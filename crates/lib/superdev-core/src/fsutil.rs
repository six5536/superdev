//! fsutil.rs — small pure file helpers shared by the engine and the
//! planners: read-or-absent, recursive listing, and parent-creating writes.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

/// File content, or None when the file is absent. Anything unreadable — a
/// binary file at a target path included — is an error, never an overwrite.
pub(crate) fn read_text(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::Io {
            path: path.into(),
            source: e,
        }),
    }
}

/// Every file under `dir`, recursively, in sorted order.
pub(crate) fn collect_files(dir: &Path, into: &mut Vec<std::path::PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| Error::Io {
            path: dir.into(),
            source: e,
        })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(&path, into)?;
        } else {
            into.push(path);
        }
    }
    Ok(())
}

pub(crate) fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::Io {
            path: parent.into(),
            source: e,
        })?;
    }
    fs::write(path, content).map_err(|e| Error::Io {
        path: path.into(),
        source: e,
    })
}
