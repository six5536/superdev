//! Issue/plan reservations and checkout recovery.
use super::*;

pub(super) fn start(root: &Path, args: &BindArgs) -> Result<u8> {
    cache::transaction(root, |transaction| start_locked(root, args, transaction))
}

pub(super) fn start_locked(
    root: &Path,
    args: &BindArgs,
    transaction: &mut cache::Transaction<'_>,
) -> Result<u8> {
    ui_authority_capability()?;
    validate_reserved_identity(args)?;
    if git::current_branch(root)? != args.default_branch {
        return Err(Error::Manifest {
            message: format!(
                "SCOPE must start on default branch `{}`",
                args.default_branch
            ),
        });
    }
    if git::reference_exists(root, &args.work_branch)? {
        return Err(Error::Manifest {
            message: format!(
                "workflow branch `{}` already exists; use resume",
                args.work_branch
            ),
        });
    }
    if git::local_work_branch_exists(root, &args.work_branch)? {
        return Err(Error::Manifest {
            message: format!(
                "SCOPE work branch `{}` already exists; select or recover its canonical workflow",
                args.work_branch
            ),
        });
    }
    let issue_path = format!("knowledge/issues/open/{}.md", args.issue);
    let plan_path = format!("knowledge/plans/open/{}.md", args.plan);
    let allowed = [
        issue_path.as_str(),
        plan_path.as_str(),
        "knowledge/issues/index.md",
        "knowledge/plans/index.md",
    ];
    let unexpected: Vec<String> = git::working_paths(root)?
        .into_iter()
        .filter(|path| !allowed.contains(&path.as_str()))
        .collect();
    if !unexpected.is_empty() {
        return Err(Error::Manifest {
            message: format!(
                "SCOPE start found unrelated working-tree changes: {}",
                unexpected.join(", ")
            ),
        });
    }
    git::require_unique_record_number(root, "issue", &args.issue)?;
    git::require_unique_record_number(root, "plan", &args.plan)?;
    validate_open_issue(root, &args.issue)?;
    let record = plan_record(root, &args.plan).map_err(|_| Error::Manifest {
        message: format!(
            "SCOPE skill must author open plan `{}` before deterministic start",
            args.plan
        ),
    })?;
    if record.lifecycle != "open"
        || record.phase != "scope"
        || record.issue != args.issue
        || record.branch != args.work_branch
    {
        return Err(Error::Manifest {
            message: "authored SCOPE plan does not match the selected issue, phase, or issue-derived branch".into(),
        });
    }
    let grammar = superdev_core::validate::schema::load_grammar(root)?;
    let targets = [root.join(&issue_path), root.join(&plan_path)];
    let report =
        superdev_core::validate::validate_repo(root, &root.join("knowledge"), &targets, &grammar)?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: format!(
                "LLM-authored SCOPE records did not validate:\n{}",
                report
                    .report
                    .render_human(superdev_core::validate::sokf::Warnings::Listed)
            ),
        });
    }
    let scope_base = git::revision(root, &args.default_branch)?;

    // Reserve ownership before mutating Git or records. This makes concurrent
    // starts serialize through the same repository lock rather than racing on
    // a prior load followed by an independent write.
    let reservation = workflow_cache(args, String::new(), Some(scope_base), None, None)?;
    transaction.bind(&reservation)?;
    let started = (|| {
        let path = root.join(&plan_path);
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let marker = format!("Workflow default branch: {}.", args.default_branch);
        if !text.lines().any(|line| line == marker) {
            apply_plan_edits_transactionally(
                root,
                &path,
                vec![completion_evidence_edit(&text, &[marker])?],
            )?;
        }
        git::commit_knowledge_changes(root, "chore(workflow): start scope")?;
        git::create_work_branch(root, &args.work_branch)?;
        let revision = plan_revision(root, &args.plan)?.1;
        let state = transaction.compare_and_swap(&args.session, "", |state| {
            state.last_plan_revision.clone_from(&revision)
        })?;
        emit("start", &state)
    })();
    if started.is_err() {
        let _ = transaction.release(&args.session);
    }
    started
}

