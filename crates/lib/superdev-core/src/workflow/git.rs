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

/// Read one repository-relative file from an immutable revision without changing the worktree.
pub fn file_at_revision(root: &Path, revision: &str, path: &str) -> Result<String> {
    validate_ref(revision)?;
    if path.starts_with('/') || path.contains("..") || path.contains(':') {
        return Err(Error::Manifest {
            message: format!("invalid repository-relative path `{path}`"),
        });
    }
    let object = format!("{revision}:{path}");
    let output = git(root, &["show", &object])?;
    String::from_utf8(output.stdout).map_err(|source| Error::Manifest {
        message: format!("`{path}` at `{revision}` is not UTF-8: {source}"),
    })
}

/// Return whether a validated ref resolves.
pub fn reference_exists(root: &Path, reference: &str) -> Result<bool> {
    validate_ref(reference)?;
    let status = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", reference])
        .current_dir(root)
        .status()
        .map_err(|source| Error::Command {
            command: "git rev-parse --verify --quiet <ref>".into(),
            status: None,
            stderr: source.to_string(),
        })?;
    Ok(status.success())
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

/// Return whether `ancestor` is reachable from `descendant`.
pub fn is_ancestor(root: &Path, ancestor: &str, descendant: &str) -> Result<bool> {
    validate_ref(ancestor)?;
    validate_ref(descendant)?;
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .current_dir(root)
        .status()
        .map_err(|source| Error::Command {
            command: "git merge-base --is-ancestor <ancestor> <descendant>".into(),
            status: None,
            stderr: source.to_string(),
        })?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        code => Err(Error::Command {
            command: "git merge-base --is-ancestor <ancestor> <descendant>".into(),
            status: code,
            stderr: "Git could not compare workflow revisions".into(),
        }),
    }
}

/// Paths changed between two revisions, without invoking a shell.
pub fn changed_paths(root: &Path, from: &str, to: &str) -> Result<Vec<String>> {
    validate_ref(from)?;
    validate_ref(to)?;
    let output = git(root, &["diff", "--name-only", from, to, "--"])?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
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
        .output()
        .map_err(|source| Error::Command {
            command: "git check-ref-format --branch <ref>".into(),
            status: None,
            stderr: source.to_string(),
        })?;
    if status.status.success() {
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

    fn command(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .expect("git runs");
        assert!(status.success(), "git {args:?}");
    }

    fn repository() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        command(dir.path(), &["init", "-q", "-b", "main"]);
        command(dir.path(), &["config", "user.email", "test@example.com"]);
        command(dir.path(), &["config", "user.name", "Test"]);
        command(dir.path(), &["config", "commit.gpgsign", "false"]);
        command(dir.path(), &["config", "merge.gpgsign", "false"]);
        std::fs::write(dir.path().join("file"), "base\n").unwrap();
        command(dir.path(), &["add", "file"]);
        command(dir.path(), &["commit", "-q", "-m", "base"]);
        dir
    }

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

    #[test]
    fn integration_creates_a_no_ff_merge_commit() {
        let dir = repository();
        let root = dir.path();
        let default = revision(root, "main").unwrap();
        create_work_branch(root, "work/059-test").unwrap();
        std::fs::write(root.join("file"), "work\n").unwrap();
        command(root, &["commit", "-q", "-am", "work"]);
        let work = revision(root, "work/059-test").unwrap();
        command(root, &["switch", "-q", "main"]);

        integrate_no_ff(root, "main", &default, "work/059-test", &work).unwrap();

        let parents = git(root, &["rev-list", "--parents", "-n", "1", "HEAD"]).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&parents.stdout)
                .split_whitespace()
                .count(),
            3
        );
    }

    #[test]
    fn integration_refuses_dirty_or_stale_tips() {
        let dir = repository();
        let root = dir.path();
        let default = revision(root, "main").unwrap();
        create_work_branch(root, "work/059-test").unwrap();
        std::fs::write(root.join("file"), "work\n").unwrap();
        command(root, &["commit", "-q", "-am", "work"]);
        let work = revision(root, "work/059-test").unwrap();
        command(root, &["switch", "-q", "main"]);

        std::fs::write(root.join("unrelated"), "dirty\n").unwrap();
        assert!(integrate_no_ff(root, "main", &default, "work/059-test", &work).is_err());
        std::fs::remove_file(root.join("unrelated")).unwrap();
        assert!(integrate_no_ff(root, "main", "0000000", "work/059-test", &work).is_err());
        assert!(integrate_no_ff(root, "main", &default, "work/059-test", &default).is_err());
    }
}
