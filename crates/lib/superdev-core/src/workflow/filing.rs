//! Human-confirmed issue/idea filing on the local default branch.

use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Command, Output};

use fs2::FileExt;
use serde::{Deserialize, Serialize};

use super::git;
use crate::error::{Error, Result};
use crate::sokf::parse_concept;
use crate::validate;

/// The two out-of-band record kinds `/file` accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilingKind {
    /// A feature issue for scoped work.
    Issue,
    /// A draft idea without an implementation obligation.
    Idea,
}

/// One confirmed filing request.
#[derive(Debug, Clone)]
pub struct FilingRequest {
    /// Record kind.
    pub kind: FilingKind,
    /// Human title, preserved in frontmatter and the heading.
    pub title: String,
    /// Human description, preserved as the record body.
    pub description: String,
    /// Default branch to advance locally.
    pub default_branch: String,
}

/// Result of one locally committed filing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilingResult {
    /// New concept ID.
    pub id: String,
    /// Repository-relative path on the default branch.
    pub path: String,
    /// Knowledge-only commit that advanced the default branch.
    pub commit: String,
}

/// File and commit one record without touching an active workflow worktree.
pub fn file(root: &Path, request: &FilingRequest) -> Result<FilingResult> {
    git::validate_ref(&request.default_branch)?;
    let title = request.title.trim();
    let description = request.description.trim();
    if title.is_empty() || description.is_empty() {
        return Err(Error::Manifest {
            message: "filing requires a non-empty title and description".into(),
        });
    }
    with_lock(root, || file_locked(root, request, title, description))
}

