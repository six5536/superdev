//! Typed scope mutations and recoverable, separately authorised BUILD startup.

use super::*;

pub(super) fn apply(
    root: &Path,
    store: &Store,
    record: &mut WorkflowRecord,
    change: &ScopeChange,
    controller: &Controller<'_>,
) -> Result<()> {
    match change {
        ScopeChange::ReturnToScope { feedback, input } => {
            if !matches!(record.phase, Phase::Build | Phase::Accept) || feedback.trim().is_empty() {
                return Err(refusal(
                    "return to SCOPE requires active execution and concrete feedback",
                ));
            }
            require_branch(root, record)?;
            verify_input(record, input, controller)?;
            record.phase = Phase::Scope;
            record.scope_step = ScopeStep::InterviewIssue;
            record.discussion = Some(match &record.discussion {
                Some(previous) => format!("{previous}\n\nRe-scope feedback: {feedback}"),
                None => feedback.clone(),
            });
            record.candidate = None;
            // Unchanged document approvals remain valid. Subsequent byte changes
            // suspend them until the diff is assessed or the human approves again.
            store.save(record)
        }
        ScopeChange::AttachPlan { plan } => {
            require_scope(record)?;
            require_branch(root, record)?;
            if record
                .plan
                .as_ref()
                .is_some_and(|existing| existing != plan)
            {
                return Err(refusal(
                    "this workflow already has a plan; select a new workflow for a new identity",
                ));
            }
            documents::reserve(root, store, DocumentKind::Plan, plan, Some(&record.id))?;
            record.plan = Some(plan.clone());
            store.save(record)
        }
        ScopeChange::RecordStep {
            entry,
            next,
            discussion,
        } => {
            require_scope(record)?;
            if let Some(entry) = entry {
                record.steps.push(entry.clone());
            }
            record.scope_step = *next;
            record.discussion = discussion.clone();
            store.save(record)
        }
        ScopeChange::Approve { .. } => {
            publication::approve(root, store, record, change, controller)
        }
        ScopeChange::RecoverPublication => publication::recover(root, store, record),
        ScopeChange::RecoverBuildStart => recover_start(root, store, record),
        ScopeChange::AssessChange {
            document,
            from,
            to,
            formatting_only,
            reason,
        } => {
            require_branch(root, record)?;
            if reason.trim().is_empty() || from == to {
                return Err(refusal(
                    "diff assessment requires distinct hashes and an explanation",
                ));
            }
            let current = documents::document(root, record, *document)?;
            if current.hash != *to {
                return Err(refusal("document changed during diff assessment"));
            }
            let approval = match document {
                DocumentKind::Issue => &mut record.issue_approval,
                DocumentKind::Plan => &mut record.plan_approval,
            };
            let previous = approval
                .as_mut()
                .ok_or_else(|| refusal("there is no original approval to assess"))?;
            let baseline = DocumentRevision {
                hash: from.clone(),
                ..current
            };
            if !previous.covers(&baseline) {
                return Err(refusal(
                    "diff baseline is not covered by the original approval",
                ));
            }
            if *formatting_only {
                if !previous.compatible.contains(to) {
                    previous.compatible.push(to.clone());
                }
            } else {
                *approval = None;
                if *document == DocumentKind::Issue {
                    record.plan_approval = None;
                }
                record.phase = Phase::Scope;
                record.scope_step = match document {
                    DocumentKind::Issue => ScopeStep::ApproveIssue,
                    DocumentKind::Plan => ScopeStep::ApprovePlan,
                };
                record.candidate = None;
            }
            let note = format!("Diff {from} -> {to}: {reason}");
            record.recovery = Some(match record.recovery.take() {
                Some(existing) => format!("{existing}\n{note}"),
                None => note,
            });
            store.save(record)
        }
        ScopeChange::StartBuild { mode, input } => {
            require_scope(record)?;
            require_branch(root, record)?;
            verify_input(record, input, controller)?;
            documents::approved_scope(root, record)?;
            git::require_clean(root)?;
            if record.work_branch.is_some() {
                // Re-scoping retains the existing work branch and completed work.
                record.phase = Phase::Build;
                record.execution_mode = Some(*mode);
                record.stage = Some(ExecutionStage::Implementation);
                return store.save(record);
            }
            let branch = branch_for_issue(&record.issue);
            git::validate_work_branch(&branch)?;
            if git::local_work_branch_exists(root, &branch)? {
                return Err(refusal(
                    "BUILD branch already exists; inspect and reconstruct that workflow instead of adopting it",
                ));
            }
            record.pending_build_start = Some(PendingBuildStart {
                branch,
                parent: git::revision(root, "HEAD")?,
                mode: *mode,
                input: input.clone(),
            });
            store.save(record)?;
            recover_start(root, store, record)
        }
        ScopeChange::AttachWorker { pid, worker } => {
            require_execution(record)?;
            if record.execution_mode != Some(ExecutionMode::Worker) {
                return Err(refusal(
                    "this workflow executes in the controlling conversation; no worker may attach",
                ));
            }
            if worker.session.trim().is_empty() {
                return Err(refusal("a worker must name its own session identity"));
            }
            if let Some(existing) = &record.worker_session
                && existing.session != worker.session
            {
                return Err(refusal(
                    "this workflow already has a persistent worker session; resume it instead of replacing it",
                ));
            }
            // Ownership is recorded before the record, so a failure here never
            // leaves a stored worker that owns nothing.
            claim::attach_child(root, &record.id, controller, *pid)?;
            record.worker_session = Some(worker.clone());
            store.save(record)
        }
        ScopeChange::DetachWorker => {
            // The session reference survives, because a later controller resumes
            // that same worker session rather than starting a new one.
            claim::detach_child(root, &record.id, controller)
        }
        ScopeChange::RecordProgress {
            stage,
            checkpoint,
            candidate,
            assessment,
        } => {
            require_execution(record)?;
            if let Some(candidate) = candidate {
                git::validate_ref(candidate)?;
            }
            record.stage = Some(*stage);
            record.checkpoint = checkpoint.clone();
            record.candidate = candidate.clone();
            if let Some(report) = assessment {
                // A report names the candidate it judged, so a later candidate
                // cannot inherit its verdict. A report is replaced only by a
                // newer one: findings outlive the candidate they were made
                // against, because returning to BUILD supersedes the candidate
                // and those findings are exactly what the correction needs.
                record.assessment = Some(report.clone().bounded());
            }
            store.save(record)
        }
        ScopeChange::ConsumeRetry { key } => {
            require_execution(record)?;
            let budget = retry_budget(&workflow_config(root), key)
                .ok_or_else(|| refusal(format!("`{key}` has no configured retry budget")))?;
            let consumed = record.retries.entry(key.clone()).or_insert(0);
            if *consumed >= budget {
                return Err(refusal(format!(
                    "`{key}` has consumed its {budget} attempts; a reset or restart does not grant more"
                )));
            }
            *consumed += 1;
            store.save(record)
        }
        ScopeChange::CommitBlock {
            block,
            message,
            areas,
        } => {
            if record.phase != Phase::Build {
                return Err(refusal("only BUILD commits a work block"));
            }
            require_branch(root, record)?;
            documents::approved_scope(root, record)?;
            if record.checkpoint.completed_blocks.contains(block) {
                return Err(refusal(format!(
                    "block {block} is already recorded as complete; inspect its commit before repeating it"
                )));
            }
            // The commit is bounded to the block's declared areas, so unrelated
            // worktree changes are refused rather than absorbed.
            let commit = git::commit_block_changes(root, message, areas)?;
            record.checkpoint.completed_blocks.push(*block);
            record.checkpoint.unfinished = String::new();
            record.candidate = Some(commit);
            record.stage = Some(ExecutionStage::Implementation);
            store.save(record)
        }
        ScopeChange::CompleteBuild => {
            require_branch(root, record)?;
            documents::approved_scope(root, record)?;
            // An uncommitted change is unfinished work, not a candidate.
            git::require_clean(root)?;
            record.phase = transition(root, record, Transition::CompleteBuild, None)?;
            record.candidate = Some(git::revision(root, "HEAD")?);
            record.stage = Some(ExecutionStage::Acceptance);
            store.save(record)
        }
        ScopeChange::ReturnToBuild { feedback } => {
            if feedback.trim().is_empty() {
                return Err(refusal(
                    "returning to BUILD requires the findings to correct",
                ));
            }
            require_branch(root, record)?;
            record.phase = transition(root, record, Transition::ReturnToBuild, None)?;
            record.stage = Some(ExecutionStage::Implementation);
            // The candidate is superseded by the correction, so no stale
            // acceptance evidence survives it.
            record.candidate = None;
            record.discussion = Some(match &record.discussion {
                Some(previous) => format!("{previous}\n\nACCEPT findings: {feedback}"),
                None => format!("ACCEPT findings: {feedback}"),
            });
            store.save(record)
        }
        ScopeChange::Accept { input } => {
            require_branch(root, record)?;
            documents::approved_scope(root, record)?;
            git::require_clean(root)?;
            let candidate = record
                .candidate
                .clone()
                .ok_or_else(|| refusal("there is no candidate to accept"))?;
            if git::revision(root, "HEAD")? != candidate {
                return Err(refusal(
                    "the candidate changed after its assessment; reassess before accepting",
                ));
            }
            if let Some(input) = input {
                verify_input(record, input, controller)?;
            }
            // Acceptance closes the workflow. It never merges, pushes,
            // releases, or deletes the work branch.
            record.phase = transition(root, record, Transition::Accept, input.as_ref())?;
            record.stage = None;
            store.save(record)
        }
        ScopeChange::Abandon { reason, input } => {
            if reason.trim().is_empty() {
                return Err(refusal("abandonment requires the human's stated reason"));
            }
            verify_input(record, input, controller)?;
            // Abandonment is human-only, so the gate is set from authenticated
            // human input rather than from any model claim.
            let gates = GateEvidence {
                human_abandonment_approved: true,
                ..GateEvidence::default()
            };
            let config = workflow_config(root);
            record.phase = apply_transition(record.phase, Transition::Abandon, &gates, &config)
                .map_err(|error| refusal(error.to_string()))?;
            record.stage = None;
            // Partial product work stays on its branch. Nothing is merged,
            // pushed, or deleted, and the reason is kept with the record.
            record.candidate = None;
            record.discussion = Some(match &record.discussion {
                Some(previous) => format!("{previous}\n\nAbandoned: {reason}"),
                None => format!("Abandoned: {reason}"),
            });
            store.save(record)
        }
    }
}

