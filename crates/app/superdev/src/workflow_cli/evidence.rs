//! Immutable candidate attestation and executable verification.
use super::*;

pub(super) fn record_evidence(root: &Path, args: &EvidenceArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        record_evidence_locked(root, args, transaction)
    })
}

pub(super) fn record_evidence_locked(
    root: &Path,
    args: &EvidenceArgs,
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
    cache::verify_authority(&state, &ui_authority_capability()?)?;
    let review_session = args.review_session.as_deref();
    if !matches!(args.kind, EvidenceKindName::Verification)
        && review_session.is_none_or(|review| review.trim().is_empty() || review == args.session)
    {
        return Err(Error::Manifest {
            message: "review evidence requires a distinct isolated reviewer session".into(),
        });
    }
    if matches!(args.kind, EvidenceKindName::Verification) && review_session.is_some() {
        return Err(Error::Manifest {
            message: "verification evidence does not accept a reviewer session".into(),
        });
    }
    validate_identity_values(root, &state.identity)?;
    git::require_clean(root)?;
    if git::current_branch(root)? != state.identity.work_branch {
        return Err(Error::Manifest {
            message: "evidence requires the checked-out work branch".into(),
        });
    }
    let record = plan_record(root, &state.identity.plan)?;
    let (path, observed) = plan_revision(root, &state.identity.plan)?;
    if matches!(args.kind, EvidenceKindName::ScopeReview) {
        let head = git::revision(root, "HEAD")?;
        if args.revision.as_deref() != Some(observed.as_str())
            || observed != args.expected_revision
            || args.candidate.as_deref() != Some(head.as_str())
        {
            return Err(Error::Manifest {
                message: "scope review must bind the checkpointed canonical plan and commit".into(),
            });
        }
        let scope_base = state
            .scope_base_revision
            .as_deref()
            .ok_or_else(|| Error::Manifest {
                message: "scope review requires its service-owned product baseline".into(),
            })?;
        git::require_knowledge_only_since(root, scope_base, &head)?;
        git::require_scope_checkpoint(root, &head, &path)?;
    } else if args.revision.is_some() || observed != args.expected_revision {
        return Err(Error::Manifest {
            message: "workflow plan revision changed".into(),
        });
    }
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let (lines, candidate, verified_default) = match args.kind {
        EvidenceKindName::ScopeReview if record.phase == "scope" => (
            vec![format!(
                "Scope requirements review: clean by isolated session {}.",
                review_session.expect("scope review session was checked")
            )],
            None,
            None,
        ),
        EvidenceKindName::Verification if record.phase == "build" => {
            let retry_state = parse_retry_state(build_state_line(&text)?)?;
            if retry_state.blocker.starts_with("final correction pending:")
                || retry_state
                    .blocker
                    .starts_with("final correction limit exhausted:")
            {
                return Err(Error::Manifest {
                    message:
                        "final-review findings require a correction checkpoint before verification"
                            .into(),
                });
            }
            let candidate = args.candidate.as_deref().ok_or_else(|| Error::Manifest {
                message: "verification evidence requires candidate H".into(),
            })?;
            let head = git::revision(root, &state.identity.work_branch)?;
            if candidate != head {
                return Err(Error::Manifest {
                    message: "verification candidate must be the clean work-branch tip".into(),
                });
            }
            let default = git::revision(root, &state.identity.default_branch)?;
            if !git::is_ancestor(root, &default, candidate)? {
                return Err(Error::Manifest {
                    message: "candidate H does not contain the default-branch tip".into(),
                });
            }
            run_plan_verification(root, &text, candidate)?;
            (Vec::new(), Some(candidate.to_string()), Some(default))
        }
        EvidenceKindName::Final if record.phase == "build" => {
            if text.contains("- [ ] Done") {
                return Err(Error::Manifest {
                    message: "final attestation requires every work block to be complete".into(),
                });
            }
            if knowledge_contains(root, &format!("PENDING ({})", state.identity.plan))? {
                return Err(Error::Manifest {
                    message: "final attestation refuses affected pending promises".into(),
                });
            }
            if primary_issue_has_unresolved_discoveries(root, &state.identity.issue)? {
                return Err(Error::Manifest {
                    message: "final attestation refuses unresolved primary-issue discoveries"
                        .into(),
                });
            }
            let candidate = args.candidate.as_deref().ok_or_else(|| Error::Manifest {
                message: "final review evidence requires candidate H".into(),
            })?;
            if state.candidate_revision.as_deref() != Some(candidate)
                || git::revision(root, &state.identity.work_branch)? != candidate
            {
                return Err(Error::Manifest {
                    message: "final review is not bound to current successful verification".into(),
                });
            }
            let default =
                state
                    .verified_default_revision
                    .clone()
                    .ok_or_else(|| Error::Manifest {
                        message: "final review is missing the verified default revision".into(),
                    })?;
            (
                vec![
                    format!("Candidate revision: {candidate}."),
                    format!("Verified default revision: {default}."),
                    format!("Final verification: passed for {candidate}."),
                    format!("Documentation verification: passed for {candidate}."),
                    format!(
                        "Final review: clean for {candidate} by isolated session {}.",
                        review_session.expect("final review session was checked")
                    ),
                ],
                Some(candidate.to_string()),
                Some(default),
            )
        }
        _ => {
            return Err(Error::Manifest {
                message: "evidence kind does not match the canonical phase".into(),
            });
        }
    };
    if matches!(args.kind, EvidenceKindName::Verification) {
        let state =
            transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
                state.candidate_revision.clone_from(&candidate);
                state
                    .verified_default_revision
                    .clone_from(&verified_default);
            })?;
        return emit("verification", &state);
    }
    let mut edits = vec![completion_evidence_edit(&text, &lines)?];
    let commit_message = if matches!(args.kind, EvidenceKindName::Final) {
        let mut retry_state = parse_retry_state(build_state_line(&text)?)?;
        if retry_state.blocker == "final correction awaiting review" {
            retry_state.final_corrections += 1;
            retry_state.blocker = "none".into();
            edits.push(ExactEdit {
                old_text: build_state_line(&text)?.into(),
                new_text: render_retry_state(&retry_state),
            });
        } else if retry_state.blocker != "none" {
            return Err(Error::Manifest {
                message: "BUILD cannot complete while a final correction is pending or exhausted"
                    .into(),
            });
        }
        edits.push(ExactEdit {
            old_text: "phase: build".into(),
            new_text: "phase: accept".into(),
        });
        "chore(workflow): attest build completion"
    } else {
        "chore(workflow): record canonical evidence"
    };
    apply_plan_edits_transactionally(root, &path, edits)?;
    git::commit_knowledge_changes(root, commit_message)?;
    let revision = plan_revision(root, &state.identity.plan)?.1;
    let state = transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
        state.last_plan_revision.clone_from(&revision);
        if candidate.is_some() {
            state.candidate_revision.clone_from(&candidate);
            state
                .verified_default_revision
                .clone_from(&verified_default);
        }
    })?;
    emit("evidence", &state)
}

