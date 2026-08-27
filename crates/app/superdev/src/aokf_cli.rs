//! aokf_cli.rs — the `aokf` and `mcp` verbs over the knowledge bundle.
//!
//! Parsing, path defaults and printed output only; the bundle work is all
//! `superdev-core`'s.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use superdev_core::aokf::{
    AokfServer, EmbeddingsConfig, Index, IndexDir, embedder_from, load_bundle, validate,
};
use superdev_core::capability::Capability;
use superdev_core::error::{Error, Result};
use superdev_core::manifest::{CONFIG_PATH, Manifest};

/// Bundle directory when the caller names none, relative to the repo root.
const BUNDLE_DIR: &str = "knowledge";

/// Where the search index lives, relative to the repo root. It is machine
/// state: `.superdev/cache/` is gitignored by `init`.
const INDEX_DIR: &str = ".superdev/cache/aokf-index";

/// Serve a project subsystem over MCP.
#[derive(clap::Subcommand)]
pub enum McpCommand {
    /// Serve the AOKF bundle over stdio
    Aokf,
}

/// Work on the AOKF knowledge bundle.
#[derive(clap::Subcommand)]
pub enum AokfCommand {
    /// Validate the bundle against the AOKF spec
    Validate {
        /// Bundle directory (default: `knowledge`)
        path: Option<PathBuf>,
        /// Emit JSON instead of text
        #[arg(long)]
        json: bool,
        /// Repository root for `/`-rooted paths (default: this repo)
        #[arg(long)]
        repo_root: Option<PathBuf>,
    },
    /// Rebuild the search index from scratch
    Index {
        /// Bundle directory (default: `knowledge`)
        path: Option<PathBuf>,
    },
    /// Claude Code hook plumbing (reads the hook payload from stdin)
    #[command(subcommand)]
    Hook(HookCommand),
}

/// One verb per hook, so future hooks slot in beside `validate`.
#[derive(clap::Subcommand)]
pub enum HookCommand {
    /// PostToolUse: validate the bundle after an Edit/Write under knowledge/
    Validate,
}

/// Serve the bundle over stdio until the client disconnects.
pub fn run_mcp(cmd: &McpCommand, root: &Path) -> Result<u8> {
    match cmd {
        McpCommand::Aokf => {
            let bundle_dir = bundle_dir(root, None);
            // Fail at startup rather than answer every tool call with the same
            // error: a client has no way to act on the latter.
            if !bundle_dir.is_dir() {
                return Err(Error::Io {
                    path: bundle_dir,
                    source: io::Error::new(
                        io::ErrorKind::NotFound,
                        "no AOKF bundle here — run `superdev init`",
                    ),
                });
            }
            let embedder = embedder(root)?;
            let index_dir = IndexDir(root.join(INDEX_DIR));
            // Sync before serving, so an unreadable bundle or an unwritable
            // index directory ends the process instead of failing every tool
            // call. It also warms the index for the client's first question.
            let (index, _) =
                Index::open_and_sync(&index_dir, &load_bundle(&bundle_dir)?, embedder.as_deref())?;
            // Never hold an index open across the rebuild a tool call may do.
            drop(index);
            let server = AokfServer::new(bundle_dir, root.to_path_buf(), index_dir, embedder);
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

/// Validate or index the bundle.
pub fn run_aokf(cmd: &AokfCommand, root: &Path) -> Result<u8> {
    match cmd {
        AokfCommand::Validate {
            path,
            json,
            repo_root,
        } => {
            let bundle_dir = bundle_dir(root, path.as_deref());
            let bundle = load_bundle(&bundle_dir)?;
            let repo_root = repo_root
                .as_deref()
                .map_or_else(|| root.to_path_buf(), |p| root.join(p));
            let report = validate(&bundle, &repo_root);
            if *json {
                let mut value = report.to_json();
                // The bundle path is the caller's string, so core leaves the
                // key to the caller. The reference validator emits it.
                if let Some(object) = value.as_object_mut() {
                    object.insert("bundle".into(), bundle_dir.display().to_string().into());
                }
                let rendered = serde_json::to_string_pretty(&value)
                    .map_err(|e| io_error(io::Error::other(e)))?;
                out(&rendered)?;
            } else {
                out(&format!(
                    "AOKF validator — bundle: {}",
                    bundle_dir.display()
                ))?;
                out(report.render_human().trim_end_matches('\n'))?;
            }
            Ok(u8::from(!report.passed()))
        }
        AokfCommand::Index { path } => {
            let bundle = load_bundle(&bundle_dir(root, path.as_deref()))?;
            let index_dir = IndexDir(root.join(INDEX_DIR));
            let (index, stats) =
                Index::force_rebuild(&index_dir, &bundle, embedder(root)?.as_deref())?;
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
            if !bundle.broken.is_empty() {
                out(&format!(
                    "skipped {} unparseable file(s) — run `superdev aokf validate`",
                    bundle.broken.len()
                ))?;
            }
            Ok(0)
        }
        AokfCommand::Hook(HookCommand::Validate) => hook_validate(root),
    }
}

/// The PostToolUse hook body. Exit 0 unless the payload names a path under
/// the bundle; then validate and exit 2 with findings on errors, which
/// Claude Code feeds back to the agent as a blocking error. An unreadable
/// payload is a loud exit 2 — a silent skip here silently stops validating
/// the bundle.
fn hook_validate(root: &Path) -> Result<u8> {
    // Hooks run with the project as the working directory, but Claude Code
    // also names it explicitly; prefer the explicit form.
    let root =
        std::env::var_os("CLAUDE_PROJECT_DIR").map_or_else(|| root.to_path_buf(), PathBuf::from);
    let mut payload = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut payload) {
        eprintln!("aokf hook: could not read the tool payload from stdin: {e}");
        return Ok(2);
    }
    let parsed: serde_json::Value = match serde_json::from_str(&payload) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("aokf hook: malformed tool payload on stdin: {e}");
            return Ok(2);
        }
    };
    let Some(file_path) = parsed["tool_input"]["file_path"].as_str() else {
        // Not a file edit: nothing to validate.
        return Ok(0);
    };
    let bundle = root.join(BUNDLE_DIR);
    let edited = Path::new(file_path);
    if !under_bundle(&bundle, edited) {
        return Ok(0);
    }
    let report = validate(&load_bundle(&bundle)?, &root);
    if report.passed() {
        return Ok(0);
    }
    eprintln!("AOKF validation failed after editing {file_path} — fix before continuing:");
    eprintln!("{}", report.render_human().trim_end_matches('\n'));
    Ok(2)
}