/// Apply one durable phase change through the pure transition table.
///
/// Project policy alone decides whether human acceptance is required. A missing
/// or unreadable manifest is treated as requiring it, so an absent policy never
/// becomes silent automatic acceptance.
fn transition(
    root: &Path,
    record: &WorkflowRecord,
    transition: Transition,
    input: Option<&HumanInput>,
) -> Result<Phase> {
    let config = workflow_config(root);
    let gates = GateEvidence {
        human_acceptance_approved: input.is_some(),
        ..GateEvidence::default()
    };
    apply_transition(record.phase, transition, &gates, &config)
        .map_err(|error| refusal(error.to_string()))
}

/// Read project workflow policy, falling back to the safe defaults.
///
/// The defaults require human acceptance, so an unreadable manifest never
/// becomes silent automatic acceptance or an unbounded correction budget.
fn workflow_config(root: &Path) -> crate::manifest::WorkflowConfig {
    crate::manifest::Manifest::load(root)
        .map(|manifest| manifest.workflow)
        .unwrap_or_default()
}

fn require_execution(record: &WorkflowRecord) -> Result<()> {
    if !matches!(record.phase, Phase::Build | Phase::Accept) {
        return Err(refusal(
            "execution facts require a started BUILD or ACCEPT phase",
        ));
    }
    Ok(())
}

fn recover_start(root: &Path, store: &Store, record: &mut WorkflowRecord) -> Result<()> {
    let pending = record
        .pending_build_start
        .clone()
        .ok_or_else(|| refusal("no pending BUILD startup to recover"))?;
    require_scope(record)?;
    documents::approved_scope(root, record)?;
    git::require_clean(root)?;
    if git::revision(root, "HEAD")? != pending.parent {
        return Err(refusal(
            "BUILD startup candidate changed; inspect the saved startup before recovery",
        ));
    }
    let branch = git::current_branch(root)?;
    if branch == record.default_branch {
        // Never adopt an independently created branch, even if its tip happens
        // to equal the expected candidate.
        if git::local_work_branch_exists(root, &pending.branch)? {
            return Err(refusal(
                "pending BUILD branch exists but is not checked out; inspect before recovery",
            ));
        }
        git::create_work_branch(root, &pending.branch)?;
    } else if branch != pending.branch {
        return Err(refusal("pending BUILD startup is on an unrelated branch"));
    }
    record.work_branch = Some(pending.branch);
    record.execution_mode = Some(pending.mode);
    record.phase = Phase::Build;
    record.stage = Some(ExecutionStage::Implementation);
    record.pending_build_start = None;
    store.save(record)
}
