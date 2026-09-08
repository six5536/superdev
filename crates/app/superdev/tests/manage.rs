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
        ".pi/extensions/superdev/prompts/file.md",
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
    assert!(!repo.join(".pi/skills/build").exists());
    assert!(!repo.join(".pi/skills/accept").exists());
    assert!(!lock.contains(".claude/skills"));
    assert!(!lock.contains("superdev hook run"));
    for (skill, command) in [
        ("scope", "/skill:scope"),
        ("build", "/skill:build"),
        ("accept", "/skill:accept"),
    ] {
        let text = sb.read(&format!(".pi/extensions/superdev/skills/{skill}/SKILL.md"));
        assert!(text.contains("disable-model-invocation: true"));
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
}

#[test]
fn workflow_start_adopts_an_llm_authored_independently_numbered_plan() {
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

    let plan = dir
        .path()
        .join("knowledge/plans/open/plan-042-canonical-recovery.md");
    fs::create_dir_all(plan.parent().unwrap()).unwrap();
    fs::write(
        &plan,
        "---\ntype: Plan\nid: plan-042-canonical-recovery\ntitle: Canonical recovery plan\ndescription: LLM-authored scope proposal.\nlifecycle: open\nphase: scope\nbranch: work/001-canonical-recovery\nlinks:\n  - rel: implements\n    to: issue-001-canonical-recovery\n---\n\n# Plan: Canonical recovery\n\n## Goal and boundaries\n\nImplement [the issue][sokf:issue-001-canonical-recovery].\n\n## Requirements\n\nSettle requirements in SCOPE.\n\n## Contract changes\n\n- none.\n\n## ADR decisions\n\n- none.\n\n## Source and interface changes\n\nSettle source surfaces in SCOPE.\n\n## Knowledge changes\n\nMaintain this issue and plan.\n\n## Documentation changes\n\nSettle documentation in SCOPE.\n\n## Work blocks\n\n### Block 1: Deliver scope\n\n- [ ] Done.\n- Dependencies: none.\n- Areas: pending SCOPE.\n- Outcome: approved work delivered.\n- Verification: pending SCOPE.\n- Tests: pending SCOPE.\n- Structural evidence: pending SCOPE.\n- Documentation: pending SCOPE.\n\n## Build state\n\nCurrent block: 1. Attempts: 0. Final corrections: 0. Blocker: scope approval pending.\n\n## Implementation decisions\n\nnone.\n\n## Follow-up issues\n\nnone.\n\n## Completion evidence\n\nScope review and approval pending.\n\n<!-- sokf:links -->\n[sokf:issue-001-canonical-recovery]: /knowledge/issues/open/issue-001-canonical-recovery.md\n",
    )
    .unwrap();

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
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args(start.clone())
        .assert()
        .failure();
    assert!(!git::reference_exists(dir.path(), "work/001-canonical-recovery").unwrap());
    let status_output = Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args(["workflow", "status", "--json"])
        .output()
        .unwrap();
    assert!(status_output.status.success());
    let status: serde_json::Value = serde_json::from_slice(&status_output.stdout).unwrap();
    assert_eq!(status["result"]["openWorkflows"], serde_json::json!([]));
    git(&["branch", "work/001-canonical-recovery"]);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .env("SUPERDEV_UI_AUTHORITY", "0123456789abcdef0123456789abcdef")
        .args(start.clone())
        .assert()
        .failure();
    assert!(
        git::working_paths(dir.path())
            .unwrap()
            .contains(&"knowledge/plans/open/plan-042-canonical-recovery.md".into())
    );
    git(&["branch", "-D", "work/001-canonical-recovery"]);
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
    git(&["switch", "-q", "main"]);
    let mut resume = vec!["workflow", "resume"];
    resume.extend([
        "--session",
        "pi-b",
        "--issue",
        "issue-001-canonical-recovery",
        "--plan",
        "plan-042-canonical-recovery",
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
            "Settle requirements in SCOPE.",
            "Implement one tested source checkpoint.",
        )
        .replace(
            "Settle source surfaces in SCOPE.",
            "Add `src/lib.rs`; no generated interface changes.",
        )
        .replace(
            "Maintain this issue and plan.",
            "Update only this canonical plan.",
        )
        .replace(
            "Settle documentation in SCOPE.",
            "No user-observable documentation surface is affected.",
        )
        .replace("- Areas: pending SCOPE.", "- Areas: `src`.")
        .replace("- Verification: pending SCOPE.", "- Verification: `true`.")
        .replace("- Tests: pending SCOPE.", "- Tests: `true`.")
        .replace(
            "- Structural evidence: pending SCOPE.",
            "- Structural evidence: `true`.",
        )
        .replace(
            "- Documentation: pending SCOPE.",
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
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("src/scope.rs"),
        "product changes belong to BUILD\n",
    )
    .unwrap();
    let before_scope_refusal = git::revision(dir.path(), "HEAD").unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "scope-checkpoint",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
        ])
        .assert()
        .failure();
    assert_eq!(
        git::revision(dir.path(), "HEAD").unwrap(),
        before_scope_refusal
    );
    assert_eq!(
        cache::load(dir.path()).unwrap().unwrap().last_plan_revision,
        revision
    );
    assert!(
        std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["add", "--", "src/scope.rs"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["commit", "-m", "feat: forbidden scope product commit"])
            .status()
            .unwrap()
            .success()
    );
    let product_commit = git::revision(dir.path(), "HEAD").unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "scope-checkpoint",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
        ])
        .assert()
        .failure();
    assert_eq!(git::revision(dir.path(), "HEAD").unwrap(), product_commit);
    assert!(
        std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["reset", "--hard", &before_scope_refusal])
            .status()
            .unwrap()
            .success()
    );
    fs::write(&plan, &scoped).unwrap();
    let cache_dir = dir.path().join(".superdev/cache");
    let mut cache_permissions = fs::metadata(&cache_dir).unwrap().permissions();
    cache_permissions.set_mode(0o555);
    fs::set_permissions(&cache_dir, cache_permissions).unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "scope-checkpoint",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
        ])
        .assert()
        .failure();
    let mut cache_permissions = fs::metadata(&cache_dir).unwrap().permissions();
    cache_permissions.set_mode(0o755);
    fs::set_permissions(&cache_dir, cache_permissions).unwrap();
    assert_eq!(
        git::revision(dir.path(), "HEAD").unwrap(),
        before_scope_refusal
    );
    assert_eq!(
        cache::load(dir.path()).unwrap().unwrap().last_plan_revision,
        revision
    );
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "scope-checkpoint",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
        ])
        .assert()
        .success();
    let revision = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    let scope_candidate = git::revision(dir.path(), "HEAD").unwrap();
    let issue_path = dir
        .path()
        .join("knowledge/issues/open/issue-001-canonical-recovery.md");
    let mut post_checkpoint_issue = fs::read_to_string(&issue_path).unwrap();
    post_checkpoint_issue.push_str("\nPost-checkpoint mutation.\n");
    fs::write(&issue_path, post_checkpoint_issue).unwrap();
    assert!(
        std::process::Command::new("git")
            .current_dir(dir.path())
            .args([
                "add",
                "--",
                "knowledge/issues/open/issue-001-canonical-recovery.md"
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["commit", "-m", "docs: mutate after scope checkpoint"])
            .status()
            .unwrap()
            .success()
    );
    let post_checkpoint_candidate = git::revision(dir.path(), "HEAD").unwrap();
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
            "review-post-checkpoint-mutation",
            "--candidate",
            &post_checkpoint_candidate,
        ])
        .assert()
        .failure();
    assert!(
        std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["reset", "--hard", &scope_candidate])
            .status()
            .unwrap()
            .success()
    );
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
            "review-wrong-candidate",
            "--candidate",
            &before_scope_refusal,
        ])
        .assert()
        .failure();
    assert_eq!(
        cache::load(dir.path()).unwrap().unwrap().last_plan_revision,
        revision
    );
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
            "--candidate",
            &scope_candidate,
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

    let out_of_scope = attempted.replace("- [ ] Done.", "- [x] Done.");
    fs::write(&plan, &out_of_scope).unwrap();
    fs::write(dir.path().join("outside"), "not scope approved\n").unwrap();
    let out_of_scope_revision = parse_concept(&plan.to_string_lossy(), &out_of_scope)
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
            &out_of_scope_revision,
        ])
        .assert()
        .failure();
    assert_eq!(fs::read_to_string(&plan).unwrap(), out_of_scope);
    assert_eq!(git::revision(dir.path(), "HEAD").unwrap(), before_refusal);
    fs::remove_file(dir.path().join("outside")).unwrap();

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
    assert!(paths.contains("knowledge/plans/open/plan-042-canonical-recovery.md"));
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

    // Exercise the complete configured human-acceptance and local integration
    // journey from this ready BUILD state without consuming the retry fixture.
    let accepted = tempfile::tempdir().unwrap();
    assert!(
        std::process::Command::new("cp")
            .args([
                "-a",
                &format!("{}/.", dir.path().display()),
                accepted.path().to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success()
    );
    let accepted_owner = cache::load(accepted.path()).unwrap().unwrap();
    let accepted_candidate = git::revision(accepted.path(), "HEAD").unwrap();
    let accepted_default = git::revision(accepted.path(), "main").unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(accepted.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "evidence",
            "--session",
            "pi-b",
            "--expected-revision",
            &accepted_owner.last_plan_revision,
            "--kind",
            "verification",
            "--candidate",
            &accepted_candidate,
        ])
        .assert()
        .success();
    let accepted_owner = cache::load(accepted.path()).unwrap().unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(accepted.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "evidence",
            "--session",
            "pi-b",
            "--expected-revision",
            &accepted_owner.last_plan_revision,
            "--kind",
            "final",
            "--review-session",
            "review-final-clean",
            "--candidate",
            &accepted_candidate,
        ])
        .assert()
        .success();
    let accepted_owner = cache::load(accepted.path()).unwrap().unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(accepted.path())
        .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
        .args([
            "workflow",
            "transition",
            "--session",
            "pi-b",
            "--expected-revision",
            &accepted_owner.last_plan_revision,
            "--phase",
            "accept",
            "--transition",
            "accept",
        ])
        .assert()
        .success();
    let closure = git::revision(accepted.path(), "HEAD").unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(accepted.path())
        .args([
            "workflow",
            "integrate",
            "--session",
            "pi-b",
            "--default-branch",
            "main",
            "--expected-default",
            &accepted_default,
            "--work-branch",
            "work/001-canonical-recovery",
            "--expected-work",
            &closure,
        ])
        .assert()
        .success();
    assert!(cache::load(accepted.path()).unwrap().is_none());
    assert_eq!(git::current_branch(accepted.path()).unwrap(), "main");
    let parents = std::process::Command::new("git")
        .args(["rev-list", "--parents", "-n", "1", "HEAD"])
        .current_dir(accepted.path())
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&parents.stdout)
            .split_whitespace()
            .count(),
        3
    );
    assert!(
        std::process::Command::new("git")
            .args([
                "cat-file",
                "-e",
                "main:knowledge/plans/done/plan-042-canonical-recovery.md",
            ])
            .current_dir(accepted.path())
            .status()
            .unwrap()
            .success()
    );

    // The first finding schedules a correction without consuming the budget. Each
    // subsequent complete valid review consumes the preceding correction cycle.
    for cycle in 1..=4 {
        let owner = cache::load(dir.path()).unwrap().unwrap();
        let candidate = git::revision(dir.path(), "HEAD").unwrap();
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
                &owner.last_plan_revision,
                "--kind",
                "verification",
                "--candidate",
                &candidate,
            ])
            .assert()
            .success();
        let verified = cache::load(dir.path()).unwrap().unwrap();
        Command::cargo_bin("superdev")
            .unwrap()
            .current_dir(dir.path())
            .env("SUPERDEV_UI_AUTHORITY", "fedcba9876543210fedcba9876543210")
            .args([
                "workflow",
                "correction",
                "--session",
                "pi-b",
                "--expected-revision",
                &verified.last_plan_revision,
                "--candidate",
                &candidate,
                "--review-session",
                &format!("review-final-{cycle}"),
                "--summary",
                "One bounded review finding",
            ])
            .assert()
            .success();
        assert!(
            fs::read_to_string(&plan)
                .unwrap()
                .contains(&format!("Final corrections: {}.", cycle - 1))
        );
        if cycle <= 3 {
            let correction_owner = cache::load(dir.path()).unwrap().unwrap();
            if cycle == 1 {
                let pending_candidate = git::revision(dir.path(), "HEAD").unwrap();
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
                        &correction_owner.last_plan_revision,
                        "--kind",
                        "verification",
                        "--candidate",
                        &pending_candidate,
                    ])
                    .assert()
                    .failure();
                let before_refusal = git::revision(dir.path(), "HEAD").unwrap();
                fs::write(dir.path().join("outside"), "not correction scoped\n").unwrap();
                Command::cargo_bin("superdev")
                    .unwrap()
                    .current_dir(dir.path())
                    .args([
                        "workflow",
                        "correction-checkpoint",
                        "--session",
                        "pi-b",
                        "--expected-revision",
                        &correction_owner.last_plan_revision,
                    ])
                    .assert()
                    .failure();
                assert_eq!(git::revision(dir.path(), "HEAD").unwrap(), before_refusal);
                assert_eq!(
                    cache::load(dir.path()).unwrap().unwrap().last_plan_revision,
                    correction_owner.last_plan_revision
                );
                fs::remove_file(dir.path().join("outside")).unwrap();
            }
            fs::write(
                dir.path().join("src/lib.rs"),
                format!("pub fn checkpointed() {{ /* correction {cycle} */ }}\n"),
            )
            .unwrap();
            Command::cargo_bin("superdev")
                .unwrap()
                .current_dir(dir.path())
                .args([
                    "workflow",
                    "correction-checkpoint",
                    "--session",
                    "pi-b",
                    "--expected-revision",
                    &correction_owner.last_plan_revision,
                ])
                .assert()
                .success();
            git::require_clean(dir.path()).unwrap();
            let checkpointed_owner = cache::load(dir.path()).unwrap().unwrap();
            assert_ne!(
                checkpointed_owner.last_plan_revision,
                correction_owner.last_plan_revision
            );
            assert!(
                fs::read_to_string(&plan)
                    .unwrap()
                    .contains("Blocker: final correction awaiting review.")
            );
        }
    }
    let exhausted = fs::read_to_string(&plan).unwrap();
    assert!(exhausted.contains("Final corrections: 3."));
    assert!(exhausted.contains("Blocker: final correction limit exhausted"));
    let owner = cache::load(dir.path()).unwrap().unwrap();
    assert!(owner.candidate_revision.is_none());
    assert!(owner.verified_default_revision.is_none());
    let candidate = git::revision(dir.path(), "HEAD").unwrap();
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
            &owner.last_plan_revision,
            "--kind",
            "verification",
            "--candidate",
            &candidate,
        ])
        .assert()
        .failure();

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
    let returned_owner = cache::load(dir.path()).unwrap().unwrap();
    assert!(returned_owner.candidate_revision.is_none());
    assert!(returned_owner.verified_default_revision.is_none());
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
    fs::write(
        dir.path()
            .join("knowledge/issues/open/issue-001-canonical-recovery.md"),
        issue.replace("- [ ] BUILD discovery:", "- [x] BUILD discovery:"),
    )
    .unwrap();

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
        .args([
            "workflow",
            "scope-checkpoint",
            "--session",
            "pi-b",
            "--expected-revision",
            &expected,
        ])
        .assert()
        .success();
    let expected = cache::load(dir.path()).unwrap().unwrap().last_plan_revision;
    let scope_candidate = git::revision(dir.path(), "HEAD").unwrap();
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
            "--candidate",
            &scope_candidate,
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
    let approved = fs::read_to_string(&plan).unwrap();
    assert!(approved.contains("phase: build"));
    assert!(approved.contains("Final corrections: 0."));
    assert!(approved.contains("Scope product baseline:"));
    assert!(approved.contains("Scope requirements review: clean"));
    assert!(approved.contains("Human scope approval: approved"));

    let owner = cache::load(dir.path()).unwrap().unwrap();
    let candidate = git::revision(dir.path(), "HEAD").unwrap();
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
            &owner.last_plan_revision,
            "--kind",
            "verification",
            "--candidate",
            &candidate,
        ])
        .assert()
        .success();
    let verified = cache::load(dir.path()).unwrap().unwrap();
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
            &verified.last_plan_revision,
            "--kind",
            "final",
            "--review-session",
            "review-clean-after-exhausted-rescope",
            "--candidate",
            &candidate,
        ])
        .assert()
        .success();
    let accepted = fs::read_to_string(&plan).unwrap();
    assert!(accepted.contains("phase: accept"));
    assert!(accepted.contains("Final corrections: 0."));
    assert!(accepted.contains("Scope product baseline:"));
    assert!(accepted.contains("Scope requirements review: clean"));
    assert!(accepted.contains("Human scope approval: approved"));

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
            "accept",
            "--transition",
            "return-to-build",
            "--feedback",
            "One within-scope ACCEPT defect",
        ])
        .assert()
        .success();
    assert!(
        fs::read_to_string(&plan)
            .unwrap()
            .contains("Blocker: final correction pending: One within-scope ACCEPT defect.")
    );
    let owner = cache::load(dir.path()).unwrap().unwrap();
    fs::write(
        dir.path().join("src/lib.rs"),
        "pub fn checkpointed() { /* accept correction */ }\n",
    )
    .unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "workflow",
            "correction-checkpoint",
            "--session",
            "pi-b",
            "--expected-revision",
            &owner.last_plan_revision,
        ])
        .assert()
        .success();
    let owner = cache::load(dir.path()).unwrap().unwrap();
    let candidate = git::revision(dir.path(), "HEAD").unwrap();
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
            &owner.last_plan_revision,
            "--kind",
            "verification",
            "--candidate",
            &candidate,
        ])
        .assert()
        .success();
    let owner = cache::load(dir.path()).unwrap().unwrap();
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
            &owner.last_plan_revision,
            "--kind",
            "final",
            "--review-session",
            "review-clean-after-accept-correction",
            "--candidate",
            &candidate,
        ])
        .assert()
        .success();
    let corrected = fs::read_to_string(&plan).unwrap();
    assert!(corrected.contains("Final corrections: 1."));
    assert!(corrected.contains("Blocker: none."));

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
            "accept",
            "--transition",
            "reject-acceptance",
            "--feedback",
            "Exercise cleanup after the clean exhausted-budget attestation.",
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
            "abandon",
            "--session",
            "pi-b",
            "--expected-revision",
            &revision,
            "--phase",
            "scope",
            "--reason",
            "Human chose not to continue.",
        ])
        .assert()
        .success();
    assert!(cache::load(dir.path()).unwrap().is_none());
    let default_plan = std::process::Command::new("git")
        .args([
            "show",
            "main:knowledge/plans/abandoned/plan-042-canonical-recovery.md",
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
