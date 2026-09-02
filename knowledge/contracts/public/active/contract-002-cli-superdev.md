---
type: Contract
id: contract-002-cli-superdev
kind: cli
title: CLI contract for superdev
description: The superdev command line — every command, argument and flag as clap declares it, the exit codes and streams each command promises, and what may change.
lifecycle: active
resource: /crates/app/superdev/src/main.rs
links:
  - rel: references
    to: adr-033-a-contract-defines-its-interface
    note: A contract carries a machine-readable definition; here it is the clap tree.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The definition is materialised from the `cli` regions and bound by the include; the exit codes and streams are bound by `tests/contract_exit_codes.rs` and `tests/cli.rs`.
---

# CLI contract: superdev

The superdev command line: the manage verbs, the knowledge verbs and the
run verbs. The Definition is the clap tree as the binary declares it,
one include per source file; a doc comment on a command or flag is its
help text and its promise. Behaviour carries what the tree cannot say:
the exit codes, the streams, and each verb's promises across its flags.
The decisions behind the shape are
[ADR-033][sokf:adr-033-a-contract-defines-its-interface] and
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source].

## Definition

<!-- sokf:include /crates/app/superdev/src/main.rs#cli -->
```rust
#[derive(Parser)]
#[command(
    name = "superdev",
    version = superdev_core::version(),
    about = "superdev — project scaffold",
    // Hand-wrapped: clap's `wrap_help` is not enabled, so a paragraph here
    // renders as one long line and runs off the terminal. roff reflows it
    // again for the man page, so the breaks cost nothing there.
    long_about = "superdev — project scaffold.\n\n\
        Sets a repository up for agent-driven development and keeps that\n\
        setup current. The skills, templates and scaffolds it writes come\n\
        from a content pack: one ships inside this binary, and `[[packs]]`\n\
        in .superdev/config.toml points at another — a git source or a\n\
        directory — to add your own or supersede superdev's. Content\n\
        releases under its own assets-vX.Y.Z tags, and `superdev update`\n\
        is the verb that goes looking for the newest one this binary can\n\
        read."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Set this repo up for agent-driven development
    Init(manage::InitArgs),
    /// Report drift between the repo and its blueprint
    Status {
        /// Exit on drift alone, ignoring external state a checkout never
        /// carries (an unbuilt code index, an uninstalled tool)
        #[arg(long)]
        drift: bool,
    },
    /// Re-apply the blueprint so the repo matches the manifest
    Sync {
        /// Print the plan without applying it
        #[arg(long)]
        dry_run: bool,
    },
    /// Bring pins current, then sync
    // Explicit, and hand-wrapped: clap joins a doc comment's lines into one
    // paragraph, and without `wrap_help` that paragraph never breaks.
    #[command(long_about = "Bring pins current, then sync.\n\n\
            A capability's pin moves to this binary's default. The pack's\n\
            moves to the newest release its source carries that this binary\n\
            can read, which may be past what it embeds — the one place\n\
            superdev reaches the network unasked. A release it cannot read\n\
            is reported and the pin stays where it was.")]
    Update {
        /// Capability to update, optionally `<capability>@<version>`
        target: Option<String>,
        /// Provider to switch the target capability to
        #[arg(long, value_name = "ID")]
        provider: Option<String>,
    },
    /// Check the SOKF knowledge and the files the grammar governs
    Validate(validate_cli::ValidateArgs),
    /// Inspect and render the shipped project templates
    #[command(subcommand)]
    Template(manage::TemplateCommand),
    /// Serve project subsystems over MCP
    #[command(subcommand)]
    Mcp(sokf_cli::McpCommand),
    /// SOKF knowledge commands
    #[command(subcommand)]
    Sokf(sokf_cli::SokfCommand),
    /// Drive the state of an unattended workflow run
    #[command(subcommand)]
    Run(run::RunCommand),
    /// Agent hook plumbing (reads the hook payload from stdin)
    #[command(subcommand)]
    Hook(validate_cli::HookCommand),
    /// Write a completion script for the given shell to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Write the man page (roff) to stdout
    #[command(hide = true)]
    Man,
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/app/superdev/src/manage.rs#cli -->
```rust
/// The `init` flags: the capability-disable switches and the template
/// selection (kebab-case comes free from clap).
#[derive(clap::Args)]
pub struct InitArgs {
    /// Skip the frontend design workflows
    #[arg(long)]
    pub no_frontend: bool,
    /// Skip the superdev skill pack
    #[arg(long)]
    pub no_skills: bool,
    /// Skip the code index
    #[arg(long)]
    pub no_code_index: bool,
    #[arg(long, value_name = "NAME", help = crate::template_select::TEMPLATE_HELP)]
    pub template: Option<String>,
    /// Project name for template substitution (default: the directory name)
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,
}

