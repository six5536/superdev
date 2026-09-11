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
use superdev_core::workflow::{cache, git};

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

/// contract-002-cli-superdev P_init-agents-chain, P_sync-agent-instructions.
#[test]
fn agent_instructions_are_inline_and_user_content_survives_init_and_sync() {
    for user in [None, Some("# Local rules\r\nKeep this tail")] {
        let sb = Sandbox::new();
        if let Some(user) = user {
            sb.write("AGENTS.md", user);
        }
        sb.superdev()
            .args(["init", "--no-skills", "--no-code-index", "--no-frontend"])
            .assert()
            .success();
        let expected = format!(
            "<!-- superdev:instructions -->\n{}<!-- /superdev:instructions -->\n{}",
            sb.read(".agents/superdev.md"),
            user.unwrap_or_default()
        );
        assert_eq!(sb.read("AGENTS.md"), expected);

        let legacy = format!("@.agents/superdev.md\n{}", user.unwrap_or_default());
        sb.write("AGENTS.md", &legacy);
        // Both status and dry-run observe drift without applying the migration.
        sb.superdev().args(["status", "--drift"]).assert().code(1);
        sb.superdev().args(["sync", "--dry-run"]).assert().success();
        assert_eq!(sb.read("AGENTS.md"), legacy);
        sb.superdev().arg("sync").assert().success();
        assert_eq!(sb.read("AGENTS.md"), expected);
        sb.superdev().arg("sync").assert().success();
        assert_eq!(sb.read("AGENTS.md"), expected);
        sb.superdev().args(["status", "--drift"]).assert().success();
        assert!(!sb.read(".superdev/lock.toml").contains("\"AGENTS.md\""));
    }
}

/// contract-011-interface-workflow P_extension-skills
/// contract-011-interface-workflow P_skill-cold-start
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
        ".pi/extensions/superdev/lib/phases.ts",
        ".pi/extensions/superdev/prompts/scope.md",
        ".pi/extensions/superdev/prompts/requirements-review.md",
        ".pi/extensions/superdev/prompts/build.md",
        ".pi/extensions/superdev/prompts/code-review.md",
        ".pi/extensions/superdev/prompts/accept.md",
        ".pi/skills/sokf-authoring/SKILL.md",
        ".pi/extensions/superdev/skills/scope/SKILL.md",
        ".pi/extensions/superdev/skills/build/SKILL.md",
        ".pi/extensions/superdev/skills/accept/SKILL.md",
    ] {
        assert!(repo.join(path).is_file(), "{path} was not materialized");
    }
    assert!(!repo.join(".claude/skills").exists());
    assert!(!repo.join(".claude/settings.json").exists());
    let lock = sb.read(".superdev/lock.toml");
    assert!(lock.contains(".pi/extensions/superdev/index.ts"));
    assert!(lock.contains(".pi/extensions/superdev/lib/phases.ts"));
    assert!(lock.contains(".pi/skills/sokf-authoring/SKILL.md"));
    assert!(lock.contains(".pi/extensions/superdev/skills/scope/SKILL.md"));
    assert!(lock.contains(".pi/extensions/superdev/skills/build/SKILL.md"));
    assert!(lock.contains(".pi/extensions/superdev/skills/accept/SKILL.md"));
    assert!(!repo.join(".pi/skills/scope").exists());
    assert!(
        repo.join(".pi/extensions/superdev/skills/file/SKILL.md")
            .is_file()
    );
    assert!(lock.contains(".pi/extensions/superdev/skills/file/SKILL.md"));
    assert!(!lock.contains(".claude/skills"));
    assert!(!lock.contains("superdev hook run"));
    for (skill, command) in [
        ("scope", "/skill:scope"),
        ("build", "/skill:build"),
        ("accept", "/skill:accept"),
    ] {
        let text = sb.read(&format!(".pi/extensions/superdev/skills/{skill}/SKILL.md"));
        assert!(!text.contains("disable-model-invocation: true"));
        assert!(text.contains("superdev_run_phase"));
        assert!(text.contains(command));
        assert!(text.contains("Examples:"));
    }
    let scope_skill = sb.read(".pi/extensions/superdev/skills/scope/SKILL.md");
    for instruction in [
        "Assume no workflow conversation is present in context",
        "Call `superdev_run_phase` with `phase: \"scope\"` and `action: \"inspect\"",
        "If the issue exists but no suitable linked plan exists",
        "Never derive a plan ID from an issue ID",
        "Discuss one unresolved decision at a time",
        "Confirm**, **Revise**, or **Cancel",
    ] {
        assert!(
            scope_skill.contains(instruction),
            "SCOPE skill omitted `{instruction}`"
        );
    }
    let scope_prompt = sb.read(".pi/extensions/superdev/prompts/scope.md");
    for instruction in [
        "Keep every plan in `phase: scope`",
        "Do not claim that requirements review or human scope approval has occurred",
        "Confirmation supplied in the task authorizes drafting only",
    ] {
        assert!(
            scope_prompt.contains(instruction),
            "isolated SCOPE prompt omitted `{instruction}`"
        );
    }
    let build_skill = sb.read(".pi/extensions/superdev/skills/build/SKILL.md");
    assert!(build_skill.contains("Assume no earlier SCOPE or BUILD conversation is present"));
    assert!(build_skill.contains("before taking any action"));
    assert!(build_skill.contains("recommend `/skill:scope <requested change>`"));
    let accept_skill = sb.read(".pi/extensions/superdev/skills/accept/SKILL.md");
    assert!(accept_skill.contains("Assume no earlier workflow discussion is present"));
    assert!(accept_skill.contains("before taking any action"));
    assert!(accept_skill.contains("destination: \"build\""));
    assert!(accept_skill.contains("destination: \"scope\""));

    let retired = b"retired managed workflow skill\n";
    let retired_path = repo.join(".claude/skills/build/SKILL.md");
    let retired_project_skill = repo.join(".pi/skills/scope/SKILL.md");
    fs::create_dir_all(retired_path.parent().unwrap()).unwrap();
    fs::create_dir_all(retired_project_skill.parent().unwrap()).unwrap();
    fs::write(&retired_path, retired).unwrap();
    fs::write(&retired_project_skill, retired).unwrap();
    let retired_hash = superdev_core::lock::sha256_hex(retired);
    let mut old_lock = lock;
    old_lock = old_lock.replacen(
        "[files]\n",
        &format!(
            "[files]\n\".claude/skills/build/SKILL.md\" = \"{retired_hash}\"\n\".pi/skills/scope/SKILL.md\" = \"{retired_hash}\"\n"
        ),
        1,
    );
    sb.write(".superdev/lock.toml", &old_lock);
    sb.superdev().arg("sync").assert().success();
    assert!(!retired_path.exists());
    assert!(!retired_project_skill.exists());
    let converged_lock = sb.read(".superdev/lock.toml");
    assert!(!converged_lock.contains(".claude/skills/build"));
    assert!(!converged_lock.contains(".pi/skills/scope"));
    assert!(
        repo.join(".pi/extensions/superdev/skills/scope/SKILL.md")
            .is_file()
    );
}

