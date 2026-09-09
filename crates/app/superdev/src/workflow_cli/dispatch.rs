//! CLI dispatch and owned operations.
use super::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Response<T: Serialize> {
    pub(super) protocol: &'static str,
    pub(super) operation: &'static str,
    pub(super) result: T,
}

pub fn run(command: &WorkflowCommand, root: &Path) -> Result<u8> {
    let root = git::repository_root(root)?;
    match command {
        WorkflowCommand::Start(args) => start(&root, &resolved_identity(&root, args)?),
        WorkflowCommand::Assess(args) => assess(&root, args),
        WorkflowCommand::Bind(args) => {
            // Bind is retained for protocol compatibility, but it never creates
            // or infers canonical records. New recovery uses `resume`.
            validate_identity(&root, args)?;
            bind(&root, args)
        }
        WorkflowCommand::Resume(args) => resume(&root, &resolved_identity(&root, args)?),
        WorkflowCommand::Status { json: _ } => {
            let owner = match cache::try_load(&root)? {
                cache::CacheSnapshot::Owned(owner) => Some(*owner),
                cache::CacheSnapshot::Unowned => None,
                cache::CacheSnapshot::Busy => {
                    let workflow_config = Manifest::load(&root)?.workflow;
                    let executable = std::env::current_exe().map_err(|source| Error::Io {
                        path: root.join("superdev"),
                        source,
                    })?;
                    return emit(
                        "status",
                        &serde_json::json!({
                            "busy": true,
                            "owner": null,
                            "phase": null,
                            "canonicalPlanRevision": null,
                            "buildState": null,
                            "maxStalledBlockAttempts": workflow_config.max_stalled_block_attempts,
                            "maxFinalCorrectionCycles": workflow_config.max_final_correction_cycles,
                            "maxScopeReviewCycles": workflow_config.max_scope_review_cycles,
                            "isolatedRoleTimeoutSeconds": workflow_config.isolated_role_timeout_seconds,
                            "maxIsolatedContextBytes": workflow_config.max_isolated_context_bytes,
                            "maxIsolatedContextLines": workflow_config.max_isolated_context_lines,
                            "maxReviewStateBytes": workflow_config.max_review_state_bytes,
                            "maxReviewFindings": workflow_config.max_review_findings,
                            "maxIsolatedArtifactBytes": workflow_config.max_isolated_artifact_bytes,
                            "maxIsolatedArtifactsPerSession": workflow_config.max_isolated_artifacts_per_session,
                            "isolatedArtifactRetentionHours": workflow_config.isolated_artifact_retention_hours,
                            "humanAcceptanceRequired": workflow_config.human_acceptance_required,
                            "executable": executable,
                            "openWorkflows": [],
                        }),
                    );
                }
            };
            let phase = owner
                .as_ref()
                .map(|state| plan_record(&root, &state.identity.plan).map(|record| record.phase))
                .transpose()?;
            let build_state = if phase.as_deref() == Some("build") {
                let state = owner.as_ref().expect("BUILD status has an owner");
                let (path, _) = plan_revision(&root, &state.identity.plan)?;
                let text =
                    fs::read_to_string(&path).map_err(|source| Error::Io { path, source })?;
                Some(parse_retry_state(build_state_line(&text)?)?)
            } else {
                None
            };
            let canonical_plan_revision = owner
                .as_ref()
                .map(|state| {
                    plan_revision(&root, &state.identity.plan).map(|(_, revision)| revision)
                })
                .transpose()?;
            let workflow_config = Manifest::load(&root)?.workflow;
            let executable = std::env::current_exe().map_err(|source| Error::Io {
                path: root.join("superdev"),
                source,
            })?;
            emit(
                "status",
                &serde_json::json!({
                    "owner": owner,
                    "defaultBranch": git::default_branch(&root, None)?,
                    "defaultRevision": git::revision(&root, &git::default_branch(&root, None)?)?,
                    "phase": phase,
                    "canonicalPlanRevision": canonical_plan_revision,
                    "buildState": build_state.as_ref().map(|state| serde_json::json!({
                        "currentBlock": state.current_block,
                        "attempts": state.attempts,
                        "finalCorrections": state.final_corrections,
                        "fingerprint": state.fingerprint,
                        "blocker": state.blocker,
                    })),
                    "maxStalledBlockAttempts": workflow_config.max_stalled_block_attempts,
                    "maxFinalCorrectionCycles": workflow_config.max_final_correction_cycles,
                    "maxScopeReviewCycles": workflow_config.max_scope_review_cycles,
                    "isolatedRoleTimeoutSeconds": workflow_config.isolated_role_timeout_seconds,
                    "maxIsolatedContextBytes": workflow_config.max_isolated_context_bytes,
                    "maxIsolatedContextLines": workflow_config.max_isolated_context_lines,
                    "maxReviewStateBytes": workflow_config.max_review_state_bytes,
                    "maxReviewFindings": workflow_config.max_review_findings,
                    "maxIsolatedArtifactBytes": workflow_config.max_isolated_artifact_bytes,
                    "maxIsolatedArtifactsPerSession": workflow_config.max_isolated_artifacts_per_session,
                    "isolatedArtifactRetentionHours": workflow_config.isolated_artifact_retention_hours,
                    "humanAcceptanceRequired": workflow_config.human_acceptance_required,
                    "executable": executable,
                    "openWorkflows": discover_open_workflows(&root)?,
                }),
            )
        }
        WorkflowCommand::ActivityStart(args) => {
            if !matches!(
                args.role.as_str(),
                "scope" | "requirements-review" | "build" | "code-review" | "accept"
            ) || args.owner_pid == 0
                || args.child_pid == 0
                || args.owner_started.trim().is_empty()
                || args.child_started.trim().is_empty()
            {
                return Err(Error::Manifest {
                    message:
                        "workflow activity requires a known role and complete process identity"
                            .into(),
                });
            }
            let state =
                cache::compare_and_swap(&root, &args.session, &args.expected_revision, |state| {
                    state.owner_pid = Some(args.owner_pid);
                    state.owner_started = Some(args.owner_started.clone());
                    state.child_role = Some(args.role.clone());
                    state.child_pid = Some(args.child_pid);
                    state.child_started = Some(args.child_started.clone());
                    state.cancelled = false;
                })?;
            emit("activity-start", &serde_json::json!({ "state": state }))
        }
        WorkflowCommand::ActivityFinish(args) => {
            let state =
                cache::compare_and_swap(&root, &args.session, &args.expected_revision, |state| {
                    state.owner_pid = None;
                    state.owner_started = None;
                    state.child_role = None;
                    state.child_pid = None;
                    state.child_started = None;
                })?;
            emit("activity-finish", &serde_json::json!({ "state": state }))
        }
        WorkflowCommand::Cancel(args) => {
            cache::release(&root, &args.session)?;
            emit(
                "cancel",
                &serde_json::json!({"phaseChanged": false, "ownershipReleased": true}),
            )
        }
        WorkflowCommand::ScopeBaseline(args) => record_scope_baseline(&root, args),
        WorkflowCommand::ScopeCheckpoint(args) => record_scope_checkpoint(&root, args),
        WorkflowCommand::Block(args) => cache::transaction(&root, |transaction| {
            let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
                message: "workflow is unowned".into(),
            })?;
            if owner.session_id != args.session
                || owner.last_plan_revision != args.expected_revision
            {
                return Err(Error::Manifest {
                    message: "workflow ownership or plan revision changed".into(),
                });
            }
            validate_identity_values(&root, &owner.identity)?;
            if git::current_branch(&root)? != owner.identity.work_branch {
                return Err(Error::Manifest {
                    message: "block checkpoint requires the checked-out work branch".into(),
                });
            }
            let record = plan_record(&root, &owner.identity.plan)?;
            if record.phase != "build" {
                return Err(Error::Manifest {
                    message: "block evidence may be recorded only during BUILD".into(),
                });
            }
            let (plan_path, observed) = plan_revision(&root, &owner.identity.plan)?;
            if observed != args.revision || observed == args.expected_revision {
                return Err(Error::Manifest {
                    message: "supplied revision must identify a changed canonical plan".into(),
                });
            }
            let relative_plan = plan_path
                .strip_prefix(&root)
                .map_err(|_| Error::Manifest {
                    message: "workflow plan is outside the repository".into(),
                })?
                .to_string_lossy();
            let previous_text = git::file_at_revision(&root, "HEAD", &relative_plan)?;
            let current_text = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
                path: plan_path.clone(),
                source,
            })?;
            let previous_retry = parse_retry_state(build_state_line(&previous_text)?)?;
            let supplied_retry = parse_retry_state(build_state_line(&current_text)?)?;
            if supplied_retry != previous_retry {
                return Err(Error::Manifest {
                    message: "BUILD checkpoint retry state is service-owned".into(),
                });
            }
            let block = previous_retry.current_block;
            let newly_completed =
                block_is_done(&current_text, block) && !block_is_done(&previous_text, block);
            if !newly_completed {
                return Err(Error::Manifest {
                    message: "BUILD checkpoint must newly complete the current stable block".into(),
                });
            }
            let approved_block = plan_block(&previous_text, block)?;
            let block_text = plan_block(&current_text, block)?;
            if verification_commands(&current_text, true)
                != verification_commands(&previous_text, true)
                || block_dependencies(block_text) != block_dependencies(approved_block)
                || block_area_paths(block_text) != block_area_paths(approved_block)
                || plan_verification_commands(block_text)
                    != plan_verification_commands(approved_block)
            {
                return Err(Error::Manifest {
                    message: "BUILD checkpoint cannot change SCOPE-approved dependencies, Areas, or Verification commands".into(),
                });
            }
            for dependency in block_dependencies(approved_block) {
                if dependency >= block || !block_is_done(&current_text, dependency) {
                    return Err(Error::Manifest {
                        message: format!(
                            "BUILD block {block} has an incomplete or invalid dependency {dependency}"
                        ),
                    });
                }
            }
            let mut areas = block_area_paths(approved_block);
            areas.push(relative_plan.to_string());
            git::validate_block_paths(&root, &areas)?;
            run_block_verification(&root, approved_block)?;
            let next_retry = retry_state_after_checkpoint(&current_text, &previous_retry);
            let current_retry_line = build_state_line(&current_text)?;
            let next_retry_line = render_retry_state(&next_retry);
            if current_retry_line != next_retry_line {
                apply_plan_edits_transactionally(
                    &root,
                    &plan_path,
                    vec![ExactEdit {
                        old_text: current_retry_line.into(),
                        new_text: next_retry_line,
                    }],
                )?;
            }
            let grammar = superdev_core::validate::schema::load_grammar(&root)?;
            let report = superdev_core::validate::validate_repo(
                &root,
                &root.join("knowledge"),
                &[],
                &grammar,
            )?;
            if !report.report.passed() {
                return Err(Error::Manifest {
                    message: "block checkpoint requires valid canonical knowledge".into(),
                });
            }
            git::commit_block_changes(
                &root,
                &format!("chore(workflow): checkpoint build block {block}"),
                &areas,
            )?;
            let revision = plan_revision(&root, &owner.identity.plan)?.1;
            let state =
                transaction.compare_and_swap(&args.session, &args.expected_revision, |state| {
                    state.last_plan_revision.clone_from(&revision)
                })?;
            emit("progress", &state)
        }),
        WorkflowCommand::Attempt(args) => record_attempt(&root, args),
        WorkflowCommand::Correction(args) => record_correction(&root, args),
        WorkflowCommand::CorrectionCheckpoint(args) => record_correction_checkpoint(&root, args),
        WorkflowCommand::Evidence(args) => record_evidence(&root, args),
        WorkflowCommand::Sync(args) => cache::transaction(&root, |transaction| {
            let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
                message: "workflow is unowned".into(),
            })?;
            if owner.session_id != args.session
                || owner.last_plan_revision != args.expected_revision
            {
                return Err(Error::Manifest {
                    message: "workflow ownership or plan revision changed".into(),
                });
            }
            validate_identity_values(&root, &owner.identity)?;
            if plan_record(&root, &owner.identity.plan)?.phase != "build" {
                return Err(Error::Manifest {
                    message: "default synchronization is permitted only during BUILD".into(),
                });
            }
            let revision = git::synchronize_default(
                &root,
                &owner.identity.default_branch,
                &args.expected_default,
                &owner.identity.work_branch,
                &args.expected_work,
            )?;
            emit(
                "synchronization",
                &serde_json::json!({"revision": revision, "state": owner}),
            )
        }),
        WorkflowCommand::Transition(args) => transition(&root, args, false, None),
        WorkflowCommand::Abandon(args) => transition(
            &root,
            &TransitionArgs {
                session: args.session.clone(),
                expected_revision: args.expected_revision.clone(),
                phase: args.phase,
                transition: TransitionName::Accept,
                feedback: None,
            },
            true,
            Some(&args.reason),
        ),
        WorkflowCommand::Integrate(args) => cache::transaction(&root, |transaction| {
            let state = transaction.load()?.ok_or_else(|| Error::Manifest {
                message: "workflow is unowned".into(),
            })?;
            if state.session_id != args.session {
                return Err(Error::Manifest {
                    message: "workflow is owned by another Pi session".into(),
                });
            }
            if args.default_branch != state.identity.default_branch
                || args.work_branch != state.identity.work_branch
            {
                return Err(Error::Manifest {
                    message: "integration refs do not match the bound workflow identity".into(),
                });
            }
            let record = plan_record_at_revision(&root, &state.identity.plan, &args.expected_work)?;
            if record.issue != state.identity.issue || record.branch != state.identity.work_branch {
                return Err(Error::Manifest {
                    message: "bound identity does not match the closure plan".into(),
                });
            }
            let issue_path = format!("knowledge/issues/done/{}.md", state.identity.issue);
            git::file_at_revision(&root, &args.expected_work, &issue_path)?;
            if record.phase != "done" || record.lifecycle != "done" {
                return Err(Error::Manifest {
                    message: "integration requires a prepared done plan".into(),
                });
            }
            let candidate = state
                .candidate_revision
                .as_deref()
                .ok_or_else(|| Error::Manifest {
                    message: "integration requires the reviewed candidate revision".into(),
                })?;
            let verified_default =
                state
                    .verified_default_revision
                    .as_deref()
                    .ok_or_else(|| Error::Manifest {
                        message: "integration requires the verified default-branch revision".into(),
                    })?;
            if args.expected_default != verified_default {
                return Err(Error::Manifest {
                    message: "expected default does not match BUILD verification".into(),
                });
            }
            if !git::is_ancestor(&root, candidate, &args.expected_work)? {
                return Err(Error::Manifest {
                    message: "closure commit does not descend from reviewed candidate H".into(),
                });
            }
            let changed = git::changed_paths(&root, candidate, &args.expected_work)?;
            if changed
                .iter()
                .any(|path| !administrative_path(path, &state.identity))
            {
                return Err(Error::Manifest {
                    message: "non-administrative changes follow reviewed candidate H".into(),
                });
            }
            git::integrate_no_ff(
                &root,
                &state.identity.default_branch,
                verified_default,
                &state.identity.work_branch,
                &args.expected_work,
            )?;
            transaction.release(&args.session)?;
            emit(
                "integrate",
                &serde_json::json!({"merged": true, "pushed": false, "branchDeleted": false}),
            )
        }),
    }
}
