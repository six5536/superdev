//! Failure boundaries and preservation of legacy recovery facts.
use std::{fs, process::Command};

use super::{
    super::{
        WorkflowCache, WorkflowIdentity, process,
        store::{self, Files},
    },
    tests::*,
    *,
};

#[test]
fn human_rescope_keeps_approvals_discussion_branch_and_execution_facts() {
    let dir = repository();
    let root = dir.path();
    let record = ready(root);
    let mut record = change(
        root,
        &record,
        ScopeChange::StartBuild {
            mode: ExecutionMode::Current,
            input: input("start"),
        },
    )
    .unwrap();
    cache::transaction(root, |_| {
        record.checkpoint.completed_blocks = vec![1];
        record.retries.insert("final-correction".into(), 2);
        record.discussion = Some("Earlier interview answer".into());
        Store::open(root)?.save(&mut record)
    })
    .unwrap();
    assert!(
        change(
            root,
            &record,
            ScopeChange::ReturnToScope {
                feedback: "New requirement".into(),
                input: input("plan"),
            }
        )
        .is_err(),
        "an approval response was reused as re-scoping permission"
    );
    let scoped = change(
        root,
        &record,
        ScopeChange::ReturnToScope {
            feedback: "New requirement".into(),
            input: input("rescope"),
        },
    )
    .unwrap();
    assert_eq!(scoped.phase, Phase::Scope);
    assert_eq!(scoped.scope_step, ScopeStep::InterviewIssue);
    assert_eq!(scoped.work_branch, record.work_branch);
    assert_eq!(scoped.checkpoint, record.checkpoint);
    assert_eq!(scoped.retries, record.retries);
    assert_eq!(scoped.issue_approval, record.issue_approval);
    assert_eq!(scoped.plan_approval, record.plan_approval);
    let discussion = scoped.discussion.unwrap();
    assert!(discussion.contains("Earlier interview answer"));
    assert!(discussion.contains("New requirement"));
    assert_eq!(
        git::current_branch(root).unwrap(),
        record.work_branch.unwrap()
    );
}

#[cfg(unix)]
fn stopped_process() -> (u32, Option<String>) {
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id();
    let started = process::start_identity(pid);
    child.wait().unwrap();
    (pid, started)
}

#[cfg(unix)]
#[test]
fn stopped_claim_recovery_preserves_approved_scope_and_retry_consumption() {
    let dir = repository();
    let root = dir.path();
    let mut record = ready(root);
    cache::transaction(root, |_| {
        record.retries.insert("final-correction".into(), 2);
        record.checkpoint.completed_blocks = vec![1, 2];
        Store::open(root)?.save(&mut record)
    })
    .unwrap();
    let mut claim = claim::load(root).unwrap().unwrap();
    let (pid, started) = stopped_process();
    claim.owner_pid = Some(pid);
    claim.owner_started = started;
    Files::open(root, ".superdev/cache")
        .unwrap()
        .write("workflow-claim.json", &claim)
        .unwrap();
    let next = Controller {
        session: "next-session",
        capability: "fedcba9876543210fedcba9876543210",
        ..controller()
    };
    let resumed = apply(
        root,
        &LocalRequest::Resume {
            id: record.id.clone(),
        },
        &next,
    )
    .unwrap();
    assert_eq!(resumed, record);
    assert!(status(root).unwrap().0[0].approval.executable);
    let pause = apply(
        root,
        &LocalRequest::Pause {
            id: record.id.clone(),
        },
        &controller(),
    );
    assert!(
        pause.is_err(),
        "the departed controller cannot release its successor"
    );
}

