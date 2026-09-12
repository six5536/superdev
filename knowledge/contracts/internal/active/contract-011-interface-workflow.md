---
type: Contract
id: contract-011-interface-workflow
kind: interface
title: Interface contract for the local workflow
description: The Rust-owned phases, transitions, transient ownership, project policy, and local integration seam used by Pi.
lifecycle: active
resource: /crates/lib/superdev-core/src/workflow
links:
  - rel: references
    to: adr-052-the-workflow-is-scope-build-accept-under-a-durable-core
    note: The workflow authority, phase ownership, isolation, cancellation, abandonment, and integration decision.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The Definition is materialized from the Rust declarations used by every adapter.
---

# Interface contract: the local workflow

The Rust service is the sole durable workflow authority under
[ADR-052][sokf:adr-052-the-workflow-is-scope-build-accept-under-a-durable-core].
Pi orchestrates user interaction and isolated children through the versioned
CLI protocol; it does not own phase state or Git policy. Contract Definition
materialization follows
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source].

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/workflow/state.rs -->
```rust
use serde::{Deserialize, Serialize};

/// Version returned by every workflow adapter response.
pub const WORKFLOW_PROTOCOL: &str = "superdev-workflow/v3";
/// Legacy ownership file, retained only for stopped-writer migration.
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
    /// Whether the interactive human accepted the candidate.
    pub human_acceptance_approved: bool,
    /// Whether the interactive human approved abandonment and disposition.
    pub human_abandonment_approved: bool,
}

/// Legacy document-bound identity, read during migration; not local approval.
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

/// Legacy transient ownership, preserved before local-record migration.
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
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/workflow/transition.rs -->
```rust
use std::fmt;

use super::{GateEvidence, Phase, Transition};
use crate::manifest::WorkflowConfig;

/// A refused workflow transition, safe to render to an adapter caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionError(pub String);

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TransitionError {}

fn require(ok: bool, message: &str) -> Result<(), TransitionError> {
    if ok {
        Ok(())
    } else {
        Err(TransitionError(message.into()))
    }
}

