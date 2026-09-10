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

fn refuse(root: &Path, args: &[&str]) {
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .env("SUPERDEV_UI_AUTHORITY", AUTHORITY)
        .args(args)
        .assert()
        .failure();
}

/// One managed repository carrying the canonical issue and authored plan that
/// `workflow start` adopts.
fn managed_repository(root: &Path) {
    command(root, &["init", "-q", "-b", "main"]);
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
    fs::create_dir_all(root.join("knowledge/issues/open")).unwrap();
    fs::write(
        root.join("knowledge/issues/open/issue-001-canonical-recovery.md"),
        include_str!("fixtures/workflow-issue.md"),
    )
    .unwrap();
    command(root, &["add", "knowledge"]);
    command(root, &["commit", "-qm", "docs: file canonical recovery"]);
    fs::create_dir_all(root.join("knowledge/plans/open")).unwrap();
    fs::write(
        root.join("knowledge/plans/open/plan-042-canonical-recovery.md"),
        include_str!("fixtures/workflow-plan.md"),
    )
    .unwrap();
}

fn identity(session: &str) -> Vec<String> {
    [
        "--session",
        session,
        "--issue",
        "issue-001-canonical-recovery",
        "--plan",
        "plan-042-canonical-recovery",
        "--work-branch",
        "work/001-canonical-recovery",
    ]
    .iter()
    .map(|value| (*value).to_owned())
    .collect()
}

/// A Pi session that stops without pausing leaves a claim behind. Recovering
/// from that must not require editing `.superdev/cache/workflow.toml` by hand.
#[cfg(unix)]
#[test]
fn a_claim_from_a_stopped_session_recovers_without_manual_repair() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    managed_repository(root);

    // Stand in for the owning Pi process, which outlives the short-lived CLI.
    let mut owner = Command::new("sleep").arg("120").spawn().unwrap();
    let owner_pid = owner.id().to_string();
    let mut start = vec!["workflow".to_owned(), "start".to_owned()];
    start.extend(identity("pi-stopped"));
    start.extend(["--owner-pid".to_owned(), owner_pid]);
    let started: Vec<&str> = start.iter().map(String::as_str).collect();
    service(root, &started);

    // While that process runs, its claim holds the checkout against everyone.
    let live = service(root, &["workflow", "status", "--json"]);
    assert_eq!(live["result"]["owner"]["session_id"], "pi-stopped");
    assert!(live["result"]["abandonedOwner"].is_null());
    let mut intruder = vec!["workflow".to_owned(), "resume".to_owned()];
    intruder.extend(identity("pi-intruder"));
    let refused: Vec<&str> = intruder.iter().map(String::as_str).collect();
    refuse(root, &refused);
    refuse(root, &["workflow", "cancel", "--session", "pi-intruder"]);

    owner.kill().unwrap();
    owner.wait().unwrap();

    // The service now decides the claim is abandoned, and says so once for
    // every consumer rather than leaving each to re-derive it.
    let stopped = service(root, &["workflow", "status", "--json"]);
    assert!(
        stopped["result"]["owner"].is_null(),
        "a claim whose owner exited no longer blocks the checkout"
    );
    assert_eq!(
        stopped["result"]["abandonedOwner"]["session_id"],
        "pi-stopped"
    );
    assert_eq!(stopped["result"]["abandonedOwner"]["child_running"], false);

    // A fresh session reclaims the checkout with no manual repair, which is the
    // behaviour whose absence wedged the workflow.
    let mut resume = vec!["workflow".to_owned(), "resume".to_owned()];
    resume.extend(identity("pi-next"));
    resume.extend(["--owner-pid".to_owned(), std::process::id().to_string()]);
    let resumed: Vec<&str> = resume.iter().map(String::as_str).collect();
    service(root, &resumed);

    let recovered = cache::load(root).unwrap().unwrap();
    assert_eq!(recovered.session_id, "pi-next");
    assert_eq!(recovered.owner_pid, Some(std::process::id()));
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-canonical-recovery"
    );
    // Canonical work is untouched by any of this; only the claim moved.
    assert!(
        git::file_at_revision(
            root,
            "work/001-canonical-recovery",
            "knowledge/plans/open/plan-042-canonical-recovery.md",
        )
        .is_ok()
    );
    service(root, &["workflow", "cancel", "--session", "pi-next"]);
}

