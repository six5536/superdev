//! Transactional canonical record edits and closure.
use super::*;

pub(super) fn apply_plan_edits_transactionally(
    root: &Path,
    live_plan: &Path,
    edits: Vec<ExactEdit>,
) -> Result<()> {
    apply_record_edits_transactionally(root, vec![(live_plan, edits)])
}

pub(super) fn apply_record_edits_transactionally(
    root: &Path,
    records: Vec<(&Path, Vec<ExactEdit>)>,
) -> Result<()> {
    let staging = stage_knowledge(root, "workflow-transition-")?;
    let staged_knowledge = staging.path().join("knowledge");
    let service = SokfService::new(
        staged_knowledge.clone(),
        root.to_path_buf(),
        IndexDir(root.join(".superdev/cache/sokf")),
        None,
    );
    for (live_record, edits) in records {
        let relative = live_record
            .strip_prefix(root.join("knowledge"))
            .map_err(|_| Error::Manifest {
                message: "workflow record is outside canonical knowledge".into(),
            })?;
        let staged_record = staged_knowledge.join(relative);
        let mutation = service.edit(
            EditRequest {
                path: staged_record.to_string_lossy().into_owned(),
                edits,
            },
            MutationPolicy::AgentSafe,
        )?;
        if mutation.validation != superdev_core::sokf::ValidationState::Valid {
            let findings = mutation
                .findings
                .iter()
                .map(|finding| finding.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(Error::Manifest {
                message: format!(
                    "workflow transition did not leave valid canonical knowledge: {findings}"
                ),
            });
        }
    }
    publish_staged_knowledge(root, &staged_knowledge)
}

pub(super) fn primary_issue_has_unresolved_discoveries(root: &Path, issue: &str) -> Result<bool> {
    let path = root
        .join("knowledge/issues/open")
        .join(format!("{issue}.md"));
    let text = fs::read_to_string(&path).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let Some(start) = text.find("## Discoveries\n") else {
        return Ok(false);
    };
    let body = &text[start + "## Discoveries\n".len()..];
    let end = body
        .find("\n## ")
        .or_else(|| body.find("\n<!-- sokf:links -->"))
        .unwrap_or(body.len());
    Ok(body[..end].lines().any(|line| line.starts_with("- [ ] ")))
}

pub(super) fn rescope_discovery_edit(
    issue: &str,
    label: &str,
    feedback: &str,
) -> Result<ExactEdit> {
    let mut feedback_lines = feedback.split('\n');
    let first = feedback_lines.next().unwrap_or_default();
    let remaining = feedback_lines.collect::<Vec<_>>();
    let item = if remaining.is_empty() {
        format!("- [ ] {label}: {first}")
    } else {
        format!(
            "- [ ] {label}: {first}\n\n      {}",
            remaining.join("\n      ")
        )
    };
    if let Some(start) = issue.find("## Discoveries\n") {
        let content_start = start + "## Discoveries\n".len();
        let end = issue[content_start..]
            .find("\n## ")
            .map(|offset| content_start + offset)
            .or_else(|| {
                issue[content_start..]
                    .find("\n<!-- sokf:links -->")
                    .map(|offset| content_start + offset)
            })
            .unwrap_or(issue.len());
        let old_text = issue[start..end].to_string();
        let new_text = format!("{}\n{}", old_text.trim_end(), item);
        Ok(ExactEdit { old_text, new_text })
    } else {
        let insertion = issue
            .find("\n## Comments\n")
            .or_else(|| issue.find("\n<!-- sokf:links -->"))
            .unwrap_or(issue.len());
        let anchor = issue[insertion..].to_string();
        Ok(ExactEdit {
            old_text: anchor.clone(),
            new_text: format!("\n## Discoveries\n\n{item}\n{anchor}"),
        })
    }
}

pub(super) fn stage_knowledge(root: &Path, prefix: &str) -> Result<tempfile::TempDir> {
    let staging_parent = root.join(".superdev/cache");
    fs::create_dir_all(&staging_parent).map_err(|source| Error::Io {
        path: staging_parent.clone(),
        source,
    })?;
    let staging = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(&staging_parent)
        .map_err(|source| Error::Io {
            path: staging_parent,
            source,
        })?;
    copy_tree(&root.join("knowledge"), &staging.path().join("knowledge"))?;
    Ok(staging)
}

pub(super) fn publish_staged_knowledge(root: &Path, staged_knowledge: &Path) -> Result<()> {
    let live = root.join("knowledge");
    let backup = root.join(".superdev/cache/workflow-knowledge-backup");
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(|source| Error::Io {
            path: backup.clone(),
            source,
        })?;
    }
    fs::rename(&live, &backup).map_err(|source| Error::Io {
        path: live.clone(),
        source,
    })?;
    if let Err(source) = fs::rename(staged_knowledge, &live) {
        let _ = fs::rename(&backup, &live);
        return Err(Error::Io { path: live, source });
    }
    fs::remove_dir_all(&backup).map_err(|source| Error::Io {
        path: backup,
        source,
    })?;
    Ok(())
}

