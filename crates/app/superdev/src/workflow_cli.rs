//! Thin CLI adapter for the versioned Rust workflow service.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;
use superdev_core::error::{Error, Result};
use superdev_core::manifest::Manifest;
use superdev_core::sokf::{
    EditRequest, ExactEdit, IndexDir, MutationPolicy, SokfService, parse_concept,
};
use superdev_core::workflow::cache;
use superdev_core::workflow::filing::{self, FilingKind, FilingRequest};
use superdev_core::workflow::git;
use superdev_core::workflow::retry::{self, RetryState};
use superdev_core::workflow::{
    GateEvidence, Phase, Transition, WORKFLOW_PROTOCOL, WorkflowCache, WorkflowIdentity,
    apply_transition,
};

// sokf:begin cli
/// Human-confirmed out-of-band issue or idea filing.
#[derive(Args)]
pub struct FileArgs {
    /// Record kind
    #[arg(long, value_enum, default_value = "issue")]
    kind: FilingKindName,
    /// Short human title
    #[arg(long)]
    title: String,
    /// Human description to preserve in the record
    #[arg(long)]
    description: String,
    /// Local default branch to advance
    #[arg(long, default_value = "main")]
    default_branch: String,
    /// Confirmation supplied only after the human approves the bounded diff
    #[arg(long)]
    human_approved: bool,
}

/// CLI spelling of fileable record kinds.
#[derive(Clone, Copy, ValueEnum)]
enum FilingKindName {
    Issue,
    Idea,
}

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
    /// Commit a validated BUILD block checkpoint
    Block(ProgressArgs),
    /// Record one normalized failed BUILD attempt
    Attempt(AttemptArgs),
    /// Count one failed final verification/review correction cycle
    Correction(CorrectionArgs),
    /// Record isolated review or final verification evidence canonically
    Evidence(EvidenceArgs),
    /// Incorporate the expected local default tip into BUILD
    Sync(SyncArgs),
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

/// One failed BUILD command, normalized and counted by Rust.
#[derive(Args)]
pub struct AttemptArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Failed command as executed without a shell
    #[arg(long)]
    command: String,
    /// Process exit status
    #[arg(long)]
    exit_status: i32,
    /// Bounded command diagnostics
    #[arg(long)]
    diagnostics: String,
}

/// One candidate-bound failed final gate.
#[derive(Args)]
pub struct CorrectionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Candidate whose final gate failed
    #[arg(long)]
    candidate: String,
    /// Fresh isolated reviewer run
    #[arg(long)]
    review_session: String,
    /// Bounded structured finding summary
    #[arg(long)]
    summary: String,
}

/// Rust-owned canonical evidence attestation.
#[derive(Args)]
pub struct EvidenceArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Reviewed SCOPE plan revision after isolated modifying work
    #[arg(long)]
    revision: Option<String>,
    /// Evidence gate being attested
    #[arg(long, value_enum)]
    kind: EvidenceKindName,
    /// Fresh isolated reviewer session ID
    #[arg(long)]
    review_session: Option<String>,
    /// Immutable candidate for final BUILD evidence
    #[arg(long)]
    candidate: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum EvidenceKindName {
    ScopeReview,
    Verification,
    Final,
}

/// Session ownership argument.
#[derive(Args)]
pub struct SessionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
}

/// Compare-and-swap arguments for BUILD synchronization.
#[derive(Args)]
pub struct SyncArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected local default-branch tip
    #[arg(long)]
    expected_default: String,
    /// Expected local work-branch tip
    #[arg(long)]
    expected_work: String,
}

/// Typed phase transition names.
#[derive(Clone, Copy, ValueEnum)]
pub enum TransitionName {
    /// SCOPE to BUILD
    ApproveScope,
    /// BUILD to SCOPE
    ReturnToScope,
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
    /// BUILD discovery or human rejection preserved verbatim on the primary issue
    #[arg(long)]
    feedback: Option<String>,
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
    /// Human-approved disposition recorded on the issue
    #[arg(long)]
    reason: String,
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

pub fn run_file(args: &FileArgs, root: &Path) -> Result<u8> {
    if !args.human_approved {
        return Err(Error::Manifest {
            message: "filing requires explicit human confirmation".into(),
        });
    }
    let root = git::repository_root(root)?;
    let result = filing::file(
        &root,
        &FilingRequest {
            kind: match args.kind {
                FilingKindName::Issue => FilingKind::Issue,
                FilingKindName::Idea => FilingKind::Idea,
            },
            title: args.title.clone(),
            description: args.description.clone(),
            default_branch: args.default_branch.clone(),
        },
    )?;
    emit("file", &result)
}

pub fn run(command: &WorkflowCommand, root: &Path) -> Result<u8> {
    let root = git::repository_root(root)?;
    match command {
        WorkflowCommand::Start(args) => start(&root, args),
        WorkflowCommand::Bind(args) => {
            // Bind is retained for protocol compatibility, but it never creates
            // or infers canonical records. New recovery uses `resume`.
            validate_identity(&root, args)?;
            bind(&root, args)
        }
        WorkflowCommand::Resume(args) => {
            validate_identity(&root, args)?;
            let record = plan_record(&root, &args.plan)?;
            if record.lifecycle != "open"
                || !matches!(record.phase.as_str(), "scope" | "build" | "accept")
            {
                return Err(Error::Manifest {
                    message: "resume requires one open canonical workflow".into(),
                });
            }
            if git::current_branch(&root)? != args.work_branch {
                return Err(Error::Manifest {
                    message: format!("resume requires checked-out branch `{}`", args.work_branch),
                });
            }
            bind(&root, args)
        }
        WorkflowCommand::Status { json: _ } => {
            let owner = cache::load(&root)?;
            let phase = owner
                .as_ref()
                .map(|state| plan_record(&root, &state.identity.plan).map(|record| record.phase))
                .transpose()?;
            let build_state = if phase.as_deref() == Some("build") {
                let state = owner.as_ref().expect("BUILD status has an owner");
                let (path, _) = plan_revision(&root, &state.identity.plan)?;
                let text =
                    fs::read_to_string(&path).map_err(|source| Error::Io { path, source })?;
                Some(parse_retry_state(build_state_line(&text)?)?)
            } else {
                None
            };
            let canonical_plan_revision = owner
                .as_ref()
                .map(|state| {
                    plan_revision(&root, &state.identity.plan).map(|(_, revision)| revision)
                })
                .transpose()?;
            let workflow_config = Manifest::load(&root)?.workflow;
            emit(
                "status",
                &serde_json::json!({
                    "owner": owner,
                    "phase": phase,
                    "canonicalPlanRevision": canonical_plan_revision,
                    "buildState": build_state.as_ref().map(|state| serde_json::json!({
                        "currentBlock": state.current_block,
                        "attempts": state.attempts,
                        "finalCorrections": state.final_corrections,
                        "fingerprint": state.fingerprint,
                        "blocker": state.blocker,
                    })),
                    "maxStalledBlockAttempts": workflow_config.max_stalled_block_attempts,
                    "maxFinalCorrectionCycles": workflow_config.max_final_correction_cycles,
                    "humanAcceptanceRequired": workflow_config.human_acceptance_required,
                    "openWorkflows": discover_open_workflows(&root)?,
                }),
            )
        }
        WorkflowCommand::Cancel(args) => {
            cache::release(&root, &args.session)?;
            emit(
                "cancel",
                &serde_json::json!({"phaseChanged": false, "ownershipReleased": true}),
            )
        }
        WorkflowCommand::Block(args) => cache::transaction(&root, |transaction| {
            let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
                message: "workflow is unowned".into(),
            })?;
            if owner.session_id != args.session
                || owner.last_plan_revision != args.expected_revision
            {
                return Err(Error::Manifest {
                    message: "workflow ownership or plan revision changed".into(),
                });
            }
            validate_identity_values(&root, &owner.identity)?;
            if git::current_branch(&root)? != owner.identity.work_branch {
                return Err(Error::Manifest {
                    message: "block checkpoint requires the checked-out work branch".into(),
                });
            }
            let record = plan_record(&root, &owner.identity.plan)?;
            if record.phase != "build" {
                return Err(Error::Manifest {
                    message: "block evidence may be recorded only during BUILD".into(),
                });
            }
            let (plan_path, observed) = plan_revision(&root, &owner.identity.plan)?;
            if observed != args.revision || observed == args.expected_revision {
                return Err(Error::Manifest {
                    message: "supplied revision must identify a changed canonical plan".into(),
                });
            }
            let relative_plan = plan_path
                .strip_prefix(&root)
                .map_err(|_| Error::Manifest {
                    message: "workflow plan is outside the repository".into(),
                })?
                .to_string_lossy();
            let previous_text = git::file_at_revision(&root, "HEAD", &relative_plan)?;
            let current_text = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
                path: plan_path.clone(),
                source,
            })?;
            let previous_retry = parse_retry_state(build_state_line(&previous_text)?)?;
            let supplied_retry = parse_retry_state(build_state_line(&current_text)?)?;
            if supplied_retry != previous_retry {
                return Err(Error::Manifest {
                    message: "BUILD checkpoint retry state is service-owned".into(),
                });
            }
            let block = previous_retry.current_block;
            let newly_completed =
                block_is_done(&current_text, block) && !block_is_done(&previous_text, block);
            if !newly_completed {
                return Err(Error::Manifest {
                    message: "BUILD checkpoint must newly complete the current stable block".into(),
                });
            }
            let approved_block = plan_block(&previous_text, block)?;
            let block_text = plan_block(&current_text, block)?;
            if block_dependencies(block_text) != block_dependencies(approved_block)
                || block_area_paths(block_text) != block_area_paths(approved_block)
                || plan_verification_commands(block_text)
                    != plan_verification_commands(approved_block)
            {
                return Err(Error::Manifest {
                    message: "BUILD checkpoint cannot change SCOPE-approved dependencies, Areas, or Verification commands".into(),
                });
            }
            for dependency in block_dependencies(approved_block) {
                if dependency >= block || !block_is_done(&current_text, dependency) {
                    return Err(Error::Manifest {
                        message: format!(
                            "BUILD block {block} has an incomplete or invalid dependency {dependency}"
                        ),
                    });
                }
            }
            let mut areas = block_area_paths(approved_block);
            areas.push(relative_plan.to_string());
            run_block_verification(&root, approved_block)?;
            let next_retry = retry_state_after_checkpoint(&current_text, &previous_retry);
            apply_plan_edits_transactionally(
                &root,
                &plan_path,
                vec![ExactEdit {
                    old_text: build_state_line(&current_text)?.into(),
                    new_text: render_retry_state(&next_retry),
                }],
            )?;
            let grammar = superdev_core::validate::schema::load_grammar(&root)?;
            let report = superdev_core::validate::validate_repo(
                &root,
                &root.join("knowledge"),
                &[],
                &grammar,
            )?;
            if !report.report.passed() {
                return Err(Error::Manifest {
                    message: "block checkpoint requires valid canonical knowledge".into(),
                });
            }
            git::commit_block_changes(
                &root,
                &format!("chore(workflow): checkpoint build block {block}"),
                &areas,
            )?;
            let revision = plan_revision(&root, &owner.identity.plan)?.1;
            let state =
                transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
                    state.last_plan_revision.clone_from(&revision)
                })?;
            emit("progress", &state)
        }),
        WorkflowCommand::Attempt(args) => record_attempt(&root, args),
        WorkflowCommand::Correction(args) => record_correction(&root, args),
        WorkflowCommand::Evidence(args) => record_evidence(&root, args),
        WorkflowCommand::Sync(args) => cache::transaction(&root, |transaction| {
            let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
                message: "workflow is unowned".into(),
            })?;
            if owner.session_id != args.session
                || owner.last_plan_revision != args.expected_revision
            {
                return Err(Error::Manifest {
                    message: "workflow ownership or plan revision changed".into(),
                });
            }
            validate_identity_values(&root, &owner.identity)?;
            if plan_record(&root, &owner.identity.plan)?.phase != "build" {
                return Err(Error::Manifest {
                    message: "default synchronization is permitted only during BUILD".into(),
                });
            }
            let revision = git::synchronize_default(
                &root,
                &owner.identity.default_branch,
                &args.expected_default,
                &owner.identity.work_branch,
                &args.expected_work,
            )?;
            emit(
                "synchronization",
                &serde_json::json!({"revision": revision, "state": owner}),
            )
        }),
        WorkflowCommand::Transition(args) => transition(&root, args, false, None),
        WorkflowCommand::Abandon(args) => transition(
            &root,
            &TransitionArgs {
                session: args.session.clone(),
                expected_revision: args.expected_revision.clone(),
                phase: args.phase,
                transition: TransitionName::Accept,
                feedback: None,
            },
            true,
            Some(&args.reason),
        ),
        WorkflowCommand::Integrate(args) => cache::transaction(&root, |transaction| {
            let state = transaction.load()?.ok_or_else(|| Error::Manifest {
                message: "workflow is unowned".into(),
            })?;
            if state.session_id != args.session {
                return Err(Error::Manifest {
                    message: "workflow is owned by another Pi session".into(),
                });
            }
            if args.default_branch != state.identity.default_branch
                || args.work_branch != state.identity.work_branch
            {
                return Err(Error::Manifest {
                    message: "integration refs do not match the bound workflow identity".into(),
                });
            }
            let record = plan_record_at_revision(&root, &state.identity.plan, &args.expected_work)?;
            if record.issue != state.identity.issue || record.branch != state.identity.work_branch {
                return Err(Error::Manifest {
                    message: "bound identity does not match the closure plan".into(),
                });
            }
            let issue_path = format!("knowledge/issues/done/{}.md", state.identity.issue);
            git::file_at_revision(&root, &args.expected_work, &issue_path)?;
            if record.phase != "done" || record.lifecycle != "done" {
                return Err(Error::Manifest {
                    message: "integration requires a prepared done plan".into(),
                });
            }
            let candidate = state
                .candidate_revision
                .as_deref()
                .ok_or_else(|| Error::Manifest {
                    message: "integration requires the reviewed candidate revision".into(),
                })?;
            let verified_default =
                state
                    .verified_default_revision
                    .as_deref()
                    .ok_or_else(|| Error::Manifest {
                        message: "integration requires the verified default-branch revision".into(),
                    })?;
            if args.expected_default != verified_default {
                return Err(Error::Manifest {
                    message: "expected default does not match BUILD verification".into(),
                });
            }
            if !git::is_ancestor(&root, candidate, &args.expected_work)? {
                return Err(Error::Manifest {
                    message: "closure commit does not descend from reviewed candidate H".into(),
                });
            }
            let changed = git::changed_paths(&root, candidate, &args.expected_work)?;
            if changed
                .iter()
                .any(|path| !administrative_path(path, &state.identity))
            {
                return Err(Error::Manifest {
                    message: "non-administrative changes follow reviewed candidate H".into(),
                });
            }
            git::integrate_no_ff(
                &root,
                &state.identity.default_branch,
                verified_default,
                &state.identity.work_branch,
                &args.expected_work,
            )?;
            transaction.release(&args.session)?;
            emit(
                "integrate",
                &serde_json::json!({"merged": true, "pushed": false, "branchDeleted": false}),
            )
        }),
    }
}

