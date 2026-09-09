//! Shell-free Git operations and invariants used by workflow transitions.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::error::{Error, Result};

mod identity;
mod integration;
pub use identity::{
    default_branch, local_branches, paths_at_revision, require_unique_record_number,
};
pub use integration::integrate_no_ff;

/// Fixed service-owned subject for immutable SCOPE proposal checkpoints.
pub const SCOPE_CHECKPOINT_MESSAGE: &str = "docs(workflow): checkpoint scope proposal";

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

fn git_with_input(root: &Path, args: &[&str], input: &str) -> Result<Output> {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| Error::Command {
            command: format!("git {}", args.join(" ")),
            status: None,
            stderr: source.to_string(),
        })?;
    child
        .stdin
        .take()
        .expect("Git stdin was piped")
        .write_all(input.as_bytes())
        .map_err(|source| Error::Command {
            command: format!("git {}", args.join(" ")),
            status: None,
            stderr: source.to_string(),
        })?;
    let output = child.wait_with_output().map_err(|source| Error::Command {
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

fn commit_paths(
    root: &Path,
    message: &str,
    paths: &[String],
    expected_parent: Option<&str>,
    allow_empty: bool,
) -> Result<String> {
    if message.trim().is_empty() || message.contains(['\n', '\r', '\0']) {
        return Err(Error::Manifest {
            message: "workflow commit message is invalid".into(),
        });
    }
    if paths.is_empty() {
        if !allow_empty {
            return Err(Error::Manifest {
                message: "workflow mutation produced no commit changes".into(),
            });
        }
        require_clean(root)?;
    }

    // Build the commit through an isolated index. A failed add/tree/commit/ref
    // operation therefore cannot stage files or otherwise alter the live index.
    let parent = revision(root, "HEAD")?;
    if expected_parent.is_some_and(|expected| expected != parent) {
        return Err(Error::Manifest {
            message: "workflow commit parent changed before publication".into(),
        });
    }
    let temporary = tempfile::tempdir().map_err(|source| Error::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let index = temporary.path().join("index");
    git_with_index(root, &index, &["read-tree", &parent])?;
    if !paths.is_empty() {
        let mut add = vec!["add", "--all", "--"];
        add.extend(paths.iter().map(String::as_str));
        git_with_index(root, &index, &add)?;
    }
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

/// Require every current worktree change to remain in canonical knowledge.
pub fn require_knowledge_only_worktree(root: &Path) -> Result<()> {
    if working_paths(root)?
        .iter()
        .any(|path| !valid_path(path) || !path.starts_with("knowledge/"))
    {
        return Err(Error::Manifest {
            message: "SCOPE attempt requires only canonical knowledge worktree changes".into(),
        });
    }
    Ok(())
}

/// Commit only canonical knowledge changes produced after a clean-tree preflight.
pub fn commit_knowledge_changes(root: &Path, message: &str) -> Result<String> {
    require_knowledge_only_worktree(root)?;
    let paths = worktree_paths(root)?;
    commit_paths(root, message, &paths, None, false)
}

/// Commit canonical knowledge only if the checked-out branch still has the expected parent.
pub fn commit_knowledge_changes_at(
    root: &Path,
    message: &str,
    expected_parent: &str,
) -> Result<String> {
    validate_ref(expected_parent)?;
    let paths = worktree_paths(root)?;
    if paths
        .iter()
        .any(|path| !valid_path(path) || !path.starts_with("knowledge/"))
    {
        return Err(Error::Manifest {
            message: "workflow commit refused changes outside canonical knowledge".into(),
        });
    }
    commit_paths(root, message, &paths, Some(expected_parent), false)
}

/// Publish an immutable SCOPE snapshot, including an empty administrative commit
/// when the valid proposal is already committed. Other workflow commits still
/// require changes. The caller holds the ownership transaction and validates SCOPE.
pub fn commit_scope_checkpoint(root: &Path, expected_parent: &str) -> Result<String> {
    validate_ref(expected_parent)?;
    require_knowledge_only_worktree(root)?;
    commit_paths(
        root,
        SCOPE_CHECKPOINT_MESSAGE,
        &worktree_paths(root)?,
        Some(expected_parent),
        true,
    )
}

/// Commit the current worktree under one caller-supplied message.
///
/// Path shapes are validated and the commit stays local. Deciding *what*
/// belongs in the commit is the caller's judgement, not this function's.
/// An empty commit is permitted so an already-committed proposal can still be
/// published as an immutable checkpoint.
pub fn commit_worktree_changes(root: &Path, message: &str) -> Result<String> {
    let paths = worktree_paths(root)?;
    if paths.iter().any(|path| !valid_path(path)) {
        return Err(Error::Manifest {
            message: "workflow commit refused an unsafe path".into(),
        });
    }
    commit_paths(root, message, &paths, None, true)
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
    commit_paths(root, message, &paths, None, false)
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

/// Return whether a validated local work branch exists.
pub fn local_work_branch_exists(root: &Path, branch: &str) -> Result<bool> {
    validate_work_branch(branch)?;
    let reference = format!("refs/heads/{branch}");
    let output = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &reference])
        .current_dir(root)
        .output()
        .map_err(|source| Error::Command {
            command: format!("git show-ref --verify --quiet {reference}"),
            status: None,
            stderr: source.to_string(),
        })?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        status => Err(Error::Command {
            command: format!("git show-ref --verify --quiet {reference}"),
            status,
            stderr: String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(8_000)
                .collect(),
        }),
    }
}

/// Create and check out a validated work branch from the current revision.
pub fn create_work_branch(root: &Path, branch: &str) -> Result<()> {
    validate_work_branch(branch)?;
    require_clean(root)?;
    git(root, &["switch", "-c", branch]).map(|_| ())
}

/// Check out an existing validated work branch without carrying local changes.
pub fn checkout_work_branch(root: &Path, branch: &str) -> Result<()> {
    validate_work_branch(branch)?;
    require_clean(root)?;
    git(root, &["switch", branch]).map(|_| ())
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

/// Resolve the immutable merge base of two validated revisions.
pub fn merge_base(root: &Path, left: &str, right: &str) -> Result<String> {
    validate_ref(left)?;
    validate_ref(right)?;
    let output = git(root, &["merge-base", left, right])?;
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    validate_ref(&revision)?;
    Ok(revision)
}

/// Return every tracked or untracked working-tree path without invoking a shell.
pub fn working_paths(root: &Path) -> Result<Vec<String>> {
    let mut paths = BTreeSet::new();
    for args in [
        ["diff", "--name-only", "--"].as_slice(),
        ["diff", "--cached", "--name-only", "--"].as_slice(),
        ["ls-files", "--others", "--exclude-standard"].as_slice(),
    ] {
        let output = git(root, args)?;
        paths.extend(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|path| !path.is_empty())
                .map(str::to_string),
        );
    }
    Ok(paths.into_iter().collect())
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

/// Recover the baseline from the most recent service-owned transition into SCOPE.
pub fn scope_base_from_history(root: &Path, work_branch: &str, plan_path: &Path) -> Result<String> {
    validate_work_branch(work_branch)?;
    let plan_path = plan_path
        .strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .filter(|path| valid_path(path) && path.starts_with("knowledge/"))
        .ok_or_else(|| Error::Manifest {
            message: "SCOPE baseline plan path is invalid".into(),
        })?;
    // Empty and issue-only checkpoints do not appear in path-filtered history.
    let history = git(root, &["log", "--first-parent", "--format=%H", work_branch])?;
    for commit in String::from_utf8_lossy(&history.stdout).lines() {
        validate_ref(commit)?;
        let subject = git(root, &["show", "-s", "--format=%s", commit])?;
        let subject = String::from_utf8_lossy(&subject.stdout);
        if !matches!(
            subject.trim(),
            "chore(workflow): start scope"
                | "chore(workflow): return to scope"
                | SCOPE_CHECKPOINT_MESSAGE
        ) {
            continue;
        }
        let parent_expression = format!("{commit}^");
        let parent = git(root, &["rev-parse", "--verify", &parent_expression])?;
        let parent = String::from_utf8_lossy(&parent.stdout).trim().to_string();
        validate_ref(&parent)?;
        if changed_paths(root, &parent, commit)?
            .iter()
            .any(|path| !valid_path(path) || !path.starts_with("knowledge/"))
        {
            continue;
        }
        let scoped = file_at_revision(root, commit, plan_path)?;
        if !scoped.lines().any(|line| line == "phase: scope") {
            continue;
        }
        if subject.trim() == SCOPE_CHECKPOINT_MESSAGE {
            return Ok(parent);
        }
        let parent_plan = file_at_revision(root, &parent, plan_path);
        if subject.trim() == "chore(workflow): start scope" && parent_plan.is_err() {
            return Ok(parent);
        }
        if subject.trim() == "chore(workflow): return to scope"
            && parent_plan.is_ok_and(|text| {
                text.lines()
                    .any(|line| matches!(line, "phase: build" | "phase: accept"))
            })
            && scoped
                .lines()
                .any(|line| line == format!("Scope product baseline: {parent}."))
        {
            return Ok(parent);
        }
    }
    Err(Error::Manifest {
        message: "could not recover the service-owned SCOPE baseline from Git history".into(),
    })
}

/// Require all commits after the service-owned SCOPE baseline to be knowledge-only.
pub fn require_knowledge_only_since(root: &Path, baseline: &str, candidate: &str) -> Result<()> {
    if !is_ancestor(root, baseline, candidate)?
        || changed_paths(root, baseline, candidate)?
            .iter()
            .any(|path| !valid_path(path) || !path.starts_with("knowledge/"))
    {
        return Err(Error::Manifest {
            message: "SCOPE may change canonical knowledge only; product changes belong to BUILD"
                .into(),
        });
    }
    Ok(())
}

/// Require a service-owned knowledge-only SCOPE snapshot containing the plan.
/// A checkpoint need not change the plan or any files.
pub fn require_scope_checkpoint(root: &Path, candidate: &str, plan_path: &Path) -> Result<()> {
    validate_ref(candidate)?;
    let plan_path = plan_path
        .strip_prefix(root)
        .ok()
        .and_then(Path::to_str)
        .filter(|path| valid_path(path) && path.starts_with("knowledge/"))
        .ok_or_else(|| Error::Manifest {
            message: "SCOPE checkpoint plan path is invalid".into(),
        })?;
    let subject = git(root, &["show", "-s", "--format=%s", candidate])?;
    if String::from_utf8_lossy(&subject.stdout).trim() != SCOPE_CHECKPOINT_MESSAGE {
        return Err(Error::Manifest {
            message: "scope review candidate is not a service-owned SCOPE checkpoint".into(),
        });
    }
    let parent_expression = format!("{candidate}^");
    let parent = git(root, &["rev-parse", "--verify", &parent_expression])?;
    let parent = String::from_utf8_lossy(&parent.stdout).trim().to_string();
    let paths = changed_paths(root, &parent, candidate)?;
    let plan = file_at_revision(root, candidate, plan_path)?;
    if !plan.lines().any(|line| line == "phase: scope")
        || paths
            .iter()
            .any(|path| !valid_path(path) || !path.starts_with("knowledge/"))
    {
        return Err(Error::Manifest {
            message: "scope review candidate is not the knowledge-only plan checkpoint".into(),
        });
    }
    Ok(())
}

/// Compensate a just-published service commit after its cache CAS fails.
pub fn rollback_commit(root: &Path, commit: &str, parent: &str) -> Result<()> {
    validate_ref(commit)?;
    validate_ref(parent)?;
    git(root, &["update-ref", "HEAD", parent, commit])?;
    git(root, &["read-tree", parent])?;
    Ok(())
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
mod tests;
