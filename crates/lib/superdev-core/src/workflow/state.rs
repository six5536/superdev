use serde::{Deserialize, Serialize};

/// Version returned by every workflow adapter response.
pub const WORKFLOW_PROTOCOL: &str = "superdev-workflow/v1";
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
    /// Return a human rejection to SCOPE as a discovery.
    RejectAcceptance,
    /// Accept the immutable candidate under project policy.
    Accept,
    /// Reopen a prepared closure when the default branch became stale.
    RecoverStaleDefault,
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

/// Gate facts computed by the Rust service before a transition.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateEvidence {
    /// Whether the human approved the complete SCOPE diff.
    pub human_scope_approved: bool,
    /// Whether the isolated requirements review has no findings.
    pub requirements_review_clean: bool,
    /// Whether the interactive human accepted the candidate.
    pub human_acceptance_approved: bool,
    /// Whether the interactive human approved abandonment and disposition.
    pub human_abandonment_approved: bool,
    /// Whether the closure commit is reachable from the default branch.
    pub closure_integrated: bool,
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
    /// Immutable candidate reviewed at the BUILD gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_revision: Option<String>,
    /// Default-branch tip incorporated before candidate verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_default_revision: Option<String>,
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
