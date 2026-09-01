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

/// Exact whole-line containment — the one rule for "the file already has
/// this line", shared by the planners and the engine's line applier.
pub(crate) fn has_line(content: &str, line: &str) -> bool {
    content.lines().any(|l| l == line)
}
