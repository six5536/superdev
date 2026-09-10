//! Durable phase transitions and their human-authority gates.
use super::*;

/// The recorded evidence line for a gate a human forced past open review
/// findings, or `None` when no override was requested.
///
/// Forcing exercises a legal transition without a precondition an adapter
/// added, so it is accepted only at a gate the human already holds and only
/// under the same interactive authority that gate demands.
fn override_note(
    args: &TransitionArgs,
    transition: Transition,
    human_authorized: bool,
) -> Result<Option<String>> {
    let Some(note) = args
        .override_note
        .as_deref()
        .map(str::trim)
        .filter(|note| !note.is_empty())
    else {
        return Ok(None);
    };
    if !human_authorized || !matches!(transition, Transition::ApproveScope | Transition::Accept) {
        return Err(Error::Manifest {
            message:
                "an override note is accepted only at a human-gated scope or acceptance transition"
                    .into(),
        });
    }
    let label = if matches!(transition, Transition::ApproveScope) {
        "Scope approval forced by the human"
    } else {
        "Acceptance forced by the human"
    };
    Ok(Some(format!(
        "{label}: {}",
        note.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    )))
}

pub(super) fn transition(
    root: &Path,
    args: &TransitionArgs,
    abandon: bool,
    abandonment_reason: Option<&str>,
) -> Result<u8> {
    cache::transaction(root, |transaction| {
        transition_locked(root, args, abandon, abandonment_reason, transaction)
    })
}

