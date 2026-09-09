//! SCOPE can review an existing valid proposal without manufacturing plan edits.
use assert_cmd::prelude::*;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use superdev_core::workflow::{cache, git};

const AUTHORITY: &str = "0123456789abcdef0123456789abcdef";
const ISSUE: &str = "knowledge/issues/open/issue-001-canonical-recovery.md";
const PLAN: &str = "knowledge/plans/open/plan-042-canonical-recovery.md";
const IDENTITY: &[&str] = &[
    "--session",
    "owner",
    "--issue",
    "issue-001-canonical-recovery",
    "--plan",
    "plan-042-canonical-recovery",
    "--work-branch",
    "work/001-canonical-recovery",
];

fn git_command(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn invoke(root: &Path, args: &[&str]) -> Output {
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .env("SUPERDEV_UI_AUTHORITY", AUTHORITY)
        .args(args)
        .output()
        .unwrap()
}

fn service(root: &Path, args: &[&str]) -> serde_json::Value {
    let output = invoke(root, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_command(root, &["init", "-q", "-b", "main"]);
    git_command(root, &["config", "user.name", "Test"]);
    git_command(root, &["config", "user.email", "test@example.com"]);
    git_command(root, &["config", "commit.gpgsign", "false"]);
    let output = invoke(root, &["init", "--no-frontend", "--no-code-index"]);
    assert!(output.status.success());
    git_command(root, &["add", "-A"]);
    git_command(root, &["commit", "-qm", "scaffold"]);
    fs::create_dir_all(root.join("knowledge/issues/open")).unwrap();
    fs::create_dir_all(root.join("knowledge/plans/open")).unwrap();
    fs::write(root.join(ISSUE), include_str!("fixtures/workflow-issue.md")).unwrap();
    fs::write(root.join(PLAN), include_str!("fixtures/workflow-plan.md")).unwrap();
    let mut args = vec!["workflow", "start"];
    args.extend_from_slice(IDENTITY);
    service(root, &args);
    // A resumed branch can already contain product work before this SCOPE attempt.
    fs::write(root.join("existing-product"), "before this attempt\n").unwrap();
    git_command(root, &["add", "existing-product"]);
    git_command(root, &["commit", "-qm", "existing product baseline"]);
    let owner = cache::load(root).unwrap().unwrap();
    service(
        root,
        &[
            "workflow",
            "scope-baseline",
            "--session",
            "owner",
            "--expected-revision",
            &owner.last_plan_revision,
        ],
    );
    dir
}

fn checkpoint(root: &Path) -> serde_json::Value {
    let owner = cache::load(root).unwrap().unwrap();
    service(
        root,
        &[
            "workflow",
            "scope-checkpoint",
            "--session",
            "owner",
            "--expected-revision",
            &owner.last_plan_revision,
        ],
    )
}

fn evidence(root: &Path, candidate: &str) -> Output {
    let owner = cache::load(root).unwrap().unwrap();
    invoke(
        root,
        &[
            "workflow",
            "evidence",
            "--session",
            "owner",
            "--expected-revision",
            &owner.last_plan_revision,
            "--revision",
            &owner.last_plan_revision,
            "--kind",
            "scope-review",
            "--review-session",
            "isolated-review",
            "--candidate",
            candidate,
        ],
    )
}

#[test]
fn unchanged_plan_checkpoints_recovers_and_records_immutable_review() {
    let dir = repository();
    let root = dir.path();
    let owner = cache::load(root).unwrap().unwrap();
    let before = git::revision(root, "HEAD").unwrap();
    let plan = fs::read(root.join(PLAN)).unwrap();
    let result = checkpoint(root);
    let candidate = result["result"]["commit"].as_str().unwrap();
    assert_ne!(candidate, before);
    assert!(
        git::changed_paths(root, &before, candidate)
            .unwrap()
            .is_empty()
    );
    assert_eq!(fs::read(root.join(PLAN)).unwrap(), plan);
    assert_eq!(cache::load(root).unwrap().unwrap(), owner);
    git::require_clean(root).unwrap();
    // A no-op checkpoint still records the latest attempt's product baseline.
    service(root, &["workflow", "cancel", "--session", "owner"]);
    let mut args = vec!["workflow", "resume"];
    args.extend_from_slice(IDENTITY);
    service(root, &args);
    assert_eq!(
        cache::load(root).unwrap().unwrap().scope_base_revision,
        Some(before.clone())
    );
    // Neither an old candidate nor arbitrary HEAD can masquerade as this review.
    assert!(!evidence(root, &before).status.success());
    let output = evidence(root, candidate);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        fs::read_to_string(root.join(PLAN))
            .unwrap()
            .contains("Scope requirements review: clean by isolated session isolated-review.")
    );
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-canonical-recovery"
    );
    git::require_clean(root).unwrap();
}

