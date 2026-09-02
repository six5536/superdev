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
them, so the Definition does not repeat them per command. The promises
below stand in verb order; the subsections carry what every verb
shares — the exit codes, the streams, the prompt, the environment, the
usage errors and the side effects.

- `P_init-outside-git` [event] WHEN `init` runs outside a git
  repository, `init` SHALL refuse.
- `P_init-rerun` [state] WHILE `.superdev/config.toml` exists, `init`
  SHALL refuse a re-run.
- `P_init-manifest-first` [ubiquitous] `init` SHALL write the manifest
  before applying, so a failed run leaves the file the retry resumes
  from.
- `P_init-agents-chain` [ubiquitous] `init` SHALL ensure `CLAUDE.md`
  carries `@AGENTS.md` and `AGENTS.md` carries `@.agents/superdev.md`,
  appending to an existing file.
- `P_init-releases-managed-name` [event] WHEN the repo already has a
  skill under a managed name, `init` SHALL release that skill into
  `custom` before anything is written.
- `P_init-keeps-existing-file` [event] WHEN a template file names a
  file that exists, `init` SHALL NOT overwrite the existing file; the
  existing file is kept and reported.
- `P_status-writes-nothing` [ubiquitous] `status` SHALL NOT write.
- `P_status-reports-leave-exit-code` [ubiquitous] `status` SHALL NOT
  let a released skill, a released orphan, the blueprint-version line
  or a `content:` line affect the exit code; each is a report of the
  layering the manifest asked for.
- `P_sync-locked-pin` [state] WHILE a registry-locked capability is
  pinned off the registry default, `sync` SHALL refuse to run.
- `P_sync-fresh-clone-mise` [event] WHEN `sync` runs on a fresh clone,
  `sync` SHALL run `mise trust` then `mise install` before any provider
  command.
- `P_sync-removes-after-writes` [ubiquitous] `sync` SHALL run the
  orphan removals after every write, so a failed write rolls back
  before anything is deleted.
- `P_sync-releases-edited-orphan` [event] WHEN an orphan carries the
  user's edits, `sync` SHALL release the orphan rather than remove it.
- `P_sync-stamps-blueprint` [event] WHEN `sync` succeeds, `sync` SHALL
  stamp this binary's version as the manifest's `blueprint`.
- `P_update-locked-version` [event] WHEN `update` is given an explicit
  version for a registry-locked capability, `update` SHALL reject the
  version.
- `P_update-provider-needs-target` [ubiquitous] `update --provider`
  SHALL require a capability target.
- `P_update-provider-default-version` [event] WHEN `--provider`
  switches a capability, `update` SHALL set that capability's version
  to the new provider's registry default.
- `P_update-bare-moves-pack-pin` [event] WHEN `update` runs bare,
  `update` SHALL move the pack pin to the newest release the default
  source carries.
- `P_update-targeted-asks-nothing` [event] WHEN `update` names a
  capability, `update` SHALL NOT ask the default source for its newest
  release.
- `P_validate-reports-both-halves-once` [ubiquitous] `validate` SHALL
  report both halves once, findings grouped by file.
- `P_validate-lists-every-error` [ubiquitous] `validate` SHALL list
  every error.
- `P_validate-warnings-on-request` [conditional] IF `--warnings` is
  absent, `validate` SHALL NOT list a warning.
- `P_validate-states-both-counts` [ubiquitous] `validate` SHALL state
  both counts, with or without `--warnings`.
- `P_hooks-share-validate-default` [ubiquitous] `hook validate` and
  `hook run` SHALL report on the default `validate` reports on, so one
  rule holds whoever ran the check (ADR-040).
- `P_validate-path-replaces-defaults` [event] WHEN a `PATH` is given,
  `validate` SHALL replace both defaults with the `PATH` for what is
  reported.
- `P_validate-document-path-parity` [event] WHEN a `PATH` names a
  document, `validate` SHALL report the document with bare-run parity.
- `P_hook-validate-no-fix` [ubiquitous] `hook validate` SHALL NOT pass
  `--fix`.
- `P_run-begin-refusal-names-owner` [event] WHEN `run begin` refuses,
  `run begin` SHALL name the owning session and `run end`.
- `P_hook-run-fails-open` [event] WHEN the run state cannot be read,
  `hook run` SHALL report it on stderr and exit `0`.
- `P_hook-run-unreadable-payload` [event] WHEN the payload cannot be
  read from stdin or parsed, `hook run` SHALL exit `2`.
- `P_hook-run-holds-on-error` [state] WHILE `validate` reports an
  error, `hook run` SHALL refuse to end the turn, naming the findings
  on stderr, so a document cannot be left with a link to a file that
  never arrived (ADR-039).
- `P_hook-run-unreadable-knowledge` [event] WHEN the knowledge cannot
  be read or checked, `hook run` SHALL let the turn end.
- `P_hook-run-hold-cap` [event] WHEN `HOLD_CAP` holds have been held in
  one session, `hook run` SHALL report and let the turn end, so a
  finding the agent cannot resolve stalls nothing.
- `P_sokf-index-rebuilds-in-full` [ubiquitous] `sokf index` SHALL
  rebuild the index in full.
- `P_sokf-index-says-lexical-only` [event] WHEN no embedding model
  loaded, `sokf index` SHALL say the index is lexical-only.