fn file_locked(
    root: &Path,
    request: &FilingRequest,
    title: &str,
    description: &str,
) -> Result<FilingResult> {
    let default_tip = git::revision(root, &request.default_branch)?;
    let current = git::current_branch(root)?;
    if current == request.default_branch {
        git::require_clean(root)?;
    } else if branch_checked_out_elsewhere(root, &request.default_branch)? {
        return Err(Error::Manifest {
            message: format!(
                "default branch `{}` is checked out in another worktree",
                request.default_branch
            ),
        });
    }

    let worktree = root
        .join(".superdev/cache")
        .join(format!("filing-worktree-{}", std::process::id()));
    if worktree.exists() {
        return Err(Error::Manifest {
            message: format!("stale filing worktree exists at {}", worktree.display()),
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
    let result = prepare_commit(&worktree, request.kind, title, description);
    let cleanup = run_git(
        root,
        &["worktree", "remove", "--force", worktree.to_str().unwrap()],
    );
    cleanup?;
    let prepared = result?;

    if git::revision(root, &request.default_branch)? != default_tip {
        return Err(Error::Manifest {
            message: "default branch moved during filing; no ref was advanced".into(),
        });
    }
    if current == request.default_branch {
        run_git(root, &["merge", "--ff-only", &prepared.commit])?;
    } else {
        run_git(
            root,
            &[
                "update-ref",
                &format!("refs/heads/{}", request.default_branch),
                &prepared.commit,
                &default_tip,
            ],
        )?;
    }
    Ok(prepared)
}

fn prepare_commit(
    worktree: &Path,
    kind: FilingKind,
    title: &str,
    description: &str,
) -> Result<FilingResult> {
    reject_duplicate(worktree, title)?;
    let (directory, prefix, heading) = match kind {
        FilingKind::Issue => ("knowledge/issues/open", "issue", "Feature"),
        FilingKind::Idea => ("knowledge/ideas", "idea", "Idea"),
    };
    let number = next_number(&worktree.join("knowledge"), prefix)?;
    let slug = slug(title);
    if slug.is_empty() {
        return Err(Error::Manifest {
            message: "filing title must contain an ASCII letter or number".into(),
        });
    }
    let id = format!("{prefix}-{number:03}-{slug}");
    let path = format!("{directory}/{id}.md");
    let yaml_title = serde_json::to_string(title).expect("title serializes");
    let yaml_description = serde_json::to_string(description).expect("description serializes");
    let body = match kind {
        FilingKind::Issue => format!(
            "---\ntype: Issue\nid: {id}\ntitle: {yaml_title}\ndescription: {yaml_description}\nkind: feature\nlifecycle: open\n---\n\n# {heading}: {}\n\n## Summary\n\n{description}\n\n## Context\n\nFiled from the human request for later SCOPE review.\n\n## Behaviour\n\n{description}\n\n## Comments\n\nCaptured without creating a plan or work branch.\n",
            title.to_lowercase()
        ),
        FilingKind::Idea => format!(
            "---\ntype: Idea\nid: {id}\ntitle: {yaml_title}\ndescription: {yaml_description}\nstatus: draft\n---\n\n# {heading}: {}\n\n{description}\n",
            title.to_lowercase()
        ),
    };
    let target = worktree.join(&path);
    fs::create_dir_all(target.parent().unwrap()).map_err(|source| Error::Io {
        path: target.parent().unwrap().into(),
        source,
    })?;
    fs::write(&target, body).map_err(|source| Error::Io {
        path: target.clone(),
        source,
    })?;
    validate::fix_repo(worktree, &worktree.join("knowledge"), &[])?;
    let grammar = validate::schema::load_grammar(worktree)?;
    let index = match kind {
        FilingKind::Issue => worktree.join("knowledge/issues/index.md"),
        FilingKind::Idea => worktree.join("knowledge/ideas/index.md"),
    };
    let report = validate::validate_repo(
        worktree,
        &worktree.join("knowledge"),
        &[target.clone(), index.clone()],
        &grammar,
    )?;
    if !report.report.passed() {
        return Err(Error::Manifest {
            message: format!(
                "filed record did not pass repository validation:\n{}",
                report.report.render_human(validate::sokf::Warnings::Listed)
            ),
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
        path.as_str(),
        index.strip_prefix(worktree).unwrap().to_str().unwrap(),
    ];
    for line in String::from_utf8_lossy(&changed.stdout).lines() {
        let changed_path = line.get(3..).unwrap_or_default();
        if !allowed.contains(&changed_path) {
            return Err(Error::Manifest {
                message: format!(
                    "filing repair touched unrelated knowledge `{changed_path}`; no commit was created"
                ),
            });
        }
    }
    run_git(worktree, &["add", "--", &path, allowed[1]])?;
    run_git(
        worktree,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            &format!("docs: file {id}"),
        ],
    )?;
    let head = run_git(worktree, &["rev-parse", "--verify", "HEAD"])?;
    let commit = String::from_utf8_lossy(&head.stdout).trim().to_string();
    Ok(FilingResult { id, path, commit })
}

fn reject_duplicate(root: &Path, title: &str) -> Result<()> {
    let needle = title.to_lowercase();
    let mut directories = vec![root.join("knowledge/issues"), root.join("knowledge/ideas")];
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
                let duplicate = parse_concept(&path.to_string_lossy(), &text)
                    .ok()
                    .and_then(|concept| concept.raw["title"].as_str().map(str::to_lowercase))
                    .is_some_and(|existing| existing == needle);
                if duplicate {
                    return Err(Error::Manifest {
                        message: format!("a record titled `{title}` already exists"),
                    });
                }
            }
        }
    }
    Ok(())
}

fn next_number(knowledge: &Path, prefix: &str) -> Result<u32> {
    let mut highest = 0;
    let mut directories = vec![knowledge.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).map_err(|source| Error::Io {
            path: directory.clone(),
            source,
        })? {
            let path = entry
                .map_err(|source| Error::Io {
                    path: directory.clone(),
                    source,
                })?
                .path();
            if path.is_dir() {
                directories.push(path);
            } else if let Some(name) = path.file_stem().and_then(|name| name.to_str())
                && let Some(number) = name
                    .strip_prefix(&format!("{prefix}-"))
                    .and_then(|rest| rest.split('-').next())
                    .and_then(|number| number.parse::<u32>().ok())
            {
                highest = highest.max(number);
            }
        }
    }
    Ok(highest + 1)
}

fn slug(title: &str) -> String {
    title
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn branch_checked_out_elsewhere(root: &Path, branch: &str) -> Result<bool> {
    let output = run_git(root, &["worktree", "list", "--porcelain"])?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line == format!("branch refs/heads/{branch}")))
}

fn with_lock<T>(root: &Path, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    let path = root.join(".superdev/cache/filing.lock");
    fs::create_dir_all(path.parent().unwrap()).map_err(|source| Error::Io {
        path: path.parent().unwrap().into(),
        source,
    })?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
    file.lock_exclusive().map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    let result = operation();
    FileExt::unlock(&file).map_err(|source| Error::Io { path, source })?;
    result
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
