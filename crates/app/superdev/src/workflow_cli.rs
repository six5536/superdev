//! Thin CLI adapter for the versioned Rust workflow service.

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use superdev_core::error::{Error, Result};
use superdev_core::manifest::Manifest;
use superdev_core::sokf::{
    EditRequest, ExactEdit, IndexDir, MutationPolicy, SokfService, parse_concept,
};
use superdev_core::workflow::cache;
use superdev_core::workflow::git;
use superdev_core::workflow::{
    GateEvidence, Phase, Transition, WORKFLOW_PROTOCOL, WorkflowCache, WorkflowIdentity,
    apply_transition,
};

// sokf:begin cli
/// Versioned workflow operations used by the Pi adapter.
#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Acquire a workflow for an issue, plan, and reserved work branch
    Start(BindArgs),
    /// Report transient ownership and canonical identity
    Status {
        /// Emit the versioned JSON protocol response
        #[arg(long)]
        json: bool,
    },
    /// Acquire or resume ownership with the same identity
    Bind(BindArgs),
    /// Apply one typed phase transition after checking supplied evidence
    Transition(TransitionArgs),
    /// Record that BUILD changed a work block
    Block(ProgressArgs),
    /// Record that BUILD changed executable evidence
    Evidence(ProgressArgs),
    /// Reconstruct and acquire ownership for a known workflow
    Resume(BindArgs),
    /// Pause by releasing transient ownership without changing plan phase
    Cancel(SessionArgs),
    /// Apply the human-only abandonment transition
    Abandon(AbandonArgs),
    /// Merge an accepted closure locally with `git merge --no-ff`
    Integrate(IntegrateArgs),
}

/// Stable workflow identity and Pi ownership arguments.
#[derive(Args)]
pub struct BindArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Primary issue ID
    #[arg(long)]
    issue: String,
    /// Implementing plan ID
    #[arg(long)]
    plan: String,
    /// Reserved work branch
    #[arg(long)]
    work_branch: String,
    /// Local default branch
    #[arg(long, default_value = "main")]
    default_branch: String,
}

/// Arguments common to compare-and-swap progress events.
#[derive(Args)]
pub struct ProgressArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// New plan content revision after the Rust-owned mutation
    #[arg(long)]
    revision: String,
}

/// Session ownership argument.
#[derive(Args)]
pub struct SessionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
}

/// Typed phase transition names.
#[derive(Clone, Copy, ValueEnum)]
pub enum TransitionName {
    /// SCOPE to BUILD
    ApproveScope,
    /// BUILD to SCOPE
    ReturnToScope,
    /// BUILD to ACCEPT
    FinishBuild,
    /// ACCEPT to SCOPE
    RejectAcceptance,
    /// ACCEPT to DONE
    Accept,
    /// Prepared DONE to BUILD after default branch drift
    RecoverStaleDefault,
}

/// Compare-and-swap transition and gate evidence.
#[derive(Args)]
pub struct TransitionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected current phase
    #[arg(long, value_enum)]
    phase: PhaseName,
    /// Enumerated transition
    #[arg(long, value_enum)]
    transition: TransitionName,
    /// Explicit human scope approval was obtained
    #[arg(long)]
    human_scope_approved: bool,
    /// Fresh isolated requirements review is clean
    #[arg(long)]
    requirements_review_clean: bool,
    /// Every work block is complete
    #[arg(long)]
    all_blocks_complete: bool,
    /// Final verification is current for the candidate
    #[arg(long)]
    final_verification_current: bool,
    /// Fresh isolated final review is clean
    #[arg(long)]
    final_review_clean: bool,
    /// Affected contracts carry no pending promises
    #[arg(long)]
    no_pending_promises: bool,
    /// Applicable documentation evidence is current
    #[arg(long)]
    documentation_current: bool,
    /// Interactive human acceptance was obtained
    #[arg(long)]
    human_acceptance_approved: bool,
}

/// CLI spelling of durable phases.
#[derive(Clone, Copy, ValueEnum)]
pub enum PhaseName {
    Scope,
    Build,
    Accept,
    Done,
    Abandoned,
}