fn start(root: &Path, args: &BindArgs) -> Result<u8> {
    ui_authority_capability()?;
    validate_reserved_identity(args)?;
    git::require_clean(root)?;
    if git::current_branch(root)? != args.default_branch {
        return Err(Error::Manifest {
            message: format!(
                "SCOPE must start on default branch `{}`",
                args.default_branch
            ),
        });
    }
    if git::reference_exists(root, &args.work_branch)? {
        return Err(Error::Manifest {
            message: format!(
                "workflow branch `{}` already exists; use resume",
                args.work_branch
            ),
        });
    }
    if plan_revision(root, &args.plan).is_ok() {
        return Err(Error::Manifest {
            message: format!("workflow plan `{}` already exists; use resume", args.plan),
        });
    }
    let issue = open_issue_record(root, &args.issue)?;

    // Reserve ownership before mutating Git or records. This makes concurrent
    // starts serialize through the same repository lock rather than racing on
    // a prior load followed by an independent write.
    let reservation = workflow_cache(args, String::new(), None, None)?;
    cache::bind(root, &reservation)?;
    let started = (|| {
        git::create_work_branch(root, &args.work_branch)?;
        create_scope_plan(root, args, &issue)?;
        git::commit_knowledge_changes(root, "chore(workflow): start scope")?;
        let revision = plan_revision(root, &args.plan)?.1;
        let state = cache::compare_and_swap(root, &args.session, "", |state| {
            state.last_plan_revision.clone_from(&revision)
        })?;
        emit("start", &state)
    })();
    if started.is_err() {
        let _ = cache::release(root, &args.session);
    }
    started
}

#[derive(Debug)]
struct IssueRecord {
    title: String,
    description: String,
}

fn open_issue_record(root: &Path, id: &str) -> Result<IssueRecord> {
    let path = root.join("knowledge/issues/open").join(format!("{id}.md"));
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let concept = parse_concept(&path.to_string_lossy(), &text).map_err(|error| Error::Sokf {
        message: error.message,
    })?;
    if concept.id.as_deref() != Some(id) || concept.lifecycle.as_deref() != Some("open") {
        return Err(Error::Manifest {
            message: format!("`{id}` is not the matching open canonical issue"),
        });
    }
    Ok(IssueRecord {
        title: concept.raw["title"].as_str().unwrap_or(id).to_string(),
        description: concept.raw["description"]
            .as_str()
            .unwrap_or("Implement the scoped issue.")
            .to_string(),
    })
}

