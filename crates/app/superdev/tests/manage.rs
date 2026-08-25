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
        // `mise exec` forwards to the fake tool, which is how a pinned tool is
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
            .env("FAKE_INSTALLED", self.dir.path().join("installed"));
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
    // A fresh AGENTS.md holds only superdev's import; the rest is the user's
    // to write. The fenced aggregator points at the enabled capabilities'
    // instruction files.
    assert_eq!(sb.read("AGENTS.md"), "@.agents/superdev.md\n");
    let aggregator = sb.read(".agents/superdev.md");
    assert!(aggregator.starts_with("<superdev-system>"), "{aggregator}");
    assert!(aggregator.contains("@aokf.md"), "{aggregator}");
    assert!(aggregator.contains("@codegraph.md"), "{aggregator}");
    assert!(aggregator.contains("@rtk.md"), "{aggregator}");
    assert!(aggregator.contains("@coding.md"), "{aggregator}");
    assert!(repo.join(".agents/coding.md").is_file());
    assert!(repo.join(".agents/professionalism.md").is_file());
    assert!(repo.join(".agents/process.md").is_file());
    assert!(repo.join(".agents/aokf.md").is_file());
    assert!(repo.join(".agents/codegraph.md").is_file());
    assert!(repo.join(".agents/rtk.md").is_file());
    // The output filter: auto_env activation, the platform-scoped rtk pins
    // (windows-arm64 deliberately absent), and the rewrite hook.
    assert!(sb.read(".miserc.toml").contains("auto_env = true"));
    let unix_pins = sb.read("mise.unix.toml");
    assert!(unix_pins.contains("http:rtk"), "{unix_pins}");
    assert!(unix_pins.contains("linux-arm64"), "{unix_pins}");
    let windows_pins = sb.read("mise.windows-x64.toml");
    assert!(windows_pins.contains("windows-x64"), "{windows_pins}");
    assert!(!windows_pins.contains("windows-arm64"), "{windows_pins}");
    let settings = sb.read(".claude/settings.json");
    assert!(settings.contains("PreToolUse"), "{settings}");
    assert!(settings.contains("rtk hook claude"), "{settings}");
    assert!(repo.join(".agents/aokf/SPEC.md").is_file());
    let mcp = sb.read(".mcp.json");
    assert!(mcp.contains("\"superdev-aokf\""), "{mcp}");
    assert!(mcp.contains("\"codegraph\""), "{mcp}");
    assert!(sb.read(".gitignore").contains(".superdev/cache/"));
    assert!(sb.read(".gitignore").contains(".codegraph/"));
    // The knowledge capability carries the converted skill set into the repo,
    // so a collaborator gets it from git alone; nothing installs at user level.
    assert!(repo.join(".claude/skills/frame/SKILL.md").is_file());
    assert!(repo.join(".claude/skills/prototype/LOGIC.md").is_file());
    assert!(
        repo.join(".claude/skills/how-do-i/SESSION-BOUNDARIES.md")
            .is_file()
    );
    let lock = sb.read(".superdev/lock.toml");
    assert!(
        lock.contains("\".claude/skills/frame/SKILL.md\""),
        "lock: {lock}"
    );
    assert!(!lock.contains("workflows"), "lock: {lock}");
    assert!(!sb.read(".superdev/config.toml").contains("workflows"));
    // codegraph comes from its checksummed release bundles, not npm.
    let pinned = sb.read(".mise.toml");
    assert!(pinned.contains("http:codegraph"), "{pinned}");
    assert!(pinned.contains("sha256:"), "{pinned}");
    assert!(!pinned.contains("npm:"), "{pinned}");
    assert!(!pinned.contains("mattpocock"), "{pinned}");
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

    // AGENTS.md is the user's: any content is fine as long as the import
    // line stays, and superdev never rewrites the rest.
    sb.write("AGENTS.md", "customised\n@.agents/superdev.md\n");
    sb.superdev().arg("status").assert().success();
    assert_eq!(sb.read("AGENTS.md"), "customised\n@.agents/superdev.md\n");
    // Deleting the line is planned work; sync appends it back and reports
    // the trim hint, leaving the user's text alone.
    sb.write("AGENTS.md", "customised\n");
    sb.superdev().arg("status").assert().code(1);
    let synced = run(sb.superdev().arg("sync"));
    assert_eq!(synced.code, 0, "stderr: {}", synced.stderr);
    assert!(
        synced.stdout.contains("AGENTS.md is yours"),
        "stdout: {}",
        synced.stdout
    );
    assert_eq!(sb.read("AGENTS.md"), "customised\n@.agents/superdev.md\n");

    // A version-less retarget re-pins to the registry default and stays clean.
    sb.superdev()
        .args(["update", "code-index"])
        .assert()
        .success();
    sb.superdev().arg("status").assert().success();
}

