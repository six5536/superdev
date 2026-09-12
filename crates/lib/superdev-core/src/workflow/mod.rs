//! Durable authority for the local SCOPE → BUILD → ACCEPT workflow.
//!
//! Harness adapters request typed transitions; they never rewrite workflow
//! state or choose project policy themselves.

pub mod abandonment;
pub mod cache;
pub mod claim;
mod documents;
pub mod git;
pub mod local;
pub mod process;
pub mod request;
pub mod service;
mod state;
pub mod store;
mod transition;

pub use state::{
    AcceptanceMode, GateEvidence, Phase, Transition, WORKFLOW_CACHE_PATH, WORKFLOW_PROTOCOL,
    WorkflowCache, WorkflowIdentity,
};
pub use transition::{TransitionError, apply_transition};