#[cfg(unix)]
#[test]
fn migration_is_repeatable_after_archival_without_inventing_approval() {
    let dir = repository();
    let root = dir.path();
    author(root, DocumentKind::Issue, ISSUE);
    author(root, DocumentKind::Plan, PLAN);
    let plan_path = root.join(format!("knowledge/plans/open/{PLAN}.md"));
    let original = fs::read_to_string(&plan_path).unwrap();
    let historical = original.replace("lifecycle: open", "lifecycle: open\nphase: build")
        + "\nHuman approval was recorded here in the old workflow.\n";
    fs::write(&plan_path, &historical).unwrap();
    let (pid, started) = stopped_process();
    let legacy = WorkflowCache {
        version: 1,
        session_id: "legacy".into(),
        identity: WorkflowIdentity {
            issue: ISSUE.into(),
            plan: PLAN.into(),
            work_branch: "work/001-local-authority".into(),
            default_branch: "main".into(),
        },
        last_plan_revision: "legacy-revision".into(),
        authority_digest: cache::authority_digest(CAPABILITY).unwrap(),
        owner_pid: Some(pid),
        owner_started: started,
        child_role: None,
        child_pid: None,
        child_started: None,
        cancelled: false,
    };
    cache::bind(root, &legacy).unwrap();
    let bytes = fs::read(root.join(super::super::WORKFLOW_CACHE_PATH)).unwrap();
    // The process stopped after backup/retirement but before record creation.
    cache::transaction(root, |transaction| transaction.archive_legacy()).unwrap();
    assert!(store::list(root).unwrap().is_empty());
    let request = LocalRequest::Migrate {
        issue: ISSUE.into(),
        plan: PLAN.into(),
        default_branch: None,
        checkpoint: Checkpoint {
            completed_blocks: vec![1],
            unfinished: "Inspect an interrupted command".into(),
            evidence: Vec::new(),
        },
    };
    let record = apply(root, &request, &controller()).unwrap();
    assert_eq!(record.phase, Phase::Scope);
    assert!(record.issue_approval.is_none() && record.plan_approval.is_none());
    assert_eq!(record.checkpoint.completed_blocks, [1]);
    assert_eq!(apply(root, &request, &controller()).unwrap(), record);
    assert_eq!(fs::read_to_string(plan_path).unwrap(), historical);
    let backup = fs::read_dir(root.join(".superdev/workflows/legacy"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(fs::read(backup).unwrap(), bytes);
}

#[cfg(unix)]
#[test]
fn migration_refuses_a_live_or_undecidable_legacy_writer() {
    let dir = repository();
    let root = dir.path();
    let (pid, started) = process::record(Some(std::process::id()));
    let mut legacy = WorkflowCache {
        version: 1,
        session_id: "legacy".into(),
        identity: WorkflowIdentity {
            issue: ISSUE.into(),
            plan: PLAN.into(),
            work_branch: "work/001-local-authority".into(),
            default_branch: "main".into(),
        },
        last_plan_revision: "legacy".into(),
        authority_digest: cache::authority_digest(CAPABILITY).unwrap(),
        owner_pid: pid,
        owner_started: started,
        child_role: None,
        child_pid: None,
        child_started: None,
        cancelled: false,
    };
    cache::bind(root, &legacy).unwrap();
    assert!(cache::transaction(root, |transaction| transaction.archive_legacy()).is_err());
    legacy.owner_pid = None;
    legacy.owner_started = None;
    cache::bind(root, &legacy).unwrap();
    assert!(cache::transaction(root, |transaction| transaction.archive_legacy()).is_err());
    assert!(root.join(super::super::WORKFLOW_CACHE_PATH).is_file());
    assert!(!root.join(".superdev/workflows/legacy").exists());
}

#[test]
fn startup_recovers_after_branch_creation_without_reusing_plan_approval_as_permission() {
    let dir = repository();
    let root = dir.path();
    let mut record = ready(root);
    let branch = "work/001-local-authority";
    cache::transaction(root, |_| {
        record.pending_build_start = Some(PendingBuildStart {
            branch: branch.into(),
            parent: git::revision(root, "HEAD")?,
            mode: ExecutionMode::Worker,
            input: input("start"),
        });
        Store::open(root)?.save(&mut record)
    })
    .unwrap();
    git::create_work_branch(root, branch).unwrap();
    let resumed = change(root, &record, ScopeChange::RecoverBuildStart).unwrap();
    assert_eq!(resumed.phase, Phase::Build);
    assert!(resumed.pending_build_start.is_none());
    assert!(change(root, &resumed, ScopeChange::RecoverBuildStart).is_err());
}

#[test]
fn unrelated_index_changes_and_failed_commit_leave_approval_absent() {
    let dir = repository();
    let root = dir.path();
    fs::create_dir_all(root.join("knowledge/issues")).unwrap();
    fs::write(root.join("knowledge/issues/index.md"), "# Issues\n").unwrap();
    git_command(root, &["add", "knowledge"]);
    git_command(root, &["commit", "-qm", "index"]);
    let record = create(root);
    author(root, DocumentKind::Issue, ISSUE);
    fs::write(
        root.join("knowledge/issues/index.md"),
        "# Issues\nUnrelated change.\n",
    )
    .unwrap();
    let document = documents::document(root, &record, DocumentKind::Issue).unwrap();
    let parent = git::revision(root, "HEAD").unwrap();
    assert!(
        change(
            root,
            &record,
            ScopeChange::Approve {
                document: DocumentKind::Issue,
                expected_hash: document.hash.clone(),
                input: input("issue"),
                indexes: vec!["knowledge/issues/index.md".into()]
            }
        )
        .is_err()
    );
    git_command(root, &["config", "user.name", ""]);
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
    assert_eq!(store::load(root, &record.id).unwrap().unwrap(), record);
    assert_eq!(git::revision(root, "HEAD").unwrap(), parent);
}

#[cfg(unix)]
#[test]
fn local_state_paths_do_not_follow_symlinks_outside_the_checkout() {
    let dir = repository();
    let root = dir.path();
    let outside = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.join(".superdev")).unwrap();
    std::os::unix::fs::symlink(outside.path(), root.join(".superdev/workflows")).unwrap();
    assert!(
        apply(
            root,
            &LocalRequest::Create {
                issue: ISSUE.into(),
                default_branch: None
            },
            &controller()
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
}
