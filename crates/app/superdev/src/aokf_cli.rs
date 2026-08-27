//! aokf_cli.rs — the `aokf` and `mcp` verbs over the knowledge bundle.
//!
//! Parsing, path defaults and printed output only; the bundle work is all
//! `superdev-core`'s.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use superdev_core::aokf::{
    AokfServer, EmbeddingsConfig, Index, IndexDir, embedder_from, load_bundle,
};
use superdev_core::capability::Capability;
use superdev_core::error::{Error, Result};
use superdev_core::format;
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

/// What one `superdev validate` run covers, and how it reports.
#[derive(clap::Args)]
pub struct ValidateArgs {
    /// Files or directories to check (default: the bundle and the trees the
    /// grammar governs)
    pub paths: Vec<PathBuf>,
    /// Emit JSON instead of text
    #[arg(long)]
    pub json: bool,
    /// Print the format grammar as prose and exit
    #[arg(long)]
    pub doc: bool,
    /// Bundle directory (default: `knowledge`)
    #[arg(long, value_name = "DIR")]
    pub bundle: Option<PathBuf>,
    /// Repository root for `/`-rooted paths (default: this repo)
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
}

/// Work on the AOKF knowledge bundle.
#[derive(clap::Subcommand)]
pub enum AokfCommand {
    /// Validate the bundle and the format files (alias of `superdev validate`)
    #[command(hide = true)]
    Validate(ValidateArgs),
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
    /// PostToolUse: validate after an Edit/Write under the bundle or a
    /// tree the format grammar governs
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

/// Validate the bundle and the format files, or render the grammar.
///
/// One report over both checks, so the hook and the merge gate cannot reach
/// different verdicts about the same repository (D-17).
pub fn run_validate(args: &ValidateArgs, root: &Path) -> Result<u8> {
    let grammar = format::load_grammar(root)?;
    if args.doc {
        return out(&format::doc::render(&grammar)).map(|()| 0);
    }
    let bundle_dir = bundle_dir(root, args.bundle.as_deref());
    let repo_root = args
        .repo_root
        .as_deref()
        .map_or_else(|| root.to_path_buf(), |p| root.join(p));
    let run = format::validate_repo(&repo_root, &bundle_dir, &args.paths, &grammar)?;
    if args.json {
        let mut value = run.report.to_json();
        // The bundle path is the caller's string, so core leaves the key to
        // the caller. The reference validator emits it. `files` joins it for
        // the same reason `concepts` sits in the report: it says what was
        // read, and a run that read nothing is otherwise a clean pass.
        if let Some(object) = value.as_object_mut() {
            object.insert("bundle".into(), bundle_dir.display().to_string().into());
            object.insert("files".into(), run.files.into());
        }
        let rendered =
            serde_json::to_string_pretty(&value).map_err(|e| io_error(io::Error::other(e)))?;
        out(&rendered)?;
    } else {
        out(&format!(
            "superdev validator — {}",
            scope(args, &grammar, run.files)
        ))?;
        out(run.report.render_human().trim_end_matches('\n'))?;
    }
    Ok(u8::from(!run.report.passed()))
}

/// What the run covered, for the line above the report: where it looked, and
/// how many format files it found there. The count is what separates a clean
/// run from one whose roots resolved to nothing.
fn scope(args: &ValidateArgs, grammar: &superdev_core::format::Grammar, files: usize) -> String {
    let where_ = if args.paths.is_empty() {
        format!(
            "bundle: {}, roots: {}",
            args.bundle
                .as_deref()
                .unwrap_or(Path::new(BUNDLE_DIR))
                .display(),
            grammar.roots.paths.join(", ")
        )
    } else {
        args.paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let plural = if files == 1 { "" } else { "s" };
    format!("{where_} ({files} format file{plural})")
}

/// Validate or index the bundle.
pub fn run_aokf(cmd: &AokfCommand, root: &Path) -> Result<u8> {
    match cmd {
        AokfCommand::Validate(args) => run_validate(args, root),
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
                    "skipped {} unparseable file(s) — run `superdev validate`",
                    bundle.broken.len()
                ))?;
            }
            Ok(0)
        }
        AokfCommand::Hook(HookCommand::Validate) => hook_validate(root),
    }
}

/// The PostToolUse hook body. Exit 0 unless the payload names a path under
/// the bundle or under a tree the format grammar governs; then validate the
/// whole set and exit 2 with findings on errors, which Claude Code feeds back
/// to the agent as a blocking error. An unreadable payload is a loud exit 2 —
/// a silent skip here silently stops validating.
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
    let grammar = format::load_grammar(&root)?;
    let bundle = root.join(BUNDLE_DIR);
    let edited = Path::new(file_path);
    // The same whole-set check the merge gate runs, fired by an edit anywhere
    // it reads: the bundle, or any tree the grammar governs. A hook narrower
    // than the gate would pass what the gate then fails.
    let watched = std::iter::once(BUNDLE_DIR)
        .chain(grammar.roots.paths.iter().map(String::as_str))
        .any(|dir| under(&root, dir, edited));
    if !watched {
        return Ok(0);
    }
    let run = format::validate_repo(&root, &bundle, &[], &grammar)?;
    if run.report.passed() {
        return Ok(0);
    }
    eprintln!("superdev validation failed after editing {file_path} — fix before continuing:");
    eprintln!("{}", run.report.render_human().trim_end_matches('\n'));
    Ok(2)
}

/// Does the edited path name a file under `<root>/<dir>`? Falling back to the
/// working directory gets a root with symlinks already resolved — macOS
/// spells a temp dir `/private/var/…` where the payload still says `/var/…`
/// — so a path that misses lexically gets a second look through the
/// resolved spelling of both sides. Resolving only as a fallback keeps a
/// symlink *into* the directory matching on its own side's name.
fn under(root: &Path, dir: &str, edited: &Path) -> bool {
    let watched = root.join(dir);
    if edited.starts_with(&watched) || edited.starts_with(dir) {
        return true;
    }
    match (watched.canonicalize(), edited.canonicalize()) {
        (Ok(watched), Ok(edited)) => edited.starts_with(watched),
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