fn create_scope_plan(root: &Path, args: &BindArgs, issue: &IssueRecord) -> Result<()> {
    let quote = |value: &str| serde_json::to_string(value).expect("string serializes");
    let content = format!(
        "---\ntype: Plan\nid: {}\ntitle: {}\ndescription: {}\nlifecycle: open\nphase: scope\nbranch: {}\nlinks:\n  - rel: implements\n    to: {}\n---\n\n# Plan: {}\n\n## Goal and boundaries\n\nImplement [{}][sokf:{}]. {}\n\n## Requirements\n\nSCOPE must replace this initial recovery-safe draft with settled requirements before approval.\n\n## Contract changes\n\n- none.\n\n## ADR decisions\n\n- none.\n\n## Source and interface changes\n\nSCOPE must identify the exact source declarations and materialized interfaces.\n\n## Knowledge changes\n\nSCOPE must identify normative and current-state knowledge changes.\n\n## Documentation changes\n\nSCOPE must map applicable surfaces from the canonical documentation map.\n\n## Work blocks\n\n### Block 1: Deliver the approved scope\n\n- [ ] Done.\n- Dependencies: none.\n- Areas: to be settled by SCOPE.\n- Outcome: the approved issue is implemented and verified.\n- Verification: executable commands must be settled by SCOPE.\n- Tests: executable contract evidence must be settled by SCOPE.\n- Structural evidence: executable structural evidence must be settled by SCOPE.\n- Documentation: applicable surfaces and commands must be settled by SCOPE.\n\n## Build state\n\nCurrent block: 1. Attempts: 0. Final corrections: 0. Blocker: scope approval pending.\n\n## Implementation decisions\n\nnone.\n\n## Follow-up issues\n\nnone.\n\n## Completion evidence\n\nScope requirements review and human approval are pending.\n",
        args.plan,
        quote(&issue.title),
        quote(&issue.description),
        args.work_branch,
        args.issue,
        issue.title,
        issue.title,
        args.issue,
        issue.description,
    );
    let path = root
        .join("knowledge/plans/open")
        .join(format!("{}.md", args.plan));
    fs::create_dir_all(path.parent().expect("plan path has a parent")).map_err(|source| {
        Error::Io {
            path: path.parent().expect("plan path has a parent").to_path_buf(),
            source,
        }
    })?;
    fs::write(&path, content).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    superdev_core::validate::fix_repo(root, &root.join("knowledge"), &[])?;
    let grammar = superdev_core::validate::schema::load_grammar(root)?;
    let report =
        superdev_core::validate::validate_repo(root, &root.join("knowledge"), &[path], &grammar)?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: format!(
                "initial scope plan did not validate:\n{}",
                report
                    .report
                    .render_human(superdev_core::validate::sokf::Warnings::Listed)
            ),
        });
    }
    Ok(())
}

fn discover_open_workflows(root: &Path) -> Result<Vec<WorkflowIdentity>> {
    let directory = root.join("knowledge/plans/open");
    let Ok(entries) = fs::read_dir(&directory) else {
        return Ok(Vec::new());
    };
    let mut workflows = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|source| Error::Io {
                path: directory.clone(),
                source,
            })?
            .path();
        let Some(plan) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        let record = plan_record(root, plan)?;
        if record.lifecycle == "open"
            && matches!(record.phase.as_str(), "scope" | "build" | "accept")
        {
            workflows.push(WorkflowIdentity {
                issue: record.issue,
                plan: plan.to_string(),
                work_branch: record.branch,
                default_branch: "main".into(),
            });
        }
    }
    workflows.sort_by(|left, right| left.plan.cmp(&right.plan));
    Ok(workflows)
}

fn bind(root: &Path, args: &BindArgs) -> Result<u8> {
    let revision = plan_revision(root, &args.plan)?.1;
    bind_with_revision(root, args, revision)
}

fn bind_with_revision(root: &Path, args: &BindArgs, revision: String) -> Result<u8> {
    let text =
        fs::read_to_string(plan_revision(root, &args.plan)?.0).map_err(|source| Error::Io {
            path: plan_revision(root, &args.plan)
                .expect("plan was just resolved")
                .0,
            source,
        })?;
    let candidate = evidence_revision(&text, "Candidate revision");
    let verified_default = evidence_revision(&text, "Verified default revision");
    let state = workflow_cache(args, revision, candidate, verified_default)?;
    cache::bind(root, &state)?;
    emit("bind", &state)
}

fn workflow_cache(
    args: &BindArgs,
    revision: String,
    candidate_revision: Option<String>,
    verified_default_revision: Option<String>,
) -> Result<WorkflowCache> {
    let capability = ui_authority_capability()?;
    Ok(WorkflowCache {
        version: 1,
        session_id: args.session.clone(),
        identity: WorkflowIdentity {
            issue: args.issue.clone(),
            plan: args.plan.clone(),
            work_branch: args.work_branch.clone(),
            default_branch: args.default_branch.clone(),
        },
        last_plan_revision: revision,
        authority_digest: cache::authority_digest(&capability)?,
        candidate_revision,
        verified_default_revision,
        child_role: None,
        child_pid: None,
        child_started: None,
        cancelled: false,
    })
}

fn ui_authority_capability() -> Result<String> {
    std::env::var("SUPERDEV_UI_AUTHORITY").map_err(|_| Error::Manifest {
        message: "workflow ownership must be established by the interactive Pi UI".into(),
    })
}

#[derive(Debug)]
struct PlanRecord {
    phase: String,
    lifecycle: String,
    branch: String,
    issue: String,
}

fn plan_record(root: &Path, id: &str) -> Result<PlanRecord> {
    let (path, _) = plan_revision(root, id)?;
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    parse_plan_record(&path.to_string_lossy(), &text)
}

fn plan_record_at_revision(root: &Path, id: &str, revision: &str) -> Result<PlanRecord> {
    let path = format!("knowledge/plans/done/{id}.md");
    let text = git::file_at_revision(root, revision, &path)?;
    parse_plan_record(&path, &text)
}

fn parse_plan_record(path: &str, text: &str) -> Result<PlanRecord> {
    let concept = parse_concept(path, text).map_err(|error| Error::Sokf {
        message: error.message,
    })?;
    let issues: Vec<String> = concept
        .links
        .iter()
        .filter(|link| link.rel.as_deref() == Some("implements"))
        .filter_map(|link| link.to.clone())
        .collect();
    if issues.len() != 1 {
        return Err(Error::Manifest {
            message: "workflow plan must implement exactly one issue".into(),
        });
    }
    Ok(PlanRecord {
        phase: concept.raw["phase"].as_str().unwrap_or_default().into(),
        lifecycle: concept.lifecycle.unwrap_or_default(),
        branch: concept.raw["branch"].as_str().unwrap_or_default().into(),
        issue: issues[0].clone(),
    })
}

fn validate_reserved_identity(args: &BindArgs) -> Result<()> {
    git::validate_work_branch(&args.work_branch)?;
    git::validate_ref(&args.default_branch)?;
    let issue_tail = args
        .issue
        .strip_prefix("issue-")
        .ok_or_else(|| Error::Manifest {
            message: "workflow issue must match issue-NNN-slug".into(),
        })?;
    let plan_tail = args
        .plan
        .strip_prefix("plan-")
        .ok_or_else(|| Error::Manifest {
            message: "workflow plan must match plan-NNN-slug".into(),
        })?;
    if issue_tail != plan_tail || args.work_branch != format!("work/{issue_tail}") {
        return Err(Error::Manifest {
            message: "issue, plan, and work branch must carry one matching NNN-slug".into(),
        });
    }
    Ok(())
}

fn validate_identity(root: &Path, args: &BindArgs) -> Result<()> {
    validate_identity_values(
        root,
        &WorkflowIdentity {
            issue: args.issue.clone(),
            plan: args.plan.clone(),
            work_branch: args.work_branch.clone(),
            default_branch: args.default_branch.clone(),
        },
    )
}

fn validate_identity_values(root: &Path, identity: &WorkflowIdentity) -> Result<()> {
    git::validate_work_branch(&identity.work_branch)?;
    git::validate_ref(&identity.default_branch)?;
    let record = plan_record(root, &identity.plan)?;
    if record.issue != identity.issue || record.branch != identity.work_branch {
        return Err(Error::Manifest {
            message: "bound identity does not match the canonical plan".into(),
        });
    }
    let issue_exists = ["open", "done", "wontfix"].iter().any(|lifecycle| {
        root.join("knowledge/issues")
            .join(lifecycle)
            .join(format!("{}.md", identity.issue))
            .is_file()
    });
    if !issue_exists {
        return Err(Error::Manifest {
            message: format!("primary issue `{}` was not found", identity.issue),
        });
    }
    Ok(())
}

fn administrative_path(path: &str, identity: &WorkflowIdentity) -> bool {
    path == "knowledge/issues/index.md"
        || path == "knowledge/plans/index.md"
        || ["open", "done", "wontfix"]
            .iter()
            .any(|state| path == format!("knowledge/issues/{state}/{}.md", identity.issue))
        || ["open", "done", "abandoned"]
            .iter()
            .any(|state| path == format!("knowledge/plans/{state}/{}.md", identity.plan))
}

fn knowledge_contains(root: &Path, needle: &str) -> Result<bool> {
    let mut directories = vec![root.join("knowledge")];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).map_err(|source| Error::Io {
            path: directory.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| Error::Io {
                path: directory.clone(),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                let text = fs::read_to_string(&path).map_err(|source| Error::Io {
                    path: path.clone(),
                    source,
                })?;
                if text.contains(needle) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn record_attempt(root: &Path, args: &AttemptArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        if owner.session_id != args.session || owner.last_plan_revision != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow ownership or plan revision changed".into(),
            });
        }
        validate_identity_values(root, &owner.identity)?;
        git::require_clean(root)?;
        if git::current_branch(root)? != owner.identity.work_branch {
            return Err(Error::Manifest {
                message: "BUILD attempt requires the checked-out work branch".into(),
            });
        }
        let record = plan_record(root, &owner.identity.plan)?;
        if record.phase != "build" {
            return Err(Error::Manifest {
                message: "failed attempts may be recorded only during BUILD".into(),
            });
        }
        if args.command.trim().is_empty()
            || args.exit_status == 0
            || args.diagnostics.trim().is_empty()
        {
            return Err(Error::Manifest {
                message: "failed attempt requires command, nonzero status, and diagnostics".into(),
            });
        }
        let (path, observed) = plan_revision(root, &owner.identity.plan)?;
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let old_line = build_state_line(&text)?;
        let current = parse_retry_state(old_line)?;
        let config = Manifest::load(root)?.workflow;
        let fingerprint =
            retry::failure_fingerprint(&args.command, args.exit_status, &args.diagnostics);
        if current.fingerprint.as_deref() == Some(&fingerprint)
            && current.attempts >= config.max_stalled_block_attempts
        {
            return Err(Error::Manifest {
                message: "equivalent BUILD failure exhausted the configured attempt limit".into(),
            });
        }
        let next = retry::record_failure(
            &current,
            &args.command,
            args.exit_status,
            &args.diagnostics,
            config.max_stalled_block_attempts,
        );
        let new_line = render_retry_state(&next);
        apply_plan_edits_transactionally(
            root,
            &path,
            vec![ExactEdit {
                old_text: old_line.into(),
                new_text: new_line,
            }],
        )?;
        git::commit_knowledge_changes(root, "chore(workflow): record failed build attempt")?;
        let revision = plan_revision(root, &owner.identity.plan)?.1;
        let state = transaction.compare_and_swap(&args.session, &observed, |state| {
            state.last_plan_revision.clone_from(&revision);
        })?;
        emit(
            "attempt",
            &serde_json::json!({
                "state": state,
                "attempts": next.attempts,
                "fingerprint": next.fingerprint,
                "stalled": next.attempts >= config.max_stalled_block_attempts,
            }),
        )
    })
}