/// Does the edited path name a file under the bundle? Falling back to the
/// working directory gets a root with symlinks already resolved — macOS
/// spells a temp dir `/private/var/…` where the payload still says `/var/…`
/// — so a path that misses lexically gets a second look through the
/// resolved spelling of both sides. Resolving only as a fallback keeps a
/// symlink *into* the bundle matching on its bundle-side name.
fn under_bundle(bundle: &Path, edited: &Path) -> bool {
    if edited.starts_with(bundle) || edited.starts_with(BUNDLE_DIR) {
        return true;
    }
    match (bundle.canonicalize(), edited.canonicalize()) {
        (Ok(bundle), Ok(edited)) => edited.starts_with(bundle),
        _ => false,
    }
}

/// The bundle to work on: what the caller named, else `<root>/knowledge`.
fn bundle_dir(root: &Path, path: Option<&Path>) -> PathBuf {
    path.map_or_else(|| root.join(BUNDLE_DIR), |path| root.join(path))
}

/// The embedder the manifest asks for, or the local default when the repo has
/// no manifest. A local model that will not load yields `None`, which leaves
/// search lexical.
fn embedder(root: &Path) -> Result<Option<Box<dyn superdev_core::aokf::Embedder>>> {
    embedder_from(embeddings(root)?.as_ref())
}

/// The knowledge capability's `[knowledge.embeddings]` table, when there is one.
fn embeddings(root: &Path) -> Result<Option<EmbeddingsConfig>> {
    if !root.join(CONFIG_PATH).is_file() {
        return Ok(None);
    }
    Ok(Manifest::load(root)?
        .configs(Capability::Knowledge)
        .first()
        .and_then(|config| config.embeddings.clone()))
}

/// The one stdout path, so `main` can keep BrokenPipe a success.
fn out(s: &str) -> Result<()> {
    writeln!(io::stdout(), "{s}").map_err(io_error)
}

/// Failures on the stdout path carry `-`, so `main` can spot a broken pipe.
fn io_error(source: io::Error) -> Error {
    Error::Io {
        path: "-".into(),
        source,
    }
}
