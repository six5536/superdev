//! Block checkpoints, scope publication, and correction accounting.
use super::*;

pub(super) fn record_scope_baseline(root: &Path, args: &ScopeBaselineArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        if owner.session_id != args.session || owner.last_plan_revision != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow ownership or plan revision changed".into(),
            });
        }
        validate_identity_values(root, &owner.identity)?;
        if git::current_branch(root)? != owner.identity.work_branch
            || plan_record(root, &owner.identity.plan)?.phase != "scope"
        {
            return Err(Error::Manifest {
                message: "SCOPE baseline requires the checked-out SCOPE work branch".into(),
            });
        }
        git::require_knowledge_only_worktree(root)?;
        let baseline = git::revision(root, &owner.identity.work_branch)?;
        if args
            .expected_work
            .as_ref()
            .is_some_and(|expected| expected != &baseline)
        {
            return Err(Error::Manifest {
                message: "work branch moved before the SCOPE baseline was recorded".into(),
            });
        }
        let state =
            transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
                state.scope_base_revision = Some(baseline.clone())
            })?;
        emit(
            "scope-baseline",
            &serde_json::json!({"baseline": baseline, "state": state}),
        )
    })
}

pub(super) fn record_scope_checkpoint(root: &Path, args: &RevisionArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        if owner.session_id != args.session || owner.last_plan_revision != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow ownership or plan revision changed".into(),
            });
        }
        validate_identity_values(root, &owner.identity)?;
        if git::current_branch(root)? != owner.identity.work_branch {
            return Err(Error::Manifest {
                message: "SCOPE checkpoint requires the checked-out work branch".into(),
            });
        }
        if plan_record(root, &owner.identity.plan)?.phase != "scope" {
            return Err(Error::Manifest {
                message: "SCOPE checkpoints are permitted only during SCOPE".into(),
            });
        }
        let scope_base = owner
            .scope_base_revision
            .as_deref()
            .ok_or_else(|| Error::Manifest {
                message: "SCOPE checkpoint requires its service-owned product baseline".into(),
            })?;
        let head = git::revision(root, &owner.identity.work_branch)?;
        git::require_knowledge_only_since(root, scope_base, &head)?;
        let (_, observed) = plan_revision(root, &owner.identity.plan)?;
        if observed == args.expected_revision {
            return Err(Error::Manifest {
                message: "SCOPE checkpoint requires a changed canonical plan".into(),
            });
        }
        let grammar = superdev_core::validate::schema::load_grammar(root)?;
        let report =
            superdev_core::validate::validate_repo(root, &root.join("knowledge"), &[], &grammar)?;
        if !report.report.passed() {
            return Err(Error::Manifest {
                message: "SCOPE checkpoint requires valid canonical knowledge".into(),
            });
        }
        let parent = head;
        let commit =
            git::commit_knowledge_changes_at(root, git::SCOPE_CHECKPOINT_MESSAGE, &parent)?;
        let revision = plan_revision(root, &owner.identity.plan)?.1;
        let state =
            match transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
                state.last_plan_revision.clone_from(&revision)
            }) {
                Ok(state) => state,
                Err(error) => {
                    git::rollback_commit(root, &commit, &parent)?;
                    return Err(error);
                }
            };
        emit(
            "scope-checkpoint",
            &serde_json::json!({"commit": commit, "revision": revision, "state": state}),
        )
    })
}

