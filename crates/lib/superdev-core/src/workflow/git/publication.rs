//! Two-stage publication of one approved document, without absorbing other edits.

use super::*;

/// An immutable commit prepared before its branch reference is advanced.
#[derive(Debug, Clone)]
pub struct PreparedPublication {
    /// Expected current parent.
    pub parent: String,
    /// Exact commit whose content recovery must verify.
    pub commit: String,
}

/// Require both local state directories to be ignored and untracked.
pub fn require_local_state_ignored(root: &Path) -> Result<()> {
    for path in [
        ".superdev/workflows/probe.json",
        ".superdev/cache/probe.json",
    ] {
        git(root, &["check-ignore", "--quiet", "--no-index", path])?;
    }
    let tracked = git(
        root,
        &["ls-files", "--", ".superdev/workflows", ".superdev/cache"],
    )?;
    if !tracked.stdout.is_empty() {
        return Err(Error::Manifest { message: "workflow state must be untracked; remove it from version control explicitly before continuing".into() });
    }
    Ok(())
}

/// Prepare only the supplied document and generated index changes in a private index.
/// No branch, live index, or worktree changes occur here.
pub fn prepare_document_publication(
    root: &Path,
    paths: &[String],
    message: &str,
) -> Result<PreparedPublication> {
    if paths.is_empty() || paths.iter().any(|path| !valid_path(path)) {
        return Err(Error::Manifest {
            message: "publication requires safe document paths".into(),
        });
    }
    let parent = revision(root, "HEAD")?;
    let temporary = tempfile::tempdir().map_err(|source| Error::Io {
        path: root.into(),
        source,
    })?;
    let index = temporary.path().join("index");
    git_with_index(root, &index, &["read-tree", &parent])?;
    let mut add = vec!["add", "--all", "--"];
    add.extend(paths.iter().map(String::as_str));
    git_with_index(root, &index, &add)?;
    let tree = git_with_index(root, &index, &["write-tree"])?;
    let tree = String::from_utf8_lossy(&tree.stdout).trim().to_string();
    let commit = git_with_index(
        root,
        &index,
        &["commit-tree", &tree, "-p", &parent, "-m", message],
    )?;
    Ok(PreparedPublication {
        parent,
        commit: String::from_utf8_lossy(&commit.stdout).trim().into(),
    })
}

/// Publish the exact prepared commit, then align only its selected index entries.
/// This is repeatable after a crash between reference and index updates.
pub fn finish_document_publication(
    root: &Path,
    parent: &str,
    commit: &str,
    branch: &str,
    paths: &[String],
) -> Result<()> {
    validate_ref(parent)?;
    validate_ref(commit)?;
    validate_ref(branch)?;
    let parents = git(root, &["show", "--no-patch", "--format=%P", commit, "--"])?;
    if String::from_utf8_lossy(&parents.stdout).trim() != parent {
        return Err(Error::Manifest {
            message: "pending publication commit has a different parent".into(),
        });
    }
    if current_branch(root)? != branch {
        return Err(Error::Manifest {
            message: format!("publication recovery requires branch `{branch}`; switch explicitly"),
        });
    }
    let observed = revision(root, "HEAD")?;
    if observed == parent {
        git(
            root,
            &[
                "update-ref",
                &format!("refs/heads/{branch}"),
                commit,
                parent,
            ],
        )?;
    } else if observed != commit {
        return Err(Error::Manifest {
            message:
                "publication branch changed; preserve the pending record and inspect the candidate"
                    .into(),
        });
    }
    let mut args = vec!["ls-tree", "-r", "-z", commit, "--"];
    args.extend(paths.iter().map(String::as_str));
    let entries = git(root, &args)?;
    let entries = String::from_utf8(entries.stdout).map_err(|error| Error::Manifest {
        message: format!("publication index paths are not UTF-8: {error}"),
    })?;
    git_with_input(root, &["update-index", "-z", "--index-info"], &entries)?;
    Ok(())
}
