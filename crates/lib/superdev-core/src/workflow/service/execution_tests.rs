//! Worker ownership, durable execution facts, and non-refilling retry budgets.
use std::process::Command;

use super::{
    super::{local::retry_budget, request::ScopeChange, store::Files},
    tests::*,
    *,
};

fn start(root: &std::path::Path, record: &WorkflowRecord) -> WorkflowRecord {
    change(
        root,
        record,
        ScopeChange::StartBuild {
            mode: ExecutionMode::Worker,
            input: input("start"),
        },
    )
    .unwrap()
}

fn started(root: &std::path::Path) -> WorkflowRecord {
    let record = ready(root);
    start(root, &record)
}

fn worker(session: &str) -> WorkerRecord {
    WorkerRecord {
        session: session.into(),
        session_file: Some("sessions/worker.jsonl".into()),
        anchor: Some("anchor-1".into()),
    }
}

#[test]
fn execution_facts_require_a_started_phase_and_a_configured_budget() {
    let dir = repository();
    let root = dir.path();
    let record = ready(root);
    for change_request in [
        ScopeChange::AttachWorker {
            pid: std::process::id(),
            worker: worker("worker-session"),
        },
        ScopeChange::DetachWorker,
        ScopeChange::RecordProgress {
            stage: ExecutionStage::Implementation,
            checkpoint: Checkpoint::default(),
            candidate: None,
            assessment: None,
        },
        ScopeChange::ConsumeRetry {
            key: "final-correction".into(),
        },
    ] {
        assert!(
            change(root, &record, change_request).is_err(),
            "an approved but unstarted workflow accepted an execution fact"
        );
    }
    let record = start(root, &record);
    assert_eq!(record.stage, Some(ExecutionStage::Implementation));
    assert!(
        change(
            root,
            &record,
            ScopeChange::ConsumeRetry {
                key: "invented-budget".into()
            }
        )
        .is_err(),
        "an unbudgeted activity was given attempts"
    );
}

#[cfg(unix)]
#[test]
fn one_worker_owns_the_checkout_and_its_session_is_reused() {
    let dir = repository();
    let root = dir.path();
    let record = started(root);
    let mut child = Command::new("sleep").arg("30").spawn().unwrap();
    let attached = change(
        root,
        &record,
        ScopeChange::AttachWorker {
            pid: child.id(),
            worker: worker("worker-session"),
        },
    )
    .unwrap();
    assert_eq!(attached.worker_session, Some(worker("worker-session")));
    assert_eq!(
        claim::load(root).unwrap().unwrap().child_pid,
        Some(child.id())
    );
    // A second worker, and any controller write, wait for the first to stop.
    assert!(
        change(
            root,
            &attached,
            ScopeChange::AttachWorker {
                pid: child.id(),
                worker: worker("another-session")
            }
        )
        .is_err()
    );
    assert!(change(root, &attached, ScopeChange::DetachWorker).is_err());
    assert!(
        change(
            root,
            &attached,
            ScopeChange::ReturnToScope {
                feedback: "New requirement".into(),
                input: input("rescope")
            }
        )
        .is_err(),
        "the controller wrote the checkout while its worker was running"
    );
    // Progress is durable while the worker runs; it is not a checkout mutation.
    let progress = change(
        root,
        &attached,
        ScopeChange::RecordProgress {
            stage: ExecutionStage::Review,
            checkpoint: Checkpoint {
                completed_blocks: vec![1, 2],
                unfinished: String::new(),
                evidence: vec!["cargo nextest: 57 passed".into()],
            },
            candidate: Some(git::revision(root, "HEAD").unwrap()),
            assessment: None,
        },
    )
    .unwrap();
    assert_eq!(progress.stage, Some(ExecutionStage::Review));
    child.kill().unwrap();
    child.wait().unwrap();
    let detached = change(root, &progress, ScopeChange::DetachWorker).unwrap();
    // The worker session survives, so a later controller resumes it rather
    // than starting a second persistent session for the same workflow.
    assert_eq!(detached.worker_session, Some(worker("worker-session")));
    assert_eq!(detached.stage, Some(ExecutionStage::Review));
    assert!(claim::load(root).unwrap().unwrap().child_pid.is_none());
}