pub(super) fn record_attempt(root: &Path, args: &AttemptArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        if owner.session_id != args.session || owner.last_plan_revision != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow ownership or plan revision changed".into(),
            });
        }
        validate_identity_values(root, &owner.identity)?;
        git::require_clean(root)?;
        if git::current_branch(root)? != owner.identity.work_branch {
            return Err(Error::Manifest {
                message: "BUILD attempt requires the checked-out work branch".into(),
            });
        }
        let record = plan_record(root, &owner.identity.plan)?;
        if record.phase != "build" {
            return Err(Error::Manifest {
                message: "failed attempts may be recorded only during BUILD".into(),
            });
        }
        if args.command.trim().is_empty()
            || args.exit_status == 0
            || args.diagnostics.trim().is_empty()
        {
            return Err(Error::Manifest {
                message: "failed attempt requires command, nonzero status, and diagnostics".into(),
            });
        }
        let (path, observed) = plan_revision(root, &owner.identity.plan)?;
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let old_line = build_state_line(&text)?;
        let current = parse_retry_state(old_line)?;
        let config = Manifest::load(root)?.workflow;
        let fingerprint =
            retry::failure_fingerprint(&args.command, args.exit_status, &args.diagnostics);
        if current.fingerprint.as_deref() == Some(&fingerprint)
            && current.attempts >= config.max_stalled_block_attempts
        {
            return Err(Error::Manifest {
                message: "equivalent BUILD failure exhausted the configured attempt limit".into(),
            });
        }
        let next = retry::record_failure(
            &current,
            &args.command,
            args.exit_status,
            &args.diagnostics,
            config.max_stalled_block_attempts,
        );
        let new_line = render_retry_state(&next);
        apply_plan_edits_transactionally(
            root,
            &path,
            vec![ExactEdit {
                old_text: old_line.into(),
                new_text: new_line,
            }],
        )?;
        git::commit_knowledge_changes(root, "chore(workflow): record failed build attempt")?;
        let revision = plan_revision(root, &owner.identity.plan)?.1;
        let state = transaction.compare_and_swap(&args.session, &observed, |state| {
            state.last_plan_revision.clone_from(&revision);
        })?;
        emit(
            "attempt",
            &serde_json::json!({
                "state": state,
                "attempts": next.attempts,
                "fingerprint": next.fingerprint,
                "stalled": next.attempts >= config.max_stalled_block_attempts,
            }),
        )
    })
}

pub(super) fn record_correction(root: &Path, args: &CorrectionArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        if owner.session_id != args.session || owner.last_plan_revision != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow ownership or plan revision changed".into(),
            });
        }
        cache::verify_authority(&owner, &ui_authority_capability()?)?;
        validate_identity_values(root, &owner.identity)?;
        git::require_clean(root)?;
        if git::current_branch(root)? != owner.identity.work_branch {
            return Err(Error::Manifest {
                message: "final correction requires the checked-out work branch".into(),
            });
        }
        if plan_record(root, &owner.identity.plan)?.phase != "build" {
            return Err(Error::Manifest {
                message: "final corrections may be recorded only during BUILD".into(),
            });
        }
        if args.review_session.trim().is_empty() || args.review_session == args.session {
            return Err(Error::Manifest {
                message: "final correction requires a distinct isolated reviewer run".into(),
            });
        }
        if owner.candidate_revision.as_deref() != Some(args.candidate.as_str())
            || git::revision(root, &owner.identity.work_branch)? != args.candidate
        {
            return Err(Error::Manifest {
                message: "final correction is not bound to the verified candidate".into(),
            });
        }
        let summary = retry::normalize_diagnostics(&args.summary).replace('\n', " | ");
        if summary.is_empty() {
            return Err(Error::Manifest {
                message: "final correction requires bounded structured findings".into(),
            });
        }
        let (path, observed) = plan_revision(root, &owner.identity.plan)?;
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let mut state = parse_retry_state(build_state_line(&text)?)?;
        let config = Manifest::load(root)?.workflow;
        if state.blocker == "final correction awaiting review" {
            state.final_corrections += 1;
        } else if state.blocker != "none" {
            return Err(Error::Manifest {
                message: "the previous final-correction cycle is not ready for review accounting"
                    .into(),
            });
        }
        state.blocker = if state.final_corrections >= config.max_final_correction_cycles {
            format!("final correction limit exhausted: {summary}")
        } else {
            format!("final correction pending: {summary}")
        };
        apply_plan_edits_transactionally(
            root,
            &path,
            vec![
                ExactEdit {
                    old_text: build_state_line(&text)?.into(),
                    new_text: render_retry_state(&state),
                },
                invalidate_final_evidence_edit(&text)?,
            ],
        )?;
        git::commit_knowledge_changes(root, "chore(workflow): record final correction")?;
        let revision = plan_revision(root, &owner.identity.plan)?.1;
        let cache = transaction.compare_and_swap(&args.session, &observed, |cache| {
            cache.last_plan_revision.clone_from(&revision);
            cache.candidate_revision = None;
            cache.verified_default_revision = None;
        })?;
        emit(
            "correction",
            &serde_json::json!({
                "state": cache,
                "finalCorrections": state.final_corrections,
                "stalled": state.final_corrections >= config.max_final_correction_cycles,
            }),
        )
    })
}

