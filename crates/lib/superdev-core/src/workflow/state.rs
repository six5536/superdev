use serde::{Deserialize, Serialize};

/// Version returned by every workflow adapter response.
pub const WORKFLOW_PROTOCOL: &str = "superdev-workflow/v2";
/// Transient, gitignored session ownership. Canonical progress remains in the plan.
pub const WORKFLOW_CACHE_PATH: &str = ".superdev/cache/workflow.toml";

/// The only durable workflow phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// Requirements and design are being scoped for human approval.
    Scope,
    /// Approved work blocks, verification, and corrections are executing.
    Build,
    /// The immutable candidate awaits configured acceptance and integration.
    Accept,
    /// The accepted closure is prepared or integrated.
    Done,
    /// A human ended the work without integrating partial product work.
    Abandoned,
}

/// A typed transition request. No stringly-typed arbitrary target is accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// Record explicit scope approval after requirements review.
    ApproveScope,
    /// Return BUILD discoveries to SCOPE without replacing the plan.
    ReturnToScope,
    /// Persist block progress while remaining in BUILD.
    RecordBuildProgress,
    /// Report BUILD complete and move the candidate to ACCEPT.
    CompleteBuild,
    /// Return a human rejection to SCOPE as a discovery.
    RejectAcceptance,
    /// Return ACCEPT findings that preserve approved intent to BUILD.
    ReturnToBuild,
    /// Accept the immutable candidate under project policy.
    Accept,
    /// End open work through the human-only disposition path.
    Abandon,
}

/// Acceptance policy observed for a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcceptanceMode {
    /// An interactive human approved the candidate.
    Human,
    /// Project configuration permits acceptance after automated gates.
    Automatic,
}

/// Human authority observed before a transition.
///
/// These record who authorized a phase change, not whether the work is good.
/// Judging a review's quality belongs to the role that performed it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateEvidence {
    /// Whether the human approved the complete SCOPE diff.
    pub human_scope_approved: bool,
    /// Whether the interactive human accepted the candidate.
    pub human_acceptance_approved: bool,
    /// Whether the interactive human approved abandonment and disposition.
    pub human_abandonment_approved: bool,
}

/// Stable identifiers for one open workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowIdentity {
    /// Canonical primary issue ID.
    pub issue: String,
    /// Canonical implementing plan ID.
    pub plan: String,
    /// Reserved `work/<number>-<slug>` branch.
    pub work_branch: String,
    /// Locally discovered integration branch.
    pub default_branch: String,
}

/// Transient session ownership; absence means unowned, never complete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowCache {
    /// Cache format version.
    pub version: u32,
    /// Owning Pi session ID.
    pub session_id: String,
    /// Stable records and refs for the workflow.
    pub identity: WorkflowIdentity,
    /// Plan revision last observed by the owning session.
    pub last_plan_revision: String,
    /// SHA-256 digest of the owning Pi UI's in-memory authority capability.
    /// The capability itself is never persisted; only this one-way digest is exposed.
    #[serde(default)]
    pub authority_digest: String,
    /// Owning Pi process ID, recorded whenever the owner supplies one.
    ///
    /// An owner that records no process is unfalsifiable, so its claim is
    /// never reclaimed. Recording this at acquisition, rather than only while
    /// an isolated child runs, is what makes an abandoned claim decidable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_pid: Option<u32>,
    /// OS-specific owning Pi process start identity, guarding PID reuse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_started: Option<String>,
    /// Active isolated child role, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_role: Option<String>,
    /// Active isolated child process ID, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_pid: Option<u32>,
    /// Active child start time, when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_started: Option<String>,
    /// Whether cancellation has been requested before ownership is released.
    #[serde(default)]
    pub cancelled: bool,
}
