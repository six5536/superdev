//! Durable authority for the local SCOPE → BUILD → ACCEPT workflow.
//!
//! Harness adapters request typed transitions; they never rewrite workflow
//! state or choose project policy themselves.

pub mod abandonment;
pub mod cache;
pub mod git;
pub mod process;
mod state;
mod transition;

pub use state::{
    AcceptanceMode, GateEvidence, Phase, Transition, WORKFLOW_CACHE_PATH, WORKFLOW_PROTOCOL,
    WorkflowCache, WorkflowIdentity,
};
pub use transition::{TransitionError, apply_transition};