#[test]
fn a_reset_pause_or_restart_never_refills_a_consumed_retry_budget() {
    let dir = repository();
    let root = dir.path();
    // Project policy owns the limit, so the test states one rather than
    // assuming a default the configuration could change. It is committed like
    // any project file: an untracked one would leave the worktree dirty.
    std::fs::create_dir_all(root.join(".superdev")).unwrap();
    std::fs::write(
        root.join(".superdev/config.toml"),
        "blueprint = \"0.2.0\"\n[workflow]\nhuman_acceptance_required = true\nmax_final_correction_cycles = 2\nmax_scope_review_cycles = 1\n",
    )
    .unwrap();
    git_command(root, &["add", ".superdev/config.toml"]);
    git_command(root, &["commit", "-qm", "chore: set workflow policy"]);
    let mut record = started(root);
    let config = crate::manifest::Manifest::load(root).unwrap().workflow;
    let budget = retry_budget(&config, "final-correction").unwrap();
    assert_eq!(budget, 2, "the configured limit was not the one applied");
    for _ in 0..budget {
        record = change(
            root,
            &record,
            ScopeChange::ConsumeRetry {
                key: "final-correction".into(),
            },
        )
        .unwrap();
    }
    assert_eq!(record.retries["final-correction"], budget);
    apply(
        root,
        &LocalRequest::Pause {
            id: record.id.clone(),
        },
        &controller(),
    )
    .unwrap();
    // Losing the transient cache loses ownership, never consumed attempts.
    std::fs::remove_dir_all(root.join(".superdev/cache")).unwrap();
    let resumed = apply(
        root,
        &LocalRequest::Resume {
            id: record.id.clone(),
        },
        &controller(),
    )
    .unwrap();
    assert_eq!(resumed.retries["final-correction"], budget);
    assert!(
        change(
            root,
            &resumed,
            ScopeChange::ConsumeRetry {
                key: "final-correction".into()
            }
        )
        .is_err(),
        "a restart granted another correction attempt"
    );
    // A separate budget is unaffected by an exhausted one.
    let other = change(
        root,
        &resumed,
        ScopeChange::ConsumeRetry {
            key: "scope-review".into(),
        },
    )
    .unwrap();
    assert_eq!(other.retries["scope-review"], 1);
    // That separate budget is also configured, and also finite.
    assert!(
        change(
            root,
            &other,
            ScopeChange::ConsumeRetry {
                key: "scope-review".into()
            }
        )
        .is_err(),
        "a configured limit of one granted a second attempt"
    );
}

#[cfg(unix)]
#[test]
fn a_stopped_worker_does_not_block_recovery_and_leaves_its_facts_intact() {
    let dir = repository();
    let root = dir.path();
    let record = started(root);
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id();
    let identity = super::super::process::start_identity(pid);
    child.wait().unwrap();
    // Attaching a process that is already gone is refused outright.
    assert!(
        change(
            root,
            &record,
            ScopeChange::AttachWorker {
                pid,
                worker: worker("worker-session")
            }
        )
        .is_err()
    );
    let mut claim = claim::load(root).unwrap().unwrap();
    claim.child_pid = Some(pid);
    claim.child_started = identity;
    Files::open(root, ".superdev/cache")
        .unwrap()
        .write("workflow-claim.json", &claim)
        .unwrap();
    let detached = change(root, &record, ScopeChange::DetachWorker).unwrap();
    assert_eq!(detached.checkpoint, record.checkpoint);
    assert_eq!(detached.phase, Phase::Build);
    assert!(claim::load(root).unwrap().unwrap().child_pid.is_none());
}

