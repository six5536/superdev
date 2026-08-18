#![cfg(unix)]
//! Smoke journeys for the manage verbs against fake `mise`/`claude`/
//! `codegraph` binaries on PATH. Five end-to-end runs of the real binary and
//! the real `SystemRunner`; orchestration details (call ordering, targeted
//! install lists, per-provider flows) are asserted once, in core, against
//! `FakeRunner`. The fakes are shell scripts, so Windows runs `tests/cli.rs`
//! only.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

/// A temp git repo plus a bin dir of fake `mise`/`claude`/`codegraph`.
struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    fn new() -> Sandbox {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap(); // presence is all superdev checks
        let bin = dir.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        // What `mise where http:mattpocock-skills` resolves to: a stand-in for
        // the unpacked upstream tarball the workflows materialiser copies from.
        for rel in [
            "skills/engineering/tdd/SKILL.md",
            "skills/engineering/code-review/SKILL.md",
            "skills/productivity/handoff/SKILL.md",
        ] {
            let p = dir.path().join("checkout").join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, "upstream skill\n").unwrap();
        }
        // `mise where` fails until `mise install` has run, as the real mise does
        // on a machine that has never installed the repo's pinned tools. `mise
        // exec` forwards to the fake tool, which is how a pinned tool is
        // reached: it is on no PATH the caller can see.
        write_fake(
            &bin,
            "mise",
            r#"#!/bin/sh
echo "mise $@" >> "$FAKE_LOG"
case "$1" in
  install) : > "$FAKE_INSTALLED" ;;
  exec)
    # `mise exec [TOOL...] -- CMD`: skip the tools superdev names.
    shift
    while [ $# -gt 0 ] && [ "$1" != "--" ]; do shift; done
    [ "$1" = "--" ] && shift
    exec "$@" ;;
  where)
    [ -f "$FAKE_INSTALLED" ] || { echo "$2 is not installed" >&2; exit 1; }
    case "$2" in
      http:mattpocock-skills) echo "$FAKE_CHECKOUT" ;;
      *) echo /tmp/fake-superpowers ;;
    esac ;;
esac
exit 0
"#,
        );
        write_fake(
            &bin,
            "claude",
            r#"#!/bin/sh
echo "claude $@" >> "$FAKE_LOG"
case "$1 $2" in
  "plugin list") cat "$FAKE_PLUGINS" 2>/dev/null ;;
  "plugin install") echo "$3" >> "$FAKE_PLUGINS" ;;
esac
exit 0
"#,
        );
        write_fake(
            &bin,
            "codegraph",
            r#"#!/bin/sh
echo "codegraph $@" >> "$FAKE_LOG"
[ -n "$FAKE_CODEGRAPH_FAILS" ] && { echo "index build failed" >&2; exit 1; }
[ "$1" = "init" ] && mkdir -p .codegraph
exit 0
"#,
        );
        Sandbox { dir }
    }

    fn repo(&self) -> PathBuf {
        self.dir.path().join("repo")
    }

    fn superdev(&self) -> Command {
        let mut cmd = Command::cargo_bin("superdev").unwrap();
        let path = format!(
            "{}:{}",
            self.dir.path().join("bin").display(),
            std::env::var("PATH").unwrap()
        );
        cmd.current_dir(self.repo())
            .env("PATH", path)
            .env("FAKE_LOG", self.dir.path().join("calls.log"))
            .env("FAKE_PLUGINS", self.dir.path().join("plugins.txt"))
            .env("FAKE_INSTALLED", self.dir.path().join("installed"))
            .env("FAKE_CHECKOUT", self.dir.path().join("checkout"));
        cmd
    }

    fn log(&self) -> String {
        fs::read_to_string(self.dir.path().join("calls.log")).unwrap_or_default()
    }

    /// Forget every trace of this machine having run superdev before: no tools
    /// installed, no plugins, no code index, and an empty call log. The
    /// committed files stay, so this is a fresh clone of the same repo.
    fn simulate_fresh_machine(&self) {
        for name in ["calls.log", "plugins.txt", "installed"] {
            let _ = fs::remove_file(self.dir.path().join(name));
        }
        fs::remove_dir_all(self.repo().join(".codegraph")).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.repo().join(rel)).unwrap()
    }

    fn write(&self, rel: &str, content: &str) {
        fs::write(self.repo().join(rel), content).unwrap();
    }
}

