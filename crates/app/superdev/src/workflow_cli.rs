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
use superdev_core::workflow::abandonment;
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
    /// Commit the current owned work-branch changes under one message
    Commit(CommitArgs),
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

/// One committed workflow checkpoint on the owned work branch.
#[derive(Args)]
pub struct CommitArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Single-line commit message
    #[arg(long)]
    message: String,
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
    /// ACCEPT to SCOPE
    RejectAcceptance,
    /// ACCEPT findings within approved intent to BUILD
    ReturnToBuild,
    /// BUILD to ACCEPT once the reviewed candidate is ready
    CompleteBuild,
    /// ACCEPT to DONE
    Accept,
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

// sokf:end cli

mod dispatch;
use dispatch::*;
mod identity;
use identity::*;
mod transition;
use transition::*;
mod records;
pub use dispatch::run;
use records::*;
