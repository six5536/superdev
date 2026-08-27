//! sokf_cli.rs — the `sokf` verbs and the MCP server over the SOKF knowledge.
//!
//! Parsing, path defaults and printed output only; the work is all
//! `superdev-core`'s.

use std::io;
use std::path::Path;
use std::path::PathBuf;

use superdev_core::error::{Error, Result};
use superdev_core::manifest::{CONFIG_PATH, Manifest};
use superdev_core::sokf::{
    EmbeddingsConfig, Index, IndexDir, SokfServer, embedder_from, load_bundle,
};

use crate::cli::{INDEX_DIR, io_error, knowledge_dir, out};

/// Serve a project subsystem over MCP.
#[derive(clap::Subcommand)]
pub enum McpCommand {
    /// Serve the SOKF knowledge over stdio
    Sokf,
}

/// Work on the SOKF knowledge.
#[derive(clap::Subcommand)]
pub enum SokfCommand {
    /// Rebuild the search index from scratch
    Index {
        /// SOKF knowledge directory (default: `knowledge`)
        path: Option<PathBuf>,
    },
}

/// Serve the SOKF knowledge over stdio until the client disconnects.
pub fn run_mcp(cmd: &McpCommand, root: &Path) -> Result<u8> {
    match cmd {
        McpCommand::Sokf => {
            let knowledge = knowledge_dir(root, None);
            // Fail at startup rather than answer every tool call with the same
            // error: a client has no way to act on the latter.
            if !knowledge.is_dir() {
                return Err(Error::Io {
                    path: knowledge,
                    source: io::Error::new(
                        io::ErrorKind::NotFound,
                        "no SOKF knowledge here — run `superdev init`",
                    ),
                });
            }
            let embedder = embedder(root)?;
            let index_dir = IndexDir(root.join(INDEX_DIR));
            // Sync before serving, so unreadable knowledge or an unwritable
            // index directory ends the process instead of failing every tool
            // call. It also warms the index for the client's first question.
            let (index, _) =
                Index::open_and_sync(&index_dir, &load_bundle(&knowledge)?, embedder.as_deref())?;
            // Never hold an index open across the rebuild a tool call may do.
            drop(index);
            let server = SokfServer::new(knowledge, root.to_path_buf(), index_dir, embedder);
            // One stdio client, and the server serialises its own tool calls:
            // a current-thread runtime is all this needs. Timers are not
            // optional — rmcp's request timeouts panic without them.
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .map_err(io_error)?;
            runtime.block_on(server.serve_stdio())?;
            Ok(0)
        }
    }
}

/// Rebuild the search index.
pub fn run_sokf(cmd: &SokfCommand, root: &Path) -> Result<u8> {
    match cmd {
        SokfCommand::Index { path } => {
            let knowledge = load_bundle(&knowledge_dir(root, path.as_deref()))?;
            let index_dir = IndexDir(root.join(INDEX_DIR));
            let (index, stats) =
                Index::force_rebuild(&index_dir, &knowledge, embedder(root)?.as_deref())?;
            out(&format!(
                "indexed {} concept(s), {} section(s) in {}",
                stats.reindexed,
                index.section_count(),
                index_dir.0.display()
            ))?;
            if stats.lexical_only {
                // embedder_from swallows a local-model load failure; without
                // this line the user would never learn search lost its vectors.
                out("search: lexical only — no embedding model loaded")?;
            }
            if !knowledge.broken.is_empty() {
                out(&format!(
                    "skipped {} unparseable file(s) — run `superdev validate`",
                    knowledge.broken.len()
                ))?;
            }
            Ok(0)
        }
    }
}

/// The embedder the manifest asks for, or the local default when the repo has
/// no manifest. A local model that will not load yields `None`, which leaves
/// search lexical.
fn embedder(root: &Path) -> Result<Option<Box<dyn superdev_core::sokf::Embedder>>> {
    embedder_from(embeddings(root)?.as_ref())
}

/// The `[knowledge.embeddings]` table, when there is one.
fn embeddings(root: &Path) -> Result<Option<EmbeddingsConfig>> {
    if !root.join(CONFIG_PATH).is_file() {
        return Ok(None);
    }
    Ok(Manifest::load(root)?.knowledge.embeddings)
}
