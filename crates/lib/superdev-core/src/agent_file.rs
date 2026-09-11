//! Compose the managed instruction prefix without rewriting user content.

use std::io;

use crate::error::{Error, Result};

pub(crate) const PATH: &str = "AGENTS.md";
const START: &str = "<!-- superdev:instructions -->";
const END: &str = "<!-- /superdev:instructions -->";
const LEGACY_IMPORT: &str = "@.agents/superdev.md";

/// Put exactly one managed block first. Recognise markers and the old import
/// only as complete lines; retain every other byte, including line endings.
/// Refuse ambiguous boundaries rather than guessing which user text to remove.
pub(crate) fn render(existing: &str, instructions: &str) -> Result<String> {
    let mut user = String::new();
    let mut inside = false;
    let mut seen = false;
    for line in existing.split_inclusive('\n') {
        let text = line.strip_suffix('\n').unwrap_or(line);
        let text = text.strip_suffix('\r').unwrap_or(text);
        match text {
            START if !seen => {
                seen = true;
                inside = true;
            }
            END if inside => inside = false,
            START | END => return Err(marker_error()),
            LEGACY_IMPORT => {}
            _ if !inside => user.push_str(line),
            _ => {}
        }
    }
    if inside {
        return Err(marker_error());
    }
    let mut next = format!("{START}\n{instructions}");
    if !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(END);
    next.push('\n');
    next.push_str(&user);
    Ok(next)
}

fn marker_error() -> Error {
    Error::Io {
        path: PATH.into(),
        source: io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid Superdev instruction markers; restore one matching start/end pair before syncing",
        ),
    }
}