pub(super) fn validate_open_issue(root: &Path, id: &str) -> Result<()> {
    let path = root.join("knowledge/issues/open").join(format!("{id}.md"));
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let concept = parse_concept(&path.to_string_lossy(), &text).map_err(|error| Error::Sokf {
        message: error.message,
    })?;
    if concept.id.as_deref() != Some(id) || concept.lifecycle.as_deref() != Some("open") {
        return Err(Error::Manifest {
            message: format!("`{id}` is not the matching open canonical issue"),
        });
    }
    Ok(())
}

pub(super) fn discover_open_workflows(root: &Path) -> Result<Vec<WorkflowIdentity>> {
    let mut workflows = Vec::new();
    for branch in git::local_branches(root)? {
        if !branch.starts_with("work/") {
            continue;
        }
        for path in git::paths_at_revision(root, &branch)? {
            if !path.starts_with("knowledge/plans/open/")
                || !path.ends_with(".md")
                || path.ends_with("/index.md")
            {
                continue;
            }
            let text = git::file_at_revision(root, &branch, &path)?;
            let record = parse_plan_record(&path, &text)?;
            if record.branch != branch
                || record.lifecycle != "open"
                || !matches!(record.phase.as_str(), "scope" | "build" | "accept")
            {
                continue;
            }
            let plan = Path::new(&path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            workflows.push(WorkflowIdentity {
                issue: record.issue,
                plan,
                work_branch: branch.clone(),
                default_branch: recorded_default(root, &text)?,
            });
        }
    }
    workflows.sort_by(|a, b| a.plan.cmp(&b.plan));
    Ok(workflows)
}

pub(super) fn recorded_default(root: &Path, text: &str) -> Result<String> {
    git::default_branch(
        root,
        text.lines()
            .find_map(|line| line.strip_prefix("Workflow default branch: "))
            .and_then(|value| value.strip_suffix('.')),
    )
}

pub(super) fn resolved_identity(root: &Path, args: &BindArgs) -> Result<BindArgs> {
    let mut args = args.clone();
    if args.default_branch.is_empty() && git::local_work_branch_exists(root, &args.work_branch)? {
        let path = format!("knowledge/plans/open/{}.md", args.plan);
        let text = git::file_at_revision(root, &args.work_branch, &path)?;
        args.default_branch = recorded_default(root, &text)?;
    } else {
        args.default_branch = git::default_branch(root, Some(&args.default_branch))?;
    }
    Ok(args)
}

pub(super) fn resume(root: &Path, args: &BindArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let capability = ui_authority_capability()?;
        if let Some(owner) = transaction.load()? {
            if owner.session_id != args.session
                || owner.identity.work_branch != args.work_branch
                || owner.identity.plan != args.plan
            {
                return Err(Error::Manifest {
                    message: "another workflow owns this checkout; pause it before switching"
                        .into(),
                });
            }
            cache::verify_authority(&owner, &capability)?;
        }
        let text = git::file_at_revision(
            root,
            &args.work_branch,
            &format!("knowledge/plans/open/{}.md", args.plan),
        )?;
        if recorded_default(root, &text)? != args.default_branch {
            return Err(Error::Manifest {
                message: "default branch differs from the reserved workflow identity".into(),
            });
        }
        if git::current_branch(root)? != args.work_branch {
            git::checkout_work_branch(root, &args.work_branch)?;
        }
        validate_resumable_identity(root, args)?;
        let (path, revision) = plan_revision(root, &args.plan)?;
        let text = fs::read_to_string(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        let baseline = if plan_record(root, &args.plan)?.phase == "scope" {
            Some(git::scope_base_from_history(
                root,
                &args.work_branch,
                &path,
            )?)
        } else {
            None
        };
        let state = workflow_cache(
            args,
            revision,
            baseline,
            evidence_revision(&text, "Candidate revision"),
            evidence_revision(&text, "Verified default revision"),
        )?;
        transaction.bind(&state)?;
        emit("resume", &state)
    })
}

