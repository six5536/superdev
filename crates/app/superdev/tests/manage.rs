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
use superdev_core::{
    sokf::parse_concept,
    workflow::{cache, git},
};

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
fn write_fake(bin: &Path, name: &str, body: &str) {
    let p = bin.join(name);
    fs::write(&p, body).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
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
        &format!("{config}\n").replace("[code-index]", "[[code-index]]"),
    );
    let exclusive = run(sb.superdev().arg("status"));
    assert_eq!(exclusive.code, 2, "stdout: {}", exclusive.stdout);
    assert!(
        exclusive
            .stderr
            .contains("code-index holds one provider — use a single [code-index] table"),
        "stderr: {}",
        exclusive.stderr
    );
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

#[test]
fn init_materializes_pi_workflow_without_claude_assets() {
    let sb = Sandbox::new();
    sb.superdev()
        .args(["init", "--no-skills", "--no-code-index", "--no-frontend"])
        .assert()
        .success();

    let repo = sb.repo();
    for path in [
        ".pi/extensions/superdev/index.ts",
        ".pi/extensions/superdev/prompts/scope.md",
        ".pi/extensions/superdev/prompts/requirements-review.md",
        ".pi/extensions/superdev/prompts/build.md",
        ".pi/extensions/superdev/prompts/code-review.md",
        ".pi/extensions/superdev/prompts/accept.md",
        ".pi/extensions/superdev/prompts/file.md",
        ".pi/skills/sokf-authoring/SKILL.md",
    ] {
        assert!(repo.join(path).is_file(), "{path} was not materialized");
    }
    assert!(!repo.join(".claude/skills").exists());
    assert!(!repo.join(".claude/settings.json").exists());
    let lock = sb.read(".superdev/lock.toml");
    assert!(lock.contains(".pi/extensions/superdev/index.ts"));
    assert!(lock.contains(".pi/skills/sokf-authoring/SKILL.md"));
    assert!(!lock.contains(".claude/skills"));
    assert!(!lock.contains("superdev hook run"));
}