/// One finished run: exit code plus captured output.
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Run to completion and decode the output. `predicates` is not a dependency
/// of this crate, so content assertions go through plain strings.
fn run(cmd: &mut Command) -> Run {
    let out = cmd.output().unwrap();
    Run {
        code: out.status.code().expect("not signalled"),
        stdout: String::from_utf8(out.stdout).unwrap(),
        stderr: String::from_utf8(out.stderr).unwrap(),
    }
}

/// Drop one `[table]` and its keys from a TOML manifest, leaving every other
/// line as it stands: the hand edit that turns a capability off.
fn remove_table(toml: &str, table: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for line in toml.lines() {
        if line.trim_start().starts_with('[') {
            skipping = line.trim() == table;
        }
        if !skipping {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// One `[table]`'s keys, up to the next header. Panics when the table is
/// missing, which is the same failure the caller's assertion would report.
fn table<'a>(toml: &'a str, header: &str) -> &'a str {
    let rest = toml
        .split_once(header)
        .unwrap_or_else(|| panic!("no {header} in: {toml}"))
        .1;
    rest.split_once("\n[").map_or(rest, |(keys, _)| keys)
}

/// The backed-up copy of `rel`, searched across the per-run stamp directories
/// the engine names by clock time.
fn backup_of(sb: &Sandbox, rel: &str) -> Option<String> {
    fs::read_dir(sb.repo().join(".superdev/cache/backup"))
        .ok()?
        .filter_map(|stamp| fs::read_to_string(stamp.ok()?.path().join(rel)).ok())
        .next()
}

fn write_fake(bin: &Path, name: &str, body: &str) {
    let p = bin.join(name);
    fs::write(&p, body).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Journey 1: init on the defaults, then live with the repo — drift is
/// reported, dry-run touches nothing, sync repairs, scaffolds stay the
/// user's.
#[test]
fn init_sets_up_a_fresh_repo() {
    let sb = Sandbox::new();
    let init = run(sb.superdev().arg("init"));
    assert_eq!(init.code, 0, "stderr: {}", init.stderr);
    let repo = sb.repo();
    assert!(repo.join(".superdev/config.toml").is_file());
    assert!(repo.join(".superdev/lock.toml").is_file());
    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join(".agents/aokf/SPEC.md").is_file());
    let mcp = sb.read(".mcp.json");
    assert!(mcp.contains("\"superdev-aokf\""), "{mcp}");
    assert!(sb.read(".gitignore").contains(".superdev/cache/"));
    assert!(sb.read(".gitignore").contains(".codegraph/"));
    // The default workflows provider materialises its skills into the repo, so
    // a collaborator gets them from git alone; nothing installs at user level.
    assert_eq!(sb.read(".claude/skills/tdd/SKILL.md"), "upstream skill\n");
    assert!(repo.join(".claude/skills/handoff/SKILL.md").is_file());
    let lock = sb.read(".superdev/lock.toml");
    let owners = lock
        .split("[owners]")
        .nth(1)
        .unwrap_or_else(|| panic!("lock: {lock}"));
    assert!(
        owners.contains("\".claude/skills/tdd/SKILL.md\" = \"workflows\""),
        "lock: {lock}"
    );
    assert!(
        table(&lock, "[components.workflows]").contains("provider = \"mattpocock-skills\""),
        "lock: {lock}"
    );
    assert!(
        init.stdout
            .contains("workflows: run /setup-matt-pocock-skills in Claude Code"),
        "stdout: {}",
        init.stdout
    );
    // codegraph comes from its checksummed release bundles, not npm.
    let pinned = sb.read(".mise.toml");
    assert!(pinned.contains("http:mattpocock-skills"), "{pinned}");
    assert!(pinned.contains("http:codegraph"), "{pinned}");
    assert!(pinned.contains("sha256:"), "{pinned}");
    assert!(!pinned.contains("npm:"), "{pinned}");
    // The real binary drove each fake through PATH — wiring, not ordering:
    // core's FakeRunner tests own the ordering and the targeted lists.
    let log = sb.log();
    assert!(log.contains("mise install"), "log: {log}");
    assert!(log.contains("claude plugin install frontend-design@claude-code-plugins"));
    assert!(
        log.contains("mise exec http:codegraph -- codegraph init"),
        "log: {log}"
    );
    sb.superdev().arg("status").assert().success();

    // Drift: an owned file edit turns status dirty; dry-run changes nothing;
    // sync repairs it.
    sb.write(".agents/aokf/SPEC.md", "tampered");
    let dirty = run(sb.superdev().arg("status"));
    assert_eq!(dirty.code, 1, "stdout: {}", dirty.stdout);
    assert!(dirty.stdout.contains("SPEC.md"), "stdout: {}", dirty.stdout);
    sb.superdev().args(["sync", "--dry-run"]).assert().success();
    assert_eq!(sb.read(".agents/aokf/SPEC.md"), "tampered");
    sb.superdev().arg("sync").assert().success();
    assert_ne!(sb.read(".agents/aokf/SPEC.md"), "tampered");

    // Scaffolds are the user's from the moment they exist: no drift, ever.
    sb.write("AGENTS.md", "customised");
    sb.superdev().arg("status").assert().success();
    assert_eq!(sb.read("AGENTS.md"), "customised");

    // A version-less retarget re-pins to the registry default and stays clean.
    sb.superdev()
        .args(["update", "code-index"])
        .assert()
        .success();
    sb.superdev().arg("status").assert().success();
}

/// Journey 2: clone the repo on a machine that has never run superdev — the
/// committed pins install without a single planned edit.
#[test]
fn sync_installs_committed_pins_on_a_fresh_clone() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    sb.simulate_fresh_machine();

    // No pin edit is planned — `.mise.toml` is committed and correct — but the
    // tools it names are not installed on this machine. (Trust-before-install
    // and the targeted tool list are asserted in core's engine tests.)
    let synced = run(sb.superdev().arg("sync"));
    assert_eq!(synced.code, 0, "stderr: {}", synced.stderr);
    assert!(sb.log().contains("mise install"), "log: {}", sb.log());
    sb.superdev().arg("status").assert().success();
}