pub(super) fn reopen_stale_closure(root: &Path, identity: &WorkflowIdentity) -> Result<()> {
    let staging = stage_knowledge(root, "workflow-reopen-")?;
    let knowledge = staging.path().join("knowledge");
    let plan_path = knowledge
        .join("plans/done")
        .join(format!("{}.md", identity.plan));
    let plan = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
        path: plan_path.clone(),
        source,
    })?;
    let plan = replace_once(&plan, "lifecycle: done", "lifecycle: open")?;
    let plan = replace_once(&plan, "phase: done", "phase: build")?;
    fs::write(&plan_path, plan).map_err(|source| Error::Io {
        path: plan_path,
        source,
    })?;

    let issue_path = knowledge
        .join("issues/done")
        .join(format!("{}.md", identity.issue));
    let issue = fs::read_to_string(&issue_path).map_err(|source| Error::Io {
        path: issue_path.clone(),
        source,
    })?;
    let issue = replace_once(&issue, "lifecycle: done", "lifecycle: open")?;
    let issue = remove_resolution(&issue);
    fs::write(&issue_path, issue).map_err(|source| Error::Io {
        path: issue_path,
        source,
    })?;

    superdev_core::validate::fix_repo(root, &knowledge, &[])?;
    let grammar = superdev_core::validate::schema::load_grammar(root)?;
    let report = superdev_core::validate::validate_repo(root, &knowledge, &[], &grammar)?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: "stale-default recovery did not produce valid canonical records".into(),
        });
    }
    publish_staged_knowledge(root, &knowledge)
}

pub(super) fn remove_resolution(issue: &str) -> String {
    let Some(start) = issue.find("\n## Resolution\n") else {
        return issue.to_string();
    };
    let tail = &issue[start + 1..];
    let end = tail["## Resolution\n".len()..]
        .find("\n## ")
        .map(|offset| start + 1 + "## Resolution\n".len() + offset)
        .or_else(|| {
            issue[start..]
                .find("\n<!-- sokf:links -->")
                .map(|offset| start + offset)
        })
        .unwrap_or(issue.len());
    format!("{}{}", issue[..start].trim_end(), &issue[end..])
}