#[test]
fn workflow_start_adopts_an_llm_authored_independently_numbered_plan() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let git = |args: &[&str]| {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    let service = |authority: &str, args: &[&str]| {
        Command::cargo_bin("superdev")
            .unwrap()
            .current_dir(root)
            .env("SUPERDEV_UI_AUTHORITY", authority)
            .args(args)
            .assert()
            .success();
    };
    let refuse = |authority: &str, args: &[&str]| {
        Command::cargo_bin("superdev")
            .unwrap()
            .current_dir(root)
            .env("SUPERDEV_UI_AUTHORITY", authority)
            .args(args)
            .assert()
            .failure();
    };
    let owner_authority = "0123456789abcdef0123456789abcdef";
    let revision = || cache::load(root).unwrap().unwrap().last_plan_revision;

    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    git(&["config", "commit.gpgsign", "false"]);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "init"]);
    fs::create_dir_all(root.join("knowledge/issues/open")).unwrap();
    fs::write(
        root.join("knowledge/issues/open/issue-001-canonical-recovery.md"),
        include_str!("fixtures/workflow-issue.md"),
    )
    .unwrap();
    git(&["add", "knowledge"]);
    git(&["commit", "-qm", "docs: file canonical recovery"]);

    let plan = root.join("knowledge/plans/open/plan-042-canonical-recovery.md");
    fs::create_dir_all(plan.parent().unwrap()).unwrap();
    fs::write(&plan, include_str!("fixtures/workflow-plan.md")).unwrap();

    let identity = [
        "--session",
        "pi-a",
        "--issue",
        "issue-001-canonical-recovery",
        "--plan",
        "plan-042-canonical-recovery",
        "--work-branch",
        "work/001-canonical-recovery",
    ];
    let mut start = vec!["workflow", "start"];
    start.extend(identity);

    // Ownership originates in the interactive UI, so a start without that
    // capability reserves nothing.
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .args(start.clone())
        .assert()
        .failure();
    assert!(!git::reference_exists(root, "work/001-canonical-recovery").unwrap());

    // An existing branch means another workflow already reserved this identity.
    git(&["branch", "work/001-canonical-recovery"]);
    refuse(owner_authority, &start);
    assert!(
        git::working_paths(root)
            .unwrap()
            .contains(&"knowledge/plans/open/plan-042-canonical-recovery.md".into()),
        "a refused start kept the authored plan"
    );
    git(&["branch", "-D", "work/001-canonical-recovery"]);

    service(owner_authority, &start);
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-canonical-recovery"
    );
    assert!(fs::read_to_string(&plan).unwrap().contains("phase: scope"));
    assert!(
        git::file_at_revision(
            root,
            "main",
            "knowledge/plans/open/plan-042-canonical-recovery.md",
        )
        .is_ok(),
        "the initial plan is reserved on the default branch"
    );

    // Scope approval is a human decision, so a foreign capability cannot make it.
    refuse(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        &[
            "workflow",
            "transition",
            "--session",
            "pi-a",
            "--expected-revision",
            &revision(),
            "--phase",
            "scope",
            "--transition",
            "approve-scope",
        ],
    );
    assert!(fs::read_to_string(&plan).unwrap().contains("phase: scope"));

    // Cancellation releases ownership and preserves the checkout.
    service(
        owner_authority,
        &["workflow", "cancel", "--session", "pi-a"],
    );
    assert!(cache::load(root).unwrap().is_none());
    git(&["switch", "-q", "main"]);

    let resume_authority = "fedcba9876543210fedcba9876543210";
    service(
        resume_authority,
        &[
            "workflow",
            "resume",
            "--session",
            "pi-b",
            "--issue",
            "issue-001-canonical-recovery",
            "--plan",
            "plan-042-canonical-recovery",
            "--work-branch",
            "work/001-canonical-recovery",
        ],
    );
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-canonical-recovery"
    );

    // The scoping role authors the proposal; the core only publishes it.
    let scoped = fs::read_to_string(&plan).unwrap().replace(
        "Settle requirements in SCOPE.",
        "Implement one tested source checkpoint.",
    );
    fs::write(&plan, &scoped).unwrap();
    let before_refusal = git::revision(root, "HEAD").unwrap();
    for (session, expected) in [("pi-b", "stale"), ("intruder", &*revision())] {
        refuse(
            resume_authority,
            &[
                "workflow",
                "commit",
                "--session",
                session,
                "--expected-revision",
                expected,
                "--message",
                "docs(workflow): checkpoint scope proposal",
            ],
        );
    }
    assert_eq!(git::revision(root, "HEAD").unwrap(), before_refusal);
    service(
        resume_authority,
        &[
            "workflow",
            "commit",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision(),
            "--message",
            "docs(workflow): checkpoint scope proposal",
        ],
    );
    assert_ne!(git::revision(root, "HEAD").unwrap(), before_refusal);
    assert!(git::working_paths(root).unwrap().is_empty());

    service(
        resume_authority,
        &[
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision(),
            "--phase",
            "scope",
            "--transition",
            "approve-scope",
        ],
    );
    assert!(fs::read_to_string(&plan).unwrap().contains("phase: build"));

    // BUILD commits product work through the same generic verb.
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn built() {}\n").unwrap();
    service(
        resume_authority,
        &[
            "workflow",
            "commit",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision(),
            "--message",
            "feat: implement the approved block",
        ],
    );

    // BUILD reaches ACCEPT through an explicit edge rather than a side effect.
    refuse(
        resume_authority,
        &[
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision(),
            "--phase",
            "scope",
            "--transition",
            "complete-build",
        ],
    );
    service(
        resume_authority,
        &[
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision(),
            "--phase",
            "build",
            "--transition",
            "complete-build",
        ],
    );
    assert!(fs::read_to_string(&plan).unwrap().contains("phase: accept"));

    // Acceptance closes the records and releases ownership without merging.
    let default_before = git::revision(root, "main").unwrap();
    service(
        resume_authority,
        &[
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision(),
            "--phase",
            "accept",
            "--transition",
            "accept",
        ],
    );
    assert!(cache::load(root).unwrap().is_none());
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-canonical-recovery",
        "the accepted branch stays checked out for a human merge"
    );
    assert_eq!(
        git::revision(root, "main").unwrap(),
        default_before,
        "acceptance never merges"
    );
    assert!(
        !std::process::Command::new("git")
            .args(["cat-file", "-e", "main:src/lib.rs"])
            .current_dir(root)
            .output()
            .unwrap()
            .status
            .success(),
        "partial product reached the default branch"
    );
    assert!(
        root.join("knowledge/plans/done/plan-042-canonical-recovery.md")
            .is_file()
    );
}