fn record_correction(root: &Path, args: &CorrectionArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        if owner.session_id != args.session || owner.last_plan_revision != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow ownership or plan revision changed".into(),
            });
        }
        cache::verify_authority(&owner, &ui_authority_capability()?)?;
        validate_identity_values(root, &owner.identity)?;
        git::require_clean(root)?;
        if git::current_branch(root)? != owner.identity.work_branch {
            return Err(Error::Manifest {
                message: "final correction requires the checked-out work branch".into(),
            });
        }
        if plan_record(root, &owner.identity.plan)?.phase != "build" {
            return Err(Error::Manifest {
                message: "final corrections may be recorded only during BUILD".into(),
            });
        }
        if args.review_session.trim().is_empty() || args.review_session == args.session {
            return Err(Error::Manifest {
                message: "final correction requires a distinct isolated reviewer run".into(),
            });
        }
        if owner.candidate_revision.as_deref() != Some(args.candidate.as_str())
            || git::revision(root, &owner.identity.work_branch)? != args.candidate
        {
            return Err(Error::Manifest {
                message: "final correction is not bound to the verified candidate".into(),
            });
        }
        let summary = retry::normalize_diagnostics(&args.summary).replace('\n', " | ");
        if summary.is_empty() {
            return Err(Error::Manifest {
                message: "final correction requires bounded structured findings".into(),
            });
        }
        let (path, observed) = plan_revision(root, &owner.identity.plan)?;
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let mut state = parse_retry_state(build_state_line(&text)?)?;
        let config = Manifest::load(root)?.workflow;
        if state.final_corrections >= config.max_final_correction_cycles {
            return Err(Error::Manifest {
                message: "configured final correction limit is already exhausted".into(),
            });
        }
        state.final_corrections += 1;
        state.blocker = if state.final_corrections >= config.max_final_correction_cycles {
            format!("final correction limit exhausted: {summary}")
        } else {
            format!(
                "final correction {} required: {summary}",
                state.final_corrections
            )
        };
        apply_plan_edits_transactionally(
            root,
            &path,
            vec![
                ExactEdit {
                    old_text: build_state_line(&text)?.into(),
                    new_text: render_retry_state(&state),
                },
                invalidate_final_evidence_edit(&text)?,
            ],
        )?;
        git::commit_knowledge_changes(root, "chore(workflow): record final correction")?;
        let revision = plan_revision(root, &owner.identity.plan)?.1;
        let cache = transaction.compare_and_swap(&args.session, &observed, |cache| {
            cache.last_plan_revision.clone_from(&revision);
            cache.candidate_revision = None;
            cache.verified_default_revision = None;
        })?;
        emit(
            "correction",
            &serde_json::json!({
                "state": cache,
                "finalCorrections": state.final_corrections,
                "stalled": state.final_corrections >= config.max_final_correction_cycles,
            }),
        )
    })
}

fn plan_block(plan: &str, block: u32) -> Result<&str> {
    let heading = format!("### Block {block}: ");
    let start = plan.find(&heading).ok_or_else(|| Error::Manifest {
        message: format!("workflow plan has no stable block {block}"),
    })?;
    let body = &plan[start..];
    let end = body[heading.len()..]
        .find("\n### Block ")
        .map(|offset| heading.len() + offset)
        .or_else(|| {
            body[heading.len()..]
                .find("\n## ")
                .map(|offset| heading.len() + offset)
        })
        .unwrap_or(body.len());
    Ok(&body[..end])
}

fn block_is_done(plan: &str, block: u32) -> bool {
    plan_block(plan, block).is_ok_and(|body| {
        body.lines()
            .any(|line| matches!(line, "- [x] Done." | "- [X] Done."))
    })
}

fn block_dependencies(block: &str) -> Vec<u32> {
    block
        .lines()
        .find(|line| line.starts_with("- Dependencies:"))
        .into_iter()
        .flat_map(|line| line.split(|character: char| !character.is_ascii_digit()))
        .filter_map(|value| value.parse().ok())
        .collect()
}

fn block_area_paths(block: &str) -> Vec<String> {
    let Some(line) = block.lines().find(|line| line.starts_with("- Areas:")) else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('`') else { break };
        let path = rest[..close].trim().trim_end_matches('/');
        if !path.is_empty() {
            paths.push(path.to_string());
        }
        rest = &rest[close + 1..];
    }
    paths
}

fn run_block_verification(root: &Path, block: &str) -> Result<()> {
    let commands = plan_verification_commands(block);
    if commands.is_empty() {
        return Err(Error::Manifest {
            message: "BUILD block checkpoint has no executable Verification commands".into(),
        });
    }
    let head = git::revision(root, "HEAD")?;
    for command in commands {
        run_verification_command(root, &command)?;
        if git::revision(root, "HEAD")? != head {
            return Err(Error::Manifest {
                message: "a block Verification command changed HEAD".into(),
            });
        }
    }
    Ok(())
}

fn build_state_line(plan: &str) -> Result<&str> {
    plan.lines()
        .find(|line| line.starts_with("Current block: "))
        .ok_or_else(|| Error::Manifest {
            message: "workflow plan has no machine-readable Build state".into(),
        })
}

fn parse_retry_state(line: &str) -> Result<RetryState> {
    let invalid = || Error::Manifest {
        message: "workflow plan Build state is malformed".into(),
    };
    let rest = line.strip_prefix("Current block: ").ok_or_else(&invalid)?;
    let (block, rest) = rest.split_once(". Attempts: ").ok_or_else(&invalid)?;
    let (attempts, rest) = rest
        .split_once(". Final corrections: ")
        .ok_or_else(&invalid)?;
    let (corrections, rest) = rest.split_once(". ").ok_or_else(&invalid)?;
    let (fingerprint, blocker) = if let Some(rest) = rest.strip_prefix("Fingerprint: ") {
        let (fingerprint, blocker) = rest.split_once(". Blocker: ").ok_or_else(&invalid)?;
        (
            (fingerprint != "none").then(|| fingerprint.to_string()),
            blocker,
        )
    } else {
        (None, rest.strip_prefix("Blocker: ").ok_or_else(&invalid)?)
    };
    Ok(RetryState {
        current_block: block.parse().map_err(|_| invalid())?,
        attempts: attempts.parse().map_err(|_| invalid())?,
        final_corrections: corrections.parse().map_err(|_| invalid())?,
        fingerprint,
        blocker: blocker.trim_end_matches('.').to_string(),
    })
}

fn render_retry_state(state: &RetryState) -> String {
    format!(
        "Current block: {}. Attempts: {}. Final corrections: {}. Fingerprint: {}. Blocker: {}.",
        state.current_block,
        state.attempts,
        state.final_corrections,
        state.fingerprint.as_deref().unwrap_or("none"),
        state.blocker.trim_end_matches('.'),
    )
}

fn retry_state_after_checkpoint(plan: &str, previous: &RetryState) -> RetryState {
    let mut current_block = previous.current_block;
    let mut candidate = current_block.saturating_add(1);
    while plan_block(plan, candidate).is_ok() {
        if !block_is_done(plan, candidate) {
            current_block = candidate;
            break;
        }
        candidate = candidate.saturating_add(1);
    }
    RetryState {
        current_block,
        attempts: 0,
        final_corrections: previous.final_corrections,
        fingerprint: None,
        blocker: "none".into(),
    }
}

fn record_evidence(root: &Path, args: &EvidenceArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        record_evidence_locked(root, args, transaction)
    })
}

