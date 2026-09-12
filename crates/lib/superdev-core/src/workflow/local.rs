//! Checkout-local workflow data. Documents describe intent; these records own progress.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::Phase;
use crate::manifest::WorkflowConfig;

/// Directory excluded from Git, independent of disposable ownership caches.
pub const WORKFLOW_RECORDS_PATH: &str = ".superdev/workflows";
/// Local record format. Unknown versions require explicit recovery, not inference.
pub const RECORD_VERSION: u32 = 1;

/// Budgeted correction activities, each bound to one project policy field.
///
/// A context reset, a pause, a worker restart, and a controller restart all
/// read the same consumed counters, so none of them refills a budget. The
/// limits are project configuration, so no plan, prompt, or model sets them.
pub const RETRY_BUDGETS: &[&str] = &["final-correction", "scope-review"];

/// The configured budget for a named activity, or `None` when it has no policy.
///
/// `final-correction` spans the complete BUILD-to-ACCEPT run rather than
/// resetting at the phase boundary, so both stages draw on one limit.
pub fn retry_budget(config: &WorkflowConfig, key: &str) -> Option<u32> {
    match key {
        "final-correction" => Some(config.max_final_correction_cycles),
        "scope-review" => Some(config.max_scope_review_cycles),
        _ => None,
    }
}

/// A document identity and its exact byte revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRevision {
    /// Canonical SOKF identity.
    pub id: String,
    /// Repository-relative canonical path.
    pub path: String,
    /// SHA-256 of the complete UTF-8 document, without normalisation.
    pub hash: String,
}

/// Human input authenticated by the controller, not a model's approval flag.
///
/// The controller must accept only Pi's trusted interactive path or ask UI.
/// The service additionally checks its private capability and record revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanInput {
    /// Pi session receiving the input.
    pub session: String,
    /// Unique event reference within that session.
    pub event: String,
    /// The actual approval response, without presentation markers.
    pub text: String,
}

/// Published document approval. Original human input remains immutable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentApproval {
    /// Exact document the human approved.
    pub original: DocumentRevision,
    /// Authenticated response for this approval.
    pub input: HumanInput,
    /// Commit containing the approved bytes.
    pub commit: String,
    /// Subsequent revisions whose actual diffs were assessed as formatting-only.
    pub compatible: Vec<String>,
    /// Issue revision bound by a plan approval; absent for issue approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_hash: Option<String>,
}

impl DocumentApproval {
    /// Whether this identity and byte revision are covered by this approval.
    pub fn covers(&self, document: &DocumentRevision) -> bool {
        self.original.id == document.id
            && self.original.path == document.path
            && (self.original.hash == document.hash || self.compatible.contains(&document.hash))
    }
}

/// SCOPE actions, composed by the current conversation rather than child agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScopeStep {
    /// Select or draft an issue.
    SelectIssue,
    /// Interview about the issue.
    InterviewIssue,
    /// Write or revise the issue.
    WriteIssue,
    /// Double-check the issue.
    CheckIssue,
    /// Await explicit issue approval.
    ApproveIssue,
    /// Write an initial implementing plan.
    WritePlan,
    /// Interview about the plan.
    InterviewPlan,
    /// Revise the plan.
    UpdatePlan,
    /// Double-check the plan.
    CheckPlan,
    /// Await explicit plan approval.
    ApprovePlan,
    /// Approved scope, with execution not yet authorised.
    Handoff,
}

/// Distinguishes performed work from a human-directed omission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StepOutcome {
    /// The action ran; this does not assert a clean review.
    Completed,
    /// The human chose to omit the action.
    Skipped,
}

/// One performed or skipped SCOPE action, including repeat actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepRecord {
    /// Action performed or omitted.
    pub step: ScopeStep,
    /// Actual disposition, never an inferred pass.
    pub outcome: StepOutcome,
    /// Findings, interview results, or the reason for a skip.
    pub note: String,
}

/// Where a persistent worker is in its stage sequence.
///
/// This is durable so that a reset, a crash, or a controller restart resumes
/// the correct checklist instead of inferring one from an absent conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionStage {
    /// Implementing approved work blocks.
    Implementation,
    /// Final verification of the complete candidate.
    Verification,
    /// Candidate-bound final code review, after a context reset.
    Review,
    /// Acceptance assessment, after a context reset.
    Acceptance,
}

/// One attached persistent worker process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRecord {
    /// Worker's own Pi session identity, reused across stages and restarts.
    pub session: String,
    /// Local session transcript path; recovery data, never approval evidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_file: Option<String>,
    /// Reset anchor inside that session file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
}

/// Execution placement selected separately from document approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    /// Execute in the controlling conversation.
    Current,
    /// Execute in one persistent SDK subprocess.
    Worker,
}

