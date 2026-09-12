//! Typed requests from the controller. No model-supplied approval booleans.

use serde::{Deserialize, Serialize};

use super::local::{
    AssessmentReport, Checkpoint, ExecutionMode, ExecutionStage, HumanInput, ScopeStep, StepRecord,
    WorkerRecord,
};

/// Document selected for approval or diff assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentKind {
    /// Primary issue.
    Issue,
    /// Implementing plan.
    Plan,
}

impl DocumentKind {
    pub(super) fn prefix(self) -> &'static str {
        match self {
            Self::Issue => "issue",
            Self::Plan => "plan",
        }
    }
}

/// One local workflow service operation, submitted by an authenticated controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LocalRequest {
    /// Reserve an issue-only workflow on the selected default branch.
    Create {
        /// Selected or not-yet-authored issue ID.
        issue: String,
        /// Explicit default branch, or automatic repository discovery.
        default_branch: Option<String>,
    },
    /// Reacquire local progress, without restoring unused SCOPE permissions.
    Resume {
        /// Internal workflow ID, never inferred from document prose.
        id: String,
    },
    /// Release controller ownership without deleting progress or approval.
    Pause {
        /// Internal workflow ID.
        id: String,
    },
    /// Import legacy progress as unapproved recovery facts; preserve the source.
    Migrate {
        /// Selected primary issue.
        issue: String,
        /// Selected implementing plan.
        plan: String,
        /// Explicit default branch or repository discovery.
        default_branch: Option<String>,
        /// Agent-reconstructed work to inspect, not approval or a passed review.
        checkpoint: Checkpoint,
    },
    /// Compare-and-swap one existing workflow action.
    Change {
        /// Internal workflow ID.
        id: String,
        /// Last observed local record generation.
        expected_revision: u64,
        /// Requested operation; Rust owns its invariants.
        change: ScopeChange,
    },
}

/// A SCOPE mutation or explicitly authorised BUILD handoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ScopeChange {
    /// Return a stopped execution to human-led SCOPE without discarding progress.
    ReturnToScope {
        /// Human-requested change or discovery to discuss.
        feedback: String,
        /// Explicit permission to return to SCOPE.
        input: HumanInput,
    },
    /// Reserve a plan independently from its issue's number.
    AttachPlan {
        /// New implementing plan identity.
        plan: String,
    },
    /// Save performed/skipped actions and discussion; this never grants approval.
    RecordStep {
        /// Repeated and skipped actions remain explicit.
        entry: Option<StepRecord>,
        /// Next action chosen by the human-led conversation.
        next: ScopeStep,
        /// Outstanding discussion or findings.
        discussion: Option<String>,
    },
    /// Publish precisely the document revision the human approved.
    Approve {
        /// Named document.
        document: DocumentKind,
        /// Exact byte hash shown to the human.
        expected_hash: String,
        /// Interactive response, authenticated outside model execution.
        input: HumanInput,
        /// Required generated index paths; unrelated deltas are refused.
        indexes: Vec<String>,
    },
    /// Complete only the exact interrupted publication saved in local state.
    RecoverPublication,
    /// Complete only a previously authorised, interrupted branch startup.
    RecoverBuildStart,
    /// Record the agent's assessment of an actual document diff.
    AssessChange {
        /// Named document.
        document: DocumentKind,
        /// Previously covered revision, not an arbitrary baseline.
        from: String,
        /// Exact new document hash.
        to: String,
        /// Whether the inspected diff changes formatting only, not meaning.
        formatting_only: bool,
        /// Explanation of the actual diff assessment.
        reason: String,
    },
    /// Start execution separately from plan approval, creating the branch here only.
    StartBuild {
        /// Human-selected execution placement.
        mode: ExecutionMode,
        /// Explicit startup permission, not the plan-approval response.
        input: HumanInput,
    },
    /// Record the one persistent worker that owns this checkout.
    AttachWorker {
        /// Running worker process, checked for liveness before it is recorded.
        pid: u32,
        /// Worker session identity, reused across stages and restarts.
        worker: WorkerRecord,
    },
    /// Release worker ownership only after the running system shows it stopped.
    DetachWorker,
    /// Save durable execution facts before a context reset or a pause.
    RecordProgress {
        /// Stage the worker will resume from, not one inferred from context.
        stage: ExecutionStage,
        /// Completed blocks, unfinished work, and candidate-bound evidence.
        checkpoint: Checkpoint,
        /// Immutable candidate under assessment, where one exists.
        candidate: Option<String>,
        /// This stage's own report, when it produced one. Saved with the
        /// progress it belongs to, so findings and stage cannot disagree.
        #[serde(default)]
        assessment: Option<AssessmentReport>,
    },
    /// Consume one bounded correction attempt for a named activity.
    ConsumeRetry {
        /// Budgeted activity name.
        key: String,
    },
    /// Commit one completed work block, bounded to the block's declared areas.
    CommitBlock {
        /// Stable plan block number.
        block: u32,
        /// Commit subject for this block.
        message: String,
        /// Path-scoped areas the block may change.
        areas: Vec<String>,
    },
    /// Move the immutable candidate from BUILD to ACCEPT.
    CompleteBuild,
    /// Return ACCEPT findings that preserve approved intent to BUILD.
    ReturnToBuild {
        /// Findings to correct within the approved plan.
        feedback: String,
    },
    /// Accept the candidate under project policy, without merging or pushing.
    Accept {
        /// Human acceptance, required when project policy demands it.
        input: Option<HumanInput>,
    },
    /// End open work on human authority, without integrating product changes.
    Abandon {
        /// The human's stated reason, retained with the closed records.
        reason: String,
        /// Explicit abandonment permission; never a model decision.
        input: HumanInput,
    },
}
