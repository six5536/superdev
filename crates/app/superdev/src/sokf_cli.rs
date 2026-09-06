//! sokf_cli.rs — the `sokf` verbs and the MCP server over the SOKF knowledge.
//!
//! Parsing, path defaults and printed output only; the work is all
//! `superdev-core`'s.

use std::io::{self, Read as _};
use std::path::Path;
use std::path::PathBuf;

use superdev_core::error::{Error, Result};
use superdev_core::manifest::{CONFIG_PATH, Manifest};
use superdev_core::sokf::{
    EditRequest, EmbeddingsConfig, ExactEdit, Index, IndexDir, MutationPolicy, MutationResult,
    SearchRequest, SokfServer, SokfService, WriteRequest, embedder_from, line_window, load_bundle,
};

use crate::cli::{INDEX_DIR, io_error, knowledge_dir, out};

// sokf:begin cli
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
    /// Orient in the SOKF knowledge
    Overview {
        /// Emit a tool-result JSON envelope
        #[arg(long)]
        json: bool,
    },
    /// Search the SOKF knowledge
    Search {
        /// What to look for, in the caller's own words
        query: String,
        /// Most sections to return
        #[arg(long)]
        limit: Option<u32>,
        /// Keep only concepts of this type; repeat for more than one
        #[arg(long = "type")]
        types: Vec<String>,
        /// Keep only concepts carrying this tag; repeat for more than one
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Keep only concepts with this lifecycle; repeat for more than one
        #[arg(long)]
        lifecycle: Vec<String>,
        /// Emit a tool-result JSON envelope
        #[arg(long)]
        json: bool,
    },
    /// Read one concept or section
    Read {
        /// Concept id or knowledge-relative path
        id: String,
        /// Heading or `parent > child` heading path
        #[arg(long)]
        heading: Option<String>,
        /// First rendered line to return, starting at 1
        #[arg(long)]
        offset: Option<usize>,
        /// Most rendered lines to return
        #[arg(long)]
        limit: Option<usize>,
        /// Emit a tool-result JSON envelope
        #[arg(long)]
        json: bool,
    },
    /// Edit an existing concept using exact replacements
    Edit {
        /// Concept id, virtual address, or knowledge path
        path: Option<String>,
        /// Exact text that must occur once in the original file
        #[arg(long, requires = "new_text")]
        old_text: Option<String>,
        /// Replacement text
        #[arg(long, requires = "old_text")]
        new_text: Option<String>,
        /// Read a coding-tool-shaped request from this file, or `-` for stdin
        #[arg(long, value_name = "FILE", conflicts_with_all = ["path", "old_text", "new_text"])]
        request_json: Option<PathBuf>,
        /// Deliberately permit a human to change `id` or `verified`
        #[arg(long)]
        allow_restricted: bool,
        /// Emit a tool-result JSON envelope
        #[arg(long)]
        json: bool,
    },
    /// Replace a concept, or create one at a physical knowledge path
    Write {
        /// Existing concept identity, or a physical path for creation
        path: Option<String>,
        /// Read the complete document from this file
        #[arg(long, value_name = "FILE", conflicts_with_all = ["content_stdin", "request_json"])]
        content_file: Option<PathBuf>,
        /// Read the complete document from stdin
        #[arg(long, conflicts_with_all = ["content_file", "request_json"])]
        content_stdin: bool,
        /// Read a coding-tool-shaped request from this file, or `-` for stdin
        #[arg(long, value_name = "FILE", conflicts_with_all = ["path", "content_file", "content_stdin"])]
        request_json: Option<PathBuf>,
        /// Deliberately permit a human to change `id` or `verified`
        #[arg(long)]
        allow_restricted: bool,
        /// Emit a tool-result JSON envelope
        #[arg(long)]
        json: bool,
    },
    /// Show the whole link graph or one concept's neighbours
    Graph {
        /// Concept id or knowledge-relative path
        id: Option<String>,
        /// Emit a tool-result JSON envelope
        #[arg(long)]
        json: bool,
    },
}
// sokf:end cli

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
        SokfCommand::Overview { json } => {
            print_service(root, *json, true, |service| service.overview())
        }
        SokfCommand::Search {
            query,
            limit,
            types,
            tags,
            lifecycle,
            json,
        } => print_service(root, *json, true, |service| {
            service.search(SearchRequest {
                query: query.clone(),
                limit: *limit,
                types: Some(types.clone()),
                tags: Some(tags.clone()),
                lifecycle: Some(lifecycle.clone()),
            })
        }),
        SokfCommand::Read {
            id,
            heading,
            offset,
            limit,
            json,
        } => print_service(root, *json, false, |service| {
            service
                .read(id, heading.as_deref())
                .and_then(|text| line_window(&text, *offset, *limit))
        }),
        SokfCommand::Edit {
            path,
            old_text,
            new_text,
            request_json,
            allow_restricted,
            json,
        } => {
            let request = if let Some(file) = request_json {
                parse_request(&read_input(file)?)?
            } else {
                EditRequest {
                    path: required(path.as_ref(), "edit requires PATH or --request-json")?.clone(),
                    edits: vec![ExactEdit {
                        old_text: required(old_text.as_ref(), "edit requires --old-text")?.clone(),
                        new_text: required(new_text.as_ref(), "edit requires --new-text")?.clone(),
                    }],
                }
            };
            let service = service(root, false)?;
            let result = service.edit(request, policy(*allow_restricted))?;
            print_mutation(&result, *json)
        }
        SokfCommand::Write {
            path,
            content_file,
            content_stdin,
            request_json,
            allow_restricted,
            json,
        } => {
            let request = if let Some(file) = request_json {
                parse_request(&read_input(file)?)?
            } else {
                let path =
                    required(path.as_ref(), "write requires PATH or --request-json")?.clone();
                let content = match (content_file, content_stdin) {
                    (Some(file), false) => read_input(file)?,
                    (None, true) => read_stdin()?,
                    _ => return sokf_error("write requires --content-file or --content-stdin"),
                };
                WriteRequest { path, content }
            };
            let service = service(root, false)?;
            let result = service.write(request, policy(*allow_restricted))?;
            print_mutation(&result, *json)
        }
        SokfCommand::Graph { id, json } => {
            print_service(root, *json, false, |service| service.graph(id.as_deref()))
        }
    }
}