#[test]
fn a_block_commit_is_bounded_to_its_areas_and_is_not_silently_repeated() {
    let dir = repository();
    let root = dir.path();
    let record = started(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/lib.rs"), "pub fn one() {}\n").unwrap();
    // An unrelated edit outside the block's declared areas is refused, not absorbed.
    std::fs::write(root.join("other.txt"), "unrelated\n").unwrap();
    assert!(
        change(
            root,
            &record,
            ScopeChange::CommitBlock {
                block: 1,
                message: "feat: block one".into(),
                areas: vec!["src".into()],
            }
        )
        .is_err()
    );
    std::fs::write(root.join("other.txt"), "initial\n").unwrap();
    let committed = change(
        root,
        &record,
        ScopeChange::CommitBlock {
            block: 1,
            message: "feat: block one".into(),
            areas: vec!["src".into()],
        },
    )
    .unwrap();
    assert_eq!(committed.checkpoint.completed_blocks, vec![1]);
    assert_eq!(
        committed.candidate,
        Some(git::revision(root, "HEAD").unwrap())
    );
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-local-authority"
    );
    // Recording the same block twice would overstate progress.
    assert!(
        change(
            root,
            &committed,
            ScopeChange::CommitBlock {
                block: 1,
                message: "feat: block one again".into(),
                areas: vec!["src".into()],
            }
        )
        .is_err()
    );
}

#[test]
fn acceptance_needs_policy_authority_a_current_candidate_and_never_merges() {
    let dir = repository();
    let root = dir.path();
    let record = started(root);
    std::fs::write(root.join("src.txt"), "product\n").unwrap();
    let record = change(
        root,
        &record,
        ScopeChange::CommitBlock {
            block: 1,
            message: "feat: the only block".into(),
            areas: vec!["src.txt".into()],
        },
    )
    .unwrap();
    let default_before = git::revision(root, "main").unwrap();
    let record = change(root, &record, ScopeChange::CompleteBuild).unwrap();
    assert_eq!(record.phase, Phase::Accept);
    assert_eq!(record.stage, Some(ExecutionStage::Acceptance));
    // No manifest is present, so policy defaults to requiring a human.
    assert!(
        change(root, &record, ScopeChange::Accept { input: None }).is_err(),
        "an absent policy became silent automatic acceptance"
    );
    // A candidate that moved after its assessment is refused.
    std::fs::write(root.join("src.txt"), "late change\n").unwrap();
    git_command(root, &["commit", "-qam", "late change"]);
    assert!(
        change(
            root,
            &record,
            ScopeChange::Accept {
                input: Some(input("accept"))
            }
        )
        .is_err()
    );
    git_command(root, &["reset", "-q", "--hard", "HEAD~1"]);
    let accepted = change(
        root,
        &record,
        ScopeChange::Accept {
            input: Some(input("accept")),
        },
    )
    .unwrap();
    assert_eq!(accepted.phase, Phase::Done);
    assert_eq!(accepted.stage, None);
    assert_eq!(
        git::revision(root, "main").unwrap(),
        default_before,
        "acceptance merged into the default branch"
    );
    assert_eq!(
        git::current_branch(root).unwrap(),
        "work/001-local-authority"
    );
}

#[test]
fn accept_findings_return_to_build_and_supersede_the_candidate() {
    let dir = repository();
    let root = dir.path();
    let record = started(root);
    let record = change(root, &record, ScopeChange::CompleteBuild).unwrap();
    assert!(record.candidate.is_some());
    assert!(
        change(
            root,
            &record,
            ScopeChange::ReturnToBuild {
                feedback: "   ".into()
            }
        )
        .is_err()
    );
    let returned = change(
        root,
        &record,
        ScopeChange::ReturnToBuild {
            feedback: "Block two omits its error path".into(),
        },
    )
    .unwrap();
    assert_eq!(returned.phase, Phase::Build);
    assert_eq!(returned.stage, Some(ExecutionStage::Implementation));
    assert_eq!(
        returned.candidate, None,
        "stale acceptance evidence survived a correction"
    );
    assert!(
        returned
            .discussion
            .unwrap()
            .contains("omits its error path")
    );
    assert_eq!(returned.plan_approval, record.plan_approval);
}