pub(super) fn assess(root: &Path, args: &RevisionArgs) -> Result<u8> {
    cache::transaction(root, |transaction| {
        let owner = transaction.load()?.ok_or_else(|| Error::Manifest {
            message: "workflow is unowned".into(),
        })?;
        cache::verify_authority(&owner, &ui_authority_capability()?)?;
        if owner.session_id != args.session
            || owner.last_plan_revision != args.expected_revision
            || plan_revision(root, &owner.identity.plan)?.1 != args.expected_revision
            || plan_record(root, &owner.identity.plan)?.phase != "accept"
        {
            return Err(Error::Manifest {
                message: "assessment requires the current owned ACCEPT revision".into(),
            });
        }
        git::require_clean(root)?;
        let candidate = owner
            .candidate_revision
            .as_deref()
            .ok_or_else(|| Error::Manifest {
                message: "assessment requires reviewed candidate H".into(),
            })?;
        let head = git::revision(root, "HEAD")?;
        if git::current_branch(root)? != owner.identity.work_branch
            || !git::is_ancestor(root, candidate, &head)?
            || git::changed_paths(root, candidate, &head)?
                .iter()
                .any(|path| !administrative_path(path, &owner.identity))
        {
            return Err(Error::Manifest {
                message: "candidate changed outside administrative workflow records".into(),
            });
        }
        emit(
            "assess",
            &serde_json::json!({ "candidate": candidate, "attestation": head }),
        )
    })
}

pub(super) fn bind(root: &Path, args: &BindArgs) -> Result<u8> {
    let revision = plan_revision(root, &args.plan)?.1;
    bind_with_revision(root, args, revision)
}

pub(super) fn bind_with_revision(root: &Path, args: &BindArgs, revision: String) -> Result<u8> {
    let plan_path = plan_revision(root, &args.plan)?.0;
    let text = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
        path: plan_path.clone(),
        source,
    })?;
    let candidate = evidence_revision(&text, "Candidate revision");
    let verified_default = evidence_revision(&text, "Verified default revision");
    let scope_base = if plan_record(root, &args.plan)?.phase == "scope" {
        Some(git::scope_base_from_history(
            root,
            &args.work_branch,
            &plan_path,
        )?)
    } else {
        None
    };
    let state = workflow_cache(args, revision, scope_base, candidate, verified_default)?;
    cache::bind(root, &state)?;
    emit("bind", &state)
}

pub(super) fn workflow_cache(
    args: &BindArgs,
    revision: String,
    scope_base_revision: Option<String>,
    candidate_revision: Option<String>,
    verified_default_revision: Option<String>,
) -> Result<WorkflowCache> {
    let capability = ui_authority_capability()?;
    Ok(WorkflowCache {
        version: 1,
        session_id: args.session.clone(),
        identity: WorkflowIdentity {
            issue: args.issue.clone(),
            plan: args.plan.clone(),
            work_branch: args.work_branch.clone(),
            default_branch: args.default_branch.clone(),
        },
        last_plan_revision: revision,
        authority_digest: cache::authority_digest(&capability)?,
        scope_base_revision,
        candidate_revision,
        verified_default_revision,
        owner_pid: None,
        owner_started: None,
        child_role: None,
        child_pid: None,
        child_started: None,
        cancelled: false,
    })
}

pub(super) fn ui_authority_capability() -> Result<String> {
    std::env::var("SUPERDEV_UI_AUTHORITY").map_err(|_| Error::Manifest {
        message: "workflow ownership must be established by the interactive Pi UI".into(),
    })
}

#[derive(Debug)]
pub(super) struct PlanRecord {
    pub(super) phase: String,
    pub(super) lifecycle: String,
    pub(super) branch: String,
    pub(super) issue: String,
}

pub(super) fn plan_record(root: &Path, id: &str) -> Result<PlanRecord> {
    let (path, _) = plan_revision(root, id)?;
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    parse_plan_record(&path.to_string_lossy(), &text)
}

pub(super) fn plan_record_at_revision(root: &Path, id: &str, revision: &str) -> Result<PlanRecord> {
    let path = format!("knowledge/plans/done/{id}.md");
    let text = git::file_at_revision(root, revision, &path)?;
    parse_plan_record(&path, &text)
}