/// The `template` subcommands: read-only views of the shipped templates.
/// Grown for the template-update skill — `render` gives it the current
/// content to compare a repo against, and the printed token lines save it
/// re-deriving slug rules it does not own.
#[derive(clap::Subcommand)]
pub enum TemplateCommand {
    /// List the shipped project templates
    List,
    /// Write a template's token-substituted tree into an empty directory
    Render {
        /// Template to render (see `template list`)
        template: String,
        /// Project name the tokens substitute to
        #[arg(long, value_name = "NAME")]
        name: String,
        /// Directory to write into — created if absent, must be empty
        #[arg(long, value_name = "DIR")]
        dir: PathBuf,
    },
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/app/superdev/src/validate_cli.rs#cli -->
```rust
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
    /// List the warnings, which a run counts but does not list
    #[arg(long)]
    pub warnings: bool,
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
```
<!-- /sokf:include -->

<!-- sokf:include /crates/app/superdev/src/sokf_cli.rs#cli -->
```rust
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
```
<!-- /sokf:include -->

<!-- sokf:include /crates/app/superdev/src/run.rs#cli -->
```rust
/// Drive the state of an unattended workflow run.
#[derive(clap::Subcommand)]
pub enum RunCommand {
    /// Arm an unattended run: create the run state exclusively
    Begin {
        /// Session that owns the run (default: $CLAUDE_SESSION_ID)
        #[arg(long, value_name = "ID")]
        session: Option<String>,
        /// The first next step, named by the Stop hook when it continues
        #[arg(long, value_name = "TEXT")]
        next: Option<String>,
    },
    /// Record a step forward: rewrite next, reset the watchdog, refresh the
    /// owner
    Advance {
        /// The next step, named by the Stop hook when it continues
        #[arg(long, value_name = "TEXT")]
        next: String,
        /// Session that owns the run (default: $CLAUDE_SESSION_ID)
        #[arg(long, value_name = "ID")]
        session: Option<String>,
    },
    /// End the run: remove the state; harmless when none exists
    End,
}
```
<!-- /sokf:include -->

## Behaviour

Every verb acts on the current directory. Every command carries `-h` and
`--help`, and the root carries `-V` and `--version`; the framework adds
them, so the Definition does not repeat them per command.

- **`init`** MUST refuse a directory that is not a git repo, and MUST refuse
  a re-run once `.superdev/config.toml` exists. It MUST write the manifest
  before applying, so a failed run leaves the file the retry resumes from.
  It MUST ensure `CLAUDE.md` carries `@AGENTS.md` and `AGENTS.md` carries
  `@.agents/superdev.md`, appending to an existing file. A skill the repo
  already has under a managed name MUST be released into `custom` before
  anything is written. A template file MUST NOT overwrite an existing file;
  the existing one is kept and reported.
- **`status`** MUST NOT write. Released skills, released orphans, the
  blueprint-version line and the `content:` lines are reports and MUST NOT
  affect the exit code, because layering is what the manifest asked for.
- **`sync`** MUST refuse to run while a registry-locked capability is pinned
  off the registry default. On a fresh clone it MUST run `mise trust` then
  `mise install` before any provider command. Orphan removals MUST run after
  every write, so a failed write rolls back before anything is deleted; an
  orphan the user has edited MUST be released rather than removed. A
  successful run MUST stamp this binary's version as the manifest's
  `blueprint`.
- **`update`** MUST reject an explicit version for a registry-locked
  capability. `--provider` MUST require a capability target, and MUST set
  that capability's version to the new provider's registry default. Bare
  `update` MUST move the pack pin to the newest release the default source
  carries; a targeted `update <capability>` MUST NOT make that request.
- **`validate`** MUST report both halves once, findings grouped by file. It
  MUST list every error, MUST NOT list a warning unless `--warnings` asks
  for one, and MUST state both counts either way; `hook validate` and
  `hook run` MUST report on the same default, so one rule holds whoever ran
  the check (ADR-040). A `PATH` MUST replace both defaults for what is
  reported, and a `PATH` naming a document MUST be reported with bare-run
  parity. `hook validate` MUST NOT pass `--fix`.
- **`run`** `begin` MUST name the owning session and `run end` when it
  refuses.
- **`hook run`** MUST fail open: an unreadable run state is reported and
  exits `0`, while an unreadable payload is a loud `2`. It MUST refuse to
  end the turn while `validate` reports an error, naming the findings on
  stderr, so a document cannot be left with a link to a file that never
  arrived (ADR-039). Knowledge it cannot read or check MUST let the turn
  end, and after `HOLD_CAP` holds in one session it MUST report and let
  the turn end, so a finding the agent cannot resolve stalls nothing.
- **`sokf index`** MUST rebuild the index in full, and MUST say so when no
  embedding model loaded and the index is lexical-only.
- **`hook validate`** MUST exit `0` unless the edited path is under the
  canonical knowledge or under a tree the grammar governs. It MUST NOT
  block on a finding only the whole tree settles — a broken body link or
  an `index.md` entry naming a missing file — because it is handed one
  edited file and cannot see whether the target arrives in the next edit;
  `hook run` is where those are judged (ADR-039).
- **`mcp sokf`** MUST serve the canonical knowledge over stdio; what it
  serves is [contract-003-api-sokf][sokf:contract-003-api-sokf]'s to bind.
- **`completions`** and **`man`** MUST render into a buffer before writing,
  so a failed write is an error and never partial output.

### Exit codes

Every command MUST exit `2` on a usage error, so the table lists `2` only
where the code carries a meaning beyond that. A closed stdout pipe MUST
end any command at `0`, silently. A hard failure or an I/O failure MUST
exit `2` with `error: <message>` on stderr; for `hook validate` and
`hook run`, `2` is the blocking code Claude Code hands back to the agent.

| Command | Code | Meaning |
|---------|------|---------|
| `superdev` | 0 | help printed |
| `superdev init` | 0 | the repo is set up |
| `superdev init` | 2 | not a git repo, a re-run, an unknown template, or a failed apply |
| `superdev status` | 0 | nothing to do |
| `superdev status` | 1 | drift, a missing component, a planned removal, or a stale pin |
| `superdev status` | 2 | an orphaned lock entry it cannot read |
| `superdev sync` | 0 | the repo matches the manifest |
| `superdev sync` | 2 | a registry-locked pin off its default, or a failed apply |
| `superdev update` | 0 | the pins are current |
| `superdev update` | 2 | an explicit version for a registry-locked capability, an unknown capability or provider, or a pin it could not prove |
| `superdev validate` | 0 | no errors |
| `superdev validate` | 1 | errors found |
| `superdev validate` | 2 | a path it could not read |
| `superdev template` | 2 | no subcommand named |
| `superdev template list` | 0 | the templates are listed |
| `superdev template render` | 0 | the tree is written |
| `superdev template render` | 2 | an unknown template, or a directory that is not empty |
| `superdev sokf` | 2 | no subcommand named |
| `superdev sokf index` | 0 | the index is rebuilt |
| `superdev sokf index` | 2 | knowledge it could not read |
| `superdev run` | 2 | no subcommand named |
| `superdev run begin` | 0 | the run is armed |
| `superdev run begin` | 2 | a run already exists |
| `superdev run advance` | 0 | the step is recorded |
| `superdev run advance` | 2 | no run, or a session that does not own it |
| `superdev run end` | 0 | no run state remains |
| `superdev hook` | 2 | no subcommand named |
| `superdev hook validate` | 0 | the edited path is outside the governed trees, or the repo still validates |
| `superdev hook validate` | 2 | findings on stderr, or a payload it cannot read |
| `superdev hook run` | 0 | no run, another session's run, no next step, or the watchdog cap reached |
| `superdev hook run` | 2 | the next step on stderr, or a payload it cannot read |
| `superdev mcp` | 2 | no subcommand named |
| `superdev mcp sokf` | 0 | the client closed stdin |
| `superdev mcp sokf` | 2 | the server could not start |
| `superdev completions` | 0 | the script is written |
| `superdev completions` | 2 | a shell it does not know |
| `superdev man` | 0 | the page is written |

### Streams

Every command MUST write its report to stdout and its diagnostics to stderr.
`completions` and `man` MUST write their generated file to stdout and
nothing else, so the output redirects cleanly. `hook validate` and
`hook run` MUST read their payload from stdin and MUST write their findings
and their next step to stderr, which is where Claude Code reads them.
`mcp sokf` MUST speak the MCP protocol over stdin and stdout, so nothing
else MAY be written to stdout for the life of the process. A closed stdout
pipe MUST end the run at `0`, silently.

`validate --json` MUST write one JSON object to stdout carrying `passed`,
`errors`, `warnings`, `concepts`, `documents`, `schemas`, `files`,
`findings` (one entry per finding, each with its file, severity and
message; warnings appear only with `--warnings`, as in the text output),
`knowledge` (the directory the run covered) and, with `--fix`, `repaired`
(each file the run rewrote).

### Prompting

On a TTY with neither `--template` nor `--name`, `init` MUST prompt for
the template and the project name; without a TTY it MUST NOT prompt and
MUST take the defaults. No other command prompts.

### Environment

`hook validate` and `hook run` MUST resolve the repository from
`CLAUDE_PROJECT_DIR` when Claude Code sets it, else from the working
directory; the variable is described by
[contract-004-config-superdev][sokf:contract-004-config-superdev]. `run
begin` and `run advance` MUST take the session from `CLAUDE_SESSION_ID`
when `--session` is absent. No other command reads the environment
beyond what `mise` and `git` read for themselves.

### Usage errors

An unknown flag, an unknown subcommand and a missing required value MUST
exit `2` with clap's usage message on stderr, from every command alike.

### Side effects

`--fix` is the one way `validate` writes: without it `validate` MUST NOT
write. `--fix` MUST write only inside the resolved knowledge directory and
MUST be idempotent. `status` MUST NOT write. `run` MUST NOT touch git,
the network, or any file outside `.superdev/cache/`. `update` is the one
verb that reaches the network unasked, to find the newest pack release.

## Stability

Unreleased. Every command, argument, flag and exit code above MAY change
without notice.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
