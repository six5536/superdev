//! validate_cli.rs — the `validate` verb and the PostToolUse hook.
//!
//! Parsing, path defaults and printed output only; the checking is all
//! `superdev-core`'s. Both surfaces run the same whole-repository check, so
//! the hook and the merge gate cannot reach different verdicts about the same
//! tree (D-17).

use std::io::{self, Read};
use std::path::{Path, PathBuf};

use superdev_core::error::Result;
use superdev_core::validate;

use crate::cli::{KNOWLEDGE_DIR, io_error, knowledge_dir, out};

/// What one `superdev validate` run covers, and how it reports.
#[derive(clap::Args)]
pub struct ValidateArgs {
    /// Files or directories to check (default: the SOKF knowledge and the
    /// trees the grammar governs)
    pub paths: Vec<PathBuf>,
    /// Repair what is mechanically repairable before checking: convert body
    /// links to the id form, refill every include block from its source, and
    /// regenerate every definition block
    #[arg(long)]
    pub fix: bool,
    /// Emit JSON instead of text
    #[arg(long)]
    pub json: bool,
    /// Print the grammar as prose and exit
    #[arg(long)]
    pub doc: bool,
    /// SOKF knowledge directory (default: `knowledge`)
    #[arg(long, value_name = "DIR")]
    pub knowledge: Option<PathBuf>,
    /// Repository root for `/`-rooted paths (default: this repo)
    #[arg(long, value_name = "DIR")]
    pub repo_root: Option<PathBuf>,
}

/// Claude Code hook plumbing (reads the hook payload from stdin).
#[derive(clap::Subcommand)]
pub enum HookCommand {
    /// PostToolUse: validate after an Edit/Write under the SOKF knowledge or
    /// a tree the grammar governs
    Validate,
    /// Stop: continue an active unattended run, or let the turn end
    Run,
}

/// Run one hook.
pub fn run_hook(cmd: &HookCommand, root: &Path) -> Result<u8> {
    match cmd {
        HookCommand::Validate => hook_validate(root),
        HookCommand::Run => crate::run::hook_run(root),
    }
}

/// Validate the SOKF knowledge and the files the grammar governs, or render
/// the grammar.
///
/// One report over both halves, so the hook and the merge gate cannot reach
/// different verdicts about the same repository (D-17).
pub fn run_validate(args: &ValidateArgs, root: &Path) -> Result<u8> {
    let grammar = validate::schema::load_grammar(root)?;
    if args.doc {
        return out(&validate::schema::doc::render(&grammar)).map(|()| 0);
    }
    let knowledge = knowledge_dir(root, args.knowledge.as_deref());
    let repo_root = args
        .repo_root
        .as_deref()
        .map_or_else(|| root.to_path_buf(), |p| root.join(p));
    // Repair first, then check: the report is then the state the repository
    // is left in, not the one it arrived in.
    let repaired = if args.fix {
        validate::fix_repo(&repo_root, &knowledge, &args.paths)?.written
    } else {
        Vec::new()
    };
    let run = validate::validate_repo(&repo_root, &knowledge, &args.paths, &grammar)?;
    if args.json {
        let mut value = run.report.to_json();
        // The knowledge path is the caller's string, so core leaves the key to
        // the caller. `files` joins it for the same reason `concepts` sits in
        // the report: it says what was read, and a run that read nothing is
        // otherwise a clean pass.
        if let Some(object) = value.as_object_mut() {
            object.insert("knowledge".into(), knowledge.display().to_string().into());
            object.insert("files".into(), run.files.into());
            object.insert("schemas".into(), run.schemas.into());
            object.insert("documents".into(), run.documents.into());
            if args.fix {
                object.insert("repaired".into(), repaired.clone().into());
            }
        }
        let rendered =
            serde_json::to_string_pretty(&value).map_err(|e| io_error(io::Error::other(e)))?;
        out(&rendered)?;
    } else {
        out(&format!(
            "superdev validator — {}",
            scope(args, &grammar, run.files)
        ))?;
        out(&format!(
            "  documents: {} checked against {} schema{}",
            run.documents,
            run.schemas,
            if run.schemas == 1 { "" } else { "s" }
        ))?;
        if run.schemas == 0 {
            // A repository with no `knowledge/schemas/` checks no document
            // against any contract. Silence here would read as a clean pass.
            out("  no schemas found — no document was checked against a contract")?;
        }
        if args.fix {
            // What was rewritten, named: `--fix` writes files, and a run that
            // says only that it passed hides which ones.
            out(&format!("  repaired: {} file(s)", repaired.len()))?;
            for path in &repaired {
                out(&format!("    {path}"))?;
            }
        }
        out(run.report.render_human().trim_end_matches('\n'))?;
    }
    Ok(u8::from(!run.report.passed()))
}

/// What the run covered, for the line above the report: where it looked, and
/// how many governed files it found there. The count is what separates a
/// clean run from one whose roots resolved to nothing.
fn scope(args: &ValidateArgs, grammar: &validate::Grammar, files: usize) -> String {
    let where_ = if args.paths.is_empty() {
        format!(
            // `./` so the value reads as a path rather than repeating the
            // label: the default directory is called `knowledge` too.
            "knowledge: ./{}, roots: {}",
            args.knowledge
                .as_deref()
                .unwrap_or(Path::new(KNOWLEDGE_DIR))
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
    format!("{where_} ({files} governed file{plural})")
}

/// The PostToolUse hook body. Exit 0 unless the payload names a path under
/// the SOKF knowledge or under a tree the grammar governs; then validate the
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
        eprintln!("superdev hook: could not read the tool payload from stdin: {e}");
        return Ok(2);
    }
    let parsed: serde_json::Value = match serde_json::from_str(&payload) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("superdev hook: malformed tool payload on stdin: {e}");
            return Ok(2);
        }
    };
    let Some(file_path) = parsed["tool_input"]["file_path"].as_str() else {
        // Not a file edit: nothing to validate.
        return Ok(0);
    };
    let grammar = validate::schema::load_grammar(&root)?;
    let knowledge = root.join(KNOWLEDGE_DIR);
    let edited = Path::new(file_path);
    // The same whole-set check the merge gate runs, fired by an edit anywhere
    // it reads: the SOKF knowledge, or any tree the grammar governs. A hook
    // narrower than the gate would pass what the gate then fails.
    let watched = std::iter::once(KNOWLEDGE_DIR)
        .chain(grammar.roots.paths.iter().map(String::as_str))
        .any(|dir| under(&root, dir, edited));
    if !watched {
        return Ok(0);
    }
    let run = validate::validate_repo(&root, &knowledge, &[], &grammar)?;
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
