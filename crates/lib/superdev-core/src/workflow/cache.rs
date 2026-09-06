//! Atomic transient Pi-session ownership for a workflow.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use fs2::FileExt;

use super::{WORKFLOW_CACHE_PATH, WorkflowCache};
use crate::error::{Error, Result};

fn path(root: &Path) -> PathBuf {
    root.join(WORKFLOW_CACHE_PATH)
}

fn lock_path(root: &Path) -> PathBuf {
    root.join(".superdev/cache/workflow.lock")
}

/// Hold the repository workflow lock for one complete read/check/write
/// transaction. The persistent empty lock file makes crash recovery safe.
fn locked<T>(root: &Path, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    let lock_path = lock_path(root);
    let parent = lock_path.parent().expect("workflow lock has a parent");
    fs::create_dir_all(parent).map_err(|source| Error::Io {
        path: parent.into(),
        source,
    })?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|source| Error::Io {
            path: lock_path.clone(),
            source,
        })?;
    file.lock_exclusive().map_err(|source| Error::Io {
        path: lock_path.clone(),
        source,
    })?;
    let result = operation();
    FileExt::unlock(&file).map_err(|source| Error::Io {
        path: lock_path,
        source,
    })?;
    result
}

/// Read transient ownership. Absence means unowned, never complete.
pub fn load(root: &Path) -> Result<Option<WorkflowCache>> {
    locked(root, || load_unlocked(root))
}

fn load_unlocked(root: &Path) -> Result<Option<WorkflowCache>> {
    let path = path(root);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(Error::Io { path, source }),
    };
    toml_edit::de::from_str(&text)
        .map(Some)
        .map_err(|error| Error::Toml {
            path,
            message: error.to_string(),
        })
}

/// Acquire unowned state or resume it with the same session and identity.
pub fn bind(root: &Path, cache: &WorkflowCache) -> Result<()> {
    if cache.version != 1 || cache.session_id.trim().is_empty() {
        return Err(Error::Manifest {
            message: "workflow ownership requires cache version 1 and a non-empty Pi session"
                .into(),
        });
    }
    locked(root, || {
        if let Some(owner) = load_unlocked(root)?
            && (owner.session_id != cache.session_id || owner.identity != cache.identity)
        {
            return Err(Error::Manifest {
                message: format!("workflow is owned by Pi session `{}`", owner.session_id),
            });
        }
        save_unlocked(root, cache)
    })
}

/// Compare-and-swap transient state for the owning session and plan revision.
pub fn compare_and_swap(
    root: &Path,
    session: &str,
    expected_revision: &str,
    update: impl FnOnce(&mut WorkflowCache),
) -> Result<WorkflowCache> {
    locked(root, || {
        let mut cache = load_unlocked(root)?.ok_or_else(|| Error::Manifest {
            message: "workflow has no owning Pi session; bind or resume it first".into(),
        })?;
        if cache.session_id != session {
            return Err(Error::Manifest {
                message: format!("workflow is owned by Pi session `{}`", cache.session_id),
            });
        }
        if cache.last_plan_revision != expected_revision {
            return Err(Error::Manifest {
                message: "workflow plan revision changed; reload status before mutating".into(),
            });
        }
        update(&mut cache);
        save_unlocked(root, &cache)?;
        Ok(cache)
    })
}

/// Release ownership for the matching session without touching canonical state.
pub fn release(root: &Path, session: &str) -> Result<()> {
    locked(root, || {
        let Some(cache) = load_unlocked(root)? else {
            return Ok(());
        };
        if cache.session_id != session {
            return Err(Error::Manifest {
                message: format!("workflow is owned by Pi session `{}`", cache.session_id),
            });
        }
        fs::remove_file(path(root)).map_err(|source| Error::Io {
            path: path(root),
            source,
        })
    })
}

fn save_unlocked(root: &Path, cache: &WorkflowCache) -> Result<()> {
    let path = path(root);
    let parent = path.parent().expect("cache path has a parent");
    fs::create_dir_all(parent).map_err(|source| Error::Io {
        path: parent.into(),
        source,
    })?;
    let text = toml_edit::ser::to_string_pretty(cache).map_err(|error| Error::Toml {
        path: path.clone(),
        message: error.to_string(),
    })?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, text).map_err(|source| Error::Io {
        path: temporary.clone(),
        source,
    })?;
    fs::rename(&temporary, &path).map_err(|source| Error::Io { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::WorkflowIdentity;

    fn state(session: &str) -> WorkflowCache {
        WorkflowCache {
            version: 1,
            session_id: session.into(),
            identity: WorkflowIdentity {
                issue: "issue-001-x".into(),
                plan: "plan-001-x".into(),
                work_branch: "work/001-x".into(),
                default_branch: "main".into(),
            },
            last_plan_revision: "one".into(),
            candidate_revision: None,
            verified_default_revision: None,
            child_role: None,
            child_pid: None,
            child_started: None,
            cancelled: false,
        }
    }

    #[test]
    fn ownership_and_revision_are_compare_and_swapped() {
        let root = tempfile::tempdir().unwrap();
        assert!(load(root.path()).unwrap().is_none());
        assert!(compare_and_swap(root.path(), "a", "one", |_| {}).is_err());
        bind(root.path(), &state("a")).unwrap();
        assert!(bind(root.path(), &state("b")).is_err());
        assert!(compare_and_swap(root.path(), "a", "stale", |_| {}).is_err());
        let changed = compare_and_swap(root.path(), "a", "one", |state| {
            state.last_plan_revision = "two".into();
        })
        .unwrap();
        assert_eq!(changed.last_plan_revision, "two");
        assert!(release(root.path(), "b").is_err());
        release(root.path(), "a").unwrap();
        assert!(load(root.path()).unwrap().is_none());
    }

    #[test]
    fn concurrent_binds_have_exactly_one_owner() {
        use std::sync::{Arc, Barrier};

        let root = Arc::new(tempfile::tempdir().unwrap());
        let barrier = Arc::new(Barrier::new(3));
        let mut threads = Vec::new();
        for session in ["a", "b"] {
            let root = Arc::clone(&root);
            let barrier = Arc::clone(&barrier);
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                bind(root.path(), &state(session)).is_ok()
            }));
        }
        barrier.wait();
        let won = threads
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .filter(|won| *won)
            .count();
        assert_eq!(won, 1);
    }
}