pub(super) fn record_correction_checkpoint(root: &Path, args: &RevisionArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        if owner.session_id != args.session || owner.last_plan_revision != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow ownership or plan revision changed".into(),
            });
        }
        validate_identity_values(root, &owner.identity)?;
        if git::current_branch(root)? != owner.identity.work_branch {
            return Err(Error::Manifest {
                message: "final correction checkpoint requires the checked-out work branch".into(),
            });
        }
        if plan_record(root, &owner.identity.plan)?.phase != "build" {
            return Err(Error::Manifest {
                message: "final correction checkpoints are permitted only during BUILD".into(),
            });
        }
        let (plan_path, observed) = plan_revision(root, &owner.identity.plan)?;
        if observed != args.expected_revision {
            return Err(Error::Manifest {
                message: "workflow plan revision changed".into(),
            });
        }
        let text = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
            path: plan_path.clone(),
            source,
        })?;
        let retry_state = parse_retry_state(build_state_line(&text)?)?;
        if !retry_state.blocker.starts_with("final correction pending:") {
            return Err(Error::Manifest {
                message: "no failed final gate is awaiting an implementation correction".into(),
            });
        }
        let mut areas = Vec::new();
        let mut block = 1;
        while let Ok(body) = plan_block(&text, block) {
            areas.extend(block_area_paths(body));
            block += 1;
        }
        areas.sort();
        areas.dedup();
        git::validate_block_paths(root, &areas)?;
        let commands = verification_commands(&text, true);
        if commands.is_empty() {
            return Err(Error::Manifest {
                message: "final correction checkpoint has no executable Verification commands"
                    .into(),
            });
        }
        let head = git::revision(root, "HEAD")?;
        for command in commands {
            run_verification_command(root, &command)?;
            if git::revision(root, "HEAD")? != head {
                return Err(Error::Manifest {
                    message: "a final correction Verification command changed HEAD".into(),
                });
            }
        }
        let relative_plan = plan_path
            .strip_prefix(root)
            .map_err(|_| Error::Manifest {
                message: "workflow plan is outside the repository".into(),
            })?
            .to_string_lossy()
            .to_string();
        let mut corrected_state = retry_state.clone();
        corrected_state.blocker = "final correction awaiting review".into();
        apply_plan_edits_transactionally(
            root,
            &plan_path,
            vec![ExactEdit {
                old_text: build_state_line(&text)?.into(),
                new_text: render_retry_state(&corrected_state),
            }],
        )?;
        areas.push(relative_plan);
        let commit = git::commit_block_changes(
            root,
            &format!(
                "fix(workflow): checkpoint final correction {}",
                retry_state.final_corrections + 1
            ),
            &areas,
        )?;
        let revision = plan_revision(root, &owner.identity.plan)?.1;
        let state = transaction.compare_and_swap(&args.session, &observed, |state| {
            state.last_plan_revision.clone_from(&revision)
        })?;
        emit(
            "correction-checkpoint",
            &serde_json::json!({"commit": commit, "revision": revision, "state": state}),
        )
    })
}

pub(super) fn plan_block(plan: &str, block: u32) -> Result<&str> {
    let heading = format!("### Block {block}: ");
    let start = plan.find(&heading).ok_or_else(|| Error::Manifest {
        message: format!("workflow plan has no stable block {block}"),
    })?;
    let body = &plan[start..];
    let end = body[heading.len()..]
        .find("\n### Block ")
        .map(|offset| heading.len() + offset)
        .or_else(|| {
            body[heading.len()..]
                .find("\n## ")
                .map(|offset| heading.len() + offset)
        })
        .unwrap_or(body.len());
    Ok(&body[..end])
}

