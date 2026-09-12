//! CLI regressions for local authority, publication, and recovery.
//! Legacy tests asserted early branch creation and phase/approval prose in plans.
//! Those mechanisms are intentionally removed by WORKFLOW_REDESIGN_2_PLAN.md.
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use assert_cmd::prelude::*;
use serde_json::{Value, json};
use superdev_core::workflow::{git, local::WorkflowRecord};

const AUTHORITY: &str = "0123456789abcdef0123456789abcdef";
const ISSUE: &str = "issue-001-canonical-recovery";
const PLAN: &str = "plan-042-canonical-recovery";

fn command(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn request(
    root: &Path,
    session: &str,
    authority: Option<&str>,
    value: Value,
) -> std::process::Output {
    let mut command = Command::cargo_bin("superdev").unwrap();
    command
        .current_dir(root)
        .args([
            "workflow",
            "apply",
            "--session",
            session,
            "--owner-pid",
            &std::process::id().to_string(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("SUPERDEV_UI_AUTHORITY");
    if let Some(authority) = authority {
        command.env("SUPERDEV_UI_AUTHORITY", authority);
    }
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(value.to_string().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn service(root: &Path, value: Value) -> WorkflowRecord {
    let output = request(root, "controller", Some(AUTHORITY), value.clone());
    assert!(
        output.status.success(),
        "{value}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["protocol"], "superdev-workflow/v3");
    serde_json::from_value(response["result"]["record"].clone()).unwrap()
}

fn managed_repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
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
    dir
}

fn change(record: &WorkflowRecord, action: Value) -> Value {
    json!({"operation":"change", "id":record.id, "expected_revision":record.revision, "change":action})
}

fn approve(root: &Path, record: &WorkflowRecord, kind: &str, id: &str) -> WorkflowRecord {
    let bytes = fs::read(root.join(format!("knowledge/{kind}s/open/{id}.md"))).unwrap();
    service(
        root,
        change(
            record,
            json!({"action":"approve", "document":kind,
        "expected_hash":superdev_core::lock::sha256_hex(&bytes), "indexes":[],
        "input":{"session":"controller","event":kind,"text":format!("Approve {id}")}}),
        ),
    )
}

#[test]
fn independent_document_approval_then_explicit_build_preserves_plan_bytes() {
    let dir = managed_repository();
    let root = dir.path();
    let record = service(root, json!({"operation":"create","issue":ISSUE}));
    assert!(record.plan.is_none());
    fs::create_dir_all(root.join("knowledge/issues/open")).unwrap();
    fs::write(
        root.join(format!("knowledge/issues/open/{ISSUE}.md")),
        include_str!("fixtures/workflow-issue.md"),
    )
    .unwrap();
    let record = approve(root, &record, "issue", ISSUE);
    let record = service(
        root,
        change(&record, json!({"action":"attach-plan","plan":PLAN})),
    );
    fs::create_dir_all(root.join("knowledge/plans/open")).unwrap();
    // Legacy metadata is tolerated but never used as execution authority.
    let plan = include_str!("fixtures/workflow-plan.md").replace("phase: scope", "phase: build");
    let path = root.join(format!("knowledge/plans/open/{PLAN}.md"));
    fs::write(&path, &plan).unwrap();
    let record = approve(root, &record, "plan", PLAN);
    assert_eq!(record.phase, superdev_core::workflow::Phase::Scope);
    assert!(record.work_branch.is_none());
    assert_eq!(git::current_branch(root).unwrap(), "main");
    let record = service(
        root,
        change(
            &record,
            json!({"action":"start-build","mode":"current",
        "input":{"session":"controller","event":"start","text":"Start BUILD here"}}),
        ),
    );
    assert_eq!(record.phase, superdev_core::workflow::Phase::Build);
    assert_eq!(fs::read_to_string(path).unwrap(), plan);
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-canonical-recovery"
    );
    assert!(
        !git::paths_at_revision(root, "HEAD")
            .unwrap()
            .iter()
            .any(|path| path.starts_with(".superdev/workflows/"))
    );
}

#[test]
fn unauthorised_and_legacy_commands_cannot_mutate_workflow_state() {
    let dir = managed_repository();
    let root = dir.path();
    let create = json!({"operation":"create","issue":ISSUE});
    assert!(
        !request(root, "controller", None, create.clone())
            .status
            .success()
    );
    let record = service(root, create);
    let pause = json!({"operation":"pause","id":record.id});
    assert!(
        !request(root, "intruder", Some(AUTHORITY), pause.clone())
            .status
            .success()
    );
    assert!(
        !request(
            root,
            "controller",
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            pause
        )
        .status
        .success()
    );
    for legacy in ["start", "bind", "transition", "commit", "resume", "abandon"] {
        Command::cargo_bin("superdev")
            .unwrap()
            .current_dir(root)
            .args(["workflow", legacy])
            .assert()
            .failure();
    }
    assert_eq!(git::current_branch(root).unwrap(), "main");
}

#[test]
fn pause_and_cache_loss_recover_only_the_local_record() {
    let dir = managed_repository();
    let root = dir.path();
    let record = service(root, json!({"operation":"create","issue":ISSUE}));
    service(root, json!({"operation":"pause","id":record.id}));
    fs::remove_dir_all(root.join(".superdev/cache")).unwrap();
    let resumed = service(root, json!({"operation":"resume","id":record.id}));
    assert_eq!(resumed, record);
    let clone = tempfile::tempdir().unwrap();
    command(clone.path(), &["clone", "-q", root.to_str().unwrap(), "."]);
    let output = request(
        clone.path(),
        "controller",
        Some(AUTHORITY),
        json!({"operation":"resume","id":record.id}),
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("fresh human approval"));
}

#[test]
fn malformed_input_has_no_approval_flag_fallback() {
    let dir = managed_repository();
    let root = dir.path();
    let record = service(root, json!({"operation":"create","issue":ISSUE}));
    let output = request(
        root,
        "controller",
        Some(AUTHORITY),
        change(
            &record,
            json!({
                "action":"approve", "document":"issue", "expected_hash":"copied", "human_approved":true
            }),
        ),
    );
    assert!(!output.status.success());
    let output = Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .args(["workflow", "status", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let status: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        status["result"]["workflows"][0]["approval"]["executable"],
        false
    );
}