fn record_evidence_locked(
    root: &Path,
    args: &EvidenceArgs,
    transaction: &mut cache::Transaction<'_>,
) -> Result<u8> {
    let state = transaction.load()?.ok_or_else(|| Error::Manifest {
        message: "workflow is unowned".into(),
    })?;
    if state.session_id != args.session || state.last_plan_revision != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow ownership or plan revision changed".into(),
        });
    }
    cache::verify_authority(&state, &ui_authority_capability()?)?;
    let review_session = args.review_session.as_deref();
    if !matches!(args.kind, EvidenceKindName::Verification)
        && review_session.is_none_or(|review| review.trim().is_empty() || review == args.session)
    {
        return Err(Error::Manifest {
            message: "review evidence requires a distinct isolated reviewer session".into(),
        });
    }
    if matches!(args.kind, EvidenceKindName::Verification) && review_session.is_some() {
        return Err(Error::Manifest {
            message: "verification evidence does not accept a reviewer session".into(),
        });
    }
    validate_identity_values(root, &state.identity)?;
    if !matches!(args.kind, EvidenceKindName::ScopeReview) {
        git::require_clean(root)?;
    }
    if git::current_branch(root)? != state.identity.work_branch {
        return Err(Error::Manifest {
            message: "evidence requires the checked-out work branch".into(),
        });
    }
    let record = plan_record(root, &state.identity.plan)?;
    let (path, observed) = plan_revision(root, &state.identity.plan)?;
    if matches!(args.kind, EvidenceKindName::ScopeReview) {
        if args.revision.as_deref() != Some(observed.as_str()) || observed == args.expected_revision
        {
            return Err(Error::Manifest {
                message: "scope review must bind the changed canonical plan revision".into(),
            });
        }
    } else if args.revision.is_some() || observed != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow plan revision changed".into(),
        });
    }
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let (lines, candidate, verified_default) = match args.kind {
        EvidenceKindName::ScopeReview if record.phase == "scope" => (
            vec![format!(
                "Scope requirements review: clean by isolated session {}.",
                review_session.expect("scope review session was checked")
            )],
            None,
            None,
        ),
        EvidenceKindName::Verification if record.phase == "build" => {
            let candidate = args.candidate.as_deref().ok_or_else(|| Error::Manifest {
                message: "verification evidence requires candidate H".into(),
            })?;
            let head = git::revision(root, &state.identity.work_branch)?;
            if candidate != head {
                return Err(Error::Manifest {
                    message: "verification candidate must be the clean work-branch tip".into(),
                });
            }
            let default = git::revision(root, &state.identity.default_branch)?;
            if !git::is_ancestor(root, &default, candidate)? {
                return Err(Error::Manifest {
                    message: "candidate H does not contain the default-branch tip".into(),
                });
            }
            run_plan_verification(root, &text, candidate)?;
            (Vec::new(), Some(candidate.to_string()), Some(default))
        }
        EvidenceKindName::Final if record.phase == "build" => {
            let retry_state = parse_retry_state(build_state_line(&text)?)?;
            if retry_state.final_corrections
                >= Manifest::load(root)?.workflow.max_final_correction_cycles
            {
                return Err(Error::Manifest {
                    message: "final correction limit is exhausted; human guidance is required"
                        .into(),
                });
            }
            if text.contains("- [ ] Done") {
                return Err(Error::Manifest {
                    message: "final attestation requires every work block to be complete".into(),
                });
            }
            if knowledge_contains(root, &format!("PENDING ({})", state.identity.plan))? {
                return Err(Error::Manifest {
                    message: "final attestation refuses affected pending promises".into(),
                });
            }
            if primary_issue_has_unresolved_discoveries(root, &state.identity.issue)? {
                return Err(Error::Manifest {
                    message: "final attestation refuses unresolved primary-issue discoveries"
                        .into(),
                });
            }
            let candidate = args.candidate.as_deref().ok_or_else(|| Error::Manifest {
                message: "final review evidence requires candidate H".into(),
            })?;
            if state.candidate_revision.as_deref() != Some(candidate)
                || git::revision(root, &state.identity.work_branch)? != candidate
            {
                return Err(Error::Manifest {
                    message: "final review is not bound to current successful verification".into(),
                });
            }
            let default =
                state
                    .verified_default_revision
                    .clone()
                    .ok_or_else(|| Error::Manifest {
                        message: "final review is missing the verified default revision".into(),
                    })?;
            (
                vec![
                    format!("Candidate revision: {candidate}."),
                    format!("Verified default revision: {default}."),
                    format!("Final verification: passed for {candidate}."),
                    format!("Documentation verification: passed for {candidate}."),
                    format!(
                        "Final review: clean for {candidate} by isolated session {}.",
                        review_session.expect("final review session was checked")
                    ),
                ],
                Some(candidate.to_string()),
                Some(default),
            )
        }
        _ => {
            return Err(Error::Manifest {
                message: "evidence kind does not match the canonical phase".into(),
            });
        }
    };
    if matches!(args.kind, EvidenceKindName::Verification) {
        let state =
            transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
                state.candidate_revision.clone_from(&candidate);
                state
                    .verified_default_revision
                    .clone_from(&verified_default);
            })?;
        return emit("verification", &state);
    }
    let mut edits = vec![completion_evidence_edit(&text, &lines)?];
    let commit_message = if matches!(args.kind, EvidenceKindName::Final) {
        edits.push(ExactEdit {
            old_text: "phase: build".into(),
            new_text: "phase: accept".into(),
        });
        "chore(workflow): attest build completion"
    } else {
        "chore(workflow): record canonical evidence"
    };
    apply_plan_edits_transactionally(root, &path, edits)?;
    git::commit_knowledge_changes(root, commit_message)?;
    let revision = plan_revision(root, &state.identity.plan)?.1;
    let state = transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
        state.last_plan_revision.clone_from(&revision);
        if candidate.is_some() {
            state.candidate_revision.clone_from(&candidate);
            state
                .verified_default_revision
                .clone_from(&verified_default);
        }
    })?;
    emit("evidence", &state)
}

fn plan_verification_commands(plan: &str) -> Vec<String> {
    plan.lines()
        .filter(|line| line.trim_start().starts_with("- Verification:"))
        .flat_map(|line| {
            let mut commands = Vec::new();
            let mut rest = line;
            while let Some(open) = rest.find('`') {
                rest = &rest[open + 1..];
                let Some(close) = rest.find('`') else { break };
                let command = rest[..close].trim();
                if !command.is_empty() {
                    commands.push(command.to_string());
                }
                rest = &rest[close + 1..];
            }
            commands
        })
        .collect()
}

fn run_verification_command(root: &Path, command: &str) -> Result<()> {
    #[cfg(unix)]
    let mut process = {
        let mut process = Command::new("sh");
        process.args(["-c", command]);
        process
    };
    #[cfg(windows)]
    let mut process = {
        let mut process = Command::new("cmd");
        process.args(["/C", command]);
        process
    };
    let status = process
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|source| Error::Command {
            command: command.into(),
            status: None,
            stderr: source.to_string(),
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Command {
            command: command.into(),
            status: status.code(),
            stderr: "plan verification failed".into(),
        })
    }
}

fn run_plan_verification(root: &Path, plan: &str, candidate: &str) -> Result<()> {
    let commands = plan_verification_commands(plan);
    if commands.is_empty() {
        return Err(Error::Manifest {
            message: "final verification has no executable plan commands".into(),
        });
    }
    for command in commands {
        run_verification_command(root, &command)?;
        if git::revision(root, "HEAD")? != candidate {
            return Err(Error::Manifest {
                message: "a verification command changed candidate H".into(),
            });
        }
    }
    git::require_clean(root)
}

fn transition(
    root: &Path,
    args: &TransitionArgs,
    abandon: bool,
    abandonment_reason: Option<&str>,
) -> Result<u8> {
    cache::transaction(root, |transaction| {
        transition_locked(root, args, abandon, abandonment_reason, transaction)
    })
}

