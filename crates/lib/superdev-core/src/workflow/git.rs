//! Shell-free Git operations and invariants used by workflow transitions.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::error::{Error, Result};

/// Run Git without a shell and return bounded command failures.
fn git(root: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|source| Error::Command {
            command: format!("git {}", args.join(" ")),
            status: None,
            stderr: source.to_string(),
        })
        .and_then(|output| {
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
        })
}

/// Discover the containing repository's absolute work-tree root.
pub fn repository_root(start: &Path) -> Result<PathBuf> {
    let output = git(start, &["rev-parse", "--show-toplevel"])?;
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    std::fs::canonicalize(&root).map_err(|source| Error::Io {
        path: root.into(),
        source,
    })
}

/// Resolve a ref to its full object ID.
pub fn revision(root: &Path, reference: &str) -> Result<String> {
    validate_ref(reference)?;
    let output = git(root, &["rev-parse", "--verify", reference])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Return the checked-out branch, refusing detached HEAD.
pub fn current_branch(root: &Path) -> Result<String> {
    let output = git(root, &["symbolic-ref", "--short", "HEAD"])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Fail unless tracked and untracked status is empty.
pub fn require_clean(root: &Path) -> Result<()> {
    let output = git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    if output.stdout.is_empty() {
        Ok(())
    } else {
        Err(Error::Manifest {
            message: "workflow requires a clean working tree and will not stash, reset, or absorb unrelated changes".into(),
        })
    }
}

/// Create and check out a validated work branch from the current revision.
pub fn create_work_branch(root: &Path, branch: &str) -> Result<()> {
    validate_work_branch(branch)?;
    require_clean(root)?;
    git(root, &["switch", "-c", branch]).map(|_| ())
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
    if current_branch(root)? != default_branch {
        return Err(Error::Manifest {
            message: format!("integration requires checked-out default branch `{default_branch}`"),
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
    git(root, &["merge", "--no-ff", work_branch]).map(|_| ())
}

/// Refuse option-like, traversal-like, or syntactically invalid refs.
pub fn validate_ref(reference: &str) -> Result<()> {
    if reference.is_empty()
        || reference.starts_with('-')
        || reference.contains("..")
        || reference.contains(['\n', '\r', '\0'])
    {
        return Err(Error::Manifest {
            message: format!("invalid Git ref `{reference}`"),
        });
    }
    let status = Command::new("git")
        .args(["check-ref-format", "--branch", reference])
        .status()
        .map_err(|source| Error::Command {
            command: "git check-ref-format --branch <ref>".into(),
            status: None,
            stderr: source.to_string(),
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Manifest {
            message: format!("invalid Git ref `{reference}`"),
        })
    }
}

/// Enforce the reserved workflow branch convention.
pub fn validate_work_branch(branch: &str) -> Result<()> {
    validate_ref(branch)?;
    let rest = branch
        .strip_prefix("work/")
        .ok_or_else(|| Error::Manifest {
            message: "workflow branches must match work/<issue-number>-<slug>".into(),
        })?;
    let (number, slug) = rest.split_once('-').ok_or_else(|| Error::Manifest {
        message: "workflow branches must match work/<issue-number>-<slug>".into(),
    })?;
    if number.len() != 3
        || !number.bytes().all(|byte| byte.is_ascii_digit())
        || slug.is_empty()
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(Error::Manifest {
            message: "workflow branches must match work/<issue-number>-<slug>".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_branch_validation_is_strict() {
        assert!(validate_work_branch("work/059-scope-build-accept").is_ok());
        for bad in [
            "main",
            "work/59-x",
            "work/059-X",
            "work/059-../main",
            "--help",
        ] {
            assert!(validate_work_branch(bad).is_err(), "{bad}");
        }
    }
}