pub(super) fn block_is_done(plan: &str, block: u32) -> bool {
    plan_block(plan, block).is_ok_and(|body| {
        body.lines()
            .any(|line| matches!(line, "- [x] Done." | "- [X] Done."))
    })
}

pub(super) fn block_dependencies(block: &str) -> Vec<u32> {
    block
        .lines()
        .find(|line| line.starts_with("- Dependencies:"))
        .into_iter()
        .flat_map(|line| line.split(|character: char| !character.is_ascii_digit()))
        .filter_map(|value| value.parse().ok())
        .collect()
}

pub(super) fn block_area_paths(block: &str) -> Vec<String> {
    let Some(line) = block.lines().find(|line| line.starts_with("- Areas:")) else {
        return Vec::new();
    };
    let mut paths = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('`') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('`') else { break };
        let path = rest[..close].trim().trim_end_matches('/');
        if !path.is_empty() {
            paths.push(path.to_string());
        }
        rest = &rest[close + 1..];
    }
    paths
}

pub(super) fn run_block_verification(root: &Path, block: &str) -> Result<()> {
    let commands = plan_verification_commands(block);
    if commands.is_empty() {
        return Err(Error::Manifest {
            message: "BUILD block checkpoint has no executable Verification commands".into(),
        });
    }
    let head = git::revision(root, "HEAD")?;
    for command in commands {
        run_verification_command(root, &command)?;
        if git::revision(root, "HEAD")? != head {
            return Err(Error::Manifest {
                message: "a block Verification command changed HEAD".into(),
            });
        }
    }
    Ok(())
}

pub(super) fn build_state_line(plan: &str) -> Result<&str> {
    plan.lines()
        .find(|line| line.starts_with("Current block: "))
        .ok_or_else(|| Error::Manifest {
            message: "workflow plan has no machine-readable Build state".into(),
        })
}

pub(super) fn parse_retry_state(line: &str) -> Result<RetryState> {
    let invalid = || Error::Manifest {
        message: "workflow plan Build state is malformed".into(),
    };
    let rest = line.strip_prefix("Current block: ").ok_or_else(&invalid)?;
    let (block, rest) = rest.split_once(". Attempts: ").ok_or_else(&invalid)?;
    let (attempts, rest) = rest
        .split_once(". Final corrections: ")
        .ok_or_else(&invalid)?;
    let (corrections, rest) = rest.split_once(". ").ok_or_else(&invalid)?;
    let (fingerprint, blocker) = if let Some(rest) = rest.strip_prefix("Fingerprint: ") {
        let (fingerprint, blocker) = rest.split_once(". Blocker: ").ok_or_else(&invalid)?;
        (
            (fingerprint != "none").then(|| fingerprint.to_string()),
            blocker,
        )
    } else {
        (None, rest.strip_prefix("Blocker: ").ok_or_else(&invalid)?)
    };
    Ok(RetryState {
        current_block: block.parse().map_err(|_| invalid())?,
        attempts: attempts.parse().map_err(|_| invalid())?,
        final_corrections: corrections.parse().map_err(|_| invalid())?,
        fingerprint,
        blocker: blocker.trim_end_matches('.').to_string(),
    })
}

pub(super) fn render_retry_state(state: &RetryState) -> String {
    format!(
        "Current block: {}. Attempts: {}. Final corrections: {}. Fingerprint: {}. Blocker: {}.",
        state.current_block,
        state.attempts,
        state.final_corrections,
        state.fingerprint.as_deref().unwrap_or("none"),
        state.blocker.trim_end_matches('.'),
    )
}

pub(super) fn retry_state_after_checkpoint(plan: &str, previous: &RetryState) -> RetryState {
    let mut current_block = previous.current_block;
    let mut candidate = current_block.saturating_add(1);
    while plan_block(plan, candidate).is_ok() {
        if !block_is_done(plan, candidate) {
            current_block = candidate;
            break;
        }
        candidate = candidate.saturating_add(1);
    }
    RetryState {
        current_block,
        attempts: 0,
        final_corrections: previous.final_corrections,
        fingerprint: None,
        blocker: "none".into(),
    }
}