fn transition_locked(
    root: &Path,
    args: &TransitionArgs,
    abandon: bool,
    abandonment_reason: Option<&str>,
    transaction: &mut cache::Transaction<'_>,
) -> Result<u8> {
    let state = transaction.load()?.ok_or_else(|| Error::Manifest {
        message: "workflow is unowned".into(),
    })?;
    if state.session_id != args.session || state.last_plan_revision != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow ownership or plan revision changed".into(),
        });
    }
    validate_identity_values(root, &state.identity)?;
    git::require_clean(root)?;
    if git::current_branch(root)? != state.identity.work_branch {
        return Err(Error::Manifest {
            message: format!(
                "workflow mutations require checked-out branch `{}`",
                state.identity.work_branch
            ),
        });
    }
    let record = plan_record(root, &state.identity.plan)?;
    let phase = phase(args.phase);
    if record.phase != phase_text(phase) {
        return Err(Error::Manifest {
            message: format!(
                "canonical plan phase is `{}`, not `{}`",
                record.phase,
                phase_text(phase)
            ),
        });
    }
    let transition = if abandon {
        Transition::Abandon
    } else {
        match args.transition {
            TransitionName::ApproveScope => Transition::ApproveScope,
            TransitionName::ReturnToScope => Transition::ReturnToScope,
            TransitionName::RejectAcceptance => Transition::RejectAcceptance,
            TransitionName::Accept => Transition::Accept,
            TransitionName::RecoverStaleDefault => Transition::RecoverStaleDefault,
        }
    };
    let rescope_feedback = if matches!(
        transition,
        Transition::ReturnToScope | Transition::RejectAcceptance
    ) {
        Some(
            args.feedback
                .as_deref()
                .filter(|feedback| !feedback.trim().is_empty())
                .ok_or_else(|| Error::Manifest {
                    message: "return to SCOPE requires verbatim discovery or rejection feedback"
                        .into(),
                })?,
        )
    } else {
        if args.feedback.is_some() {
            return Err(Error::Manifest {
                message: "feedback is accepted only when returning to SCOPE".into(),
            });
        }
        None
    };
    let plan_text =
        fs::read_to_string(plan_revision(root, &state.identity.plan)?.0).map_err(|source| {
            Error::Io {
                path: plan_revision(root, &state.identity.plan)
                    .expect("plan was just read")
                    .0,
                source,
            }
        })?;
    if matches!(transition, Transition::ApproveScope) {
        let scope_base = evidence_revision(&plan_text, "Scope product baseline")
            .unwrap_or(git::revision(root, &state.identity.default_branch)?);
        let head = git::revision(root, &state.identity.work_branch)?;
        if !git::is_ancestor(root, &scope_base, &head)?
            || git::changed_paths(root, &scope_base, &head)?
                .iter()
                .any(|path| !path.starts_with("knowledge/"))
        {
            return Err(Error::Manifest {
                message:
                    "SCOPE may change canonical knowledge only; product changes belong to BUILD"
                        .into(),
            });
        }
    }
    let candidate_revision = evidence_revision(&plan_text, "Candidate revision");
    let verified_default_revision = evidence_revision(&plan_text, "Verified default revision");
    let config = Manifest::load(root)?.workflow;
    let human_authorized = matches!(
        transition,
        Transition::ApproveScope | Transition::RejectAcceptance | Transition::Abandon
    ) || matches!(transition, Transition::Accept)
        && config.human_acceptance_required;
    if human_authorized {
        cache::verify_authority(&state, &ui_authority_capability()?)?;
    }
    if matches!(transition, Transition::Accept) {
        let verified =
            state
                .verified_default_revision
                .as_deref()
                .ok_or_else(|| Error::Manifest {
                    message: "acceptance requires a verified default revision".into(),
                })?;
        if git::revision(root, &state.identity.default_branch)? != verified {
            return Err(Error::Manifest {
                message: "default branch advanced; return ACCEPT to BUILD before closure".into(),
            });
        }
    }
    if matches!(transition, Transition::RecoverStaleDefault) && phase == Phase::Accept {
        let verified =
            state
                .verified_default_revision
                .as_deref()
                .ok_or_else(|| Error::Manifest {
                    message: "stale-default recovery requires a verified default revision".into(),
                })?;
        if git::revision(root, &state.identity.default_branch)? == verified {
            return Err(Error::Manifest {
                message: "default branch still matches BUILD verification".into(),
            });
        }
    }
    let gates = GateEvidence {
        human_scope_approved: human_authorized && matches!(transition, Transition::ApproveScope),
        requirements_review_clean: plan_text
            .lines()
            .any(|line| line.starts_with("Scope requirements review: clean by isolated session ")),
        human_acceptance_approved: human_authorized && matches!(transition, Transition::Accept),
        human_abandonment_approved: human_authorized && abandon,
        closure_integrated: if phase == Phase::Done {
            git::is_ancestor(
                root,
                &git::revision(root, &state.identity.work_branch)?,
                &git::revision(root, &state.identity.default_branch)?,
            )?
        } else {
            false
        },
    };
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
    let mut edits = vec![ExactEdit {
        old_text: format!("phase: {}", phase_text(phase)),
        new_text: format!("phase: {}", phase_text(next)),
    }];
    if matches!(transition, Transition::ApproveScope) {
        edits.push(completion_evidence_edit(
            &plan_text,
            &["Human scope approval: approved.".into()],
        )?);
    }
    let scope_baseline = if matches!(
        transition,
        Transition::ReturnToScope | Transition::RejectAcceptance
    ) {
        Some(format!(
            "Scope product baseline: {}.",
            git::revision(root, &state.identity.work_branch)?
        ))
    } else {
        None
    };
    if matches!(transition, Transition::RejectAcceptance) {
        edits.push(invalidate_scope_evidence_edit(
            &plan_text,
            scope_baseline
                .as_ref()
                .expect("rejection has a scope baseline"),
            true,
        )?);
    } else if matches!(transition, Transition::ReturnToScope) {
        edits.push(invalidate_scope_evidence_edit(
            &plan_text,
            scope_baseline
                .as_ref()
                .expect("return has a scope baseline"),
            false,
        )?);
    } else if matches!(transition, Transition::RecoverStaleDefault) && phase == Phase::Accept {
        edits.push(invalidate_final_evidence_edit(&plan_text)?);
    }
    if matches!(next, Phase::Done | Phase::Abandoned) {
        edits.push(ExactEdit {
            old_text: "lifecycle: open".into(),
            new_text: format!("lifecycle: {}", phase_text(next)),
        });
    }
    // Validate immutable-candidate ancestry before writing closure records.
    // A rejected acceptance attempt must not leave a partially closed plan or
    // issue behind.
    if matches!(transition, Transition::Accept) {
        let candidate = state
            .candidate_revision
            .as_deref()
            .ok_or_else(|| Error::Manifest {
                message: "acceptance requires an immutable reviewed candidate H".into(),
            })?;
        let head = git::revision(root, &state.identity.work_branch)?;
        if !git::is_ancestor(root, candidate, &head)?
            || git::changed_paths(root, candidate, &head)?
                .iter()
                .any(|path| !administrative_path(path, &state.identity))
        {
            return Err(Error::Manifest {
                message: "candidate H changed outside administrative workflow records".into(),
            });
        }
    }
    if matches!(transition, Transition::RecoverStaleDefault) && phase == Phase::Done {
        reopen_stale_closure(root, &state.identity)?;
    } else if matches!(next, Phase::Done | Phase::Abandoned) {
        close_records(root, &state.identity, next, abandonment_reason)?;
    } else if let Some(feedback) = rescope_feedback {
        let issue_path = root
            .join("knowledge/issues/open")
            .join(format!("{}.md", state.identity.issue));
        let issue_text = fs::read_to_string(&issue_path).map_err(|source| Error::Io {
            path: issue_path.clone(),
            source,
        })?;
        apply_record_edits_transactionally(
            root,
            vec![
                (&path, edits),
                (
                    &issue_path,
                    vec![rescope_discovery_edit(
                        &issue_text,
                        if matches!(transition, Transition::RejectAcceptance) {
                            "ACCEPT rejection"
                        } else {
                            "BUILD discovery"
                        },
                        feedback,
                    )?],
                ),
            ],
        )?;
    } else {
        apply_plan_edits_transactionally(root, &path, edits)?;
    }
    let commit_message = match transition {
        Transition::ApproveScope => "chore(workflow): approve scope",
        Transition::ReturnToScope | Transition::RejectAcceptance => {
            "chore(workflow): return to scope"
        }
        Transition::RecordBuildProgress => "chore(workflow): record build progress",
        Transition::Accept => "chore(workflow): close accepted work",
        Transition::RecoverStaleDefault => "chore(workflow): reopen stale closure",
        Transition::Abandon => "chore(workflow): close abandoned work",
    };
    git::commit_knowledge_changes(root, commit_message)?;
    if matches!(transition, Transition::Abandon) {
        filing::publish_abandonment(
            root,
            &state.identity.default_branch,
            &state.identity.issue,
            &state.identity.plan,
        )?;
    }
    let revision = plan_revision(root, &state.identity.plan)?.1;
    let state = transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
        state.last_plan_revision.clone_from(&revision);
        if let Some(candidate) = candidate_revision {
            state.candidate_revision = Some(candidate);
            state.verified_default_revision = verified_default_revision;
        } else if matches!(
            transition,
            Transition::RecoverStaleDefault | Transition::RejectAcceptance
        ) {
            state.candidate_revision = None;
            state.verified_default_revision = None;
        }
    })?;
    if matches!(transition, Transition::Abandon) {
        transaction.release(&args.session)?;
    }
    emit(
        "transition",
        &serde_json::json!({
            "phase": phase_text(next),
            "state": state,
            "ownershipReleased": matches!(transition, Transition::Abandon),
        }),
    )
}

fn apply_plan_edits_transactionally(
    root: &Path,
    live_plan: &Path,
    edits: Vec<ExactEdit>,
) -> Result<()> {
    apply_record_edits_transactionally(root, vec![(live_plan, edits)])
}