pub(super) fn close_records(
    root: &Path,
    identity: &WorkflowIdentity,
    next: Phase,
    abandonment_reason: Option<&str>,
) -> Result<()> {
    // Build and validate the complete closure away from the live knowledge
    // tree. A failed mutation, repair, or validation therefore leaves the
    // canonical records byte-for-byte unchanged.
    let staging = stage_knowledge(root, "workflow-closure-")?;
    let staged_knowledge = staging.path().join("knowledge");

    let plan_path = ["open", "done", "abandoned"]
        .into_iter()
        .map(|lifecycle| {
            staged_knowledge
                .join("plans")
                .join(lifecycle)
                .join(format!("{}.md", identity.plan))
        })
        .find(|path| path.is_file())
        .ok_or_else(|| Error::Manifest {
            message: format!("workflow plan `{}` was not found", identity.plan),
        })?;
    let plan = fs::read_to_string(&plan_path).map_err(|source| Error::Io {
        path: plan_path.clone(),
        source,
    })?;
    let plan = replace_once(
        &plan,
        "lifecycle: open",
        &format!("lifecycle: {}", phase_text(next)),
    )?;
    let current_phase = ["scope", "build", "accept"]
        .into_iter()
        .find(|phase| plan.matches(&format!("phase: {phase}")).count() == 1)
        .ok_or_else(|| Error::Manifest {
            message: "workflow closure could not identify the open plan phase".into(),
        })?;
    let plan = replace_once(
        &plan,
        &format!("phase: {current_phase}"),
        &format!("phase: {}", phase_text(next)),
    )?;
    fs::write(&plan_path, plan).map_err(|source| Error::Io {
        path: plan_path,
        source,
    })?;

    let issue_path = staged_knowledge
        .join("issues/open")
        .join(format!("{}.md", identity.issue));
    let issue = fs::read_to_string(&issue_path).map_err(|source| Error::Io {
        path: issue_path.clone(),
        source,
    })?;
    let issue_lifecycle = if next == Phase::Done {
        "done"
    } else {
        "wontfix"
    };
    let issue = replace_once(
        &issue,
        "lifecycle: open",
        &format!("lifecycle: {issue_lifecycle}"),
    )?;
    let resolution = if next == Phase::Done {
        "The configured acceptance gate approved the reviewed candidate for local integration."
            .to_string()
    } else {
        format!(
            "The human abandoned this workflow without integrating partial product work. {}",
            abandonment_reason.unwrap_or("No additional reason was supplied.")
        )
    };
    let issue = if issue.contains("\n## Comments\n") {
        replace_once(
            &issue,
            "\n## Comments\n",
            &format!("\n## Resolution\n\n{resolution}\n\n## Comments\n"),
        )?
    } else if issue.contains("\n<!-- sokf:links -->") {
        replace_once(
            &issue,
            "\n<!-- sokf:links -->",
            &format!("\n## Resolution\n\n{resolution}\n\n<!-- sokf:links -->"),
        )?
    } else {
        format!("{}\n\n## Resolution\n\n{resolution}\n", issue.trim_end())
    };
    fs::write(&issue_path, issue).map_err(|source| Error::Io {
        path: issue_path,
        source,
    })?;

    superdev_core::validate::fix_repo(root, &staged_knowledge, &[])?;
    let grammar = superdev_core::validate::schema::load_grammar(root)?;
    let report = superdev_core::validate::validate_repo(root, &staged_knowledge, &[], &grammar)?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: format!(
                "workflow closure did not validate:\n{}",
                report
                    .report
                    .render_human(superdev_core::validate::sokf::Warnings::Listed)
            ),
        });
    }

    publish_staged_knowledge(root, &staged_knowledge)
}

pub(super) fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|source_error| Error::Io {
        path: destination.to_path_buf(),
        source: source_error,
    })?;
    for entry in fs::read_dir(source).map_err(|source_error| Error::Io {
        path: source.to_path_buf(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| Error::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|source_error| Error::Io {
                path: from.clone(),
                source: source_error,
            })?
            .is_dir()
        {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|source_error| Error::Io {
                path: to,
                source: source_error,
            })?;
        }
    }
    Ok(())
}

pub(super) fn invalidate_final_evidence_edit(plan: &str) -> Result<ExactEdit> {
    filter_completion_evidence(
        plan,
        &[
            "Candidate revision: ",
            "Verified default revision: ",
            "Final verification: ",
            "Documentation verification: ",
            "Final review: ",
        ],
        &[],
    )
}

pub(super) fn invalidate_scope_evidence_edit(
    plan: &str,
    baseline: &str,
    invalidate_final: bool,
) -> Result<ExactEdit> {
    let mut prefixes = vec![
        "Scope requirements review: ",
        "Human scope approval: ",
        "Scope product baseline: ",
    ];
    if invalidate_final {
        prefixes.extend([
            "Candidate revision: ",
            "Verified default revision: ",
            "Final verification: ",
            "Documentation verification: ",
            "Final review: ",
        ]);
    }
    filter_completion_evidence(plan, &prefixes, &[baseline.to_string()])
}

