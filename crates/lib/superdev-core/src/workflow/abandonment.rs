//! Human-approved abandonment publication, independent of product integration.
use super::git;
use crate::{
    error::{Error, Result},
    validate,
};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

/// Publish a human-approved abandoned issue and plan on the local default
/// branch without carrying any product commit from the work branch.
pub fn publish_abandonment(
    root: &Path,
    default_branch: &str,
    issue: &str,
    plan: &str,
) -> Result<String> {
    git::validate_ref(default_branch)?;
    (|| {
        let default_tip = git::revision(root, default_branch)?;
        if branch_checked_out_elsewhere(root, default_branch)? {
            return Err(Error::Manifest {
                message: format!(
                    "default branch `{default_branch}` is checked out in another worktree"
                ),
            });
        }
        let worktree = root
            .join(".superdev/cache")
            .join(format!("abandonment-worktree-{}", std::process::id()));
        if worktree.exists() {
            return Err(Error::Manifest {
                message: format!(
                    "stale abandonment worktree exists at {}",
                    worktree.display()
                ),
            });
        }
        run_git(
            root,
            &[
                "worktree",
                "add",
                "--detach",
                worktree.to_str().unwrap(),
                &default_tip,
            ],
        )?;
        let prepared = prepare_abandonment(root, &worktree, issue, plan);
        let cleanup = run_git(
            root,
            &["worktree", "remove", "--force", worktree.to_str().unwrap()],
        );
        cleanup?;
        let commit = prepared?;
        if git::revision(root, default_branch)? != default_tip {
            return Err(Error::Manifest {
                message: "default branch moved during abandonment; no ref was advanced".into(),
            });
        }
        run_git(
            root,
            &[
                "update-ref",
                &format!("refs/heads/{default_branch}"),
                &commit,
                &default_tip,
            ],
        )?;
        Ok(commit)
    })()
}

fn prepare_abandonment(source: &Path, worktree: &Path, issue: &str, plan: &str) -> Result<String> {
    let issue_from = source.join(format!("knowledge/issues/wontfix/{issue}.md"));
    let plan_from = source.join(format!("knowledge/plans/abandoned/{plan}.md"));
    if !issue_from.is_file() || !plan_from.is_file() {
        return Err(Error::Manifest {
            message: "abandonment publication requires closed canonical issue and plan records"
                .into(),
        });
    }
    let issue_to = worktree.join(format!("knowledge/issues/wontfix/{issue}.md"));
    let plan_to = worktree.join(format!("knowledge/plans/abandoned/{plan}.md"));
    fs::create_dir_all(issue_to.parent().unwrap()).map_err(|source| Error::Io {
        path: issue_to.clone(),
        source,
    })?;
    fs::create_dir_all(plan_to.parent().unwrap()).map_err(|source| Error::Io {
        path: plan_to.clone(),
        source,
    })?;
    fs::copy(&issue_from, &issue_to).map_err(|source| Error::Io {
        path: issue_to,
        source,
    })?;
    fs::copy(&plan_from, &plan_to).map_err(|source| Error::Io {
        path: plan_to,
        source,
    })?;
    for obsolete in [
        worktree.join(format!("knowledge/issues/open/{issue}.md")),
        worktree.join(format!("knowledge/plans/open/{plan}.md")),
    ] {
        if obsolete.exists() {
            fs::remove_file(&obsolete).map_err(|source| Error::Io {
                path: obsolete,
                source,
            })?;
        }
    }
    validate::fix_repo(worktree, &worktree.join("knowledge"), &[])?;
    let grammar = validate::schema::load_grammar(worktree)?;
    let report = validate::validate_repo(worktree, &worktree.join("knowledge"), &[], &grammar)?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: "abandonment records did not validate on the default branch".into(),
        });
    }
    let changed = run_git(
        worktree,
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--",
            "knowledge",
        ],
    )?;
    let allowed = [
        format!("knowledge/issues/open/{issue}.md"),
        format!("knowledge/issues/wontfix/{issue}.md"),
        format!("knowledge/plans/open/{plan}.md"),
        format!("knowledge/plans/abandoned/{plan}.md"),
        "knowledge/issues/index.md".into(),
        "knowledge/plans/index.md".into(),
    ];
    for line in String::from_utf8_lossy(&changed.stdout).lines() {
        let path = line.get(3..).unwrap_or_default();
        if !allowed.iter().any(|allowed| allowed == path) {
            return Err(Error::Manifest {
                message: format!("abandonment repair touched unrelated knowledge `{path}`"),
            });
        }
    }
    run_git(worktree, &["add", "--all", "--", "knowledge"])?;
    run_git(
        worktree,
        &[
            "commit",
            "--no-verify",
            "--no-gpg-sign",
            "-m",
            &format!("docs: abandon {plan}"),
        ],
    )?;
    let head = run_git(worktree, &["rev-parse", "--verify", "HEAD"])?;
    Ok(String::from_utf8_lossy(&head.stdout).trim().to_string())
}

fn branch_checked_out_elsewhere(root: &Path, branch: &str) -> Result<bool> {
    let output = run_git(root, &["worktree", "list", "--porcelain"])?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line == format!("branch refs/heads/{branch}")))
}

fn run_git(root: &Path, args: &[&str]) -> Result<Output> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|source| Error::Command {
            command: format!("git {}", args.join(" ")),
            status: None,
            stderr: source.to_string(),
        })?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(Error::Command {
            command: format!("git {}", args.join(" ")),
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(8_000)
                .collect(),
        })
    }
}