#[test]
fn workflow_start_creates_a_canonical_scope_plan_and_resume_adopts_it() {
    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "init"]);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "file",
            "--title",
            "Canonical recovery",
            "--description",
            "Create and recover one canonical workflow.",
            "--human-approved",
        ])
        .assert()
        .success();

    let identity = [
        "--session",
        "pi-a",
        "--issue",
        "issue-001-canonical-recovery",
        "--plan",
        "plan-001-canonical-recovery",
        "--work-branch",
        "work/001-canonical-recovery",
    ];
    let mut start = vec!["workflow", "start"];
    start.extend(identity);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args(start.clone())
        .assert()
        .failure();
    assert!(!git::reference_exists(dir.path(), "work/001-canonical-recovery").unwrap());
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "0123456789abcdef0123456789abcdef")
        .args(start)
        .assert()
        .success();
    assert_eq!(
        git::current_branch(dir.path()).unwrap(),
        "work/001-canonical-recovery"
    );
    let plan = dir
        .path()
        .join("knowledge/plans/open/plan-001-canonical-recovery.md");
    assert!(plan.is_file());
    assert!(fs::read_to_string(&plan).unwrap().contains("phase: scope"));
    let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .args([
            "workflow",
            "transition",
            "--session",
            "pi-a",
            "--expected-revision",
            &revision,
            "--phase",
            "scope",
            "--transition",
            "approve-scope",
        ])
        .assert()
        .failure();
    assert!(fs::read_to_string(&plan).unwrap().contains("phase: scope"));

    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["workflow", "cancel", "--session", "pi-a"])
        .assert()
        .success();
    let mut resume = vec!["workflow", "resume"];
    resume.extend([
        "--session",
        "pi-b",
        "--issue",
        "issue-001-canonical-recovery",
        "--plan",
        "plan-001-canonical-recovery",
        "--work-branch",
        "work/001-canonical-recovery",
    ]);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args(resume)
        .assert()
        .success();

    let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    let scoped = fs::read_to_string(&plan)
        .unwrap()
        .replace(
            "SCOPE must replace this initial recovery-safe draft with settled requirements before approval.",
            "Implement one tested source checkpoint.",
        )
        .replace(
            "SCOPE must identify the exact source declarations and materialized interfaces.",
            "Add `src/lib.rs`; no generated interface changes.",
        )
        .replace(
            "SCOPE must identify normative and current-state knowledge changes.",
            "Update only this canonical plan.",
        )
        .replace(
            "SCOPE must map applicable surfaces from the canonical documentation map.",
            "No user-observable documentation surface is affected.",
        )
        .replace("- Areas: to be settled by SCOPE.", "- Areas: `src`.")
        .replace(
            "- Verification: executable commands must be settled by SCOPE.",
            "- Verification: `true`.",
        )
        .replace(
            "- Tests: executable contract evidence must be settled by SCOPE.",
            "- Tests: `true`.",
        )
        .replace(
            "- Structural evidence: executable structural evidence must be settled by SCOPE.",
            "- Structural evidence: `true`.",
        )
        .replace(
            "- Documentation: applicable surfaces and commands must be settled by SCOPE.",
            "- Documentation: none; no user-observable change.",
        );
    fs::write(&plan, &scoped).unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["validate", "--warnings"])
        .assert()
        .success();
    let scoped_revision = parse_concept(&plan.to_string_lossy(), &scoped)
        .unwrap()
        .content_hash;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "evidence",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
            "--revision",
            &scoped_revision,
            "--kind",
            "scope-review",
            "--review-session",
            "review-scope",
        ])
        .assert()
        .success();
    let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
            "--phase",
            "scope",
            "--transition",
            "approve-scope",
        ])
        .assert()
        .success();

    let attempt_head = git::revision(dir.path(), "HEAD").unwrap();
    for diagnostics in [
        "tests   failed at src/lib.rs",
        " tests failed at src/lib.rs ",
        "tests failed   at src/lib.rs",
    ] {
        let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
        Command::cargo_bin("superdev")
            .unwrap()
            .current_dir(dir.path())
            .args([
                "workflow",
                "attempt",
                "--session",
                "pi-b",
                "--expected-revision",
                &revision,
                "--command",
                "cargo test",
                "--exit-status",
                "1",
                "--diagnostics",
                diagnostics,
            ])
            .assert()
            .success();
    }
    let stalled = fs::read_to_string(&plan).unwrap();
    assert!(stalled.contains("Attempts: 3."));
    assert!(stalled.contains("Fingerprint: "));
    assert!(stalled.contains("Blocker: stalled after 3 equivalent failures"));
    assert_ne!(git::revision(dir.path(), "HEAD").unwrap(), attempt_head);
    let stalled_head = git::revision(dir.path(), "HEAD").unwrap();
    let stalled_revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "attempt",
            "--session",
            "pi-b",
            "--expected-revision",
            &stalled_revision,
            "--command",
            "cargo test",
            "--exit-status",
            "1",
            "--diagnostics",
            "tests failed at src/lib.rs",
        ])
        .assert()
        .failure();
    assert_eq!(git::revision(dir.path(), "HEAD").unwrap(), stalled_head);
    assert_eq!(
        cache::load(dir.path()).unwrap().unwrap().last_plan_revision,
        stalled_revision
    );

    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "attempt",
            "--session",
            "pi-b",
            "--expected-revision",
            &stalled_revision,
            "--command",
            "cargo test",
            "--exit-status",
            "1",
            "--diagnostics",
            "a new failure",
        ])
        .assert()
        .success();
    let attempted = fs::read_to_string(&plan).unwrap();
    assert!(attempted.contains("Attempts: 1."));
    assert!(attempted.contains("Blocker: retrying after failure"));

    let expected = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    let tampered = attempted
        .replace("- [ ] Done.", "- [x] Done.")
        .replace("Attempts: 1.", "Attempts: 0.")
        .replace("- Areas: to be settled by SCOPE.", "- Areas: `src`.")
        .replace(
            "- Verification: executable commands must be settled by SCOPE.",
            "- Verification: `true`.",
        );
    fs::write(&plan, &tampered).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "pub fn checkpointed() {}\n").unwrap();
    let tampered_revision = parse_concept(&plan.to_string_lossy(), &tampered)
        .unwrap()
        .content_hash;
    let before_refusal = git::revision(dir.path(), "HEAD").unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "block",
            "--session",
            "pi-b",
            "--expected-revision",
            &expected,
            "--revision",
            &tampered_revision,
        ])
        .assert()
        .failure();
    assert_eq!(git::revision(dir.path(), "HEAD").unwrap(), before_refusal);

    let broadened = attempted
        .replace("- [ ] Done.", "- [x] Done.")
        .replace("- Areas: `src`.", "- Areas: `src` and `outside`.");
    fs::write(&plan, &broadened).unwrap();
    fs::write(dir.path().join("outside"), "not scope approved\n").unwrap();
    let broadened_revision = parse_concept(&plan.to_string_lossy(), &broadened)
        .unwrap()
        .content_hash;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "block",
            "--session",
            "pi-b",
            "--expected-revision",
            &expected,
            "--revision",
            &broadened_revision,
        ])
        .assert()
        .failure();
    assert_eq!(git::revision(dir.path(), "HEAD").unwrap(), before_refusal);
    fs::remove_file(dir.path().join("outside")).unwrap();

    let text = attempted
        .replace("- [ ] Done.", "- [x] Done.")
        .replace("- Areas: to be settled by SCOPE.", "- Areas: `src`.")
        .replace(
            "- Verification: executable commands must be settled by SCOPE.",
            "- Verification: `true`.",
        )
        .replace("Blocker: scope approval pending.", "Blocker: none.");
    fs::write(&plan, &text).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "pub fn checkpointed() {}\n").unwrap();
    let changed = parse_concept(&plan.to_string_lossy(), &text)
        .unwrap()
        .content_hash;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "block",
            "--session",
            "pi-b",
            "--expected-revision",
            &expected,
            "--revision",
            &changed,
        ])
        .assert()
        .success();
    git::require_clean(dir.path()).unwrap();
    let paths = std::process::Command::new("git")
        .args(["show", "--format=", "--name-only", "HEAD"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let paths = String::from_utf8(paths.stdout).unwrap();
    assert!(paths.contains("src/lib.rs"));
    assert!(paths.contains("knowledge/plans/open/plan-001-canonical-recovery.md"));
    let checkpointed = fs::read_to_string(&plan).unwrap();
    assert!(checkpointed.contains("Attempts: 0."));
    assert!(checkpointed.contains("Fingerprint: none."));
    assert!(checkpointed.contains("Blocker: none."));
    assert_eq!(
        cache::load(dir.path()).unwrap().unwrap().last_plan_revision,
        parse_concept(&plan.to_string_lossy(), &checkpointed)
            .unwrap()
            .content_hash
    );

    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "file",
            "--title",
            "Default branch advance",
            "--description",
            "Exercise BUILD synchronization with a concurrent knowledge filing.",
            "--human-approved",
        ])
        .assert()
        .success();
    let owner = cache::load(dir.path()).unwrap().unwrap();
    let expected_default = git::revision(dir.path(), "main").unwrap();
    let expected_work = git::revision(dir.path(), "work/001-canonical-recovery").unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "sync",
            "--session",
            "pi-b",
            "--expected-revision",
            &owner.last_plan_revision,
            "--expected-default",
            &expected_default,
            "--expected-work",
            &expected_work,
        ])
        .assert()
        .success();
    let synchronized = git::revision(dir.path(), "work/001-canonical-recovery").unwrap();
    assert!(git::is_ancestor(dir.path(), &expected_default, &synchronized).unwrap());
    assert_eq!(
        cache::load(dir.path()).unwrap().unwrap().last_plan_revision,
        owner.last_plan_revision
    );

    let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
            "--phase",
            "build",
            "--transition",
            "return-to-scope",
            "--feedback",
            "The public behavior needs a clarified requirement.",
        ])
        .assert()
        .success();
    let returned = fs::read_to_string(&plan).unwrap();
    assert!(returned.contains("phase: scope"));
    assert!(returned.contains("Scope product baseline: "));
    assert!(!returned.contains("Scope requirements review: clean"));
    assert!(!returned.contains("Human scope approval: approved"));
    let issue = fs::read_to_string(
        dir.path()
            .join("knowledge/issues/open/issue-001-canonical-recovery.md"),
    )
    .unwrap();
    assert!(issue.contains("- [ ] BUILD discovery:"));
    assert!(issue.contains("The public behavior needs a clarified requirement."));

    let expected = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    let rescoped = returned.replace(
        "Implement one tested source checkpoint.",
        "Implement one tested source checkpoint with clarified public behavior.",
    );
    fs::write(&plan, &rescoped).unwrap();
    let rescoped_revision = parse_concept(&plan.to_string_lossy(), &rescoped)
        .unwrap()
        .content_hash;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "evidence",
            "--session",
            "pi-b",
            "--expected-revision",
            &expected,
            "--revision",
            &rescoped_revision,
            "--kind",
            "scope-review",
            "--review-session",
            "review-rescope",
        ])
        .assert()
        .success();
    let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
            "--phase",
            "scope",
            "--transition",
            "approve-scope",
        ])
        .assert()
        .success();
    assert!(fs::read_to_string(&plan).unwrap().contains("phase: build"));

    let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "abandon",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
            "--phase",
            "build",
            "--reason",
            "Human chose not to continue.",
        ])
        .assert()
        .success();
    assert!(cache::load(dir.path()).unwrap().is_none());
    let default_plan = std::process::Command::new("git")
        .args([
            "show",
            "main:knowledge/plans/abandoned/plan-001-canonical-recovery.md",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(default_plan.status.success());
    assert!(
        String::from_utf8(default_plan.stdout)
            .unwrap()
            .contains("phase: abandoned")
    );
    let default_product = std::process::Command::new("git")
        .args(["cat-file", "-e", "main:src/lib.rs"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        !default_product.status.success(),
        "partial product reached main"
    );
}

#[test]
fn file_commits_on_default_without_a_workflow_branch() {
    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "init"]);

    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "file",
            "--kind",
            "issue",
            "--title",
            "Safe filing works",
            "--description",
            "Capture this request without creating a workflow branch.",
            "--human-approved",
        ])
        .assert()
        .success();

    assert!(
        dir.path()
            .join("knowledge/issues/open/issue-001-safe-filing-works.md")
            .is_file()
    );
    let message = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&message.stdout).trim(),
        "docs: file issue-001-safe-filing-works"
    );

    git(&["switch", "-q", "-c", "work/001-active"]);
    let work_tip = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir.path())
        .output()
        .unwrap()
        .stdout;
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "file",
            "--kind",
            "idea",
            "--title",
            "An independent thought",
            "--description",
            "Keep this idea on the default branch while BUILD is active.",
            "--human-approved",
        ])
        .assert()
        .success();
    assert_eq!(git::current_branch(dir.path()).unwrap(), "work/001-active");
    let after = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir.path())
        .output()
        .unwrap()
        .stdout;
    assert_eq!(after, work_tip);
    assert!(git::revision(dir.path(), "main").is_ok());

    let binary = assert_cmd::cargo::cargo_bin("superdev");
    let mut first = std::process::Command::new(&binary);
    first.current_dir(dir.path()).args([
        "file",
        "--kind",
        "issue",
        "--title",
        "Concurrent filing one",
        "--description",
        "The filing lock must allocate this independently.",
        "--human-approved",
    ]);
    let mut second = std::process::Command::new(&binary);
    second.current_dir(dir.path()).args([
        "file",
        "--kind",
        "issue",
        "--title",
        "Concurrent filing two",
        "--description",
        "The filing lock must serialize this allocation.",
        "--human-approved",
    ]);
    let first = first.spawn().unwrap();
    let second = second.spawn().unwrap();
    assert!(first.wait_with_output().unwrap().status.success());
    assert!(second.wait_with_output().unwrap().status.success());
    assert!(
        git::file_at_revision(
            dir.path(),
            "main",
            "knowledge/issues/open/issue-002-concurrent-filing-one.md"
        )
        .is_ok()
            || git::file_at_revision(
                dir.path(),
                "main",
                "knowledge/issues/open/issue-003-concurrent-filing-one.md"
            )
            .is_ok()
    );
    assert!(
        git::file_at_revision(
            dir.path(),
            "main",
            "knowledge/issues/open/issue-002-concurrent-filing-two.md"
        )
        .is_ok()
            || git::file_at_revision(
                dir.path(),
                "main",
                "knowledge/issues/open/issue-003-concurrent-filing-two.md"
            )
            .is_ok()
    );
    assert_eq!(git::current_branch(dir.path()).unwrap(), "work/001-active");
    let after_concurrent = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir.path())
        .output()
        .unwrap()
        .stdout;
    assert_eq!(after_concurrent, work_tip);
    let cache_entries = fs::read_dir(dir.path().join(".superdev/cache"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        cache_entries
            .iter()
            .all(|entry| !entry.starts_with("filing-worktree-")),
        "stale filing worktree: {cache_entries:?}"
    );

    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "file",
            "--kind",
            "issue",
            "--title",
            "SAFE FILING WORKS",
            "--description",
            "A differently cased duplicate must not allocate another record.",
            "--human-approved",
        ])
        .assert()
        .failure();
    assert_eq!(git::current_branch(dir.path()).unwrap(), "work/001-active");
}