pub(super) fn parse_plan_record(path: &str, text: &str) -> Result<PlanRecord> {
    let concept = parse_concept(path, text).map_err(|error| Error::Sokf {
        message: error.message,
    })?;
    let issues: Vec<String> = concept
        .links
        .iter()
        .filter(|link| link.rel.as_deref() == Some("implements"))
        .filter_map(|link| link.to.clone())
        .collect();
    if issues.len() != 1 {
        return Err(Error::Manifest {
            message: "workflow plan must implement exactly one issue".into(),
        });
    }
    Ok(PlanRecord {
        phase: concept.raw["phase"].as_str().unwrap_or_default().into(),
        lifecycle: concept.lifecycle.unwrap_or_default(),
        branch: concept.raw["branch"].as_str().unwrap_or_default().into(),
        issue: issues[0].clone(),
    })
}

pub(super) fn validate_reserved_identity(args: &BindArgs) -> Result<()> {
    git::validate_work_branch(&args.work_branch)?;
    git::validate_ref(&args.default_branch)?;
    let issue_tail = args
        .issue
        .strip_prefix("issue-")
        .ok_or_else(|| Error::Manifest {
            message: "workflow issue must match issue-NNN-slug".into(),
        })?;
    let plan_tail = args
        .plan
        .strip_prefix("plan-")
        .ok_or_else(|| Error::Manifest {
            message: "workflow plan must match plan-NNN-slug".into(),
        })?;
    if plan_tail.is_empty() || args.work_branch != format!("work/{issue_tail}") {
        return Err(Error::Manifest {
            message:
                "work branch must derive from the issue identity; plan identity is independent"
                    .into(),
        });
    }
    Ok(())
}

pub(super) fn validate_resumable_identity(root: &Path, args: &BindArgs) -> Result<()> {
    validate_identity(root, args)?;
    let record = plan_record(root, &args.plan)?;
    if record.lifecycle != "open" || !matches!(record.phase.as_str(), "scope" | "build" | "accept")
    {
        return Err(Error::Manifest {
            message: "resume requires one open canonical workflow".into(),
        });
    }
    Ok(())
}

pub(super) fn validate_identity(root: &Path, args: &BindArgs) -> Result<()> {
    validate_identity_values(
        root,
        &WorkflowIdentity {
            issue: args.issue.clone(),
            plan: args.plan.clone(),
            work_branch: args.work_branch.clone(),
            default_branch: args.default_branch.clone(),
        },
    )
}

pub(super) fn validate_identity_values(root: &Path, identity: &WorkflowIdentity) -> Result<()> {
    git::validate_work_branch(&identity.work_branch)?;
    git::validate_ref(&identity.default_branch)?;
    let record = plan_record(root, &identity.plan)?;
    if record.issue != identity.issue || record.branch != identity.work_branch {
        return Err(Error::Manifest {
            message: "bound identity does not match the canonical plan".into(),
        });
    }
    let issue_exists = ["open", "done", "wontfix"].iter().any(|lifecycle| {
        root.join("knowledge/issues")
            .join(lifecycle)
            .join(format!("{}.md", identity.issue))
            .is_file()
    });
    if !issue_exists {
        return Err(Error::Manifest {
            message: format!("primary issue `{}` was not found", identity.issue),
        });
    }
    Ok(())
}

pub(super) fn administrative_path(path: &str, identity: &WorkflowIdentity) -> bool {
    path == "knowledge/issues/index.md"
        || path == "knowledge/plans/index.md"
        || ["open", "done", "wontfix"]
            .iter()
            .any(|state| path == format!("knowledge/issues/{state}/{}.md", identity.issue))
        || ["open", "done", "abandoned"]
            .iter()
            .any(|state| path == format!("knowledge/plans/{state}/{}.md", identity.plan))
}

pub(super) fn knowledge_contains(root: &Path, needle: &str) -> Result<bool> {
    let mut directories = vec![root.join("knowledge")];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).map_err(|source| Error::Io {
            path: directory.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| Error::Io {
                path: directory.clone(),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                let text = fs::read_to_string(&path).map_err(|source| Error::Io {
                    path: path.clone(),
                    source,
                })?;
                if text.contains(needle) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
