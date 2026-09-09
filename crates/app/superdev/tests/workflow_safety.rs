//! Regression coverage for single-checkout identity reservation and recovery.
use assert_cmd::prelude::*;
use std::{fs, path::Path, process::Command};
use superdev_core::workflow::{cache, git};

const AUTHORITY: &str = "0123456789abcdef0123456789abcdef";

fn command(root: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
}

fn service(root: &Path, args: &[&str]) -> serde_json::Value {
    let output = Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .env("SUPERDEV_UI_AUTHORITY", AUTHORITY)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn trunk_reserves_independent_plans_and_recovers_each_branch_without_cache() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    command(root, &["init", "-q", "-b", "trunk"]);
    command(root, &["config", "user.name", "Test"]);
    command(root, &["config", "user.email", "test@example.com"]);
    command(root, &["config", "commit.gpgsign", "false"]);
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    command(root, &["add", "-A"]);
    command(root, &["commit", "-qm", "init"]);
    for (number, title, slug, plan_number, kind) in [
        (
            "001",
            "Canonical recovery",
            "canonical-recovery",
            "042",
            "bug",
        ),
        ("002", "Second issue", "second-issue", "043", "chore"),
    ] {
        let issue = include_str!("fixtures/workflow-issue.md")
            .replace("001-canonical-recovery", &format!("{number}-{slug}"))
            .replace("Canonical recovery", title)
            .replace("kind: feature", &format!("kind: {kind}"))
            .replace(
                "# Feature:",
                if kind == "bug" { "# Bug:" } else { "# Chore:" },
            );
        fs::create_dir_all(root.join("knowledge/issues/open")).unwrap();
        fs::write(
            root.join(format!("knowledge/issues/open/issue-{number}-{slug}.md")),
            issue,
        )
        .unwrap();
        command(root, &["add", "knowledge"]);
        command(root, &["commit", "-qm", "docs: file issue"]);
        let issue = fs::read_to_string(
            root.join(format!("knowledge/issues/open/issue-{number}-{slug}.md")),
        )
        .unwrap();
        assert!(issue.contains(&format!("kind: {kind}")));
        let plan = include_str!("fixtures/workflow-plan.md")
            .replace("plan-042", &format!("plan-{plan_number}"))
            .replace("issue-001", &format!("issue-{number}"))
            .replace("work/001", &format!("work/{number}"))
            .replace("canonical-recovery", slug);
        let plan_id = format!("plan-{plan_number}-{slug}");
        fs::create_dir_all(root.join("knowledge/plans/open")).unwrap();
        fs::write(
            root.join(format!("knowledge/plans/open/{plan_id}.md")),
            plan,
        )
        .unwrap();
        let branch = format!("work/{number}-{slug}");
        service(
            root,
            &[
                "workflow",
                "start",
                "--session",
                "owner",
                "--issue",
                &format!("issue-{number}-{slug}"),
                "--plan",
                &plan_id,
                "--work-branch",
                &branch,
            ],
        );
        assert_eq!(git::current_branch(root).unwrap(), branch);
        assert!(
            git::file_at_revision(root, "trunk", &format!("knowledge/plans/open/{plan_id}.md"))
                .unwrap()
                .contains("Workflow default branch: trunk.")
        );
        // A checkpoint commit is refused unless the caller owns the workflow at
        // the current plan revision.
        let owner = cache::load(root).unwrap().unwrap();
        for (session, revision) in [("owner", "stale"), ("intruder", &*owner.last_plan_revision)] {
            Command::cargo_bin("superdev")
                .unwrap()
                .current_dir(root)
                .env("SUPERDEV_UI_AUTHORITY", AUTHORITY)
                .args([
                    "workflow",
                    "commit",
                    "--session",
                    session,
                    "--expected-revision",
                    revision,
                    "--message",
                    "docs(workflow): checkpoint",
                ])
                .assert()
                .failure();
        }
        let before = git::revision(root, &branch).unwrap();
        let checkpoint = service(
            root,
            &[
                "workflow",
                "commit",
                "--session",
                "owner",
                "--expected-revision",
                &owner.last_plan_revision,
                "--message",
                "docs(workflow): checkpoint",
            ],
        );
        assert_eq!(
            checkpoint["result"]["commit"],
            git::revision(root, &branch).unwrap()
        );
        assert_ne!(git::revision(root, &branch).unwrap(), before);
        // A second session cannot switch the shared checkout under the owner.
        Command::cargo_bin("superdev")
            .unwrap()
            .current_dir(root)
            .env("SUPERDEV_UI_AUTHORITY", AUTHORITY)
            .args([
                "workflow",
                "resume",
                "--session",
                "intruder",
                "--issue",
                "issue-001-canonical-recovery",
                "--plan",
                "plan-042-canonical-recovery",
                "--work-branch",
                "work/001-canonical-recovery",
            ])
            .assert()
            .failure();
        assert_eq!(git::current_branch(root).unwrap(), branch);
        service(root, &["workflow", "cancel", "--session", "owner"]);
        command(root, &["switch", "trunk"]);
    }
    let status = service(root, &["workflow", "status", "--json"]);
    assert_eq!(status["result"]["defaultBranch"], "trunk");
    assert_eq!(
        status["result"]["openWorkflows"].as_array().unwrap().len(),
        2
    );
    for workflow in status["result"]["openWorkflows"].as_array().unwrap() {
        assert_eq!(workflow["default_branch"], "trunk");
    }
    service(
        root,
        &[
            "workflow",
            "resume",
            "--session",
            "new-session",
            "--issue",
            "issue-001-canonical-recovery",
            "--plan",
            "plan-042-canonical-recovery",
            "--work-branch",
            "work/001-canonical-recovery",
        ],
    );
    assert_eq!(
        cache::load(root).unwrap().unwrap().identity.default_branch,
        "trunk"
    );
    service(root, &["workflow", "cancel", "--session", "new-session"]);
}