/// Journey 3: switch the workflows provider both ways — each switch sweeps
/// the other provider's footprint and reports the steps only the user can
/// take.
#[test]
fn update_provider_switches_and_sweeps_both_directions() {
    let sb = Sandbox::new();
    sb.superdev()
        .args(["init", "--workflows-provider", "superpowers"])
        .assert()
        .success();
    // The plugin flow's manifest record, for contrast with journey 1's.
    let config = sb.read(".superdev/config.toml");
    let workflows = table(&config, "[workflows]");
    assert!(workflows.contains("provider = \"superpowers\""), "{config}");
    // Init writes the registry version, not a placeholder.
    assert!(workflows.contains("version = \"6.2.0\""), "{config}");
    assert!(!sb.repo().join(".claude/skills/tdd").exists());

    let onto_files =
        run(sb
            .superdev()
            .args(["update", "workflows", "--provider", "mattpocock-skills"]));
    assert_eq!(onto_files.code, 0, "stderr: {}", onto_files.stderr);
    assert!(
        onto_files
            .stdout
            .contains("unpin http:superpowers in .mise.toml"),
        "stdout: {}",
        onto_files.stdout
    );
    // The plugin is a user-level install superdev cannot take back, and the
    // knowledge scaffold's import names the old provider: both are the user's
    // to finish.
    assert!(
        onto_files
            .stdout
            .contains("claude plugin uninstall superpowers"),
        "stdout: {}",
        onto_files.stdout
    );
    assert!(
        onto_files
            .stdout
            .contains("update the .agents import in AGENTS.md"),
        "stdout: {}",
        onto_files.stdout
    );
    let mise = sb.read(".mise.toml");
    assert!(!mise.contains("http:superpowers"), "{mise}");
    assert!(mise.contains("http:mattpocock-skills"), "{mise}");
    assert_eq!(sb.read(".claude/skills/tdd/SKILL.md"), "upstream skill\n");
    let lock = sb.read(".superdev/lock.toml");
    assert!(!lock.contains(".mise.toml:http:superpowers"), "{lock}");
    sb.superdev().arg("status").assert().success();

    // And back. The materialised set is superdev's own, so the switch sweeps
    // it rather than leaving two providers' skills side by side.
    let onto_plugin = run(sb
        .superdev()
        .args(["update", "workflows", "--provider", "superpowers"]));
    assert_eq!(onto_plugin.code, 0, "stderr: {}", onto_plugin.stderr);
    for skill in ["tdd", "code-review", "handoff"] {
        assert!(
            !sb.repo()
                .join(format!(".claude/skills/{skill}/SKILL.md"))
                .exists(),
            "{skill} survived the switch"
        );
    }
    // Swept, not shredded: the removal leaves the file under the cache, so a
    // switch made by mistake costs nothing.
    assert_eq!(
        backup_of(&sb, ".claude/skills/tdd/SKILL.md"),
        Some("upstream skill\n".to_string())
    );
    let lock = sb.read(".superdev/lock.toml");
    assert!(!lock.contains("[owners]"), "{lock}");
    let mise = sb.read(".mise.toml");
    assert!(!mise.contains("http:mattpocock-skills"), "{mise}");
    assert!(mise.contains("http:superpowers"), "{mise}");
    sb.superdev().arg("status").assert().success();
}

