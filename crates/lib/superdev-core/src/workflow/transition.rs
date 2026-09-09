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
