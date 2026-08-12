//! Error type shared across superdev-core.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Any failure superdev-core can produce.
#[derive(Debug)]
pub enum Error {
    /// Filesystem operation failed.
    Io {
        /// Path the operation touched.
        path: PathBuf,
        /// Underlying error.
        source: io::Error,
    },
    /// A structured config file failed to parse or serialise. Named for TOML,
    /// which is most of them; it also carries JSON failures (`.mcp.json`).
    Toml {
        /// File concerned.
        path: PathBuf,
        /// Parser message.
        message: String,
    },
    /// An external command failed.
    Command {
        /// The command line that ran.
        command: String,
        /// Exit status, if the process ran at all.
        status: Option<i32>,
        /// Captured stderr, verbatim.
        stderr: String,
    },
    /// The manifest is invalid.
    Manifest {
        /// What is wrong.
        message: String,
    },
    /// An embedding model could not be loaded or called.
    Embedding {
        /// What failed, and where.
        message: String,
    },
    /// The search index could not be built, read or written.
    Index {
        /// What failed.
        message: String,
    },
    /// The MCP server could not start or could not run to completion.
    Mcp {
        /// What failed.
        message: String,
    },
}

/// Alias used across the crate.
pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Error::Toml { path, message } => write!(f, "{}: {message}", path.display()),
            Error::Command {
                command,
                status,
                stderr,
            } => {
                let stderr = stderr.replace('\n', " ");
                let stderr = stderr.trim();
                // No status means the process never ran, so there is no exit
                // code to name — the stderr says why (usually "not found").
                match status {
                    Some(status) => write!(f, "`{command}` failed (exit {status}): {stderr}"),
                    None => write!(f, "`{command}` failed: {stderr}"),
                }
            }
            Error::Manifest { message } => write!(f, "manifest: {message}"),
            Error::Embedding { message } => write!(f, "embedding: {message}"),
            Error::Index { message } => write!(f, "index: {message}"),
            Error::Mcp { message } => write!(f, "mcp: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use super::*;

    #[test]
    fn error_display_is_one_line() {
        let e = Error::Command {
            command: "claude plugin list".into(),
            status: Some(1),
            stderr: "boom".into(),
        };
        let s = e.to_string();
        assert!(s.contains("claude plugin list"));
        assert!(s.contains("boom"));
        assert!(!s.contains('\n'));
    }

    #[test]
    fn every_variant_displays() {
        let io = Error::Io {
            path: PathBuf::from("/tmp/a"),
            source: io::Error::other("nope"),
        };
        assert_eq!(io.to_string(), "/tmp/a: nope");
        assert!(io.source().is_some());

        let toml = Error::Toml {
            path: PathBuf::from("/tmp/b.toml"),
            message: "bad key".into(),
        };
        assert_eq!(toml.to_string(), "/tmp/b.toml: bad key");
        assert!(toml.source().is_none());

        let never_ran = Error::Command {
            command: "git".into(),
            status: None,
            stderr: "not found".into(),
        };
        assert_eq!(never_ran.to_string(), "`git` failed: not found");

        assert_eq!(
            Error::Manifest {
                message: "no name".into()
            }
            .to_string(),
            "manifest: no name"
        );

        assert_eq!(
            Error::Embedding {
                message: "OPENAI_API_KEY is not set".into()
            }
            .to_string(),
            "embedding: OPENAI_API_KEY is not set"
        );

        assert_eq!(
            Error::Index {
                message: "no such directory".into()
            }
            .to_string(),
            "index: no such directory"
        );

        assert_eq!(
            Error::Mcp {
                message: "transport closed".into()
            }
            .to_string(),
            "mcp: transport closed"
        );
    }
}
