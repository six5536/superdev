//! Local workflow service. All mutations share the existing checkout lock.

use std::path::Path;

use serde::Serialize;

use super::{
    GateEvidence, Phase, Transition, cache,
    claim::{self, Claim, Controller},
    documents, git,
    local::*,
    request::{DocumentKind, LocalRequest, ScopeChange},
    store::{Store, refusal},
    transition::apply_transition,
};
use crate::error::Result;

mod changes;
#[cfg(test)]
mod execution_tests;
mod publication;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod tests;

pub use super::WORKFLOW_PROTOCOL as LOCAL_WORKFLOW_PROTOCOL;

/// Observed approval usability, including explicit suspension diagnostics.
#[derive(Debug, Serialize)]
pub struct ApprovalStatus {
    /// Both approvals cover the current canonical bytes and linked issue.
    pub executable: bool,
    /// Missing or changed evidence; not silently treated as valid.
    pub reason: Option<String>,
}

/// A consistent local record and its current document validity.
#[derive(Debug, Serialize)]
pub struct LocalStatus {
    /// Durable progress and approval receipts.
    pub record: WorkflowRecord,
    /// Current validity, not a phase inferred from plan prose.
    pub approval: ApprovalStatus,
}

/// Observational status never acquires writer ownership.
pub fn status(root: &Path) -> Result<(Vec<LocalStatus>, Option<Claim>)> {
    let root = git::repository_root(root)?;
    let root = root.as_path();
    cache::transaction(root, |_| {
        let records = Store::open(root)?
            .list()?
            .into_iter()
            .map(|record| {
                let result = documents::approved_scope(root, &record);
                LocalStatus {
                    record,
                    approval: ApprovalStatus {
                        executable: result.is_ok(),
                        reason: result.err().map(|error| error.to_string()),
                    },
                }
            })
            .collect();
        Ok((records, claim::load(root)?))
    })
}

/// Apply one typed controller request. The capability is not a model tool argument.
/// Unknown local state and stale generations fail before a transition is applied.
pub fn apply(
    root: &Path,
    request: &LocalRequest,
    controller: &Controller<'_>,
) -> Result<WorkflowRecord> {
    let root = git::repository_root(root)?;
    let root = root.as_path();
    cache::authority_digest(controller.capability)?;
    git::require_local_state_ignored(root)?;
    cache::transaction(root, |transaction| {
        let store = Store::open(root)?;
        if !matches!(request, LocalRequest::Migrate { .. }) && transaction.load()?.is_some() {
            return Err(refusal(
                "legacy ownership exists; stop its writers and migrate before using local records",
            ));
        }
        match request {
            LocalRequest::Create {
                issue,
                default_branch,
            } => {
                let record = new_record(root, &store, issue, default_branch.as_deref())?;
                require_branch(root, &record)?;
                // Check the incumbent before reserving a document identity.
                // A failed claim write leaves discoverable progress, never an
                // orphaned claim that names a missing workflow.
                claim::available(root, &record.id, controller)?;
                store.create(&record)?;
                claim::acquire(root, &record.id, controller)?;
                Ok(record)
            }
            LocalRequest::Migrate {
                issue,
                plan,
                default_branch,
                checkpoint,
            } => {
                // Back up exact legacy bytes before conversion. Re-running after
                // any failure never interprets legacy phase/prose as approval.
                transaction.archive_legacy()?;
                if let Some(record) = store
                    .list()?
                    .into_iter()
                    .find(|record| record.issue == *issue && record.plan.as_ref() == Some(plan))
                {
                    claim::acquire(root, &record.id, controller)?;
                    return Ok(record);
                }
                let mut record = new_record(root, &store, issue, default_branch.as_deref())?;
                documents::reserve(root, &store, DocumentKind::Plan, plan, None)?;
                record.plan = Some(plan.clone());
                let current = git::current_branch(root)?;
                let expected = branch_for_issue(issue);
                if current == expected {
                    record.work_branch = Some(current);
                }
                require_branch(root, &record)?;
                documents::document(root, &record, DocumentKind::Issue)?;
                documents::document(root, &record, DocumentKind::Plan)?;
                record.checkpoint = checkpoint.clone();
                record.recovery = Some("Legacy progress is an inspection hint, not approval. Inspect partial files and uncertain commands; request fresh document approval before execution.".into());
                record.scope_step = ScopeStep::ApproveIssue;
                claim::available(root, &record.id, controller)?;
                store.create(&record)?;
                claim::acquire(root, &record.id, controller)?;
                Ok(record)
            }
            LocalRequest::Resume { id } => {
                let record = required_record(&store, id)?;
                claim::acquire(root, id, controller)?;
                // No phase or step changes. Unused conversational permissions
                // are deliberately absent from the durable record.
                Ok(record)
            }
            LocalRequest::Pause { id } => {
                let record = required_record(&store, id)?;
                claim::release(root, id, controller)?;
                Ok(record)
            }
            LocalRequest::Change {
                id,
                expected_revision,
                change,
            } => {
                let mut record = required_record(&store, id)?;
                if record.revision != *expected_revision {
                    return Err(refusal(
                        "workflow revision changed; reload before continuing",
                    ));
                }
                // Execution operations belong to an attached worker's own
                // stages, so requiring a stopped worker would mean stopping it
                // between every block. The controller issues these only at a
                // settled boundary, never during a stage turn, and it is the
                // only holder of the capability these operations require.
                //
                // SCOPE mutations keep the stricter rule: a running worker owns
                // the checkout, so the controller must not author beside it.
                if matches!(
                    change,
                    ScopeChange::AttachWorker { .. }
                        | ScopeChange::DetachWorker
                        | ScopeChange::RecordProgress { .. }
                        | ScopeChange::ConsumeRetry { .. }
                        | ScopeChange::CommitBlock { .. }
                        | ScopeChange::CompleteBuild
                        | ScopeChange::ReturnToBuild { .. }
                        | ScopeChange::Accept { .. }
                        | ScopeChange::Abandon { .. }
                ) {
                    claim::verify_active(root, id, controller)?;
                } else {
                    claim::verify(root, id, controller)?;
                }
                if record.pending_publication.is_some()
                    && !matches!(change, ScopeChange::RecoverPublication)
                {
                    return Err(refusal(
                        "recover the pending publication before another mutation",
                    ));
                }
                if record.pending_build_start.is_some()
                    && !matches!(change, ScopeChange::RecoverBuildStart)
                {
                    return Err(refusal(
                        "recover the pending BUILD startup before another mutation",
                    ));
                }
                changes::apply(root, &store, &mut record, change, controller)?;
                Ok(record)
            }
        }
    })
}