/// The `update` verb's whole surface: retarget every pin at once, switch a
/// provider, and the two refusals — a disabled slot, and a provider switch
/// on a slot holding several packs, where there is no single entry to move.
#[test]
fn update_retargets_switches_and_refuses() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();

    // No target: every enabled capability's pin moves to this binary's
    // registry default, and the repo stays converged.
    sb.superdev().arg("update").assert().success();
    sb.superdev().arg("status").assert().success();

    // An explicit provider switch rewrites the slot's one entry.
    sb.superdev()
        .args(["update", "frontend", "--provider", "frontend-design"])
        .assert()
        .success();
    assert!(
        sb.read(".superdev/config.toml")
            .contains("provider = \"frontend-design\""),
    );

    // A capability the manifest does not enable cannot be updated.
    let config = sb.read(".superdev/config.toml");
    sb.write(
        ".superdev/config.toml",
        &remove_table(&config, "[frontend]"),
    );
    let disabled = run(sb.superdev().args(["update", "frontend"]));
    assert_eq!(disabled.code, 2, "stdout: {}", disabled.stdout);
    assert!(
        disabled.stderr.contains("`frontend` is not enabled"),
        "stderr: {}",
        disabled.stderr
    );

    // A many slot holding two packs has no single entry to switch, so the
    // provider switch refuses and says where to make the change by hand.
    let config = sb.read(".superdev/config.toml");
    let two_packs = format!(
        "{}\n[[skills]]\nprovider = \"superdev-skills\"\nversion = \"{}\"\n\n[[skills]]\nprovider = \"another-pack\"\n",
        remove_table(&config, "[skills]"),
        env!("CARGO_PKG_VERSION")
    );
    sb.write(".superdev/config.toml", &two_packs);
    let several = run(sb
        .superdev()
        .args(["update", "skills", "--provider", "superdev-skills"]));
    assert_eq!(several.code, 2, "stdout: {}", several.stdout);
    assert!(
        several.stderr.contains("skills holds several packs"),
        "stderr: {}",
        several.stderr
    );
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

/// Journey 3: a repo still carrying the removed workflows capability. The
/// manifest load fails with the guided error; once the table is deleted, sync
/// swaps same-named skills to knowledge ownership and sweeps the dropped
/// upstream files.
#[test]
fn skills_entries_are_a_set_with_guided_refusals() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    let config = sb.read(".superdev/config.toml");
    let without_skills = remove_table(&config, "[skills]");

    // The single [skills] table and a one-entry [[skills]] array are the
    // same manifest; status accepts both identically.
    sb.write(
        ".superdev/config.toml",
        &format!(
            "{without_skills}\n[[skills]]\nprovider = \"superdev-skills\"\nversion = \"{}\"\n",
            env!("CARGO_PKG_VERSION")
        ),
    );
    let accepted = run(sb.superdev().arg("status"));
    assert_ne!(accepted.code, 2, "stderr: {}", accepted.stderr);

    // A second entry naming an unregistered pack gets the provider listing.
    sb.write(
        ".superdev/config.toml",
        &format!(
            "{without_skills}\n[[skills]]\nprovider = \"superdev-skills\"\nversion = \"{}\"\n\n[[skills]]\nprovider = \"another-pack\"\n",
            env!("CARGO_PKG_VERSION")
        ),
    );
    let unknown = run(sb.superdev().arg("status"));
    assert_eq!(unknown.code, 2, "stdout: {}", unknown.stdout);
    assert!(
        unknown
            .stderr
            .contains("skills provider must be one of: superdev-skills"),
        "stderr: {}",
        unknown.stderr
    );

    // The same pack twice is refused at load, naming the duplicate.
    sb.write(
        ".superdev/config.toml",
        &format!(
            "{without_skills}\n[[skills]]\nprovider = \"superdev-skills\"\n\n[[skills]]\nprovider = \"superdev-skills\"\n"
        ),
    );
    let duplicate = run(sb.superdev().arg("status"));
    assert_eq!(duplicate.code, 2, "stdout: {}", duplicate.stdout);
    assert!(
        duplicate
            .stderr
            .contains("skills lists provider `superdev-skills` more than once"),
        "stderr: {}",
        duplicate.stderr
    );

    // The array form on an exclusive slot is refused with the way out.
    sb.write(
        ".superdev/config.toml",
        &format!("{config}\n").replace("[knowledge]", "[[knowledge]]"),
    );
    let exclusive = run(sb.superdev().arg("status"));
    assert_eq!(exclusive.code, 2, "stdout: {}", exclusive.stdout);
    assert!(
        exclusive
            .stderr
            .contains("knowledge holds one provider — use a single [knowledge] table"),
        "stderr: {}",
        exclusive.stderr
    );
}