/// Apply one explicitly enumerated transition after checking its gates.
///
/// Git reachability and document evidence are resolved by callers into
/// `GateEvidence`; this pure function owns the legal transition table.
pub fn apply_transition(
    phase: Phase,
    transition: Transition,
    gates: &GateEvidence,
    config: &WorkflowConfig,
) -> Result<Phase, TransitionError> {
    use Phase::{Abandoned, Accept, Build, Done, Scope};
    use Transition::{
        Abandon, Accept as AcceptTransition, CompleteBuild, RecordBuildProgress, RejectAcceptance,
        ReturnToBuild, ReturnToScope,
    };

    match (phase, transition) {
        (Build, ReturnToScope) => Ok(Scope),
        (Build, RecordBuildProgress) => Ok(Build),
        (Build, CompleteBuild) => Ok(Accept),
        (Accept, RejectAcceptance) => Ok(Scope),
        (Accept, ReturnToBuild) => Ok(Build),
        (Accept, AcceptTransition) => {
            if config.human_acceptance_required {
                require(
                    gates.human_acceptance_approved,
                    "human acceptance is required",
                )?;
            }
            Ok(Done)
        }
        (Scope | Build | Accept, Abandon) => {
            require(
                gates.human_abandonment_approved,
                "abandonment is human-only",
            )?;
            Ok(Abandoned)
        }
        _ => Err(TransitionError(format!(
            "transition {transition:?} is not legal from {phase:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(human: bool) -> WorkflowConfig {
        WorkflowConfig {
            human_acceptance_required: human,
            ..WorkflowConfig::default()
        }
    }

    #[test]
    fn execution_transitions_cannot_bypass_local_scope_approval_and_startup() {
        for transition in [
            Transition::RecordBuildProgress,
            Transition::CompleteBuild,
            Transition::ReturnToBuild,
            Transition::Accept,
        ] {
            assert!(
                apply_transition(
                    Phase::Scope,
                    transition,
                    &GateEvidence::default(),
                    &config(false),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn build_completion_is_an_explicit_edge_rather_than_an_evidence_side_effect() {
        assert_eq!(
            apply_transition(
                Phase::Build,
                Transition::CompleteBuild,
                &GateEvidence::default(),
                &config(true),
            ),
            Ok(Phase::Accept)
        );
        assert!(
            apply_transition(
                Phase::Scope,
                Transition::CompleteBuild,
                &GateEvidence::default(),
                &config(true),
            )
            .is_err()
        );
    }

    #[test]
    fn within_scope_accept_findings_return_to_build() {
        assert_eq!(
            apply_transition(
                Phase::Accept,
                Transition::ReturnToBuild,
                &GateEvidence::default(),
                &config(true),
            ),
            Ok(Phase::Build)
        );
    }

    #[test]
    fn project_policy_alone_controls_human_acceptance() {
        let gates = GateEvidence::default();
        assert!(
            apply_transition(Phase::Accept, Transition::Accept, &gates, &config(true)).is_err()
        );
        assert_eq!(
            apply_transition(Phase::Accept, Transition::Accept, &gates, &config(false)),
            Ok(Phase::Done)
        );
    }

    #[test]
    fn cancellation_is_not_a_durable_transition_and_abandonment_is_human_only() {
        let gates = GateEvidence::default();
        assert!(
            apply_transition(Phase::Build, Transition::Abandon, &gates, &config(true)).is_err()
        );
        assert!(apply_transition(Phase::Done, Transition::Abandon, &gates, &config(true)).is_err());
    }
}
```
<!-- /sokf:include -->

## Behaviour

### Module boundaries

- `P_rust-authority` [ubiquitous] The Rust workflow service SHALL own every durable phase transition, document approval, ownership comparison, and automatic Git operation.
- `P_local-records` [ubiquitous] Durable progress and approvals SHALL live in checkout-local records under `.superdev/workflows/`, excluded from Git.
- `P_cache-transient` [ubiquitous] The transient cache SHALL carry only the live claim, so its absence means unowned rather than complete.
- `P_controller-orchestrates` [ubiquitous] The Pi controller SHALL own human interaction, question handling, and worker lifecycle without owning phase state or Git policy.
- `P_extension-skills` [ubiquitous] The Superdev extension SHALL register its bundled file, scope, build, accept, grill-me, and double-check skills from `.pi/extensions/superdev/skills/` through Pi's `resources_discover` event, exposing native `/skill:*` commands without copies in `.pi/skills/`.
- `P_skills-own-checklists` [ubiquitous] Each invoked skill SHALL carry its own stage checklist, so no separate role prompt can drift from it.
- `P_service-snapshot` [event] WHEN Pi initializes its workflow service, the adapter SHALL retain a private verified native executable snapshot independent of subsequent checkout switches and rebuilds.
- `P_skill-cold-start` [event] WHEN a workflow skill starts, the skill SHALL reconstruct identity, phase, and saved progress from the service before acting.
- `P_questions-ui` [event] WHEN a skill asks a question, the controller SHALL offer stable choice identities, a separately identified recommended choice, a typed answer, and Continue, Discuss, Do something else, and Pause controls.
- `P_question-single-use` [event] WHEN a pending question resolves, the controller SHALL consume it before any asynchronous mutation, so one action cannot be authorised twice.
- `P_discussion-preserves-question` [event] WHEN a human discusses a pending question, the controller SHALL retain that question rather than reopen or discard it.

### Key flows

- `P_scope-in-conversation` [ubiquitous] SCOPE SHALL run in the controlling conversation, invoking the grill-me and double-check skills there rather than delegating them to child sessions.
- `P_step-permission` [event] WHEN SCOPE advances to a drafting, interview, or review action, the controller SHALL hold explicit human permission naming that action or its group.
- `P_permission-not-durable` [ubiquitous] Step permission SHALL belong to the current continuation alone, so a pause or resume asks again while valid document approvals remain.
- `P_skips-recorded-honestly` [event] WHEN a human skips or repeats an action, the service SHALL record its actual disposition rather than report a skipped action as passed.
- `P_discussion-not-approval` [ubiquitous] Discussion and ambiguous agreement SHALL neither advance state nor approve a document.
- `P_human-input-only` [ubiquitous] Approval SHALL come only from Pi's interactive input path or its ask UI, so extension, worker, and RPC messages and model flags confer no authority.
- `P_approval-binds-revision` [event] WHEN a human approves a document, the service SHALL bind that approval to its exact byte revision and refuse a revision that changed after the question was asked.
- `P_plan-binds-issue` [event] WHEN a plan is approved, the service SHALL also bind the approved issue revision it implements.
- `P_independent-identities` [ubiquitous] Issue and plan numbers SHALL be reserved independently, so neither is derived from the other.
- `P_publication-scoped` [event] WHEN an approved document is published, the service SHALL commit only that document and its required generated indexes.
- `P_publication-recoverable` [event] WHEN publication is interrupted, the service SHALL verify the saved parent, paths, bytes, and branch before completing it, so a failure leaves approval absent rather than assumed.
- `P_unexplained-edit-suspends` [event] WHEN an approved document changes without assessment, the service SHALL suspend use of its approval until the actual diff is assessed.
- `P_formatting-diff-preserves` [event] WHEN an assessed diff changes formatting alone, the service SHALL retain the original human input and record the new compatible revision.
- `P_substantive-diff-clears` [event] WHEN an assessed diff changes meaning, the service SHALL clear the affected approvals while retaining recorded progress.
- `P_branch-switch-requested` [event] WHEN SCOPE begins on a branch other than the default, the service SHALL refuse and ask the human to switch rather than create a worktree or branch.

- `P_explicit-build-start` [event] WHEN BUILD starts, the service SHALL require separate human input, valid approvals, the expected branch, and a clean worktree.
- `P_branch-at-startup` [ubiquitous] The work branch SHALL be created at BUILD startup alone, so plan approval by itself starts nothing.
- `P_startup-recoverable` [event] WHEN branch creation is interrupted, the service SHALL recover only that authorised startup rather than adopt an independently existing branch.
- `P_one-worker` [ubiquitous] At most one worker SHALL execute per checkout.
- `P_worker-session-persists` [ubiquitous] A worker's session identity SHALL persist across stages, pauses, and restarts.
- `P_single-writer` [event] WHEN a worker is running, the service SHALL refuse controller writes to the checkout.
- `P_worker-has-no-authority` [ubiquitous] The worker SHALL hold no controller capability, so it can neither approve documents nor claim ownership.
- `P_worker-questions-routed` [event] WHEN the worker asks a question, the controller SHALL present it to the human and return the human's own control over their private channel.
- `P_control-not-invented` [event] WHEN a question is discussed, paused, or not put to the human, the controller SHALL report that outcome rather than deliver a fabricated answer.
- `P_stage-owns-instructions` [ubiquitous] Each stage turn SHALL carry its own checklist and the durable facts it rebuilds from, because a reset leaves no conversation to recover them from.
- `P_assessment-may-inspect` [ubiquitous] Review and acceptance SHALL keep the tools needed to inspect a candidate, including a shell, because an assessment that cannot run a diff or a check is made by reading alone.
- `P_assessment-bounded-by-state` [event] WHEN an assessment stage finishes, the controller SHALL compare the checkout's complete Git state with the state it started from.
- `P_changed-checkout-voids-assessment` [event] WHEN that comparison differs, or either state cannot be read, the controller SHALL record the assessment as void with its diagnostic rather than report a pass.
- `P_stage-reset` [event] WHEN a stage boundary is reached, the worker SHALL reset context to its own anchor so review and acceptance do not inherit the implementation conversation or its verdicts.
- `P_reset-on-request` [event] WHEN a completed block or a filling context requires it, BUILD SHALL be able to reset mid-stage as well as at a stage boundary.
- `P_reset-only-when-settled` [ubiquitous] Context reset SHALL occur only at a settled tool boundary.
- `P_current-mode-discloses` [event] WHEN execution runs in the controlling conversation, the controller SHALL disclose that no context reset is available rather than imply clean assessment context.
- `P_checkpoint-before-reset` [event] WHEN context is reset, the service SHALL already hold the stage, completed blocks, unfinished work, and evidence needed to resume.
- `P_progress-is-recordable` [event] WHEN a stage runs, its caller SHALL be able to record partial work and verification evidence durably, so a later reset or crash resumes from facts rather than from an empty checkpoint.
- `P_assessment-findings-durable` [event] WHEN an assessment stage finishes, its own report SHALL be saved with the progress it belongs to, because the conversation that received it does not survive a compaction, a reset, or a closed session.
- `P_assessment-report-bound` [ubiquitous] A saved report SHALL name the candidate it judged, so a reader can tell whether it still describes the current one.
- `P_findings-outlive-their-candidate` [event] WHEN a correction supersedes the candidate, the report SHALL be retained, because those findings are what the correction works from; only a newer report replaces it.
- `P_block-commit-bounded` [event] WHEN BUILD commits a work block, the service SHALL commit only paths inside that block's declared areas.
- `P_block-commit-refuses-strays` [event] WHEN changes reach outside a block's declared areas, the service SHALL refuse rather than absorb them.
- `P_block-committed-once` [event] WHEN a block is already recorded complete, the service SHALL refuse to record it again.
- `P_bounded-retries` [ubiquitous] Correction budgets SHALL be consumed durably, so a reset, pause, worker restart, or controller restart grants no further attempt.
- `P_complete-build-needs-clean` [event] WHEN BUILD completes, the service SHALL require a clean worktree before moving the candidate to ACCEPT.
- `P_accept-policy` [event] WHEN ACCEPT decides a candidate, the service SHALL derive human acceptance solely from project configuration.
- `P_absent-policy-refuses` [event] WHEN the acceptance policy cannot be read, the service SHALL refuse acceptance rather than treat it as automatic.
- `P_candidate-must-be-current` [event] WHEN the candidate changed after its assessment, the service SHALL refuse acceptance until it is reassessed.
- `P_accept-does-not-integrate` [event] WHEN acceptance succeeds, the service SHALL leave the work branch checked out without merging, pushing, releasing, or deleting it.
- `P_accept-routes-findings` [event] WHEN ACCEPT returns within-scope findings, the service SHALL return the workflow to BUILD and supersede the candidate so stale acceptance evidence cannot outlive the correction.
- `P_rescope-preserves-work` [event] WHEN changed intent returns a workflow to SCOPE, the service SHALL retain its branch, approvals, checkpoint, and consumed retries.
- `P_abandon-human-only` [event] WHEN abandonment is requested, the service SHALL require authenticated human input and a stated reason.
- `P_abandon-preserves-work` [event] WHEN abandonment closes a workflow, the service SHALL leave partial product work on its branch without merging or deleting it.
- `P_no-model-override` [ubiquitous] Abandonment SHALL exist only as a typed human command, absent from every tool, skill, and prompt the model reads.

### Cross-cutting concerns

- `P_records-are-local` [ubiquitous] A local record SHALL name its checkout, so a clone or copied record inherits no approval.
- `P_missing-state-refuses` [event] WHEN local state is absent, the service SHALL require human reconstruction rather than infer permission from document lifecycle, plan prose, or Git history.
- `P_unknown-version-refuses` [event] WHEN a local record names an unsupported version, the service SHALL preserve it and refuse rather than guess its meaning.
- `P_migration-preserves-legacy` [event] WHEN legacy ownership is migrated, the service SHALL archive its exact bytes before retirement and import reconstructed progress as unapproved inspection facts.
- `P_migration-refuses-live-writer` [event] WHEN a legacy writer is live or its liveness is unknown, the service SHALL refuse migration.
- `P_liveness-favours-incumbent` [ubiquitous] The service SHALL treat a writer whose liveness cannot be determined as still executing.
- `P_worker-shutdown-bounded` [event] WHEN a worker is stopped, the controller SHALL request an orderly stop, wait a bounded time, and then terminate its process tree.
- `P_disconnect-pauses` [event] WHEN the worker loses its controller, it SHALL let the running turn reach its own boundary within a bounded wait before ending, rather than cutting it off.
- `P_interrupted-is-not-complete` [event] WHEN a stage is interrupted, the controller SHALL report it as interrupted rather than as finished work.
- `P_git-preserves-unrelated` [ubiquitous] Automatic Git operations SHALL NOT stash, reset, discard, absorb, or implicitly resolve unrelated changes.
- `P_git-no-shell` [ubiquitous] Workflow Git operations SHALL validate refs and invoke Git with argument arrays without a shell.
- `P_transition-atomic` [ubiquitous] Publication, transition, and closure operations SHALL hold the repository workflow lock through their durable write.
- `P_state-injected-as-data` [ubiquitous] Before every controller turn, current workflow state SHALL be injected as data rather than recovered from conversation or compaction summaries.
- `P_transcripts-private` [ubiquitous] Worker transcripts SHALL stay in ignored local storage as recovery data, never as approval or progress authority.

## Stability

Internal and unreleased; the protocol is versioned so adapter incompatibility
fails explicitly.

- `P_versioned` [ubiquitous] Every machine-readable workflow response SHALL name `superdev-workflow/v3`.

<!-- sokf:links -->
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-052-the-workflow-is-scope-build-accept-under-a-durable-core]: /knowledge/adrs/deprecated/adr-052-the-workflow-is-scope-build-accept-under-a-durable-core.md