/// Journey 4: disable a capability — the orphan sweep unpins what superdev
/// owns and leaves the user's own pins exactly as written.
#[test]
fn disabling_code_index_unpins_codegraph_and_keeps_user_pins() {
    let sb = Sandbox::new();
    // On superpowers, so the plugin-installing workflows provider stays covered
    // end to end alongside the default materialising one.
    sb.superdev()
        .args(["init", "--workflows-provider", "superpowers"])
        .assert()
        .success();
    let log = sb.log();
    assert!(
        log.contains("claude plugin marketplace add /tmp/fake-superpowers"),
        "log: {log}"
    );
    assert!(log.contains("claude plugin install superpowers@superpowers-dev"));

    // A pin of the user's own, in the file superdev shares with them.
    let mise = sb.read(".mise.toml");
    assert!(mise.contains("[tools]"), "{mise}");
    sb.write(
        ".mise.toml",
        &mise.replace("[tools]", "[tools]\nnode = \"24\""),
    );

    let config = sb.read(".superdev/config.toml");
    let edited = remove_table(&config, "[code-index]");
    assert!(!edited.contains("codegraph"), "{edited}");
    sb.write(".superdev/config.toml", &edited);

    let dirty = run(sb.superdev().arg("status"));
    assert_eq!(dirty.code, 1, "stdout: {}", dirty.stdout);
    assert!(
        dirty.stdout.contains("unpin http:codegraph in .mise.toml"),
        "stdout: {}",
        dirty.stdout
    );

    let synced = run(sb.superdev().arg("sync"));
    assert_eq!(synced.code, 0, "stderr: {}", synced.stderr);
    // Only superdev's own pin goes: the user's and the other capability's stay.
    let mise = sb.read(".mise.toml");
    assert!(!mise.contains("http:codegraph"), "{mise}");
    assert!(mise.contains("node = \"24\""), "{mise}");
    assert!(mise.contains("http:superpowers"), "{mise}");
    let lock = sb.read(".superdev/lock.toml");
    assert!(!lock.contains(".mise.toml:http:codegraph"), "{lock}");
    assert!(!lock.contains("[components.code-index]"), "{lock}");
    assert!(lock.contains(".mise.toml:http:superpowers"), "{lock}");
    sb.superdev().arg("status").assert().success();
}

/// Journey 5: a provider command fails mid-init — the manifest survives with
/// a pointer to it, and `sync` resumes.
#[test]
fn a_failed_init_reports_the_manifest_it_leaves_behind() {
    let sb = Sandbox::new();
    let failed = run(sb.superdev().env("FAKE_CODEGRAPH_FAILS", "1").arg("init"));
    assert_eq!(failed.code, 2, "stdout: {}", failed.stdout);
    assert!(
        failed
            .stdout
            .contains("left in place: .superdev/config.toml"),
        "stdout: {}",
        failed.stdout
    );
    // The manifest is kept deliberately: it is what the retry resumes from.
    assert!(sb.repo().join(".superdev/config.toml").is_file());
    assert!(!sb.repo().join(".superdev/lock.toml").exists());
    sb.superdev().arg("sync").assert().success();
}
