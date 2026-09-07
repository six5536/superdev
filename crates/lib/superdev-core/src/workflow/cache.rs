//! Atomic transient Pi-session ownership for a workflow.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use fs2::FileExt;
use sha2::{Digest, Sha256};

use super::{WORKFLOW_CACHE_PATH, WorkflowCache};
use crate::error::{Error, Result};

fn path(root: &Path) -> PathBuf {
    root.join(WORKFLOW_CACHE_PATH)
}

fn lock_path(root: &Path) -> PathBuf {
    root.join(".superdev/cache/workflow.lock")
}

/// Digest an unpersisted UI authority capability for transient ownership.
pub fn authority_digest(capability: &str) -> Result<String> {
    if capability.len() < 32 {
        return Err(Error::Manifest {
            message: "workflow UI authority capability is absent or too short".into(),
        });
    }
    Ok(Sha256::digest(capability.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Verify UI authority without exposing or persisting the capability itself.
pub fn verify_authority(cache: &WorkflowCache, capability: &str) -> Result<()> {
    let supplied = authority_digest(capability)?;
    let expected = cache.authority_digest.as_bytes();
    let supplied = supplied.as_bytes();
    let mut difference = expected.len() ^ supplied.len();
    for index in 0..expected.len().max(supplied.len()) {
        difference |= usize::from(
            expected.get(index).copied().unwrap_or_default()
                ^ supplied.get(index).copied().unwrap_or_default(),
        );
    }
    if difference == 0 {
        Ok(())
    } else {
        Err(Error::Manifest {
            message: "interactive Pi UI authority is absent or invalid".into(),
        })
    }
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

/// A repository-wide workflow transaction. While this value is borrowed, no
/// other workflow cache transaction can interleave canonical publication with
/// its ownership compare-and-swap.
pub struct Transaction<'a> {
    root: &'a Path,
}

impl Transaction<'_> {
    /// Read transient ownership without reacquiring the transaction lock.
    pub fn load(&self) -> Result<Option<WorkflowCache>> {
        load_unlocked(self.root)
    }

    /// Compare and swap ownership state while retaining the transaction lock.
    pub fn compare_and_swap(
        &mut self,
        session: &str,
        expected_revision: &str,
        update: impl FnOnce(&mut WorkflowCache),
    ) -> Result<WorkflowCache> {
        compare_and_swap_unlocked(self.root, session, expected_revision, update)
    }

    /// Release matching ownership while retaining the transaction lock.
    pub fn release(&mut self, session: &str) -> Result<()> {
        release_unlocked(self.root, session)
    }
}

/// Hold the repository workflow lock across canonical validation, publication,
/// and the cache compare-and-swap performed by `operation`.
pub fn transaction<T>(
    root: &Path,
    operation: impl FnOnce(&mut Transaction<'_>) -> Result<T>,
) -> Result<T> {
    locked(root, || operation(&mut Transaction { root }))
}

/// A non-blocking ownership read used by observational status adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheSnapshot {
    /// The lock was acquired and an owner was read consistently.
    Owned(Box<WorkflowCache>),
    /// The lock was acquired and no owner exists.
    Unowned,
    /// Another workflow transaction currently owns the repository lock.
    Busy,
}

/// Read transient ownership. Absence means unowned, never complete.
pub fn load(root: &Path) -> Result<Option<WorkflowCache>> {
    locked(root, || load_unlocked(root))
}

/// Read ownership without waiting behind a workflow transaction.
///
/// A busy result is distinct from absent ownership so status adapters cannot
/// mistake an in-progress publication for an unowned workflow.
pub fn try_load(root: &Path) -> Result<CacheSnapshot> {
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
    match file.try_lock_exclusive() {
        Ok(()) => {
            let result = load_unlocked(root);
            FileExt::unlock(&file).map_err(|source| Error::Io {
                path: lock_path,
                source,
            })?;
            result.map(|owner| match owner {
                Some(owner) => CacheSnapshot::Owned(Box::new(owner)),
                None => CacheSnapshot::Unowned,
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(CacheSnapshot::Busy),
        Err(source) => Err(Error::Io {
            path: lock_path,
            source,
        }),
    }
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
    if cache.version != 1
        || cache.session_id.trim().is_empty()
        || cache.authority_digest.len() != 64
        || !cache
            .authority_digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(Error::Manifest {
            message: "workflow ownership requires cache version 1, a non-empty Pi session, and UI authority"
                .into(),
        });
    }
    locked(root, || {
        if let Some(owner) = load_unlocked(root)?
            && (owner.session_id != cache.session_id
                || owner.identity != cache.identity
                || owner.authority_digest != cache.authority_digest)
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
        compare_and_swap_unlocked(root, session, expected_revision, update)
    })
}

fn compare_and_swap_unlocked(
    root: &Path,
    session: &str,
    expected_revision: &str,
    update: impl FnOnce(&mut WorkflowCache),
) -> Result<WorkflowCache> {
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
}

/// Release ownership for the matching session without touching canonical state.
pub fn release(root: &Path, session: &str) -> Result<()> {
    locked(root, || release_unlocked(root, session))
}

fn release_unlocked(root: &Path, session: &str) -> Result<()> {
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
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|source| Error::Io {
        path: parent.into(),
        source,
    })?;
    temporary
        .write_all(text.as_bytes())
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|source| Error::Io {
            path: temporary.path().into(),
            source,
        })?;
    temporary
        .persist(&path)
        .map(|_| ())
        .map_err(|error| Error::Io {
            path,
            source: error.error,
        })
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
            authority_digest: authority_digest("0123456789abcdef0123456789abcdef").unwrap(),
            scope_base_revision: None,
            candidate_revision: None,
            verified_default_revision: None,
            child_role: None,
            child_pid: None,
            child_started: None,
            cancelled: false,
        }
    }

    #[test]
    fn authority_capabilities_are_hashed_and_compared() {
        let state = state("a");
        assert!(!state.authority_digest.contains("0123456789abcdef"));
        assert!(verify_authority(&state, "0123456789abcdef0123456789abcdef").is_ok());
        assert!(verify_authority(&state, "fedcba9876543210fedcba9876543210").is_err());
        assert!(authority_digest("short").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn cache_save_does_not_follow_the_legacy_predictable_temporary_symlink() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let cache = path(root.path());
        fs::create_dir_all(cache.parent().unwrap()).unwrap();
        let victim = root.path().join("victim");
        fs::write(&victim, "preserve\n").unwrap();
        let legacy_temporary = cache.with_extension(format!("tmp-{}", std::process::id()));
        symlink(&victim, legacy_temporary).unwrap();

        bind(root.path(), &state("a")).unwrap();
        assert_eq!(fs::read_to_string(victim).unwrap(), "preserve\n");
        assert_eq!(load(root.path()).unwrap().unwrap().session_id, "a");
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
    fn a_repository_transaction_serializes_publication_and_cache_cas() {
        use std::sync::mpsc;
        use std::time::Duration;

        let root = std::sync::Arc::new(tempfile::tempdir().unwrap());
        bind(root.path(), &state("a")).unwrap();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let held_root = std::sync::Arc::clone(&root);
        let held = std::thread::spawn(move || {
            transaction(held_root.path(), |transaction| {
                assert!(transaction.load()?.is_some());
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                Ok(())
            })
            .unwrap();
        });
        entered_rx.recv().unwrap();
        assert_eq!(try_load(root.path()).unwrap(), CacheSnapshot::Busy);

        let (finished_tx, finished_rx) = mpsc::channel();
        let waiting_root = std::sync::Arc::clone(&root);
        let waiting = std::thread::spawn(move || {
            compare_and_swap(waiting_root.path(), "a", "one", |cache| {
                cache.last_plan_revision = "two".into();
            })
            .unwrap();
            finished_tx.send(()).unwrap();
        });
        assert!(finished_rx.recv_timeout(Duration::from_millis(50)).is_err());
        release_tx.send(()).unwrap();
        finished_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        held.join().unwrap();
        waiting.join().unwrap();
        assert_eq!(
            load(root.path()).unwrap().unwrap().last_plan_revision,
            "two"
        );
        assert!(matches!(
            try_load(root.path()).unwrap(),
            CacheSnapshot::Owned(cache) if cache.last_plan_revision == "two"
        ));
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