pub(super) fn transition_locked(
    root: &Path,
    args: &TransitionArgs,
    abandon: bool,
    abandonment_reason: Option<&str>,
    transaction: &mut cache::Transaction<'_>,
) -> Result<u8> {
    let state = transaction.load()?.ok_or_else(|| Error::Manifest {
        message: "workflow is unowned".into(),
    })?;
    if state.session_id != args.session || state.last_plan_revision != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow ownership or plan revision changed".into(),
        });
    }
    validate_identity_values(root, &state.identity)?;
    git::require_clean(root)?;
    if git::current_branch(root)? != state.identity.work_branch {
        return Err(Error::Manifest {
            message: format!(
                "workflow mutations require checked-out branch `{}`",
                state.identity.work_branch
            ),
        });
    }
    let record = plan_record(root, &state.identity.plan)?;
    let phase = phase(args.phase);
    if record.phase != phase_text(phase) {
        return Err(Error::Manifest {
            message: format!(
                "canonical plan phase is `{}`, not `{}`",
                record.phase,
                phase_text(phase)
            ),
        });
    }
    let transition = if abandon {
        Transition::Abandon
    } else {
        match args.transition {
            TransitionName::ApproveScope => Transition::ApproveScope,
            TransitionName::ReturnToScope => Transition::ReturnToScope,
            TransitionName::RejectAcceptance => Transition::RejectAcceptance,
            TransitionName::ReturnToBuild => Transition::ReturnToBuild,
            TransitionName::CompleteBuild => Transition::CompleteBuild,
            TransitionName::Accept => Transition::Accept,
        }
    };
    let feedback = args
        .feedback
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let rescope_feedback = if matches!(
        transition,
        Transition::ReturnToScope | Transition::RejectAcceptance
    ) {
        Some(feedback.ok_or_else(|| Error::Manifest {
            message: "return to SCOPE requires verbatim discovery or rejection feedback".into(),
        })?)
    } else {
        None
    };
    if matches!(transition, Transition::ReturnToBuild) && feedback.is_none() {
        return Err(Error::Manifest {
            message: "return to BUILD requires the complete ACCEPT finding set".into(),
        });
    }
    if feedback.is_some()
        && rescope_feedback.is_none()
        && !matches!(transition, Transition::ReturnToBuild)
    {
        return Err(Error::Manifest {
            message: "feedback is accepted only when routing findings to SCOPE or BUILD".into(),
        });
    }
    let config = Manifest::load(root)?.workflow;
    // Only a human may approve scope, accept under policy, reject an
    // acceptance, or abandon. Everything else is the caller's own judgement.
    let human_authorized = matches!(
        transition,
        Transition::ApproveScope | Transition::RejectAcceptance | Transition::Abandon
    ) || matches!(transition, Transition::Accept)
        && config.human_acceptance_required;
    if human_authorized {
        cache::verify_authority(&state, &ui_authority_capability()?)?;
    }
    let override_note = override_note(args, transition, human_authorized)?;
    let gates = GateEvidence {
        human_scope_approved: human_authorized && matches!(transition, Transition::ApproveScope),
        human_acceptance_approved: human_authorized && matches!(transition, Transition::Accept),
        human_abandonment_approved: human_authorized && abandon,
    };
    let next =
        apply_transition(phase, transition, &gates, &config).map_err(|error| Error::Manifest {
            message: error.to_string(),
        })?;
    let (path, observed) = plan_revision(root, &state.identity.plan)?;
    if observed != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow plan revision changed".into(),
        });
    }
    let mut edits = vec![ExactEdit {
        old_text: format!("phase: {}", phase_text(phase)),
        new_text: format!("phase: {}", phase_text(next)),
    }];
    if matches!(next, Phase::Done | Phase::Abandoned) {
        edits.push(ExactEdit {
            old_text: "lifecycle: open".into(),
            new_text: format!("lifecycle: {}", phase_text(next)),
        });
    }
    // A forced gate differs from a clean one, so the difference is recorded
    // where the later phases read it rather than left in session memory.
    if let Some(note) = override_note.as_deref()
        && !matches!(next, Phase::Done | Phase::Abandoned)
    {
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        edits.push(completion_evidence_edit(&text, &[note.to_owned()])?);
    }
    if matches!(next, Phase::Done | Phase::Abandoned) {
        close_records(
            root,
            &state.identity,
            next,
            abandonment_reason,
            override_note.as_deref(),
        )?;
    } else if let Some(feedback) = rescope_feedback {
        let issue_path = root
            .join("knowledge/issues/open")
            .join(format!("{}.md", state.identity.issue));
        let issue_text = fs::read_to_string(&issue_path).map_err(|source| Error::Io {
            path: issue_path.clone(),
            source,
        })?;
        apply_record_edits_transactionally(
            root,
            vec![
                (&path, edits),
                (
                    &issue_path,
                    vec![rescope_discovery_edit(
                        &issue_text,
                        if matches!(transition, Transition::RejectAcceptance) {
                            "ACCEPT rejection"
                        } else {
                            "BUILD discovery"
                        },
                        feedback,
                    )?],
                ),
            ],
        )?;
    } else {
        apply_plan_edits_transactionally(root, &path, edits)?;
    }
    let commit_message = match transition {
        Transition::ApproveScope => "chore(workflow): approve scope",
        Transition::ReturnToScope | Transition::RejectAcceptance => {
            "chore(workflow): return to scope"
        }
        Transition::RecordBuildProgress => "chore(workflow): record build progress",
        Transition::CompleteBuild => "chore(workflow): complete build",
        Transition::ReturnToBuild => "chore(workflow): return acceptance findings to build",
        Transition::Accept => "chore(workflow): close accepted work",
        Transition::Abandon => "chore(workflow): close abandoned work",
    };
    git::commit_knowledge_changes(root, commit_message)?;
    if matches!(transition, Transition::Abandon) {
        abandonment::publish_abandonment(
            root,
            &state.identity.default_branch,
            &state.identity.issue,
            &state.identity.plan,
        )?;
    }
    let revision = plan_revision(root, &state.identity.plan)?.1;
    let state = transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
        state.last_plan_revision.clone_from(&revision);
    })?;
    if matches!(transition, Transition::Abandon | Transition::Accept) {
        transaction.release(&args.session)?;
    }
    emit(
        "transition",
        &serde_json::json!({
            "phase": phase_text(next),
            "state": state,
            "ownershipReleased": matches!(transition, Transition::Abandon | Transition::Accept),
        }),
    )
}
