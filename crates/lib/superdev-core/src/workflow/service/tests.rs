//! Behavioural coverage for the replacement local authority, not legacy plan phases.
use std::{fs, process::Command};

use super::{
    super::{
        request::ScopeChange,
        store::{self, Files},
    },
    *,
};

pub(super) const ISSUE: &str = "issue-001-local-authority";
pub(super) const PLAN: &str = "plan-042-independent-plan";
pub(super) const CAPABILITY: &str = "0123456789abcdef0123456789abcdef";

pub(super) fn controller() -> Controller<'static> {
    Controller {
        session: "controller",
        pid: std::process::id(),
        capability: CAPABILITY,
    }
}

pub(super) fn git_command(root: &Path, args: &[&str]) -> String {
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
    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub(super) fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    git_command(root, &["init", "-q", "-b", "main"]);
    git_command(root, &["config", "user.name", "Test"]);
    git_command(root, &["config", "user.email", "test@example.com"]);
    git_command(root, &["config", "commit.gpgsign", "false"]);
    fs::write(
        root.join(".gitignore"),
        ".superdev/cache/\n.superdev/workflows/\n",
    )
    .unwrap();
    fs::write(root.join("other.txt"), "initial\n").unwrap();
    git_command(root, &["add", "."]);
    git_command(root, &["commit", "-qm", "init"]);
    dir
}

pub(super) fn author(root: &Path, kind: DocumentKind, id: &str) {
    let folder = root.join(format!("knowledge/{}s/open", kind.prefix()));
    fs::create_dir_all(&folder).unwrap();
    let type_name = if kind == DocumentKind::Issue {
        "Issue"
    } else {
        "Plan"
    };
    let links = if kind == DocumentKind::Plan {
        format!("links:\n  - rel: implements\n    to: {ISSUE}\n")
    } else {
        String::new()
    };
    fs::write(folder.join(format!("{id}.md")), format!("---\ntype: {type_name}\nid: {id}\ntitle: Local authority\ndescription: Test local authority.\nlifecycle: open\n{links}---\n\n# Local authority\n\nIntent.\n")).unwrap();
}

pub(super) fn create(root: &Path) -> WorkflowRecord {
    apply(
        root,
        &LocalRequest::Create {
            issue: ISSUE.into(),
            default_branch: None,
        },
        &controller(),
    )
    .unwrap()
}

pub(super) fn change(
    root: &Path,
    record: &WorkflowRecord,
    change: ScopeChange,
) -> Result<WorkflowRecord> {
    apply(
        root,
        &LocalRequest::Change {
            id: record.id.clone(),
            expected_revision: record.revision,
            change,
        },
        &controller(),
    )
}

pub(super) fn input(event: &str) -> HumanInput {
    HumanInput {
        session: "controller".into(),
        event: event.into(),
        text: format!("Approve {event}"),
    }
}

fn approve(root: &Path, record: &WorkflowRecord, kind: DocumentKind) -> WorkflowRecord {
    let document = documents::document(root, record, kind).unwrap();
    change(
        root,
        record,
        ScopeChange::Approve {
            document: kind,
            expected_hash: document.hash,
            input: input(kind.prefix()),
            indexes: Vec::new(),
        },
    )
    .unwrap()
}

pub(super) fn ready(root: &Path) -> WorkflowRecord {
    let record = create(root);
    author(root, DocumentKind::Issue, ISSUE);
    let record = approve(root, &record, DocumentKind::Issue);
    let record = change(root, &record, ScopeChange::AttachPlan { plan: PLAN.into() }).unwrap();
    author(root, DocumentKind::Plan, PLAN);
    approve(root, &record, DocumentKind::Plan)
}

