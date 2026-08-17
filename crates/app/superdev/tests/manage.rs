#![cfg(unix)]
//! End-to-end tests for the manage verbs against fake `mise`/`claude`/
//! `codegraph` binaries on PATH. The fakes are shell scripts, so Windows runs
//! `tests/cli.rs` only.

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

    /// Hand-edit the manifest's workflows pin, as a user with a stale checkout
    /// (or a newer superdev's config) would leave it.
    fn pin_workflows(&self, version: &str) {
        let config = self.read(".superdev/config.toml");
        self.write(
            ".superdev/config.toml",
            &config.replace("version = \"1.2.3\"", &format!("version = \"{version}\"")),
        );
        assert!(self.read(".superdev/config.toml").contains(version));
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

fn write_fake(bin: &Path, name: &str, body: &str) {
    let p = bin.join(name);
    fs::write(&p, body).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn init_sets_up_a_fresh_repo() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    let repo = sb.repo();
    assert!(repo.join(".superdev/config.toml").is_file());
    assert!(repo.join(".superdev/lock.toml").is_file());
    assert!(repo.join("AGENTS.md").is_file());
    assert!(repo.join(".agents/aokf/SPEC.md").is_file());
    let mcp = sb.read(".mcp.json");
    assert!(mcp.contains("\"superdev-aokf\""), "{mcp}");
    assert!(mcp.contains("\"mcp\""), "{mcp}");
    assert!(sb.read(".gitignore").contains(".superdev/cache/"));
    assert!(sb.read(".gitignore").contains(".codegraph/"));
    assert!(sb.read(".mise.toml").contains("http:mattpocock-skills"));
    // The default workflows provider materialises its skills into the repo, so
    // a collaborator gets them from git alone.
    assert_eq!(sb.read(".claude/skills/tdd/SKILL.md"), "upstream skill\n");
    assert!(repo.join(".claude/skills/handoff/SKILL.md").is_file());
    // codegraph comes from its checksummed release bundles, not npm: the
    // bundles vendor their own Node, so the repo needs none.
    let pinned = sb.read(".mise.toml");
    assert!(pinned.contains("http:codegraph"), "{pinned}");
    assert!(pinned.contains("linux-arm64"), "{pinned}");
    assert!(pinned.contains("sha256:"), "{pinned}");
    assert!(!pinned.contains("npm:"), "{pinned}");
    let log = sb.log();
    assert!(log.contains("mise trust"), "log: {log}");
    assert!(log.contains("mise install"), "log: {log}");
    assert!(
        log.contains("mise where http:mattpocock-skills"),
        "log: {log}"
    );
    assert!(log.contains("claude plugin install frontend-design@claude-code-plugins"));
    assert!(
        log.contains("mise exec http:codegraph -- codegraph init"),
        "log: {log}"
    );
}

#[test]
fn status_is_clean_after_init_and_dirty_after_tamper() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    sb.superdev().arg("status").assert().success();
    sb.write(".agents/aokf/SPEC.md", "tampered");
    let dirty = run(sb.superdev().arg("status"));
    assert_eq!(dirty.code, 1, "stdout: {}", dirty.stdout);
    assert!(dirty.stdout.contains("SPEC.md"), "stdout: {}", dirty.stdout);
    sb.superdev().arg("sync").assert().success();
    sb.superdev().arg("status").assert().success();
    assert_ne!(sb.read(".agents/aokf/SPEC.md"), "tampered");
}

#[test]
fn sync_installs_committed_pins_on_a_fresh_clone() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    sb.simulate_fresh_machine();

    // No pin edit is planned — `.mise.toml` is committed and correct — but the
    // tools it names are not installed on this machine.
    let synced = run(sb.superdev().arg("sync"));
    assert_eq!(synced.code, 0, "stderr: {}", synced.stderr);
    let log = sb.log();
    let install = log
        .find("mise install")
        .unwrap_or_else(|| panic!("log: {log}"));
    // mise refuses to install from a config it has not been told to trust.
    let trust = log
        .find("mise trust")
        .unwrap_or_else(|| panic!("log: {log}"));
    let plugin = log.find("claude plugin install").unwrap();
    let index = log
        .find("mise exec http:codegraph -- codegraph init")
        .unwrap();
    assert!(trust < install, "log: {log}");
    assert!(install < plugin && install < index, "log: {log}");
    sb.superdev().arg("status").assert().success();
}

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

#[test]
fn init_refuses_reruns_and_non_git_dirs() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    let rerun = run(sb.superdev().arg("init"));
    assert_eq!(rerun.code, 2);
    assert!(rerun.stderr.contains("sync"), "stderr: {}", rerun.stderr);

    let plain = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("superdev").unwrap();
    let not_git = run(cmd.current_dir(plain.path()).arg("init"));
    assert_eq!(not_git.code, 2);
    assert!(not_git.stderr.contains("git"), "stderr: {}", not_git.stderr);
}

#[test]
fn scaffolds_survive_user_edits_and_sync_dry_run_changes_nothing() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    sb.write("AGENTS.md", "customised");
    sb.superdev().arg("status").assert().success(); // scaffolds never drift
    sb.write(".agents/aokf/SPEC.md", "tampered");
    sb.superdev().args(["sync", "--dry-run"]).assert().success();
    assert_eq!(sb.read(".agents/aokf/SPEC.md"), "tampered");
    assert_eq!(sb.read("AGENTS.md"), "customised");
}

#[test]
fn update_refuses_hand_picked_checksum_pinned_versions() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    // Both capabilities superdev downloads by checksum ship exactly the
    // version this binary carries digests for.
    for target in ["code-index@9.9.9", "workflows@9.9.9"] {
        let refused = run(sb.superdev().args(["update", target]));
        assert_eq!(refused.code, 2, "stdout: {}", refused.stdout);
        assert!(
            refused.stderr.contains("registry default"),
            "stderr: {}",
            refused.stderr
        );
    }
    assert!(!sb.read(".superdev/config.toml").contains("9.9.9"));
    sb.superdev().args(["update", "flying"]).assert().code(2);
    // Retargeting a capability with no explicit version still works.
    sb.superdev()
        .args(["update", "code-index"])
        .assert()
        .success();
    sb.superdev().arg("status").assert().success();
}

#[test]
fn a_stale_workflows_pin_makes_status_dirty() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    sb.pin_workflows("1.0.0");
    let stale = run(sb.superdev().arg("status"));
    assert_eq!(stale.code, 1, "stdout: {}", stale.stdout);
    assert!(
        stale
            .stdout
            .contains("workflows: pinned 1.0.0, registry has 1.2.3"),
        "stdout: {}",
        stale.stdout
    );
    assert!(
        stale.stdout.contains("superdev update"),
        "stdout: {}",
        stale.stdout
    );
}

#[test]
fn sync_refuses_a_stale_workflows_pin() {
    let sb = Sandbox::new();
    sb.superdev().arg("init").assert().success();
    sb.pin_workflows("1.0.0");
    let refused = run(sb.superdev().arg("sync"));
    assert_eq!(refused.code, 2, "stdout: {}", refused.stdout);
    assert!(
        refused.stderr.contains("run `superdev update`"),
        "stderr: {}",
        refused.stderr
    );
    // `update` is the way out, and it leaves the repo in sync again.
    sb.superdev().arg("update").assert().success();
    sb.superdev().arg("status").assert().success();
}

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