pub(super) fn plan_verification_commands(plan: &str) -> Vec<String> {
    verification_commands(plan, false)
}

pub(super) fn verification_commands(plan: &str, include_final: bool) -> Vec<String> {
    plan.lines()
        .filter(|line| {
            line.trim_start().starts_with("- Verification:")
                || (include_final && line.trim_start().starts_with("- Final verification:"))
        })
        .flat_map(|line| {
            let mut commands = Vec::new();
            let mut rest = line;
            while let Some(open) = rest.find('`') {
                rest = &rest[open + 1..];
                let Some(close) = rest.find('`') else { break };
                let command = rest[..close].trim();
                if !command.is_empty() {
                    commands.push(command.to_string());
                }
                rest = &rest[close + 1..];
            }
            commands
        })
        .collect()
}

pub(super) fn run_verification_command(root: &Path, command: &str) -> Result<()> {
    #[cfg(unix)]
    let mut process = {
        let mut process = Command::new("sh");
        process.args(["-c", command]);
        process
    };
    #[cfg(windows)]
    let mut process = {
        let mut process = Command::new("cmd");
        process.args(["/C", command]);
        process
    };
    let output = process
        .current_dir(root)
        .env("SUPERDEV_VERIFICATION_ACTIVE", "1")
        .env_remove("SUPERDEV_UI_AUTHORITY")
        .stdin(Stdio::null())
        .output()
        .map_err(|source| Error::Command {
            command: command.into(),
            status: None,
            stderr: source.to_string(),
        })?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let diagnostics = format!("{stderr}\n{stdout}");
        let diagnostics = diagnostics.chars().rev().take(16_000).collect::<String>();
        let diagnostics = diagnostics.chars().rev().collect();
        Err(Error::Command {
            command: command.into(),
            status: output.status.code(),
            stderr: diagnostics,
        })
    }
}

pub(super) fn run_plan_verification(root: &Path, plan: &str, candidate: &str) -> Result<()> {
    if plan.contains("- [ ] Done") {
        return Err(Error::Manifest {
            message: "final verification requires every block to be complete".into(),
        });
    }
    let commands = verification_commands(plan, true);
    if commands.is_empty() {
        return Err(Error::Manifest {
            message: "final verification has no executable plan commands".into(),
        });
    }
    for command in commands {
        run_verification_command(root, &command)?;
        if git::revision(root, "HEAD")? != candidate {
            return Err(Error::Manifest {
                message: "a verification command changed candidate H".into(),
            });
        }
    }
    git::require_clean(root)
}