fn apply_record_edits_transactionally(
    root: &Path,
    records: Vec<(&Path, Vec<ExactEdit>)>,
) -> Result<()> {
    let staging = stage_knowledge(root, "workflow-transition-")?;
    let staged_knowledge = staging.path().join("knowledge");
    let service = SokfService::new(
        staged_knowledge.clone(),
        root.to_path_buf(),
        IndexDir(root.join(".superdev/cache/sokf")),
        None,
    );
    for (live_record, edits) in records {
        let relative = live_record
            .strip_prefix(root.join("knowledge"))
            .map_err(|_| Error::Manifest {
                message: "workflow record is outside canonical knowledge".into(),
            })?;
        let staged_record = staged_knowledge.join(relative);
        let mutation = service.edit(
            EditRequest {
                path: staged_record.to_string_lossy().into_owned(),
                edits,
            },
            MutationPolicy::AgentSafe,
        )?;
        if mutation.validation != superdev_core::sokf::ValidationState::Valid {
            let findings = mutation
                .findings
                .iter()
                .map(|finding| finding.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(Error::Manifest {
                message: format!(
                    "workflow transition did not leave valid canonical knowledge: {findings}"
                ),
            });
        }
    }
    publish_staged_knowledge(root, &staged_knowledge)
}

fn primary_issue_has_unresolved_discoveries(root: &Path, issue: &str) -> Result<bool> {
    let path = root
        .join("knowledge/issues/open")
        .join(format!("{issue}.md"));
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let Some(start) = text.find("## Discoveries\n") else {
        return Ok(false);
    };
    let body = &text[start + "## Discoveries\n".len()..];
    let end = body
        .find("\n## ")
        .or_else(|| body.find("\n<!-- sokf:links -->"))
        .unwrap_or(body.len());
    Ok(body[..end].lines().any(|line| line.starts_with("- [ ] ")))
}

fn rescope_discovery_edit(issue: &str, label: &str, feedback: &str) -> Result<ExactEdit> {
    let mut feedback_lines = feedback.split('\n');
    let first = feedback_lines.next().unwrap_or_default();
    let remaining = feedback_lines.collect::<Vec<_>>();
    let item = if remaining.is_empty() {
        format!("- [ ] {label}: {first}")
    } else {
        format!(
            "- [ ] {label}: {first}\n\n      {}",
            remaining.join("\n      ")
        )
    };
    if let Some(start) = issue.find("## Discoveries\n") {
        let content_start = start + "## Discoveries\n".len();
        let end = issue[content_start..]
            .find("\n## ")
            .map(|offset| content_start + offset)
            .or_else(|| {
                issue[content_start..]
                    .find("\n<!-- sokf:links -->")
                    .map(|offset| content_start + offset)
            })
            .unwrap_or(issue.len());
        let old_text = issue[start..end].to_string();
        let new_text = format!("{}\n{}", old_text.trim_end(), item);
        Ok(ExactEdit { old_text, new_text })
    } else {
        let insertion = issue
            .find("\n## Comments\n")
            .or_else(|| issue.find("\n<!-- sokf:links -->"))
            .unwrap_or(issue.len());
        let anchor = issue[insertion..].to_string();
        Ok(ExactEdit {
            old_text: anchor.clone(),
            new_text: format!("\n## Discoveries\n\n{item}\n{anchor}"),
        })
    }
}

fn stage_knowledge(root: &Path, prefix: &str) -> Result<tempfile::TempDir> {
    let staging_parent = root.join(".superdev/cache");
    fs::create_dir_all(&staging_parent).map_err(|source| Error::Io {
        path: staging_parent.clone(),
        source,
    })?;
    let staging = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(&staging_parent)
        .map_err(|source| Error::Io {
            path: staging_parent,
            source,
        })?;
    copy_tree(&root.join("knowledge"), &staging.path().join("knowledge"))?;
    Ok(staging)
}

fn publish_staged_knowledge(root: &Path, staged_knowledge: &Path) -> Result<()> {
    let live = root.join("knowledge");
    let backup = root.join(".superdev/cache/workflow-knowledge-backup");
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(|source| Error::Io {
            path: backup.clone(),
            source,
        })?;
    }
    fs::rename(&live, &backup).map_err(|source| Error::Io {
        path: live.clone(),
        source,
    })?;
    if let Err(source) = fs::rename(staged_knowledge, &live) {
        let _ = fs::rename(&backup, &live);
        return Err(Error::Io { path: live, source });
    }
    fs::remove_dir_all(&backup).map_err(|source| Error::Io {
        path: backup,
        source,
    })?;
    Ok(())
}

fn reopen_stale_closure(root: &Path, identity: &WorkflowIdentity) -> Result<()> {
    let staging = stage_knowledge(root, "workflow-reopen-")?;
    let knowledge = staging.path().join("knowledge");
    let plan_path = knowledge
        .join("plans/done")
        .join(format!("{}.md", identity.plan));
    let plan = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
        path: plan_path.clone(),
        source,
    })?;
    let plan = replace_once(&plan, "lifecycle: done", "lifecycle: open")?;
    let plan = replace_once(&plan, "phase: done", "phase: build")?;
    fs::write(&plan_path, plan).map_err(|source| Error::Io {
        path: plan_path,
        source,
    })?;

    let issue_path = knowledge
        .join("issues/done")
        .join(format!("{}.md", identity.issue));
    let issue = fs::read_to_string(&issue_path).map_err(|source| Error::Io {
        path: issue_path.clone(),
        source,
    })?;
    let issue = replace_once(&issue, "lifecycle: done", "lifecycle: open")?;
    let issue = remove_resolution(&issue);
    fs::write(&issue_path, issue).map_err(|source| Error::Io {
        path: issue_path,
        source,
    })?;

    superdev_core::validate::fix_repo(root, &knowledge, &[])?;
    let grammar = superdev_core::validate::schema::load_grammar(root)?;
    let report = superdev_core::validate::validate_repo(root, &knowledge, &[], &grammar)?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: "stale-default recovery did not produce valid canonical records".into(),
        });
    }
    publish_staged_knowledge(root, &knowledge)
}

fn remove_resolution(issue: &str) -> String {
    let Some(start) = issue.find("\n## Resolution\n") else {
        return issue.to_string();
    };
    let tail = &issue[start + 1..];
    let end = tail["## Resolution\n".len()..]
        .find("\n## ")
        .map(|offset| start + 1 + "## Resolution\n".len() + offset)
        .or_else(|| {
            issue[start..]
                .find("\n<!-- sokf:links -->")
                .map(|offset| start + offset)
        })
        .unwrap_or(issue.len());
    format!("{}{}", issue[..start].trim_end(), &issue[end..])
}

fn close_records(
    root: &Path,
    identity: &WorkflowIdentity,
    next: Phase,
    abandonment_reason: Option<&str>,
) -> Result<()> {
    // Build and validate the complete closure away from the live knowledge
    // tree. A failed mutation, repair, or validation therefore leaves the
    // canonical records byte-for-byte unchanged.
    let staging = stage_knowledge(root, "workflow-closure-")?;
    let staged_knowledge = staging.path().join("knowledge");

    let plan_path = ["open", "done", "abandoned"]
        .into_iter()
        .map(|lifecycle| {
            staged_knowledge
                .join("plans")
                .join(lifecycle)
                .join(format!("{}.md", identity.plan))
        })
        .find(|path| path.is_file())
        .ok_or_else(|| Error::Manifest {
            message: format!("workflow plan `{}` was not found", identity.plan),
        })?;
    let plan = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
        path: plan_path.clone(),
        source,
    })?;
    let plan = replace_once(
        &plan,
        "lifecycle: open",
        &format!("lifecycle: {}", phase_text(next)),
    )?;
    let current_phase = ["scope", "build", "accept"]
        .into_iter()
        .find(|phase| plan.matches(&format!("phase: {phase}")).count() == 1)
        .ok_or_else(|| Error::Manifest {
            message: "workflow closure could not identify the open plan phase".into(),
        })?;
    let plan = replace_once(
        &plan,
        &format!("phase: {current_phase}"),
        &format!("phase: {}", phase_text(next)),
    )?;
    fs::write(&plan_path, plan).map_err(|source| Error::Io {
        path: plan_path,
        source,
    })?;

    let issue_path = staged_knowledge
        .join("issues/open")
        .join(format!("{}.md", identity.issue));
    let issue = fs::read_to_string(&issue_path).map_err(|source| Error::Io {
        path: issue_path.clone(),
        source,
    })?;
    let issue_lifecycle = if next == Phase::Done {
        "done"
    } else {
        "wontfix"
    };
    let issue = replace_once(
        &issue,
        "lifecycle: open",
        &format!("lifecycle: {issue_lifecycle}"),
    )?;
    let resolution = if next == Phase::Done {
        "The configured acceptance gate approved the reviewed candidate for local integration."
            .to_string()
    } else {
        format!(
            "The human abandoned this workflow without integrating partial product work. {}",
            abandonment_reason.unwrap_or("No additional reason was supplied.")
        )
    };
    let issue = if issue.contains("\n## Comments\n") {
        replace_once(
            &issue,
            "\n## Comments\n",
            &format!("\n## Resolution\n\n{resolution}\n\n## Comments\n"),
        )?
    } else if issue.contains("\n<!-- sokf:links -->") {
        replace_once(
            &issue,
            "\n<!-- sokf:links -->",
            &format!("\n## Resolution\n\n{resolution}\n\n<!-- sokf:links -->"),
        )?
    } else {
        format!("{}\n\n## Resolution\n\n{resolution}\n", issue.trim_end())
    };
    fs::write(&issue_path, issue).map_err(|source| Error::Io {
        path: issue_path,
        source,
    })?;

    superdev_core::validate::fix_repo(root, &staged_knowledge, &[])?;
    let grammar = superdev_core::validate::schema::load_grammar(root)?;
    let report = superdev_core::validate::validate_repo(root, &staged_knowledge, &[], &grammar)?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: format!(
                "workflow closure did not validate:\n{}",
                report
                    .report
                    .render_human(superdev_core::validate::sokf::Warnings::Listed)
            ),
        });
    }

    publish_staged_knowledge(root, &staged_knowledge)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|source_error| Error::Io {
        path: destination.to_path_buf(),
        source: source_error,
    })?;
    for entry in fs::read_dir(source).map_err(|source_error| Error::Io {
        path: source.to_path_buf(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| Error::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|source_error| Error::Io {
                path: from.clone(),
                source: source_error,
            })?
            .is_dir()
        {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|source_error| Error::Io {
                path: to,
                source: source_error,
            })?;
        }
    }
    Ok(())
}

fn invalidate_final_evidence_edit(plan: &str) -> Result<ExactEdit> {
    filter_completion_evidence(
        plan,
        &[
            "Candidate revision: ",
            "Verified default revision: ",
            "Final verification: ",
            "Documentation verification: ",
            "Final review: ",
        ],
        &[],
    )
}

fn invalidate_scope_evidence_edit(
    plan: &str,
    baseline: &str,
    invalidate_final: bool,
) -> Result<ExactEdit> {
    let mut prefixes = vec![
        "Scope requirements review: ",
        "Human scope approval: ",
        "Scope product baseline: ",
    ];
    if invalidate_final {
        prefixes.extend([
            "Candidate revision: ",
            "Verified default revision: ",
            "Final verification: ",
            "Documentation verification: ",
            "Final review: ",
        ]);
    }
    filter_completion_evidence(plan, &prefixes, &[baseline.to_string()])
}

