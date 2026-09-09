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
        WorkflowCommand::Commit(args) => cache::transaction(&root, |transaction| {
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
                    message: format!(
                        "workflow commits require checked-out branch `{}`",
                        owner.identity.work_branch
                    ),
                });
            }
            let parent = git::revision(&root, "HEAD")?;
            let commit = git::commit_worktree_changes(&root, &args.message)?;
            let revision = plan_revision(&root, &owner.identity.plan)?.1;
            let state = match transaction.compare_and_swap(
                &args.session,
                &args.expected_revision,
                |state| state.last_plan_revision.clone_from(&revision),
            ) {
                Ok(state) => state,
                Err(error) => {
                    git::rollback_commit(&root, &commit, &parent)?;
                    return Err(error);
                }
            };
            emit(
                "commit",
                &serde_json::json!({"commit": commit, "revision": revision, "state": state}),
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
    }
}