/// A claim recording no owning process cannot be proven abandoned, so it stays
/// until a human decides. That decision releases the claim they were shown.
#[test]
fn an_undecidable_claim_waits_for_the_human_who_was_shown_it() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    managed_repository(root);

    // A claim from before owner processes were recorded, or from a platform
    // that exposes none.
    let mut start = vec!["workflow".to_owned(), "start".to_owned()];
    start.extend(identity("pi-unfalsifiable"));
    let started: Vec<&str> = start.iter().map(String::as_str).collect();
    service(root, &started);
    assert_eq!(cache::load(root).unwrap().unwrap().owner_pid, None);

    // Uncertainty favours the incumbent, so the claim still holds the checkout.
    let status = service(root, &["workflow", "status", "--json"]);
    assert_eq!(status["result"]["owner"]["session_id"], "pi-unfalsifiable");
    assert!(status["result"]["abandonedOwner"].is_null());
    refuse(root, &["workflow", "cancel", "--session", "pi-other"]);

    // A human approves one claim they were shown. Naming a different claim is
    // refused rather than releasing work they never saw.
    refuse(
        root,
        &[
            "workflow",
            "cancel",
            "--session",
            "pi-other",
            "--human-release",
        ],
    );
    assert_eq!(
        cache::load(root).unwrap().unwrap().session_id,
        "pi-unfalsifiable"
    );

    service(
        root,
        &[
            "workflow",
            "cancel",
            "--session",
            "pi-unfalsifiable",
            "--human-release",
        ],
    );
    // Releasing pauses rather than destroys: the claim is gone, the workflow
    // remains open and resumable.
    assert!(cache::load(root).unwrap().is_none());
    let paused = service(root, &["workflow", "status", "--json"]);
    assert_eq!(
        paused["result"]["openWorkflows"][0]["plan"],
        "plan-042-canonical-recovery"
    );
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

/// A gate the contract assigns to the human is theirs to force. The service
/// records the decision at that gate alone, and refuses the note anywhere else.
#[test]
fn a_forced_human_gate_is_recorded_and_refused_elsewhere() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    managed_repository(root);
    let plan = root.join("knowledge/plans/open/plan-042-canonical-recovery.md");
    let revision = || cache::load(root).unwrap().unwrap().last_plan_revision;

    let mut start = vec!["workflow".to_owned(), "start".to_owned()];
    start.extend(identity("pi-force"));
    let started: Vec<&str> = start.iter().map(String::as_str).collect();
    service(root, &started);

    service(
        root,
        &[
            "workflow",
            "transition",
            "--session",
            "pi-force",
            "--expected-revision",
            &revision(),
            "--phase",
            "scope",
            "--transition",
            "approve-scope",
            "--override-note",
            "Good enough to build; 2 unanswered review findings (sub-1, mech-1)",
        ],
    );
    let approved = fs::read_to_string(&plan).unwrap();
    assert!(approved.contains("phase: build"), "the gate advanced");
    assert!(
        approved.contains(
            "Scope approval forced by the human: Good enough to build; 2 unanswered review findings (sub-1, mech-1)"
        ),
        "the forced gate left no record:\n{approved}"
    );

    // BUILD's completion edge belongs to the service, not the human, so it
    // carries no override record.
    refuse(
        root,
        &[
            "workflow",
            "transition",
            "--session",
            "pi-force",
            "--expected-revision",
            &revision(),
            "--phase",
            "build",
            "--transition",
            "complete-build",
            "--override-note",
            "forced past the code review",
        ],
    );
    let after = fs::read_to_string(&plan).unwrap();
    assert!(
        after.contains("phase: build"),
        "a refused override changed the phase"
    );
    assert!(
        !after.contains("forced past the code review"),
        "a refused override still reached the plan"
    );
    service(root, &["workflow", "cancel", "--session", "pi-force"]);
}