#[test]
fn a_workflows_manifest_errors_and_sync_migrates_after_the_table_goes() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    let config = sb.read(".superdev/config.toml");

    // The pre-removal manifest shape: every verb refuses with the way out.
    sb.write(
        ".superdev/config.toml",
        &format!("{config}\n[workflows]\nprovider = \"mattpocock-skills\"\nversion = \"1.2.3\"\n"),
    );
    let refused = run(sb.superdev().arg("status"));
    assert_eq!(refused.code, 2, "stdout: {}", refused.stdout);
    assert!(
        refused
            .stderr
            .contains("the workflows capability was removed"),
        "stderr: {}",
        refused.stderr
    );
    assert!(
        refused.stderr.contains("claude plugin install superpowers"),
        "stderr: {}",
        refused.stderr
    );
    // The error never rewrites the manifest: config.toml is the user's file.
    assert!(sb.read(".superdev/config.toml").contains("[workflows]"));

    // The user deletes the table by hand. `update workflows` is now just an
    // unknown capability.
    sb.write(".superdev/config.toml", &config);
    let unknown = run(sb.superdev().args(["update", "workflows"]));
    assert_eq!(unknown.code, 2, "stdout: {}", unknown.stdout);
    assert!(
        unknown.stderr.contains("unknown capability `workflows`"),
        "stderr: {}",
        unknown.stderr
    );

    // Rewind the repo's files to what the old provider left behind: a
    // same-named skill it wrote, a dropped upstream skill, and the override
    // file, all attributed to `workflows` in the lock.
    let upstream = "upstream skill\n";
    let hash = superdev_core::lock::sha256_hex(upstream.as_bytes());
    for rel in [
        ".claude/skills/frame/SKILL.md",
        ".claude/skills/ask-matt/SKILL.md",
        ".agents/MATT-POCOCK-SKILLS.md",
    ] {
        let p = sb.repo().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, upstream).unwrap();
    }
    let lock = sb.read(".superdev/lock.toml");
    let mut edited = String::new();
    for line in lock.lines() {
        if line.starts_with("\".claude/skills/frame/SKILL.md\"") {
            edited.push_str(&format!("\".claude/skills/frame/SKILL.md\" = \"{hash}\"\n"));
            continue;
        }
        edited.push_str(line);
        edited.push('\n');
        if line == "[files]" {
            edited.push_str(&format!(
                "\".claude/skills/ask-matt/SKILL.md\" = \"{hash}\"\n"
            ));
            edited.push_str(&format!("\".agents/MATT-POCOCK-SKILLS.md\" = \"{hash}\"\n"));
        }
    }
    // The attribution a pre-removal binary recorded — including one on a
    // file sync has no reason to touch, which only a wholesale clear retires.
    edited.push_str("\n[owners]\n");
    for key in [
        ".claude/skills/frame/SKILL.md",
        ".claude/skills/ask-matt/SKILL.md",
        ".agents/MATT-POCOCK-SKILLS.md",
        ".claude/skills/wizard/SKILL.md",
    ] {
        edited.push_str(&format!("\"{key}\" = \"workflows\"\n"));
    }
    sb.write(".superdev/lock.toml", &edited);

    // The pending swap is planned work: status exits 1 until sync runs.
    let pending = run(sb.superdev().arg("status"));
    assert_eq!(pending.code, 1, "stdout: {}", pending.stdout);

    let synced = run(sb.superdev().arg("sync"));
    assert_eq!(synced.code, 0, "stderr: {}", synced.stderr);
    // The same-named skill is superdev's again — the shipped content, with
    // the legacy attribution retired from the lock.
    assert_ne!(sb.read(".claude/skills/frame/SKILL.md"), upstream);
    let lock = sb.read(".superdev/lock.toml");
    assert!(
        lock.contains("\".claude/skills/frame/SKILL.md\""),
        "lock: {lock}"
    );
    assert!(!lock.contains("workflows"), "lock: {lock}");
    assert!(!lock.contains("[owners]"), "lock: {lock}");
    // The dropped files are swept with a backup, not shredded.
    assert!(!sb.repo().join(".claude/skills/ask-matt/SKILL.md").exists());
    assert!(!sb.repo().join(".agents/MATT-POCOCK-SKILLS.md").exists());
    assert_eq!(
        backup_of(&sb, ".claude/skills/ask-matt/SKILL.md"),
        Some(upstream.to_string())
    );
    sb.superdev().arg("status").assert().success();
}