fn filter_completion_evidence(
    plan: &str,
    prefixes: &[&str],
    appended_lines: &[String],
) -> Result<ExactEdit> {
    let marker = "## Completion evidence\n\n";
    let start = plan.find(marker).ok_or_else(|| Error::Manifest {
        message: "workflow plan has no Completion evidence section".into(),
    })? + marker.len();
    let tail = &plan[start..];
    let end = tail
        .find("\n## ")
        .or_else(|| tail.find("\n<!-- sokf:links -->"))
        .unwrap_or(tail.len());
    let old = &tail[..end];
    let mut retained = old
        .lines()
        .filter(|line| !prefixes.iter().any(|prefix| line.starts_with(prefix)))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();
    for line in appended_lines {
        if !retained.lines().any(|existing| existing == line) {
            if !retained.is_empty() {
                retained.push_str("\n\n");
            }
            retained.push_str(line);
        }
    }
    Ok(ExactEdit {
        old_text: format!("{marker}{old}"),
        new_text: format!("{marker}{retained}\n"),
    })
}

fn completion_evidence_edit(plan: &str, lines: &[String]) -> Result<ExactEdit> {
    let marker = "## Completion evidence\n\n";
    let start = plan.find(marker).ok_or_else(|| Error::Manifest {
        message: "workflow plan has no Completion evidence section".into(),
    })? + marker.len();
    let tail = &plan[start..];
    let end = tail
        .find("\n## ")
        .or_else(|| tail.find("\n<!-- sokf:links -->"))
        .unwrap_or(tail.len());
    let old = &tail[..end];
    let mut new = old.trim_end().to_string();
    for line in lines {
        if !new.lines().any(|existing| existing == line) {
            if !new.is_empty() {
                new.push_str("\n\n");
            }
            new.push_str(line);
        }
    }
    new.push('\n');
    Ok(ExactEdit {
        old_text: format!("{marker}{old}"),
        new_text: format!("{marker}{new}"),
    })
}

fn evidence_revision(plan: &str, label: &str) -> Option<String> {
    let prefix = format!("{label}: ");
    plan.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|revision| revision.trim_end_matches('.').to_string())
        .filter(|revision| {
            revision.len() >= 7
                && revision
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
        })
}

fn replace_once(text: &str, old: &str, new: &str) -> Result<String> {
    if text.matches(old).count() != 1 {
        return Err(Error::Manifest {
            message: format!("workflow expected exactly one `{old}` field"),
        });
    }
    Ok(text.replacen(old, new, 1))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completion_evidence_is_service_owned_and_recoverable() {
        let plan = "## Completion evidence\n\nScope evidence.\n\n<!-- sokf:links -->\n";
        let edit = completion_evidence_edit(
            plan,
            &[
                "Candidate revision: abcdef1.".into(),
                "Verified default revision: 1234567.".into(),
            ],
        )
        .unwrap();
        let changed = plan.replace(&edit.old_text, &edit.new_text);
        assert_eq!(
            evidence_revision(&changed, "Candidate revision").as_deref(),
            Some("abcdef1")
        );
        assert_eq!(
            evidence_revision(&changed, "Verified default revision").as_deref(),
            Some("1234567")
        );
    }

    #[test]
    fn block_completion_is_scoped_to_the_requested_stable_block() {
        let plan = "### Block 1: first\n\n- [x] Done.\n\n### Block 2: second\n\n- [ ] Done.\n\n## Build state\n";
        assert!(block_is_done(plan, 1));
        assert!(!block_is_done(plan, 2));
        assert!(!block_is_done(plan, 3));
    }

    #[test]
    fn block_scope_and_dependencies_are_derived_from_the_current_block() {
        let plan = "### Block 2: build\n\n- [ ] Done.\n- Dependencies: Blocks 1 and 3.\n- Areas: `src/workflow` and `knowledge/contracts/a.md`.\n- Verification: `cargo test -p app`.\n\n## Build state\n";
        let block = plan_block(plan, 2).unwrap();
        assert_eq!(block_dependencies(block), vec![1, 3]);
        assert_eq!(
            block_area_paths(block),
            vec!["src/workflow", "knowledge/contracts/a.md"]
        );
        assert_eq!(plan_verification_commands(block), vec!["cargo test -p app"]);
    }

    #[test]
    fn retry_state_round_trips_legacy_and_fingerprinted_lines() {
        let legacy = parse_retry_state(
            "Current block: 2. Attempts: 1. Final corrections: 3. Blocker: none.",
        )
        .unwrap();
        assert_eq!(legacy.current_block, 2);
        assert_eq!(legacy.fingerprint, None);
        let rendered = render_retry_state(&RetryState {
            fingerprint: Some("abc123".into()),
            blocker: "retrying".into(),
            ..legacy
        });
        assert_eq!(parse_retry_state(&rendered).unwrap().attempts, 1);
        assert!(rendered.contains("Fingerprint: abc123."));
    }

    #[test]
    fn successful_checkpoint_advances_and_resets_only_build_retry_state() {
        let plan = "### Block 1: first\n\n- [x] Done.\n\n### Block 2: second\n\n- [x] Done.\n\n### Block 3: third\n\n- [ ] Done.\n\n## Build state\n";
        let previous = RetryState {
            current_block: 1,
            attempts: 3,
            final_corrections: 2,
            fingerprint: Some("failure".into()),
            blocker: "stalled".into(),
        };
        let reset = retry_state_after_checkpoint(plan, &previous);
        assert_eq!(reset.current_block, 3);
        assert_eq!(reset.attempts, 0);
        assert_eq!(reset.final_corrections, 2);
        assert_eq!(reset.fingerprint, None);
        assert_eq!(reset.blocker, "none");
    }

    #[test]
    fn acceptance_rejection_invalidates_only_final_evidence() {
        let plan = "## Completion evidence\n\nScope requirements review: clean.\n\nCandidate revision: abcdef1.\n\nVerified default revision: 1234567.\n\nFinal verification: passed for abcdef1.\n\nDocumentation verification: passed for abcdef1.\n\nFinal review: clean for abcdef1 by isolated session review.\n\n## Follow-up\n";
        let edit = invalidate_final_evidence_edit(plan).unwrap();
        let changed = plan.replace(&edit.old_text, &edit.new_text);
        assert!(changed.contains("Scope requirements review: clean."));
        assert!(!changed.contains("Candidate revision:"));
        assert!(!changed.contains("Final review:"));
        assert!(changed.contains("## Follow-up"));
    }

    #[test]
    fn rescope_invalidates_prior_scope_evidence_and_replaces_the_product_baseline() {
        let plan = "## Completion evidence\n\nScope requirements review: clean by isolated session old.\n\nHuman scope approval: approved.\n\nScope product baseline: 1111111.\n\nBlock evidence remains.\n\n## Follow-up\n";
        let edit = invalidate_scope_evidence_edit(plan, "Scope product baseline: 2222222.", false)
            .unwrap();
        let changed = plan.replace(&edit.old_text, &edit.new_text);
        assert!(!changed.contains("isolated session old"));
        assert!(!changed.contains("Human scope approval"));
        assert!(!changed.contains("1111111"));
        assert!(changed.contains("Scope product baseline: 2222222."));
        assert!(changed.contains("Block evidence remains."));
    }

    #[test]
    fn unresolved_discoveries_are_limited_to_the_discovery_section() {
        let dir = tempfile::tempdir().unwrap();
        let issues = dir.path().join("knowledge/issues/open");
        fs::create_dir_all(&issues).unwrap();
        fs::write(
            issues.join("issue-001-test.md"),
            "## Discoveries\n\n- [x] settled\n\n## Comments\n\n- [ ] not a discovery\n",
        )
        .unwrap();
        assert!(!primary_issue_has_unresolved_discoveries(dir.path(), "issue-001-test").unwrap());
        fs::write(
            issues.join("issue-001-test.md"),
            "## Discoveries\n\n- [ ] unresolved\n\n## Comments\n\nnone\n",
        )
        .unwrap();
        assert!(primary_issue_has_unresolved_discoveries(dir.path(), "issue-001-test").unwrap());
    }

    #[test]
    fn rejection_feedback_becomes_an_unresolved_discovery() {
        let issue = "## Behaviour\n\nExpected.\n\n## Comments\n\nPrior.\n";
        let edit =
            rescope_discovery_edit(issue, "ACCEPT rejection", "First line\nsecond line").unwrap();
        let changed = issue.replace(&edit.old_text, &edit.new_text);
        assert!(changed.contains(
            "## Discoveries\n\n- [ ] ACCEPT rejection: First line\n\n      second line\n"
        ));
        assert!(changed.find("## Discoveries").unwrap() < changed.find("## Comments").unwrap());
    }

    #[test]
    fn rejection_appends_without_erasing_existing_discoveries() {
        let issue = "## Discoveries\n\n- [x] Existing.\n\n## Comments\n\nnone.\n";
        let edit = rescope_discovery_edit(issue, "ACCEPT rejection", "Rejected because X").unwrap();
        let changed = issue.replace(&edit.old_text, &edit.new_text);
        assert!(
            changed.contains("- [x] Existing.\n- [ ] ACCEPT rejection: Rejected because X"),
            "{changed}"
        );
    }

    #[test]
    fn verification_commands_are_extracted_only_from_executable_entries() {
        let plan = "- Outcome: ignore `not-a-command`.\n- Verification: `cargo test -p one` and `npm test`.\n- Verification: prose only.\n";
        assert_eq!(
            plan_verification_commands(plan),
            vec!["cargo test -p one", "npm test"]
        );
    }

    #[test]
    fn malformed_or_absent_evidence_is_not_recovered() {
        assert!(
            evidence_revision("Candidate revision: not-a-sha.", "Candidate revision").is_none()
        );
        assert!(evidence_revision("none.", "Candidate revision").is_none());
    }
}