#[test]
fn issue_only_scope_changes_do_not_require_plan_edits() {
    let dir = repository();
    let root = dir.path();
    let owner = cache::load(root).unwrap().unwrap();
    let before = git::revision(root, "HEAD").unwrap();
    let issue = fs::read_to_string(root.join(ISSUE)).unwrap();
    fs::write(
        root.join(ISSUE),
        format!("{issue}\nClarify the existing requirement.\n"),
    )
    .unwrap();
    let result = checkpoint(root);
    let candidate = result["result"]["commit"].as_str().unwrap();
    assert_eq!(
        git::changed_paths(root, &before, candidate).unwrap(),
        vec![ISSUE]
    );
    assert_eq!(
        cache::load(root).unwrap().unwrap().last_plan_revision,
        owner.last_plan_revision
    );
    let output = evidence(root, candidate);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn unchanged_checkpoint_preserves_rejection_guards() {
    for case in [
        "owner",
        "revision",
        "branch",
        "untracked-product",
        "staged-product",
        "unstaged-product",
        "index-only",
        "invalid-knowledge",
        "committed-product",
    ] {
        let dir = repository();
        let root = dir.path();
        match case {
            "branch" => {
                git_command(root, &["switch", "main"]);
            }
            "untracked-product" => {
                fs::write(root.join("unrelated"), "preserve").unwrap();
            }
            "staged-product" | "unstaged-product" | "index-only" | "committed-product" => {
                fs::write(root.join("existing-product"), "changed\n").unwrap();
                if case != "unstaged-product" {
                    git_command(root, &["add", "existing-product"]);
                }
                if case == "index-only" {
                    fs::write(root.join("existing-product"), "before this attempt\n").unwrap();
                }
                if case == "committed-product" {
                    git_command(root, &["commit", "-qm", "forbidden product change"]);
                }
            }
            "invalid-knowledge" => {
                fs::write(root.join(ISSUE), "invalid concept\n").unwrap();
            }
            _ => {}
        }
        let owner = cache::load(root).unwrap().unwrap();
        let before = git::revision(root, "HEAD").unwrap();
        let status = git_command(root, &["status", "--porcelain"]);
        let index = git_command(root, &["write-tree"]);
        let output = invoke(
            root,
            &[
                "workflow",
                "scope-checkpoint",
                "--session",
                if case == "owner" { "intruder" } else { "owner" },
                "--expected-revision",
                if case == "revision" {
                    "stale"
                } else {
                    &owner.last_plan_revision
                },
            ],
        );
        assert!(!output.status.success(), "{case} was not refused");
        assert_eq!(git::revision(root, "HEAD").unwrap(), before, "{case}");
        assert_eq!(
            git_command(root, &["status", "--porcelain"]),
            status,
            "{case}"
        );
        assert_eq!(git_command(root, &["write-tree"]), index, "{case}");
        assert_eq!(cache::load(root).unwrap().unwrap(), owner, "{case}");
    }
}

#[test]
fn ordinary_commits_are_not_scope_review_checkpoints() {
    let dir = repository();
    let root = dir.path();
    let before = git::revision(root, "HEAD").unwrap();
    git_command(
        root,
        &["commit", "--allow-empty", "-qm", "ordinary snapshot"],
    );
    let candidate = git::revision(root, "HEAD").unwrap();
    assert!(!evidence(root, &candidate).status.success());
    assert_ne!(candidate, before);
    assert_eq!(git::revision(root, "HEAD").unwrap(), candidate);
}
