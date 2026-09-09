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
    /// Issue category (independent of issue versus idea capture)
    #[arg(long, value_parser = ["bug", "feature", "chore"], default_value = "feature")]
    issue_kind: String,
    /// Short human title
    #[arg(long)]
    title: String,
    /// Human description to preserve in the record
    #[arg(long)]
    description: String,
    /// Local default branch to advance
    #[arg(long, default_value = "")]
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
    /// Validate the service attestation above the immutable reviewed candidate
    Assess(RevisionArgs),
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
    /// Adopt the current committed work tip as the next SCOPE attempt baseline
    ScopeBaseline(ScopeBaselineArgs),
    /// Commit one review-ready, knowledge-only SCOPE proposal
    ScopeCheckpoint(RevisionArgs),
    /// Commit a validated BUILD block checkpoint
    Block(ProgressArgs),
    /// Record one normalized failed BUILD attempt
    Attempt(AttemptArgs),
    /// Count one failed final verification/review correction cycle
    Correction(CorrectionArgs),
    /// Commit one path-scoped implementation correction after a failed final gate
    CorrectionCheckpoint(RevisionArgs),
    /// Record isolated review or final verification evidence canonically
    Evidence(EvidenceArgs),
    /// Incorporate the expected local default tip into BUILD
    Sync(SyncArgs),
    /// Reconstruct and acquire ownership for a known workflow
    Resume(BindArgs),
    /// Record one active isolated child for cross-instance status and recovery
    ActivityStart(ActivityStartArgs),
    /// Clear the active isolated child after exit
    ActivityFinish(RevisionArgs),
    /// Pause by releasing transient ownership without changing plan phase
    Cancel(SessionArgs),
    /// Apply the human-only abandonment transition
    Abandon(AbandonArgs),
    /// Merge an accepted closure locally with `git merge --no-ff`
    Integrate(IntegrateArgs),
}

/// Stable workflow identity and Pi ownership arguments.
#[derive(Args, Clone)]
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
    #[arg(long, default_value = "")]
    default_branch: String,
}

/// Active parent and child process identity.
#[derive(Args)]
pub struct ActivityStartArgs {
    /// Owning Pi session ID.
    #[arg(long)]
    session: String,
    /// Expected current plan content revision.
    #[arg(long)]
    expected_revision: String,
    /// Isolated workflow role.
    #[arg(long)]
    role: String,
    /// Owning Pi process ID.
    #[arg(long)]
    owner_pid: u32,
    /// OS process-start identity for the owning Pi.
    #[arg(long)]
    owner_started: String,
    /// Isolated child process ID.
    #[arg(long)]
    child_pid: u32,
    /// OS process-start identity for the child.
    #[arg(long)]
    child_started: String,
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

/// Compare-and-swap arguments for a SCOPE attempt baseline.
#[derive(Args)]
pub struct ScopeBaselineArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected work-branch tip; omitted callers use the owned tip under the repository lock
    #[arg(long)]
    expected_work: Option<String>,
}

/// Session and plan compare-and-swap arguments.
#[derive(Args)]
pub struct RevisionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
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
    /// ACCEPT findings within approved intent to BUILD
    ReturnToBuild,
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

mod dispatch;
use dispatch::*;
mod identity;
use identity::*;
mod build;
use build::*;
mod evidence;
use evidence::*;
mod transition;
use transition::*;
mod records;
use records::*;
#[cfg(test)]
mod tests;
pub use dispatch::{run, run_file};