pub(super) fn filter_completion_evidence(
    plan: &str,
    prefixes: &[&str],
    appended_lines: &[String],
) -> Result<ExactEdit> {
    let marker = "## Completion evidence\n\n";
    let start = plan.find(marker).ok_or_else(|| Error::Manifest {
        message: "workflow plan has no Completion evidence section".into(),
    })? + marker.len();
    let tail = &plan[start..];
    let end = tail
        .find("\n## ")
        .or_else(|| tail.find("\n<!-- sokf:links -->"))
        .unwrap_or(tail.len());
    let old = &tail[..end];
    let mut retained = old
        .lines()
        .filter(|line| !prefixes.iter().any(|prefix| line.starts_with(prefix)))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();
    for line in appended_lines {
        if !retained.lines().any(|existing| existing == line) {
            if !retained.is_empty() {
                retained.push_str("\n\n");
            }
            retained.push_str(line);
        }
    }
    Ok(ExactEdit {
        old_text: format!("{marker}{old}"),
        new_text: format!("{marker}{retained}\n"),
    })
}

pub(super) fn completion_evidence_edit(plan: &str, lines: &[String]) -> Result<ExactEdit> {
    let marker = "## Completion evidence\n\n";
    let start = plan.find(marker).ok_or_else(|| Error::Manifest {
        message: "workflow plan has no Completion evidence section".into(),
    })? + marker.len();
    let tail = &plan[start..];
    let end = tail
        .find("\n## ")
        .or_else(|| tail.find("\n<!-- sokf:links -->"))
        .unwrap_or(tail.len());
    let old = &tail[..end];
    let mut new = old.trim_end().to_string();
    for line in lines {
        if !new.lines().any(|existing| existing == line) {
            if !new.is_empty() {
                new.push_str("\n\n");
            }
            new.push_str(line);
        }
    }
    new.push('\n');
    Ok(ExactEdit {
        old_text: format!("{marker}{old}"),
        new_text: format!("{marker}{new}"),
    })
}

pub(super) fn evidence_revision(plan: &str, label: &str) -> Option<String> {
    let prefix = format!("{label}: ");
    plan.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .map(|revision| revision.trim_end_matches('.').to_string())
        .filter(|revision| {
            revision.len() >= 7
                && revision
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
        })
}

pub(super) fn replace_once(text: &str, old: &str, new: &str) -> Result<String> {
    if text.matches(old).count() != 1 {
        return Err(Error::Manifest {
            message: format!("workflow expected exactly one `{old}` field"),
        });
    }
    Ok(text.replacen(old, new, 1))
}

pub(super) fn plan_revision(root: &Path, id: &str) -> Result<(PathBuf, String)> {
    for lifecycle in ["open", "done", "abandoned"] {
        let dir = root.join("knowledge/plans").join(lifecycle);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_stem().and_then(|name| name.to_str()) != Some(id) {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            let concept =
                parse_concept(&path.to_string_lossy(), &text).map_err(|error| Error::Sokf {
                    message: error.message,
                })?;
            return Ok((path, concept.content_hash));
        }
    }
    Err(Error::Manifest {
        message: format!("workflow plan `{id}` was not found"),
    })
}

pub(super) fn phase(value: PhaseName) -> Phase {
    match value {
        PhaseName::Scope => Phase::Scope,
        PhaseName::Build => Phase::Build,
        PhaseName::Accept => Phase::Accept,
        PhaseName::Done => Phase::Done,
        PhaseName::Abandoned => Phase::Abandoned,
    }
}
pub(super) fn phase_text(value: Phase) -> &'static str {
    match value {
        Phase::Scope => "scope",
        Phase::Build => "build",
        Phase::Accept => "accept",
        Phase::Done => "done",
        Phase::Abandoned => "abandoned",
    }
}

pub(super) fn emit<T: Serialize>(operation: &'static str, result: &T) -> Result<u8> {
    println!(
        "{}",
        serde_json::to_string(&Response {
            protocol: WORKFLOW_PROTOCOL,
            operation,
            result
        })
        .expect("workflow response serializes")
    );
    Ok(0)
}
