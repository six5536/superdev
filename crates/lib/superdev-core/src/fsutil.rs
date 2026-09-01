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

/// A governed document's text, with CRLF line endings normalised to LF.
///
/// Every parser the validator runs — the frontmatter split, the fence scan,
/// the generated-block comparison — is written against LF. A checkout that
/// carries CRLF therefore registers no schema and reports every generated
/// block as ungenerated, which is what Windows saw (I040). This is the one
/// place that decides it, so a parser downstream never has to.
///
/// The engine's reads stay byte-exact through [`read_text`]: those hash what
/// they read to tell a superdev-written file from a user-edited one, and a
/// normalised read would report a CRLF file as drifted on every run.
pub(crate) fn read_document(path: &Path) -> Result<String> {
    let text = fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.into(),
        source,
    })?;
    Ok(lf(text))
}

/// `text` with CRLF line endings replaced by LF. A lone CR is left alone: it
/// ends no line in any format superdev reads, and rewriting it would edit
/// content rather than line endings.
pub(crate) fn lf(text: String) -> String {
    if text.contains("\r\n") {
        return text.replace("\r\n", "\n");
    }
    text
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
