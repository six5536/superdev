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
        Abandon, Accept as AcceptTransition, ApproveScope, FinishBuild, RecordBuildProgress,
        RecoverStaleDefault, RejectAcceptance, ReturnToScope,
    };

    match (phase, transition) {
        (Scope, ApproveScope) => {
            require(gates.human_scope_approved, "scope approval is absent")?;
            require(
                gates.requirements_review_clean,
                "requirements review is not clean",
            )?;
            Ok(Build)
        }
        (Build, ReturnToScope) => Ok(Scope),
        (Build, RecordBuildProgress) => Ok(Build),
        (Build, FinishBuild) => {
            require(gates.all_blocks_complete, "work blocks are incomplete")?;
            require(
                gates.final_verification_current,
                "final verification is stale or absent",
            )?;
            require(gates.final_review_clean, "final review is not clean")?;
            require(
                gates.no_pending_promises,
                "affected contract promises remain PENDING",
            )?;
            require(
                gates.documentation_current,
                "documentation evidence is stale or absent",
            )?;
            Ok(Accept)
        }
        (Accept, RejectAcceptance) => Ok(Scope),
        (Accept, AcceptTransition) => {
            if config.human_acceptance_required {
                require(
                    gates.human_acceptance_approved,
                    "human acceptance is required",
                )?;
            }
            Ok(Done)
        }
        (Done, RecoverStaleDefault) if !gates.closure_integrated => Ok(Build),
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
            max_stalled_block_attempts: 3,
            max_final_correction_cycles: 3,
        }
    }

    #[test]
    fn scope_requires_both_human_approval_and_clean_review() {
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
        assert!(
            apply_transition(
                Phase::Scope,
                Transition::ApproveScope,
                &gates,
                &config(true)
            )
            .is_err()
        );
        gates.requirements_review_clean = true;
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
    fn finish_build_requires_all_executable_evidence() {
        let complete = GateEvidence {
            all_blocks_complete: true,
            final_verification_current: true,
            final_review_clean: true,
            no_pending_promises: true,
            documentation_current: true,
            ..GateEvidence::default()
        };
        assert_eq!(
            apply_transition(
                Phase::Build,
                Transition::FinishBuild,
                &complete,
                &config(true)
            ),
            Ok(Phase::Accept)
        );
        for incomplete in [
            GateEvidence {
                all_blocks_complete: false,
                ..complete.clone()
            },
            GateEvidence {
                final_review_clean: false,
                ..complete.clone()
            },
            GateEvidence {
                documentation_current: false,
                ..complete.clone()
            },
        ] {
            assert!(
                apply_transition(
                    Phase::Build,
                    Transition::FinishBuild,
                    &incomplete,
                    &config(true)
                )
                .is_err()
            );
        }
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