/// Build the shared service, run one operation, and print its text result.
fn print_service(
    root: &Path,
    json: bool,
    needs_embedder: bool,
    operation: impl FnOnce(&SokfService) -> Result<String>,
) -> Result<u8> {
    let service = service(root, needs_embedder)?;
    let text = operation(&service)?;
    print_tool_result(&text, json, serde_json::json!({}))
}

fn service(root: &Path, needs_embedder: bool) -> Result<SokfService> {
    Ok(SokfService::new(
        knowledge_dir(root, None),
        root.to_path_buf(),
        IndexDir(root.join(INDEX_DIR)),
        needs_embedder
            .then(|| embedder(root))
            .transpose()?
            .flatten(),
    ))
}

fn print_mutation(result: &MutationResult, json: bool) -> Result<u8> {
    let validation = match result.validation {
        superdev_core::sokf::ValidationState::Valid => "valid",
        superdev_core::sokf::ValidationState::Invalid => "invalid",
        superdev_core::sokf::ValidationState::Unknown => "unknown",
    };
    let mut text = format!(
        "applied: {}\nvalidation: {validation}\nresolved: {}\nfinal: {}",
        result.applied, result.resolved_path, result.final_path
    );
    for change in &result.changes {
        text.push_str(&format!(
            "\n\n{:?}: {}\n{}",
            change.source, change.path, change.diff
        ));
    }
    for finding in &result.findings {
        text.push_str(&format!(
            "\n[{}] {}{}",
            finding.severity,
            finding
                .path
                .as_deref()
                .map_or_else(String::new, |path| format!("{path}: ")),
            finding.message
        ));
    }
    let details =
        serde_json::to_value(result).map_err(|error| io_error(io::Error::other(error)))?;
    print_tool_result(&text, json, details)
}

fn print_tool_result(text: &str, json: bool, details: serde_json::Value) -> Result<u8> {
    if json {
        let value = serde_json::json!({
            "protocol": "sokf-tools/v1",
            "content": [{ "type": "text", "text": text }],
            "details": details
        });
        let rendered =
            serde_json::to_string_pretty(&value).map_err(|e| io_error(io::Error::other(e)))?;
        out(&rendered)?;
    } else {
        out(text)?;
    }
    Ok(0)
}

fn policy(allow_restricted: bool) -> MutationPolicy {
    if allow_restricted {
        MutationPolicy::HumanOverride
    } else {
        MutationPolicy::AgentSafe
    }
}

fn read_input(path: &Path) -> Result<String> {
    if path == Path::new("-") {
        return read_stdin();
    }
    std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_stdin() -> Result<String> {
    let mut text = String::new();
    io::stdin().read_to_string(&mut text).map_err(io_error)?;
    Ok(text)
}

fn parse_request<T: serde::de::DeserializeOwned>(text: &str) -> Result<T> {
    serde_json::from_str(text).map_err(|error| Error::Sokf {
        message: format!("malformed request JSON: {error}"),
    })
}

fn required<'a, T>(value: Option<&'a T>, message: &str) -> Result<&'a T> {
    value.ok_or_else(|| Error::Sokf {
        message: message.into(),
    })
}

fn sokf_error<T>(message: &str) -> Result<T> {
    Err(Error::Sokf {
        message: message.into(),
    })
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
