//! Explicit local integration helpers.
use super::*;

fn branch_checked_out_elsewhere(root: &Path, branch: &str) -> Result<bool> {
    let output = git(root, &["worktree", "list", "--porcelain"])?;
    let root = std::fs::canonicalize(root).map_err(|source| Error::Io {
        path: root.into(),
        source,
    })?;
    let mut worktree = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            worktree = Some(PathBuf::from(path));
        } else if line == format!("branch refs/heads/{branch}") {
            let Some(path) = worktree.as_ref() else {
                continue;
            };
            let path = std::fs::canonicalize(path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            if path != root {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Merge a prepared work branch into the checked-out default branch.
///
/// Both tips are compare-and-swapped immediately before `git merge --no-ff`.
pub fn integrate_no_ff(
    root: &Path,
    default_branch: &str,
    expected_default: &str,
    work_branch: &str,
    expected_work: &str,
) -> Result<()> {
    validate_ref(default_branch)?;
    validate_work_branch(work_branch)?;
    require_clean(root)?;
    let original_branch = current_branch(root)?;
    if branch_checked_out_elsewhere(root, default_branch)? {
        return Err(Error::Manifest {
            message: format!(
                "default branch `{default_branch}` is checked out in another worktree"
            ),
        });
    }
    if revision(root, default_branch)? != expected_default {
        return Err(Error::Manifest {
            message: "default branch moved; return the prepared closure to BUILD".into(),
        });
    }
    if revision(root, work_branch)? != expected_work {
        return Err(Error::Manifest {
            message: "work branch moved after acceptance attestation".into(),
        });
    }
    let temporary = tempfile::tempdir().map_err(|source| Error::Io {
        path: root.into(),
        source,
    })?;
    let worktree = temporary.path().join("integration");
    let worktree_text = worktree.to_string_lossy().into_owned();
    git(
        root,
        &[
            "worktree",
            "add",
            "--detach",
            &worktree_text,
            expected_default,
        ],
    )?;
    let prepared = git(
        &worktree,
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "commit.gpgsign=false",
            "merge",
            "--no-ff",
            "--no-edit",
            "-m",
            "chore(workflow): integrate accepted work",
            expected_work,
        ],
    )
    .and_then(|_| revision(&worktree, "HEAD"));
    let cleanup = git(root, &["worktree", "remove", "--force", &worktree_text]);
    cleanup?;
    let prepared = prepared?;

    if original_branch == default_branch {
        git(
            root,
            &["-c", "core.hooksPath=/dev/null", "switch", work_branch],
        )?;
    } else if original_branch != work_branch {
        return Err(Error::Manifest {
            message: "integration requires the checked-out work or default branch".into(),
        });
    }
    let reservation = temporary.path().join("default-reservation");
    let reservation_text = reservation.to_string_lossy().into_owned();
    if let Err(error) = git(
        root,
        &[
            "worktree",
            "add",
            "--quiet",
            &reservation_text,
            default_branch,
        ],
    ) {
        if original_branch == default_branch {
            let _ = git(
                root,
                &["-c", "core.hooksPath=/dev/null", "switch", default_branch],
            );
        }
        return Err(error);
    }
    git(
        root,
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "switch",
            "--detach",
            &prepared,
        ],
    )?;
    let default_ref = format!("refs/heads/{default_branch}");
    git(root, &["symbolic-ref", "HEAD", &default_ref])?;
    git(root, &["worktree", "remove", "--force", &reservation_text])?;

    let transaction = format!(
        "start\nverify refs/heads/{work_branch} {expected_work}\nupdate refs/heads/{default_branch} {prepared} {expected_default}\nprepare\ncommit\n"
    );
    match git_with_input(root, &["update-ref", "--stdin"], &transaction) {
        Ok(_) => Ok(()),
        Err(error) => {
            // Publication did not occur. Restore a coherent checkout before
            // reporting the compare-and-swap failure.
            git(root, &["update-ref", "--no-deref", "HEAD", &prepared])?;
            git(
                root,
                &["-c", "core.hooksPath=/dev/null", "switch", &original_branch],
            )?;
            Err(error)
        }
    }
}