/// Human-only abandonment request.
#[derive(Args)]
pub struct AbandonArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected current phase
    #[arg(long, value_enum)]
    phase: PhaseName,
    /// Set only by the interactive Pi command after confirmation
    #[arg(long)]
    human_approved: bool,
}

/// Compare-and-swap local integration arguments.
#[derive(Args)]
pub struct IntegrateArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Local default branch
    #[arg(long)]
    default_branch: String,
    /// Expected default branch tip
    #[arg(long)]
    expected_default: String,
    /// Accepted work branch
    #[arg(long)]
    work_branch: String,
    /// Expected closure commit
    #[arg(long)]
    expected_work: String,
}
// sokf:end cli

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Response<T: Serialize> {
    protocol: &'static str,
    operation: &'static str,
    result: T,
}

pub fn run(command: &WorkflowCommand, root: &Path) -> Result<u8> {
    let root = git::repository_root(root)?;
    match command {
        WorkflowCommand::Start(args)
        | WorkflowCommand::Bind(args)
        | WorkflowCommand::Resume(args) => {
            git::validate_work_branch(&args.work_branch)?;
            let revision = plan_revision(&root, &args.plan)?.1;
            let state = WorkflowCache {
                version: 1,
                session_id: args.session.clone(),
                identity: WorkflowIdentity {
                    issue: args.issue.clone(),
                    plan: args.plan.clone(),
                    work_branch: args.work_branch.clone(),
                    default_branch: args.default_branch.clone(),
                },
                last_plan_revision: revision,
                child_role: None,
                child_pid: None,
                child_started: None,
                cancelled: false,
            };
            cache::bind(&root, &state)?;
            emit("bind", &state)
        }
        WorkflowCommand::Status { json: _ } => emit("status", &cache::load(&root)?),
        WorkflowCommand::Cancel(args) => {
            cache::release(&root, &args.session)?;
            emit(
                "cancel",
                &serde_json::json!({"phaseChanged": false, "ownershipReleased": true}),
            )
        }
        WorkflowCommand::Block(args) | WorkflowCommand::Evidence(args) => {
            let state =
                cache::compare_and_swap(&root, &args.session, &args.expected_revision, |state| {
                    state.last_plan_revision.clone_from(&args.revision)
                })?;
            emit("progress", &state)
        }
        WorkflowCommand::Transition(args) => transition(&root, args, false),
        WorkflowCommand::Abandon(args) => {
            if !args.human_approved {
                return Err(Error::Manifest {
                    message: "abandonment requires interactive human approval".into(),
                });
            }
            transition(
                &root,
                &TransitionArgs {
                    session: args.session.clone(),
                    expected_revision: args.expected_revision.clone(),
                    phase: args.phase,
                    transition: TransitionName::Accept,
                    human_scope_approved: false,
                    requirements_review_clean: false,
                    all_blocks_complete: false,
                    final_verification_current: false,
                    final_review_clean: false,
                    no_pending_promises: false,
                    documentation_current: false,
                    human_acceptance_approved: false,
                },
                true,
            )
        }
        WorkflowCommand::Integrate(args) => {
            let state = cache::load(&root)?.ok_or_else(|| Error::Manifest {
                message: "workflow is unowned".into(),
            })?;
            if state.session_id != args.session {
                return Err(Error::Manifest {
                    message: "workflow is owned by another Pi session".into(),
                });
            }
            git::integrate_no_ff(
                &root,
                &args.default_branch,
                &args.expected_default,
                &args.work_branch,
                &args.expected_work,
            )?;
            cache::release(&root, &args.session)?;
            emit(
                "integrate",
                &serde_json::json!({"merged": true, "pushed": false, "branchDeleted": false}),
            )
        }
    }
}