#[test]
fn issue_only_scope_survives_pause_restart_and_cache_loss() {
    let dir = repository();
    let root = dir.path();
    let record = create(root);
    assert!(record.plan.is_none());
    assert!(record.work_branch.is_none());
    let record = change(
        root,
        &record,
        ScopeChange::RecordStep {
            entry: Some(StepRecord {
                step: ScopeStep::CheckIssue,
                outcome: StepOutcome::Skipped,
                note: "Human chose direct approval".into(),
            }),
            next: ScopeStep::ApproveIssue,
            discussion: Some("Open finding".into()),
        },
    )
    .unwrap();
    apply(
        root,
        &LocalRequest::Pause {
            id: record.id.clone(),
        },
        &controller(),
    )
    .unwrap();
    fs::remove_dir_all(root.join(".superdev/cache")).unwrap();
    let resumed = apply(
        root,
        &LocalRequest::Resume {
            id: record.id.clone(),
        },
        &controller(),
    )
    .unwrap();
    assert_eq!(record, resumed);
    assert_eq!(resumed.steps[0].outcome, StepOutcome::Skipped);
    assert!(
        !serde_json::to_string(&resumed)
            .unwrap()
            .contains("permission")
    );
}

#[test]
fn approval_does_not_start_build_or_create_a_branch() {
    let dir = repository();
    let root = dir.path();
    let record = ready(root);
    assert_eq!(record.phase, Phase::Scope);
    assert_eq!(record.scope_step, ScopeStep::Handoff);
    assert_eq!(git::current_branch(root).unwrap(), "main");
    assert!(git_command(root, &["branch", "--list", "work/*"]).is_empty());
    assert!(status(root).unwrap().0[0].approval.executable);
    assert!(
        change(
            root,
            &record,
            ScopeChange::StartBuild {
                mode: ExecutionMode::Worker,
                input: input("plan")
            }
        )
        .is_err()
    );
    let built = change(
        root,
        &record,
        ScopeChange::StartBuild {
            mode: ExecutionMode::Worker,
            input: input("start"),
        },
    )
    .unwrap();
    assert_eq!(built.phase, Phase::Build);
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-local-authority"
    );
    assert!(built.pending_build_start.is_none());
    assert_eq!(
        git_command(root, &["log", "--format=%s"]).lines().count(),
        3
    );
}

#[test]
fn approval_is_document_scoped_and_preserves_unrelated_staging() {
    let dir = repository();
    let root = dir.path();
    let record = create(root);
    author(root, DocumentKind::Issue, ISSUE);
    author(root, DocumentKind::Plan, PLAN);
    fs::write(root.join("other.txt"), "unrelated staged\n").unwrap();
    git_command(root, &["add", "other.txt"]);
    fs::write(root.join("other.txt"), "unrelated unstaged\n").unwrap();
    let before_index = git_command(root, &["show", ":other.txt"]);
    let record = approve(root, &record, DocumentKind::Issue);
    assert!(record.plan_approval.is_none());
    assert_eq!(git_command(root, &["show", ":other.txt"]), before_index);
    assert_eq!(
        fs::read_to_string(root.join("other.txt")).unwrap(),
        "unrelated unstaged\n"
    );
    assert_eq!(
        git_command(
            root,
            &["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"]
        ),
        format!("knowledge/issues/open/{ISSUE}.md")
    );
    assert!(
        git_command(root, &["ls-tree", "-r", "--name-only", "HEAD"])
            .lines()
            .all(|path| !path.starts_with(".superdev/") && !path.contains(PLAN))
    );
}

#[test]
fn stale_questions_and_model_flags_never_approve() {
    let dir = repository();
    let root = dir.path();
    let record = create(root);
    author(root, DocumentKind::Issue, ISSUE);
    assert!(
        change(
            root,
            &record,
            ScopeChange::Approve {
                document: DocumentKind::Issue,
                expected_hash: "stale".into(),
                input: input("issue"),
                indexes: Vec::new()
            }
        )
        .is_err()
    );
    let request = serde_json::json!({"operation":"change", "id":record.id,"expected_revision":record.revision,
        "change":{"action":"approve","document":"issue","expected_hash":"stale","human_approved":true}});
    assert!(serde_json::from_value::<LocalRequest>(request).is_err());
    let wrong = Controller {
        capability: "another-capability-that-is-long-enough",
        ..controller()
    };
    assert!(
        apply(
            root,
            &LocalRequest::Pause {
                id: record.id.clone()
            },
            &wrong
        )
        .is_err()
    );
    let approved = approve(root, &record, DocumentKind::Issue);
    assert!(change(root, &record, ScopeChange::AttachPlan { plan: PLAN.into() }).is_err());
    assert!(approved.issue_approval.is_some());
}