/// Largest assessment report retained. A longer one keeps its head and says so.
///
/// A report is model prose, so it has no natural bound. Truncating visibly is
/// better than refusing the write, which would lose the findings entirely.
pub const MAX_ASSESSMENT_BYTES: usize = 64 * 1024;

/// One assessment stage's own report, bound to the candidate it judged.
///
/// Review and acceptance findings are durable because the conversation that
/// received them is not: a compaction, a reset, or a closed session would
/// otherwise lose them, leaving a record that cannot distinguish a review that
/// found nothing from one whose findings were dropped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssessmentReport {
    /// Which assessment produced this report.
    pub stage: ExecutionStage,
    /// The candidate it judged. A later candidate makes the report stale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<String>,
    /// What the stage reported, as it reported it.
    pub findings: String,
    /// Why the report cannot stand as a verdict, when it cannot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub void: Option<String>,
}

impl AssessmentReport {
    /// Retain the report within its bound, marking any truncation.
    ///
    /// Truncation respects character boundaries, because a report cut mid-character
    /// would not survive its own round trip through JSON.
    pub fn bounded(mut self) -> Self {
        if self.findings.len() > MAX_ASSESSMENT_BYTES {
            let mut end = MAX_ASSESSMENT_BYTES;
            while end > 0 && !self.findings.is_char_boundary(end) {
                end -= 1;
            }
            let dropped = self.findings.len() - end;
            self.findings.truncate(end);
            self.findings
                .push_str(&format!("\n\n[{dropped} further bytes were not retained]"));
        }
        self
    }

    /// Whether this report judged the candidate that is current now.
    pub fn covers(&self, candidate: Option<&str>) -> bool {
        self.candidate.as_deref() == candidate
    }
}

/// Durable implementation facts, not a transcript or a new permission grant.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    /// Stable numbers of completed plan blocks.
    pub completed_blocks: Vec<u32>,
    /// Partial work and uncertain command outcomes to inspect before continuing.
    pub unfinished: String,
    /// Candidate-bound verification evidence references.
    pub evidence: Vec<String>,
}

/// Commit/local-record coordination. Approval is unavailable while this exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingPublication {
    /// Document being published.
    pub document: DocumentRevision,
    /// Response authorising precisely these bytes.
    pub input: HumanInput,
    /// Parent of the prepared commit.
    pub parent: String,
    /// Exact prepared commit, known before moving the branch reference.
    pub commit: String,
    /// Branch on which publication must finish.
    pub branch: String,
    /// Only these document/index paths may be included.
    pub paths: Vec<String>,
    /// Issue revision for plan approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_hash: Option<String>,
}

/// An authorised BUILD startup whose branch/state writes may need recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingBuildStart {
    /// New branch whose absence was checked before saving this intent.
    pub branch: String,
    /// Exact approved starting commit.
    pub parent: String,
    /// Human-selected execution placement.
    pub mode: ExecutionMode,
    /// Explicit execution permission, distinct from plan approval.
    pub input: HumanInput,
}

/// One checkout's durable workflow, usable before a plan exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRecord {
    /// Supported storage schema version.
    pub version: u32,
    /// Internal identity, unrelated to document numbering.
    pub id: String,
    /// Compare-and-swap generation; every durable mutation increments it.
    pub revision: u64,
    /// Canonical checkout root; copied local state is not transferable authority.
    pub checkout: String,
    /// Selected default branch.
    pub default_branch: String,
    /// Existing work branch; absent until explicit BUILD startup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_branch: Option<String>,
    /// Current execution phase, never read from plan prose.
    pub phase: Phase,
    /// Next human-led SCOPE action.
    pub scope_step: ScopeStep,
    /// Performed and skipped actions, including repetitions.
    pub steps: Vec<StepRecord>,
    /// Outstanding discussion, which does not grant permission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discussion: Option<String>,
    /// Selected issue identity, reserved independently of any plan.
    pub issue: String,
    /// Selected plan identity, absent during issue-only SCOPE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// Published issue approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_approval: Option<DocumentApproval>,
    /// Published plan approval, also bound to its issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_approval: Option<DocumentApproval>,
    /// Interrupted publication, recovered before any further mutation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_publication: Option<PendingPublication>,
    /// Interrupted branch creation after explicit execution permission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_build_start: Option<PendingBuildStart>,
    /// Selected execution placement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ExecutionMode>,
    /// Local persistent worker session reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_session: Option<WorkerRecord>,
    /// Current execution stage; absent until BUILD starts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<ExecutionStage>,
    /// Last settled checkpoint; approval invalidation does not erase it.
    pub checkpoint: Checkpoint,
    /// Consumed retries, retained across pauses, restarts, and context resets.
    pub retries: BTreeMap<String, u32>,
    /// Immutable candidate currently assessed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<String>,
    /// The last assessment's report. Retained across corrections, so findings
    /// outlive the conversation that received them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment: Option<AssessmentReport>,
    /// Migration or recovery facts; never interpreted as approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery: Option<String>,
}
