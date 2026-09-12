//! Capability-relative atomic JSON storage under the repository workflow lock.

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};
use serde::{Serialize, de::DeserializeOwned};

use super::local::{RECORD_VERSION, WORKFLOW_RECORDS_PATH, WorkflowRecord};
use crate::error::{Error, Result};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_RECORD_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn refusal(message: impl Into<String>) -> Error {
    Error::Manifest {
        message: message.into(),
    }
}

pub(super) fn hash(bytes: &[u8]) -> String {
    crate::lock::sha256_hex(bytes)
}

pub(super) fn identifier(id: &str) -> Result<()> {
    if id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(refusal("invalid internal workflow ID"));
    }
    Ok(())
}

/// Read a durable record without acquiring execution ownership.
/// Absence is not evidence of approval or completion.
pub fn load(root: &Path, id: &str) -> Result<Option<WorkflowRecord>> {
    super::cache::transaction(root, |_| Store::open(root)?.load(id))
}

/// List this checkout's local workflows, without inspecting plan phase prose.
pub fn list(root: &Path) -> Result<Vec<WorkflowRecord>> {
    super::cache::transaction(root, |_| Store::open(root)?.list())
}

pub(super) struct Files {
    directory: Dir,
    display: PathBuf,
}

impl Files {
    pub(super) fn open(root: &Path, relative: &str) -> Result<Self> {
        let repository =
            Dir::open_ambient_dir(root, ambient_authority()).map_err(|source| Error::Io {
                path: root.into(),
                source,
            })?;
        repository
            .create_dir_all(relative)
            .map_err(|source| Error::Io {
                path: root.join(relative),
                source,
            })?;
        let directory = repository.open_dir(relative).map_err(|source| Error::Io {
            path: root.join(relative),
            source,
        })?;
        Ok(Self {
            directory,
            display: root.join(relative),
        })
    }

    pub(super) fn read<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>> {
        let path = self.display.join(name);
        let file = match self.directory.open(name) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(Error::Io { path, source }),
        };
        let mut bytes = Vec::new();
        file.take(MAX_RECORD_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            return Err(refusal(format!(
                "local state {} exceeds the size limit",
                path.display()
            )));
        }
        serde_json::from_slice(&bytes).map(Some).map_err(|error| {
            refusal(format!("corrupt or incompatible local state {}: {error}; preserve the file and reconstruct with human approval", path.display()))
        })
    }

    pub(super) fn write<T: Serialize>(&self, name: &str, value: &T) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(value)
            .map_err(|error| refusal(format!("local state did not serialise: {error}")))?;
        self.write_bytes(name, &bytes)
    }

    pub(super) fn write_bytes(&self, name: &str, bytes: &[u8]) -> Result<()> {
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            return Err(refusal("local state exceeds the size limit"));
        }
        let temporary = format!(
            ".{}-{}.tmp",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = self
            .directory
            .open_with(&temporary, &options)
            .map_err(|source| Error::Io {
                path: self.display.join(&temporary),
                source,
            })?;
        let result = file.write_all(bytes).and_then(|()| file.sync_all());
        drop(file);
        let result = result.and_then(|()| self.directory.rename(&temporary, &self.directory, name));
        if let Err(source) = result {
            // A failed write never replaces the previous record. A leftover
            // temporary file is not a record and is safe to inspect or remove.
            let _ = self.directory.remove_file(&temporary);
            return Err(Error::Io {
                path: self.display.join(name),
                source,
            });
        }
        #[cfg(unix)]
        self.directory
            .open(".")
            .and_then(|file| file.sync_all())
            .map_err(|source| Error::Io {
                path: self.display.clone(),
                source,
            })?;
        Ok(())
    }

    pub(super) fn remove(&self, name: &str) -> Result<()> {
        self.directory
            .remove_file(name)
            .map_err(|source| Error::Io {
                path: self.display.join(name),
                source,
            })
    }
}

/// Methods are used only while `cache::transaction` holds the checkout lock.
pub(super) struct Store {
    files: Files,
    checkout: String,
}

impl Store {
    pub(super) fn open(root: &Path) -> Result<Self> {
        let checkout = root
            .canonicalize()
            .map_err(|source| Error::Io {
                path: root.into(),
                source,
            })?
            .to_string_lossy()
            .into_owned();
        Ok(Self {
            files: Files::open(root, WORKFLOW_RECORDS_PATH)?,
            checkout,
        })
    }

    pub(super) fn new_id(&self) -> String {
        // This is uniqueness, not a credential. Exclusive creation and the
        // repository lock remain the collision barrier.
        let seed = format!(
            "{}:{:?}:{}:{}",
            self.checkout,
            std::time::SystemTime::now(),
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        hash(seed.as_bytes())[..32].into()
    }

    pub(super) fn checkout(&self) -> &str {
        &self.checkout
    }

    pub(super) fn load(&self, id: &str) -> Result<Option<WorkflowRecord>> {
        identifier(id)?;
        let record: Option<WorkflowRecord> = self.files.read(&format!("{id}.json"))?;
        if let Some(record) = &record {
            self.validate(record)?;
            if record.id != id {
                return Err(refusal("workflow filename and identity differ"));
            }
        }
        Ok(record)
    }

    pub(super) fn list(&self) -> Result<Vec<WorkflowRecord>> {
        let mut records = Vec::new();
        for entry in self.files.directory.entries().map_err(|source| Error::Io {
            path: self.files.display.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| Error::Io {
                path: self.files.display.clone(),
                source,
            })?;
            let name = entry.file_name();
            if let Some(id) = name.to_str().and_then(|name| name.strip_suffix(".json")) {
                records.push(
                    self.load(id)?
                        .ok_or_else(|| refusal("workflow vanished during read"))?,
                );
            }
        }
        records.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(records)
    }

    pub(super) fn create(&self, record: &WorkflowRecord) -> Result<()> {
        if self.load(&record.id)?.is_some() {
            return Err(refusal("workflow ID is already reserved"));
        }
        if record.revision != 0 {
            return Err(refusal("new workflow revision must be zero"));
        }
        self.validate(record)?;
        self.files.write(&format!("{}.json", record.id), record)
    }

    pub(super) fn save(&self, record: &mut WorkflowRecord) -> Result<()> {
        let previous = self
            .load(&record.id)?
            .ok_or_else(|| refusal("local workflow is missing; reconstruct with human approval"))?;
        if previous.revision != record.revision {
            return Err(refusal(
                "workflow revision changed; reload before continuing",
            ));
        }
        self.validate(record)?;
        let mut next = record.clone();
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or_else(|| refusal("workflow revision exhausted"))?;
        self.files.write(&format!("{}.json", record.id), &next)?;
        *record = next;
        Ok(())
    }

    fn validate(&self, record: &WorkflowRecord) -> Result<()> {
        identifier(&record.id)?;
        if record.version != RECORD_VERSION {
            return Err(refusal(format!(
                "unsupported workflow record version {}; preserve local state before recovery",
                record.version
            )));
        }
        if record.checkout != self.checkout {
            return Err(refusal(
                "workflow belongs to another checkout; reconstruct locally with fresh approval",
            ));
        }
        Ok(())
    }
}