/// Journey: disable the output filter — the sweep removes the auto_env
/// activation, both platform pin files, the instruction file and the hook
/// key, and the aggregator loses the import.
#[test]
fn disabling_bash_output_filter_sweeps_the_wiring() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    let config = sb.read(".superdev/config.toml");
    let edited = remove_table(&config, "[bash-output-filter]");
    assert!(!edited.contains("rtk"), "{edited}");
    sb.write(".superdev/config.toml", &edited);

    let dirty = run(sb.superdev().arg("status"));
    assert_eq!(dirty.code, 1, "stdout: {}", dirty.stdout);
    let synced = run(sb.superdev().arg("sync"));
    assert_eq!(synced.code, 0, "stderr: {}", synced.stderr);

    let repo = sb.repo();
    assert!(!repo.join(".miserc.toml").exists());
    assert!(!repo.join("mise.unix.toml").exists());
    assert!(!repo.join("mise.windows-x64.toml").exists());
    assert!(!repo.join(".agents/rtk.md").exists());
    // The hook key goes; the knowledge capability's validate hook stays.
    let settings = sb.read(".claude/settings.json");
    assert!(!settings.contains("rtk hook claude"), "{settings}");
    assert!(
        settings.contains("superdev aokf hook validate"),
        "{settings}"
    );
    let aggregator = sb.read(".agents/superdev.md");
    assert!(!aggregator.contains("@rtk.md"), "{aggregator}");
    let lock = sb.read(".superdev/lock.toml");
    assert!(!lock.contains("rtk"), "{lock}");
    sb.superdev().arg("status").assert().success();
}

/// Journey 4: disable a capability — the orphan sweep unpins what superdev
/// owns and leaves the user's own pins exactly as written.
#[test]
fn disabling_code_index_unpins_codegraph_and_keeps_user_pins() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();

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
    // Only superdev's own pin goes: the user's stays.
    let mise = sb.read(".mise.toml");
    assert!(!mise.contains("http:codegraph"), "{mise}");
    assert!(mise.contains("node = \"24\""), "{mise}");
    let lock = sb.read(".superdev/lock.toml");
    assert!(!lock.contains(".mise.toml:http:codegraph"), "{lock}");
    assert!(!lock.contains("[components.code-index]"), "{lock}");
    // The capabilities still enabled keep their records: the sweep is targeted.
    assert!(lock.contains("[components.knowledge]"), "{lock}");
    // The agent wiring goes with the capability: instruction file, MCP key
    // and aggregator import — while the knowledge wiring stays.
    assert!(!sb.repo().join(".agents/codegraph.md").exists());
    let mcp = sb.read(".mcp.json");
    assert!(!mcp.contains("\"codegraph\""), "{mcp}");
    assert!(mcp.contains("\"superdev-aokf\""), "{mcp}");
    let aggregator = sb.read(".agents/superdev.md");
    assert!(!aggregator.contains("@codegraph.md"), "{aggregator}");
    assert!(aggregator.contains("@aokf.md"), "{aggregator}");
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