#[test]
fn unknown_edits_suspend_and_assessed_edits_preserve_or_clear_approval() {
    let dir = repository();
    let root = dir.path();
    let mut record = ready(root);
    let issue = documents::document(root, &record, DocumentKind::Issue).unwrap();
    let original_input = record.issue_approval.as_ref().unwrap().input.clone();
    let original = fs::read_to_string(root.join(&issue.path)).unwrap();
    fs::write(root.join(&issue.path), format!("{original}\n")).unwrap();
    assert!(!status(root).unwrap().0[0].approval.executable);
    let updated = documents::document(root, &record, DocumentKind::Issue).unwrap();
    record = change(
        root,
        &record,
        ScopeChange::AssessChange {
            document: DocumentKind::Issue,
            from: issue.hash,
            to: updated.hash.clone(),
            formatting_only: true,
            reason: "Only a trailing empty line was added".into(),
        },
    )
    .unwrap();
    assert_eq!(
        record.issue_approval.as_ref().unwrap().input,
        original_input
    );
    assert!(status(root).unwrap().0[0].approval.executable);
    fs::write(
        root.join(&issue.path),
        original.replace("Intent.", "Different intent."),
    )
    .unwrap();
    let changed = documents::document(root, &record, DocumentKind::Issue).unwrap();
    record = change(
        root,
        &record,
        ScopeChange::AssessChange {
            document: DocumentKind::Issue,
            from: updated.hash,
            to: changed.hash,
            formatting_only: false,
            reason: "Required behaviour changed".into(),
        },
    )
    .unwrap();
    assert!(record.issue_approval.is_none() && record.plan_approval.is_none());
    assert_eq!(record.scope_step, ScopeStep::ApproveIssue);
}

#[test]
fn publication_failure_is_pending_not_approval_and_recovery_is_exact() {
    let dir = repository();
    let root = dir.path();
    let record = create(root);
    author(root, DocumentKind::Issue, ISSUE);
    let lock = root.join(".git/refs/heads/main.lock");
    fs::write(&lock, "another Git writer").unwrap();
    let document = documents::document(root, &record, DocumentKind::Issue).unwrap();
    assert!(
        change(
            root,
            &record,
            ScopeChange::Approve {
                document: DocumentKind::Issue,
                expected_hash: document.hash,
                input: input("issue"),
                indexes: Vec::new()
            }
        )
        .is_err()
    );
    let pending = store::load(root, &record.id).unwrap().unwrap();
    assert!(pending.issue_approval.is_none());
    assert!(pending.pending_publication.is_some());
    fs::remove_file(lock).unwrap();
    // Simulate the other crash boundary: Git finished but the JSON completion did not.
    let publication = pending.pending_publication.as_ref().unwrap();
    git::finish_document_publication(
        root,
        &publication.parent,
        &publication.commit,
        &publication.branch,
        &publication.paths,
    )
    .unwrap();
    assert!(
        store::load(root, &record.id)
            .unwrap()
            .unwrap()
            .issue_approval
            .is_none()
    );
    let recovered = change(root, &pending, ScopeChange::RecoverPublication).unwrap();
    assert!(recovered.issue_approval.is_some());
    assert!(change(root, &pending, ScopeChange::RecoverPublication).is_err());
}

