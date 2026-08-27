//! cli.rs — what the verb modules share: where the SOKF knowledge lives, and
//! the one path to stdout.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use superdev_core::error::{Error, Result};

/// The SOKF knowledge directory when the caller names none, relative to the
/// repo root.
pub const KNOWLEDGE_DIR: &str = "knowledge";

/// Where the search index lives, relative to the repo root. It is machine
/// state: `.superdev/cache/` is gitignored by `init`.
pub const INDEX_DIR: &str = ".superdev/cache/sokf-index";

/// The knowledge to work on: what the caller named, else `<root>/knowledge`.
pub fn knowledge_dir(root: &Path, path: Option<&Path>) -> PathBuf {
    path.map_or_else(|| root.join(KNOWLEDGE_DIR), |path| root.join(path))
}

/// The one stdout path, so `main` can keep BrokenPipe a success.
pub fn out(s: &str) -> Result<()> {
    writeln!(io::stdout(), "{s}").map_err(io_error)
}

/// Failures on the stdout path carry `-`, so `main` can spot a broken pipe.
pub fn io_error(source: io::Error) -> Error {
    Error::Io {
        path: "-".into(),
        source,
    }
}
