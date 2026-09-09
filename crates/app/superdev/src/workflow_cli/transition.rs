//! Durable phase transitions and their evidence gates.
use super::*;

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
            TransitionName::RecoverStaleDefault => Transition::RecoverStaleDefault,
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
    let accept_build_feedback = if matches!(transition, Transition::ReturnToBuild) {
        Some(feedback.ok_or_else(|| Error::Manifest {
            message: "return to BUILD requires the complete ACCEPT finding set".into(),
        })?)
    } else {
        None
    };
    if feedback.is_some() && rescope_feedback.is_none() && accept_build_feedback.is_none() {
        return Err(Error::Manifest {
            message: "feedback is accepted only when routing findings to SCOPE or BUILD".into(),
        });
    }
    let plan_text =
        fs::read_to_string(plan_revision(root, &state.identity.plan)?.0).map_err(|source| {
            Error::Io {
                path: plan_revision(root, &state.identity.plan)
                    .expect("plan was just read")
                    .0,
                source,
            }
        })?;
    let mut scope_approval_parent = None;
    if matches!(transition, Transition::ApproveScope) {
        let scope_base = state
            .scope_base_revision
            .as_deref()
            .ok_or_else(|| Error::Manifest {
                message: "scope approval requires its service-owned product baseline".into(),
            })?;
        let head = git::revision(root, &state.identity.work_branch)?;
        git::require_knowledge_only_since(root, scope_base, &head)?;
        scope_approval_parent = Some(head);
    }
    let candidate_revision = evidence_revision(&plan_text, "Candidate revision");
    let verified_default_revision = evidence_revision(&plan_text, "Verified default revision");
    let config = Manifest::load(root)?.workflow;
    let human_authorized = matches!(
        transition,
        Transition::ApproveScope | Transition::RejectAcceptance | Transition::Abandon
    ) || matches!(transition, Transition::Accept)
        && config.human_acceptance_required;
    if human_authorized {
        cache::verify_authority(&state, &ui_authority_capability()?)?;
    }
    if matches!(transition, Transition::Accept) {
        let verified =
            state
                .verified_default_revision
                .as_deref()
                .ok_or_else(|| Error::Manifest {
                    message: "acceptance requires a verified default revision".into(),
                })?;
        if git::revision(root, &state.identity.default_branch)? != verified {
            return Err(Error::Manifest {
                message: "default branch advanced; return ACCEPT to BUILD before closure".into(),
            });
        }
    }
    if matches!(transition, Transition::RecoverStaleDefault) && phase == Phase::Accept {
        let verified =
            state
                .verified_default_revision
                .as_deref()
                .ok_or_else(|| Error::Manifest {
                    message: "stale-default recovery requires a verified default revision".into(),
                })?;
        if git::revision(root, &state.identity.default_branch)? == verified {
            return Err(Error::Manifest {
                message: "default branch still matches BUILD verification".into(),
            });
        }
    }
    let gates = GateEvidence {
        human_scope_approved: human_authorized && matches!(transition, Transition::ApproveScope),
        requirements_review_clean: plan_text
            .lines()
            .any(|line| line.starts_with("Scope requirements review: clean by isolated session ")),
        human_acceptance_approved: human_authorized && matches!(transition, Transition::Accept),
        human_abandonment_approved: human_authorized && abandon,
        closure_integrated: if phase == Phase::Done {
            git::is_ancestor(
                root,
                &git::revision(root, &state.identity.work_branch)?,
                &git::revision(root, &state.identity.default_branch)?,
            )?
        } else {
            false
        },
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
    if matches!(transition, Transition::ApproveScope) {
        edits.push(completion_evidence_edit(
            &plan_text,
            &["Human scope approval: approved.".into()],
        )?);
        let old_retry = build_state_line(&plan_text)?;
        let mut reset_retry = parse_retry_state(old_retry)?;
        reset_retry.final_corrections = 0;
        if reset_retry.blocker.starts_with("final correction ") {
            reset_retry.blocker = "none".into();
        }
        let new_retry = render_retry_state(&reset_retry);
        if old_retry != new_retry {
            edits.push(ExactEdit {
                old_text: old_retry.into(),
                new_text: new_retry,
            });
        }
    }
    let scope_baseline_revision = if matches!(
        transition,
        Transition::ReturnToScope | Transition::RejectAcceptance
    ) {
        Some(git::revision(root, &state.identity.work_branch)?)
    } else {
        None
    };
    let scope_baseline = scope_baseline_revision
        .as_ref()
        .map(|revision| format!("Scope product baseline: {revision}."));
    if matches!(transition, Transition::RejectAcceptance) {
        edits.push(invalidate_scope_evidence_edit(
            &plan_text,
            scope_baseline
                .as_ref()
                .expect("rejection has a scope baseline"),
            true,
        )?);
    } else if matches!(transition, Transition::ReturnToScope) {
        edits.push(invalidate_scope_evidence_edit(
            &plan_text,
            scope_baseline
                .as_ref()
                .expect("return has a scope baseline"),
            false,
        )?);
    } else if matches!(
        transition,
        Transition::RecoverStaleDefault | Transition::ReturnToBuild
    ) && phase == Phase::Accept
    {
        if let Some(feedback) = accept_build_feedback.as_ref() {
            let old_retry = build_state_line(&plan_text)?;
            let mut retry_state = parse_retry_state(old_retry)?;
            if retry_state.blocker != "none" {
                return Err(Error::Manifest {
                    message: "ACCEPT findings cannot replace an unresolved final correction".into(),
                });
            }
            let summary = retry::normalize_diagnostics(feedback).replace('\n', " | ");
            retry_state.blocker =
                if retry_state.final_corrections >= config.max_final_correction_cycles {
                    format!("final correction limit exhausted: {summary}")
                } else {
                    format!("final correction pending: {summary}")
                };
            edits.push(ExactEdit {
                old_text: old_retry.into(),
                new_text: render_retry_state(&retry_state),
            });
        }
        edits.push(if let Some(feedback) = accept_build_feedback {
            filter_completion_evidence(
                &plan_text,
                &[
                    "Candidate revision: ",
                    "Verified default revision: ",
                    "Final verification: ",
                    "Documentation verification: ",
                    "Final review: ",
                    "Pending ACCEPT correction: ",
                ],
                &[format!("Pending ACCEPT correction: {feedback}")],
            )?
        } else {
            invalidate_final_evidence_edit(&plan_text)?
        });
    }
    if matches!(next, Phase::Done | Phase::Abandoned) {
        edits.push(ExactEdit {
            old_text: "lifecycle: open".into(),
            new_text: format!("lifecycle: {}", phase_text(next)),
        });
    }
    // Validate immutable-candidate ancestry before writing closure records.
    // A rejected acceptance attempt must not leave a partially closed plan or
    // issue behind.
    if matches!(transition, Transition::Accept) {
        let candidate = state
            .candidate_revision
            .as_deref()
            .ok_or_else(|| Error::Manifest {
                message: "acceptance requires an immutable reviewed candidate H".into(),
            })?;
        let head = git::revision(root, &state.identity.work_branch)?;
        if !git::is_ancestor(root, candidate, &head)?
            || git::changed_paths(root, candidate, &head)?
                .iter()
                .any(|path| !administrative_path(path, &state.identity))
        {
            return Err(Error::Manifest {
                message: "candidate H changed outside administrative workflow records".into(),
            });
        }
    }
    if matches!(transition, Transition::RecoverStaleDefault) && phase == Phase::Done {
        reopen_stale_closure(root, &state.identity)?;
    } else if matches!(next, Phase::Done | Phase::Abandoned) {
        close_records(root, &state.identity, next, abandonment_reason)?;
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
        Transition::RecoverStaleDefault => "chore(workflow): reopen stale closure",
        Transition::Abandon => "chore(workflow): close abandoned work",
    };
    if let Some(parent) = &scope_approval_parent {
        git::commit_knowledge_changes_at(root, commit_message, parent)?;
    } else {
        git::commit_knowledge_changes(root, commit_message)?;
    }
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
        if matches!(transition, Transition::ApproveScope) {
            state.scope_base_revision = None;
        } else if let Some(scope_base) = &scope_baseline_revision {
            state.scope_base_revision = Some(scope_base.clone());
        }
        if let Some(candidate) = candidate_revision {
            state.candidate_revision = Some(candidate);
            state.verified_default_revision = verified_default_revision;
        } else if matches!(
            transition,
            Transition::ReturnToScope
                | Transition::ReturnToBuild
                | Transition::RecoverStaleDefault
                | Transition::RejectAcceptance
        ) {
            state.candidate_revision = None;
            state.verified_default_revision = None;
        }
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
