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
    /// Owning Pi process ID while an isolated child is active.
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
        Abandon, Accept as AcceptTransition, ApproveScope, CompleteBuild, RecordBuildProgress,
        RejectAcceptance, ReturnToBuild, ReturnToScope,
    };

    match (phase, transition) {
        (Scope, ApproveScope) => {
            require(gates.human_scope_approved, "scope approval is absent")?;
            Ok(Build)
        }
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
    fn scope_requires_human_approval() {
        let mut gates = GateEvidence::default();
        assert!(
            apply_transition(
                Phase::Scope,
                Transition::ApproveScope,
                &gates,
                &config(true)
            )
            .is_err()
        );
        gates.human_scope_approved = true;
        assert_eq!(
            apply_transition(
                Phase::Scope,
                Transition::ApproveScope,
                &gates,
                &config(true)
            ),
            Ok(Phase::Build)
        );
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

- `P_rust-authority` [ubiquitous] The Rust workflow service SHALL validate every durable phase transition, plan revision comparison, ownership comparison, and automatic Git operation.
- `P_pi-orchestrates` [ubiquitous] Pi SHALL orchestrate discoverable scope, build, and accept skills through typed tools while withholding direct shell and Git access from children and resolving BUILD's executable and digest together at invocation time.
- `P_extension-skills` [ubiquitous] The Superdev extension SHALL register its bundled file, scope, build, and accept skills from `.pi/extensions/superdev/skills/` through Pi's `resources_discover` event on startup and reload, exposing native `/skill:*` commands without copies in `.pi/skills/`.
- `P_service-snapshot` [event] WHEN Pi initializes its workflow service, the adapter SHALL bootstrap the trusted launcher in place and retain a private native executable snapshot independent of subsequent checkout switches and rebuilds.
- `P_baseline-compatible` [event] WHEN a SCOPE baseline caller omits its optional expected work tip, Rust SHALL resolve the owned work-branch tip under the repository lock while retaining session, plan-revision, phase, and checkout checks. Explicit expected tips retain compare-and-swap rejection.
- `P_skill-cold-start` [event] WHEN a workflow skill starts, the skill SHALL reconstruct canonical identity and phase before mutation.
- `P_questions-persisted` [event] WHEN review requires intent, Pi SHALL persist post-transition revision-bound questions and confirmed answers for one batched correction and re-review.
- `P_questions-ui` [event] WHEN a skill asks a question, Pi SHALL offer concrete choices, a recommendation, a typed answer, and chat discussion through the public typed tool.
- `P_typed-role-result` [ubiquitous] Isolated roles SHALL terminate with one validated bounded typed result and exhaustive reviewer checklist.
- `P_isolated-trace` [ubiquitous] Pi SHALL retain bounded isolated output and diagnostics in private temporary artifacts with their paths reported on failure.

### Key flows

- `P_scope-gate` [event] WHEN SCOPE enters BUILD, the service SHALL require explicit human scope approval and a clean isolated requirements review.
- `P_build-gate` [event] WHEN BUILD enters ACCEPT, the service SHALL require complete blocks, current executable and documentation evidence, no affected pending promise, and a clean fresh isolated final review.
- `P_accept-policy` [event] WHEN ACCEPT decides a candidate, the service SHALL derive human acceptance solely from project configuration before committing accepted closure on the work branch and releasing ownership without merging.
- `P_accept-assessment` [event] WHEN ACCEPT assesses reviewed candidate H, the service SHALL validate the current plan revision and allow only administrative record differences between H and the clean checked-out attestation snapshot.
- `P_accept-routes-findings` [event] WHEN ACCEPT reports findings, Pi SHALL automatically return within-scope corrections to BUILD and route intent-changing findings to human SCOPE discussion.
- `P_scope-bootstrap` [event] WHEN a new workflow starts, the service SHALL validate and commit the initial issue and independently numbered plan on the default branch under the repository lock before switching the shared checkout to the work branch.
- `P_identity-reservation` [event] WHEN an issue or initial plan reserves a numeric identity, the service SHALL refuse a number already held by another canonical identity in the local repository.
- `P_default-recovery` [event] WHEN a workflow resumes, the service SHALL recover the recorded default branch and current plan from its work-branch snapshot before binding the shared checkout.
- `P_one-checkout-owner` [ubiquitous] The service SHALL permit only one executing workflow owner per checkout.
- `P_issue-capture` [event] WHEN `/skill:file` captures an issue or idea, the skill SHALL direct the LLM to choose an unused number, author the schema-conforming record and index entry, validate, and commit only those paths on the discovered default branch without pausing active work.
- `P_file-skill-only` [ubiquitous] Pi SHALL expose capture only through the native `file` skill, without issue or file command aliases, a dedicated filing tool, or a filing child role.
- `P_file-skill-worktree` [event] WHEN capture starts outside the default branch, the skill SHALL direct the LLM to use an existing or temporary default-branch worktree through ordinary tools while preserving the caller's branch, pending edits, and workflow ownership.
- `P_accept-stale-default` [event] WHEN the verified default revision advances before closure, the service SHALL invalidate final evidence and return the same plan to BUILD without preparing DONE records.
- `P_ui-authority-service` [event] WHEN scope approval, configured human acceptance, rejection, or abandonment changes durable state, the service SHALL require the owning Pi UI's unpersisted capability.
- `P_ui-authority-adapter` [event] WHEN an action requires human authority, Pi SHALL expose its capability to the service only after interactive confirmation.
- `P_cancel-pauses` [event] WHEN cancellation occurs, the service SHALL release transient ownership without changing the canonical phase or deleting uncommitted SCOPE drafts.
- `P_abandon-human-only` [event] WHEN abandonment is requested, the service SHALL require interactive human approval while excluding partial product work from integration.
- `P_abandon-default-records` [event] WHEN abandonment closes the workflow, the service SHALL publish the closed issue and plan in a detached worktree and compare-and-swap the local default ref without carrying work-branch product history.
- `P_evidence-durable` [event] WHEN a Pi-bound isolated scope review or final BUILD review completes cleanly, the evidence command SHALL require UI authority and record the extension-issued single-use review run plus immutable revisions in canonical Completion evidence.
- `P_scope-publication` [event] WHEN isolated SCOPE preparation completes, a SCOPE checkpoint SHALL reject every product change after the service-owned SCOPE baseline, validate the complete knowledge snapshot, and publish an immutable knowledge-only checkpoint for review even when the plan is unchanged or no files changed. An unchanged snapshot uses an empty administrative commit without editing the plan; review evidence binds the canonical plan revision and that checkpoint commit.
- `P_attestation-atomic` [event] WHEN verified BUILD receives a clean final review, the evidence command SHALL record candidate-bound evidence and enter ACCEPT in one administrative attestation commit.
- `P_discoveries-resolved` [event] WHEN BUILD requests final attestation, the service SHALL refuse any unchecked discovery on the primary issue.
- `P_build-synchronizes-default` [event] WHEN BUILD finalizes a candidate, the service SHALL compare expected default and work tips, prove a conflict-free merge without touching the worktree, and incorporate the default through a hook-free fast-forward before verification.
- `P_verification-executed` [event] WHEN all blocks are complete and BUILD requests verification evidence, the service SHALL execute approved Verification and Final verification commands without UI authority and reject failures, candidate movement, or a dirty result.
- `P_verification-staged` [event] WHEN BUILD checkpoints a block, the service SHALL execute that block's focused Verification commands without executing the separate Final verification entries declared for complete suites.
- `P_final-correction-service` [event] WHEN immutable final review reports findings, the service SHALL schedule a bounded correction, invalidate candidate evidence, require a checkpoint within approved Areas, and count the cycle after correction and complete valid re-review.
- `P_final-correction-adapter` [event] WHEN a final correction remains within the configured limit, Pi SHALL schedule correction, verification, and a fresh immutable review.
- `P_gates-derived` [ubiquitous] Phase transitions SHALL calculate non-human gates from canonical evidence and repository state rather than caller-provided boolean flags.
- `P_resume-recovers-evidence` [event] WHEN ownership resumes, the service SHALL reconstruct the SCOPE product baseline from service-owned Git transition history and candidate and verified-default revisions from canonical Completion evidence rather than treating cache loss as evidence loss.
- `P_closure-transactional` [event] WHEN acceptance or abandonment closes records, the service SHALL stage, repair, and validate the complete knowledge closure before publishing it.
- `P_service-owned-commits` [event] WHEN canonical evidence or a durable transition is published after a clean-tree preflight, the service SHALL create a knowledge-only commit through an isolated index without invoking hooks or signing, leaving the live index, worktree, and HEAD unchanged if commit construction fails.
- `P_rescope-preserves-identity` [event] WHEN BUILD returns a discovery to SCOPE, the service SHALL preserve it verbatim on the primary issue, retain the same issue, plan, branch, and stable block numbers, invalidate prior scope approval evidence and transient candidate attestations, and permit only knowledge changes after the recorded product baseline until fresh review and approval.
- `P_rejection-preserves-feedback` [event] WHEN a human rejects a candidate, the service SHALL return the same plan to SCOPE, preserve the verbatim feedback as an unresolved primary-issue discovery, invalidate prior scope approval and final evidence, and record the rejected product baseline.
- `P_rejection-invalidates-final` [event] WHEN ACCEPT returns to SCOPE, the service SHALL remove the rejected candidate's verification and review attestations.
- `P_transition-atomic` [ubiquitous] Evidence, transition, reopening, closure, and integration operations SHALL hold the repository workflow lock through canonical publication and the ownership compare-and-swap.

### Cross-cutting concerns

- `P_git-preserves-unrelated` [ubiquitous] Automatic Git operations SHALL NOT stash, reset, discard, absorb, or implicitly resolve unrelated changes.
- `P_git-no-shell` [ubiquitous] Workflow Git operations SHALL validate refs and invoke Git with argument arrays without a shell.
- `P_bounded-retries` [ubiquitous] Retry and correction limits SHALL be positive project configuration values that plans, prompts, adapters, and models cannot override.
- `P_failure-fingerprint` [event] WHEN BUILD records a failed command, the service SHALL normalize bounded diagnostics and durably count consecutive equivalent fingerprints against the configured limit.
- `P_retry-reset` [event] WHEN BUILD checkpoints its newly completed current stable block, the service SHALL reject caller edits to retry state, reset that block's attempts and fingerprint, preserve final-correction accounting, and advance to the next incomplete stable block when one exists.
- `P_block-checkpoint` [event] WHEN BUILD checkpoints a newly completed stable block, the service SHALL reject changes to its SCOPE-approved dependencies, path-scoped Areas, or executable Verification commands, execute that approved verification, and commit only changes within the approved Areas plus the owning plan.
- `P_cache-transient` [ubiquitous] Absence of `.superdev/cache/workflow.toml` SHALL mean unowned rather than complete.
- `P_status-during-transaction` [event] WHEN observational status encounters a held workflow transaction, it SHALL return an explicit busy snapshot without waiting or representing that state as unowned.
- `P_compaction-reloads-state` [ubiquitous] Before every parent agent turn, Pi SHALL reload canonical phase, identity, plan revision, and BUILD state from Rust into the turn context rather than rely on conversation or compaction summaries.

## Stability

Internal and unreleased; the protocol is versioned so adapter incompatibility
fails explicitly.

- `P_versioned` [ubiquitous] Every machine-readable workflow response SHALL name `superdev-workflow/v2`.

<!-- sokf:links -->
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-052-the-workflow-is-scope-build-accept-under-a-durable-core]: /knowledge/adrs/deprecated/adr-052-the-workflow-is-scope-build-accept-under-a-durable-core.md