fn transition(root: &Path, args: &TransitionArgs, abandon: bool) -> Result<u8> {
    let state = cache::load(root)?.ok_or_else(|| Error::Manifest {
        message: "workflow is unowned".into(),
    })?;
    if state.session_id != args.session || state.last_plan_revision != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow ownership or plan revision changed".into(),
        });
    }
    let phase = phase(args.phase);
    let transition = if abandon {
        Transition::Abandon
    } else {
        match args.transition {
            TransitionName::ApproveScope => Transition::ApproveScope,
            TransitionName::ReturnToScope => Transition::ReturnToScope,
            TransitionName::FinishBuild => Transition::FinishBuild,
            TransitionName::RejectAcceptance => Transition::RejectAcceptance,
            TransitionName::Accept => Transition::Accept,
            TransitionName::RecoverStaleDefault => Transition::RecoverStaleDefault,
        }
    };
    let gates = GateEvidence {
        human_scope_approved: args.human_scope_approved,
        requirements_review_clean: args.requirements_review_clean,
        all_blocks_complete: args.all_blocks_complete,
        final_verification_current: args.final_verification_current,
        final_review_clean: args.final_review_clean,
        no_pending_promises: args.no_pending_promises,
        documentation_current: args.documentation_current,
        human_acceptance_approved: args.human_acceptance_approved,
        human_abandonment_approved: abandon,
        closure_integrated: false,
    };
    let config = Manifest::load(root)?.workflow;
    let next =
        apply_transition(phase, transition, &gates, &config).map_err(|error| Error::Manifest {
            message: error.to_string(),
        })?;
    let (path, observed) = plan_revision(root, &state.identity.plan)?;
    if observed != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow plan revision changed".into(),
        });
    }
    let service = SokfService::new(
        root.join("knowledge"),
        root.to_path_buf(),
        IndexDir(root.join(".superdev/cache/sokf")),
        None,
    );
    let mut edits = vec![ExactEdit {
        old_text: format!("phase: {}", phase_text(phase)),
        new_text: format!("phase: {}", phase_text(next)),
    }];
    if matches!(next, Phase::Done | Phase::Abandoned) {
        edits.push(ExactEdit {
            old_text: "lifecycle: open".into(),
            new_text: format!("lifecycle: {}", phase_text(next)),
        });
    }
    service.edit(
        EditRequest {
            path: path.to_string_lossy().into_owned(),
            edits,
        },
        MutationPolicy::AgentSafe,
    )?;
    let revision = plan_revision(root, &state.identity.plan)?.1;
    let state = cache::compare_and_swap(root, &args.session, &args.expected_revision, |state| {
        state.last_plan_revision.clone_from(&revision)
    })?;
    emit(
        "transition",
        &serde_json::json!({"phase": phase_text(next), "state": state}),
    )
}

fn plan_revision(root: &Path, id: &str) -> Result<(PathBuf, String)> {
    for lifecycle in ["open", "done", "abandoned"] {
        let dir = root.join("knowledge/plans").join(lifecycle);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_stem().and_then(|name| name.to_str()) != Some(id) {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            let concept =
                parse_concept(&path.to_string_lossy(), &text).map_err(|error| Error::Sokf {
                    message: error.message,
                })?;
            return Ok((path, concept.content_hash));
        }
    }
    Err(Error::Manifest {
        message: format!("workflow plan `{id}` was not found"),
    })
}

fn phase(value: PhaseName) -> Phase {
    match value {
        PhaseName::Scope => Phase::Scope,
        PhaseName::Build => Phase::Build,
        PhaseName::Accept => Phase::Accept,
        PhaseName::Done => Phase::Done,
        PhaseName::Abandoned => Phase::Abandoned,
    }
}
fn phase_text(value: Phase) -> &'static str {
    match value {
        Phase::Scope => "scope",
        Phase::Build => "build",
        Phase::Accept => "accept",
        Phase::Done => "done",
        Phase::Abandoned => "abandoned",
    }
}

fn emit<T: Serialize>(operation: &'static str, result: &T) -> Result<u8> {
    println!(
        "{}",
        serde_json::to_string(&Response {
            protocol: WORKFLOW_PROTOCOL,
            operation,
            result
        })
        .expect("workflow response serializes")
    );
    Ok(0)
}