#[test]
fn failed_record_write_does_not_publish_the_prepared_commit() {
    let dir = repository();
    let root = dir.path();
    let record = create(root);
    author(root, DocumentKind::Issue, ISSUE);
    let document = documents::document(root, &record, DocumentKind::Issue).unwrap();
    let parent = git::revision(root, "HEAD").unwrap();
    let mut response = input("issue");
    response.text = "x".repeat(4 * 1024 * 1024);
    assert!(
        change(
            root,
            &record,
            ScopeChange::Approve {
                document: DocumentKind::Issue,
                expected_hash: document.hash,
                input: response,
                indexes: Vec::new()
            }
        )
        .is_err()
    );
    assert_eq!(git::revision(root, "HEAD").unwrap(), parent);
    assert_eq!(store::load(root, &record.id).unwrap().unwrap(), record);
}

#[test]
fn missing_copied_and_unknown_version_records_cannot_transfer_authority() {
    let dir = repository();
    let root = dir.path();
    let record = ready(root);
    let clone = tempfile::tempdir().unwrap();
    git_command(clone.path(), &["clone", "-q", root.to_str().unwrap(), "."]);
    assert!(status(clone.path()).unwrap().0.is_empty());
    assert!(
        apply(
            clone.path(),
            &LocalRequest::Resume {
                id: record.id.clone()
            },
            &controller()
        )
        .is_err()
    );
    fs::create_dir_all(clone.path().join(".superdev/workflows")).unwrap();
    let path = format!(".superdev/workflows/{}.json", record.id);
    fs::copy(root.join(&path), clone.path().join(&path)).unwrap();
    assert!(store::load(clone.path(), &record.id).is_err());
    let mut unknown = record.clone();
    unknown.version = 99;
    Files::open(root, WORKFLOW_RECORDS_PATH)
        .unwrap()
        .write(&format!("{}.json", record.id), &unknown)
        .unwrap();
    assert!(
        store::load(root, &record.id)
            .unwrap_err()
            .to_string()
            .contains("unsupported")
    );
}

#[test]
fn wrong_branch_dirty_build_and_number_collisions_refuse_without_switching() {
    let dir = repository();
    let root = dir.path();
    git_command(root, &["switch", "-qc", "unrelated"]);
    assert!(
        apply(
            root,
            &LocalRequest::Create {
                issue: ISSUE.into(),
                default_branch: Some("main".into())
            },
            &controller()
        )
        .is_err()
    );
    assert!(status(root).unwrap().0.is_empty());
    git_command(root, &["switch", "-q", "main"]);
    let record = ready(root);
    fs::write(root.join("unrelated.txt"), "leave this alone").unwrap();
    assert!(
        change(
            root,
            &record,
            ScopeChange::StartBuild {
                mode: ExecutionMode::Current,
                input: input("start")
            }
        )
        .is_err()
    );
    assert_eq!(git::current_branch(root).unwrap(), "main");
    assert!(
        apply(
            root,
            &LocalRequest::Create {
                issue: "issue-001-collision".into(),
                default_branch: None
            },
            &controller()
        )
        .is_err()
    );
    assert_eq!(status(root).unwrap().0.len(), 1);
}

#[test]
fn live_owner_is_not_displaced_and_no_child_is_assumed_stopped() {
    let dir = repository();
    let root = dir.path();
    let record = create(root);
    let other = Controller {
        session: "other",
        ..controller()
    };
    let nested = root.join("nested");
    fs::create_dir(&nested).unwrap();
    assert_eq!(status(&nested).unwrap().0[0].record, record);
    assert!(
        apply(
            &nested,
            &LocalRequest::Resume {
                id: record.id.clone()
            },
            &other
        )
        .is_err()
    );
    let mut claim = claim::load(root).unwrap().unwrap();
    let (pid, started) = super::super::process::record(Some(std::process::id()));
    claim.child_pid = pid;
    claim.child_started = started;
    Files::open(root, ".superdev/cache")
        .unwrap()
        .write("workflow-claim.json", &claim)
        .unwrap();
    assert!(
        apply(
            root,
            &LocalRequest::Pause {
                id: record.id.clone()
            },
            &controller()
        )
        .is_err()
    );
    assert!(
        apply(
            root,
            &LocalRequest::Resume {
                id: record.id.clone()
            },
            &controller()
        )
        .is_err()
    );
    assert!(claim::load(root).unwrap().is_some());
}
