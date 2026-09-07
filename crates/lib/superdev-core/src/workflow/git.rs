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

fn worktree_paths(root: &Path) -> Result<Vec<String>> {
    let tracked = git(root, &["diff", "--name-only", "HEAD", "--"])?;
    let untracked = git(root, &["ls-files", "--others", "--exclude-standard", "--"])?;
    let tracked_paths = String::from_utf8_lossy(&tracked.stdout);
    let untracked_paths = String::from_utf8_lossy(&untracked.stdout);
    let mut paths = tracked_paths
        .lines()
        .chain(untracked_paths.lines())
        .map(str::to_string)
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains("..")
        && !path.contains(['\n', '\r', '\0'])
}

fn commit_paths(root: &Path, message: &str, paths: &[String]) -> Result<String> {
    if message.trim().is_empty() || message.contains(['\n', '\r', '\0']) {
        return Err(Error::Manifest {
            message: "workflow commit message is invalid".into(),
        });
    }
    if paths.is_empty() {
        return Err(Error::Manifest {
            message: "workflow mutation produced no commit changes".into(),
        });
    }

    // Build the commit through an isolated index. A failed add/tree/commit/ref
    // operation therefore cannot stage files or otherwise alter the live index.
    let parent = revision(root, "HEAD")?;
    let temporary = tempfile::tempdir().map_err(|source| Error::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let index = temporary.path().join("index");
    git_with_index(root, &index, &["read-tree", &parent])?;
    let mut add = vec!["add", "--all", "--"];
    add.extend(paths.iter().map(String::as_str));
    git_with_index(root, &index, &add)?;
    let tree_output = git_with_index(root, &index, &["write-tree"])?;
    let tree = String::from_utf8_lossy(&tree_output.stdout)
        .trim()
        .to_string();
    validate_ref(&tree)?;
    let commit_output = git_with_index(
        root,
        &index,
        &["commit-tree", &tree, "-p", &parent, "-m", message],
    )?;
    let commit = String::from_utf8_lossy(&commit_output.stdout)
        .trim()
        .to_string();
    validate_ref(&commit)?;
    git(root, &["update-ref", "HEAD", &commit, &parent])?;
    // Every live-index/worktree change was included above. Align only the
    // index with the newly published tree; `read-tree` never changes files.
    git(root, &["read-tree", &commit])?;
    require_clean(root)?;
    Ok(commit)
}

fn git_with_index(root: &Path, index: &Path, args: &[&str]) -> Result<Output> {
    let output = Command::new("git")
        .args(args)
        .env("GIT_INDEX_FILE", index)
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

/// Commit only canonical knowledge changes produced after a clean-tree preflight.
pub fn commit_knowledge_changes(root: &Path, message: &str) -> Result<String> {
    let paths = worktree_paths(root)?;
    if paths
        .iter()
        .any(|path| !valid_path(path) || !path.starts_with("knowledge/"))
    {
        return Err(Error::Manifest {
            message: "workflow commit refused changes outside canonical knowledge".into(),
        });
    }
    commit_paths(root, message, &paths)
}

/// Commit one product-bearing BUILD checkpoint without absorbing paths outside
/// the current block's canonical Areas declarations.
pub fn commit_block_changes(
    root: &Path,
    message: &str,
    allowed_areas: &[String],
) -> Result<String> {
    validate_block_paths(root, allowed_areas)?;
    let paths = worktree_paths(root)?;
    commit_paths(root, message, &paths)
}

/// Refuse a BUILD checkpoint's current changes before any service-owned edit.
pub fn validate_block_paths(root: &Path, allowed_areas: &[String]) -> Result<()> {
    if allowed_areas.is_empty() || allowed_areas.iter().any(|area| !valid_path(area)) {
        return Err(Error::Manifest {
            message: "BUILD checkpoint has no valid path-scoped Areas".into(),
        });
    }
    let paths = worktree_paths(root)?;
    let allowed = |path: &str| {
        allowed_areas.iter().any(|area| {
            path == area || path.starts_with(&format!("{}/", area.trim_end_matches('/')))
        })
    };
    if paths.iter().any(|path| !valid_path(path) || !allowed(path)) {
        return Err(Error::Manifest {
            message: "BUILD checkpoint refused a change outside the current block Areas".into(),
        });
    }
    Ok(())
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

/// Incorporate an expected default tip into the checked-out work branch.
///
/// `merge-tree` proves the merge in the object database before the worktree is
/// touched. A conflict therefore leaves refs, index, and files unchanged. The
/// final fast-forward disables hooks and occurs only after both refs are
/// compare-and-swapped again.
pub fn synchronize_default(
    root: &Path,
    default_branch: &str,
    expected_default: &str,
    work_branch: &str,
    expected_work: &str,
) -> Result<String> {
    validate_ref(default_branch)?;
    validate_work_branch(work_branch)?;
    validate_ref(expected_default)?;
    validate_ref(expected_work)?;
    require_clean(root)?;
    if current_branch(root)? != work_branch {
        return Err(Error::Manifest {
            message: format!("synchronization requires checked-out branch `{work_branch}`"),
        });
    }
    if revision(root, default_branch)? != expected_default
        || revision(root, work_branch)? != expected_work
    {
        return Err(Error::Manifest {
            message: "workflow synchronization tips changed before merge".into(),
        });
    }
    if is_ancestor(root, expected_default, expected_work)? {
        return Ok(expected_work.to_string());
    }

    let merged = git(
        root,
        &[
            "merge-tree",
            "--write-tree",
            expected_work,
            expected_default,
        ],
    )?;
    let tree = String::from_utf8_lossy(&merged.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    validate_ref(&tree)?;
    let commit = git(
        root,
        &[
            "commit-tree",
            &tree,
            "-p",
            expected_work,
            "-p",
            expected_default,
            "-m",
            "chore(workflow): synchronize default branch",
        ],
    )?;
    let commit = String::from_utf8_lossy(&commit.stdout).trim().to_string();
    validate_ref(&commit)?;

    if revision(root, default_branch)? != expected_default
        || revision(root, work_branch)? != expected_work
    {
        return Err(Error::Manifest {
            message: "workflow synchronization tips changed; no ref was advanced".into(),
        });
    }
    git(
        root,
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "merge",
            "--ff-only",
            "--no-edit",
            &commit,
        ],
    )?;
    require_clean(root)?;
    if revision(root, work_branch)? != commit {
        return Err(Error::Manifest {
            message: "work branch did not reach the prepared synchronization commit".into(),
        });
    }
    Ok(commit)
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
    if current_branch(root)? != default_branch {
        git(root, &["switch", default_branch])?;
    }
    git(
        root,
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
            work_branch,
        ],
    )
    .map(|_| ())
}

/// Refuse option-like, traversal-like, or syntactically invalid refs.
pub fn validate_ref(reference: &str) -> Result<()> {
    if reference == "HEAD" {
        return Ok(());
    }
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
    fn canonical_knowledge_commit_refuses_unrelated_changes() {
        let dir = repository();
        std::fs::create_dir_all(dir.path().join("knowledge")).unwrap();
        std::fs::write(dir.path().join("knowledge/plan.md"), "plan\n").unwrap();
        let revision = commit_knowledge_changes(dir.path(), "chore: record evidence").unwrap();
        assert_eq!(revision, super::revision(dir.path(), "HEAD").unwrap());
        assert!(
            String::from_utf8_lossy(
                &git(dir.path(), &["show", "--format=", "--name-only", "HEAD"])
                    .unwrap()
                    .stdout
            )
            .contains("knowledge/plan.md")
        );

        std::fs::write(dir.path().join("knowledge/plan.md"), "changed\n").unwrap();
        std::fs::write(dir.path().join("unrelated"), "must remain\n").unwrap();
        assert!(commit_knowledge_changes(dir.path(), "chore: unsafe").is_err());
        assert!(
            !git(dir.path(), &["status", "--porcelain=v1"])
                .unwrap()
                .stdout
                .is_empty()
        );
    }

    #[test]
    fn failed_commit_construction_preserves_head_index_and_worktree() {
        let dir = repository();
        let root = dir.path();
        let head = revision(root, "HEAD").unwrap();
        std::fs::create_dir_all(root.join("knowledge")).unwrap();
        std::fs::write(root.join("knowledge/record.md"), "pending\n").unwrap();
        command(root, &["config", "user.name", ""]);
        command(root, &["config", "user.email", ""]);

        assert!(commit_knowledge_changes(root, "chore: must fail").is_err());
        assert_eq!(revision(root, "HEAD").unwrap(), head);
        assert_eq!(
            std::fs::read_to_string(root.join("knowledge/record.md")).unwrap(),
            "pending\n"
        );
        assert!(
            Command::new("git")
                .args(["diff", "--cached", "--quiet"])
                .current_dir(root)
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    fn block_commit_accepts_declared_product_and_knowledge_paths() {
        let dir = repository();
        let root = dir.path();
        std::fs::create_dir_all(root.join("knowledge/plans/open")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("knowledge/plans/open/plan.md"), "done\n").unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn built() {}\n").unwrap();
        let areas = vec!["knowledge/plans".into(), "src".into()];
        commit_block_changes(root, "feat: checkpoint block", &areas).unwrap();
        require_clean(root).unwrap();
    }

    #[test]
    fn block_commit_refuses_and_preserves_out_of_scope_changes() {
        let dir = repository();
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), "pub fn built() {}\n").unwrap();
        std::fs::write(root.join("unrelated"), "leave me\n").unwrap();
        assert!(commit_block_changes(root, "feat: unsafe", &["src".into()]).is_err());
        assert!(root.join("unrelated").exists());
        assert!(
            !git(root, &["status", "--porcelain=v1"])
                .unwrap()
                .stdout
                .is_empty()
        );
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
    fn synchronization_merges_expected_tips_without_exposing_conflicts() {
        let dir = repository();
        let root = dir.path();
        create_work_branch(root, "work/059-test").unwrap();
        std::fs::write(root.join("work-file"), "work\n").unwrap();
        command(root, &["add", "work-file"]);
        command(root, &["commit", "-q", "-m", "work"]);
        let work = revision(root, "work/059-test").unwrap();
        command(root, &["switch", "-q", "main"]);
        std::fs::write(root.join("default-file"), "default\n").unwrap();
        command(root, &["add", "default-file"]);
        command(root, &["commit", "-q", "-m", "default"]);
        let default = revision(root, "main").unwrap();
        command(root, &["switch", "-q", "work/059-test"]);

        let synchronized =
            synchronize_default(root, "main", &default, "work/059-test", &work).unwrap();
        assert_eq!(revision(root, "work/059-test").unwrap(), synchronized);
        assert!(is_ancestor(root, &default, &synchronized).unwrap());
        assert!(root.join("default-file").is_file());
        require_clean(root).unwrap();

        command(root, &["switch", "-q", "main"]);
        std::fs::write(root.join("file"), "default conflict\n").unwrap();
        command(root, &["commit", "-q", "-am", "default conflict"]);
        let conflicting_default = revision(root, "main").unwrap();
        command(root, &["switch", "-q", "work/059-test"]);
        std::fs::write(root.join("file"), "work conflict\n").unwrap();
        command(root, &["commit", "-q", "-am", "work conflict"]);
        let conflicting_work = revision(root, "work/059-test").unwrap();

        assert!(
            synchronize_default(
                root,
                "main",
                &conflicting_default,
                "work/059-test",
                &conflicting_work,
            )
            .is_err()
        );
        assert_eq!(revision(root, "work/059-test").unwrap(), conflicting_work);
        require_clean(root).unwrap();
    }

    #[test]
    fn synchronization_refuses_dirty_or_stale_tips_without_moving_the_branch() {
        let dir = repository();
        let root = dir.path();
        let default = revision(root, "main").unwrap();
        create_work_branch(root, "work/059-test").unwrap();
        std::fs::write(root.join("work-file"), "work\n").unwrap();
        command(root, &["add", "work-file"]);
        command(root, &["commit", "-q", "-m", "work"]);
        let work = revision(root, "work/059-test").unwrap();

        std::fs::write(root.join("dirty"), "leave this alone\n").unwrap();
        assert!(synchronize_default(root, "main", &default, "work/059-test", &work).is_err());
        assert_eq!(revision(root, "work/059-test").unwrap(), work);
        assert!(root.join("dirty").is_file());
        std::fs::remove_file(root.join("dirty")).unwrap();

        assert!(synchronize_default(root, "main", &work, "work/059-test", &work).is_err());
        assert!(synchronize_default(root, "main", &default, "work/059-test", &default).is_err());
        assert_eq!(revision(root, "work/059-test").unwrap(), work);
        require_clean(root).unwrap();
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
        let hook = root.join(".git/hooks/pre-merge-commit");
        std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&hook).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&hook, permissions).unwrap();
        }
        command(root, &["config", "commit.gpgsign", "true"]);

        integrate_no_ff(root, "main", &default, "work/059-test", &work).unwrap();
        assert_eq!(current_branch(root).unwrap(), "main");

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