- `P_hook-validate-ungoverned-path` [event] WHEN the edited path is
  outside the canonical knowledge and outside every tree the grammar
  governs, `hook validate` SHALL exit `0`.
- `P_hook-validate-leaves-tree-findings` [ubiquitous] `hook validate`
  SHALL NOT block on a finding only the whole tree settles — a broken
  body link or an `index.md` entry naming a missing file: the hook is
  handed one edited file and cannot see whether the target arrives in
  the next edit, and `hook run` judges those findings (ADR-039).
- `P_mcp-sokf-serves-knowledge` [ubiquitous] `mcp sokf` SHALL serve
  the canonical knowledge over stdio; what it serves is
  [contract-003-api-sokf][sokf:contract-003-api-sokf]'s to bind.
- `P_completions-man-buffer-first` [ubiquitous] `completions` and
  `man` SHALL render into a buffer before writing, so a failed write is
  an error and never partial output.

### Exit codes

The table lists `2` only where the code carries a meaning beyond a
usage error. For `hook validate` and `hook run`, `2` is the blocking
code Claude Code hands back to the agent.

- `P_usage-error-exits-2` [event] WHEN a usage error occurs, every
  command SHALL exit `2`.
- `P_closed-stdout-exits-0` [event] WHEN the stdout pipe closes, every
  command SHALL end at `0`, silently.
- `P_hard-failure-exits-2` [event] WHEN a hard failure or an I/O
  failure occurs, every command SHALL exit `2` with `error: <message>`
  on stderr.

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

Claude Code reads a hook's stderr. A closed stdout pipe ends the run as
`P_closed-stdout-exits-0` says.

- `P_report-stdout-diagnostics-stderr` [ubiquitous] Every command SHALL
  write its report to stdout and its diagnostics to stderr.
- `P_completions-man-stdout-only` [ubiquitous] `completions` and `man`
  SHALL write their generated file to stdout and nothing else, so the
  output redirects cleanly.
- `P_hooks-read-stdin` [ubiquitous] `hook validate` and `hook run`
  SHALL read their payload from stdin.
- `P_hooks-write-stderr` [ubiquitous] `hook validate` and `hook run`
  SHALL write their findings and their next step to stderr.
- `P_mcp-sokf-speaks-mcp` [ubiquitous] `mcp sokf` SHALL speak the MCP
  protocol over stdin and stdout.
- `P_mcp-sokf-stdout-reserved` [state] WHILE `mcp sokf` runs, `mcp
  sokf` SHALL NOT write anything beyond the protocol to stdout.
- `P_validate-json-shape` [event] WHEN `--json` is given, `validate`
  SHALL write one JSON object to stdout carrying `passed`, `errors`,
  `warnings`, `concepts`, `documents`, `schemas`, `files`, `findings`
  (one entry per finding, each with its file, severity and message;
  warnings appear only with `--warnings`, as in the text output),
  `knowledge` (the directory the run covered) and, with `--fix`,
  `repaired` (each file the run rewrote).

### Prompting

`init` is the one command that prompts; no other command prompts.

- `P_init-prompts-on-tty` [event] WHEN `init` runs on a TTY with
  neither `--template` nor `--name`, `init` SHALL prompt for the
  template and the project name.
- `P_init-no-prompt-without-tty` [event] WHEN `init` runs without a
  TTY, `init` SHALL NOT prompt.
- `P_init-defaults-without-tty` [event] WHEN `init` runs without a TTY,
  `init` SHALL take the defaults.

### Environment

`CLAUDE_PROJECT_DIR` is described by
[contract-004-config-superdev][sokf:contract-004-config-superdev]. No
other command reads the environment beyond what `mise` and `git` read
for themselves.

- `P_hooks-resolve-project-dir` [event] WHEN Claude Code sets
  `CLAUDE_PROJECT_DIR`, `hook validate` and `hook run` SHALL resolve
  the repository from `CLAUDE_PROJECT_DIR`.
- `P_hooks-resolve-working-dir` [conditional] IF `CLAUDE_PROJECT_DIR`
  is unset, `hook validate` and `hook run` SHALL resolve the repository
  from the working directory.
- `P_run-session-from-env` [event] WHEN `--session` is absent, `run
  begin` and `run advance` SHALL take the session from
  `CLAUDE_SESSION_ID`.

### Usage errors

Clap reports a usage error from every command alike.

- `P_usage-message-on-stderr` [event] WHEN an unknown flag, an unknown
  subcommand or a missing required value is given, every command SHALL
  exit `2` with clap's usage message on stderr.

### Side effects

`--fix` is the one way `validate` writes; `status` writes nothing
(`P_status-writes-nothing`). `update` is the one verb that reaches the
network unasked, to find the newest pack release.

- `P_validate-writes-only-with-fix` [event] WHEN `validate` runs
  without `--fix`, `validate` SHALL NOT write.
- `P_fix-writes-inside-knowledge` [ubiquitous] `validate --fix` SHALL
  write only inside the resolved knowledge directory.
- `P_fix-idempotent` [ubiquitous] `validate --fix` SHALL be idempotent.
- `P_run-touches-cache-only` [ubiquitous] `run` SHALL NOT touch git,
  the network, or any file outside `.superdev/cache/`.

## Stability

Unreleased.

- `P_unreleased` [ubiquitous] Every command, argument, flag and exit
  code above MAY change without notice.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