fn new_record(
    root: &Path,
    store: &Store,
    issue: &str,
    default: Option<&str>,
) -> Result<WorkflowRecord> {
    documents::reserve(root, store, DocumentKind::Issue, issue, None)?;
    Ok(WorkflowRecord {
        version: RECORD_VERSION,
        id: store.new_id(),
        revision: 0,
        checkout: store.checkout().into(),
        default_branch: git::default_branch(root, default)?,
        work_branch: None,
        phase: Phase::Scope,
        scope_step: ScopeStep::SelectIssue,
        steps: Vec::new(),
        discussion: None,
        issue: issue.into(),
        plan: None,
        issue_approval: None,
        plan_approval: None,
        pending_publication: None,
        pending_build_start: None,
        execution_mode: None,
        worker_session: None,
        stage: None,
        checkpoint: Checkpoint::default(),
        retries: Default::default(),
        candidate: None,
        assessment: None,
        recovery: None,
    })
}

fn required_record(store: &Store, id: &str) -> Result<WorkflowRecord> {
    store.load(id)?.ok_or_else(|| refusal("local workflow is missing; reconstruct with fresh human approval, not plan prose or Git history"))
}

fn require_branch(root: &Path, record: &WorkflowRecord) -> Result<()> {
    let expected = record
        .work_branch
        .as_ref()
        .unwrap_or(&record.default_branch);
    if git::current_branch(root)? != *expected {
        return Err(refusal(format!(
            "switch explicitly to `{expected}` before authoring or publishing this SCOPE"
        )));
    }
    Ok(())
}

fn require_scope(record: &WorkflowRecord) -> Result<()> {
    if record.phase != Phase::Scope {
        return Err(refusal("this action requires human-led SCOPE"));
    }
    Ok(())
}

fn branch_for_issue(issue: &str) -> String {
    format!(
        "work/{}",
        issue
            .strip_prefix("issue-")
            .expect("validated issue identity")
    )
}

fn verify_input(
    record: &WorkflowRecord,
    input: &HumanInput,
    controller: &Controller<'_>,
) -> Result<()> {
    if input.session != controller.session
        || input.event.trim().is_empty()
        || input.text.trim().is_empty()
    {
        return Err(refusal(
            "human input must identify the controlling session, event, and actual response",
        ));
    }
    for approval in [&record.issue_approval, &record.plan_approval]
        .into_iter()
        .flatten()
    {
        if approval.input.session == input.session && approval.input.event == input.event {
            return Err(refusal(
                "this input already approved a different action; obtain a new response",
            ));
        }
    }
    Ok(())
}