/// An assessment's findings outlive the conversation that received them, and
/// say honestly whether they still describe the current candidate.
#[test]
fn assessment_findings_are_durable_and_bound_to_their_candidate() {
    let dir = repository();
    let root = dir.path();
    let record = started(root);
    let candidate = git::revision(root, "HEAD").unwrap();
    let reviewed = change(
        root,
        &record,
        ScopeChange::RecordProgress {
            stage: ExecutionStage::Review,
            checkpoint: record.checkpoint.clone(),
            candidate: Some(candidate.clone()),
            assessment: Some(AssessmentReport {
                stage: ExecutionStage::Review,
                candidate: Some(candidate.clone()),
                findings: "Block one omits its error path".into(),
                void: None,
            }),
        },
    )
    .unwrap();
    let report = reviewed.assessment.clone().unwrap();
    assert!(report.findings.contains("omits its error path"));
    assert!(report.covers(Some(&candidate)));

    // ACCEPT findings supersede the candidate. The findings survive that: they
    // are exactly what the correction needs, and losing them here would leave a
    // record that cannot tell a clean review from a lost one.
    let accepting = change(root, &reviewed, ScopeChange::CompleteBuild).unwrap();
    let returned = change(
        root,
        &accepting,
        ScopeChange::ReturnToBuild {
            feedback: "Correct the error path".into(),
        },
    )
    .unwrap();
    assert_eq!(returned.candidate, None);
    let retained = returned.assessment.clone().unwrap();
    assert!(retained.findings.contains("omits its error path"));
    assert!(
        !retained.covers(returned.candidate.as_deref()),
        "a superseded candidate still claimed its old verdict"
    );

    // Recording implementation progress does not silently discard them.
    let corrected = change(
        root,
        &returned,
        ScopeChange::RecordProgress {
            stage: ExecutionStage::Implementation,
            checkpoint: returned.checkpoint.clone(),
            candidate: None,
            assessment: None,
        },
    )
    .unwrap();
    assert_eq!(corrected.assessment, returned.assessment);

    // A newer report replaces the older one.
    let reassessed = change(
        root,
        &corrected,
        ScopeChange::RecordProgress {
            stage: ExecutionStage::Review,
            checkpoint: corrected.checkpoint.clone(),
            candidate: Some(candidate.clone()),
            assessment: Some(AssessmentReport {
                stage: ExecutionStage::Review,
                candidate: Some(candidate.clone()),
                findings: "No actionable findings".into(),
                void: None,
            }),
        },
    )
    .unwrap();
    assert_eq!(
        reassessed.assessment.unwrap().findings,
        "No actionable findings"
    );
}

/// A report has no natural length, so it is truncated visibly rather than
/// refused: losing the findings entirely would be the worse outcome.
#[test]
fn an_oversized_report_keeps_its_head_and_says_what_it_dropped() {
    let report = AssessmentReport {
        stage: ExecutionStage::Acceptance,
        candidate: None,
        findings: "f".repeat(super::super::local::MAX_ASSESSMENT_BYTES + 5_000),
        void: None,
    }
    .bounded();
    assert!(report.findings.len() < super::super::local::MAX_ASSESSMENT_BYTES + 200);
    assert!(report.findings.starts_with("fff"));
    assert!(
        report
            .findings
            .contains("5000 further bytes were not retained")
    );

    // Truncation respects character boundaries, so the report survives its own
    // round trip rather than becoming invalid JSON.
    let multibyte = AssessmentReport {
        stage: ExecutionStage::Review,
        candidate: None,
        findings: "é".repeat(super::super::local::MAX_ASSESSMENT_BYTES),
        void: None,
    }
    .bounded();
    let encoded = serde_json::to_string(&multibyte).unwrap();
    assert_eq!(
        serde_json::from_str::<AssessmentReport>(&encoded).unwrap(),
        multibyte
    );
}
