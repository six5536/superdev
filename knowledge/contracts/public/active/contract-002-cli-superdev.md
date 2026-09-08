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

The superdev command line: the manage verbs, knowledge verbs, and the
local workflow adapter. The Definition is the clap tree as the binary declares it,
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
    /// File a human-confirmed issue or idea on the local default branch
    File(workflow_cli::FileArgs),
    /// Drive the local SCOPE → BUILD → ACCEPT workflow
    #[command(subcommand)]
    Workflow(workflow_cli::WorkflowCommand),
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

/// Legacy adapter hook plumbing (reads the hook payload from stdin).
#[derive(clap::Subcommand)]
pub enum HookCommand {
    /// PostToolUse: validate after an Edit/Write under the SOKF knowledge or
    /// a tree the grammar governs
    Validate,
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
```
<!-- /sokf:include -->

<!-- sokf:include /crates/app/superdev/src/workflow_cli.rs#cli -->
```rust
/// Human-confirmed out-of-band issue or idea filing.
#[derive(Args)]
pub struct FileArgs {
    /// Record kind
    #[arg(long, value_enum, default_value = "issue")]
    kind: FilingKindName,
    /// Short human title
    #[arg(long)]
    title: String,
    /// Human description to preserve in the record
    #[arg(long)]
    description: String,
    /// Local default branch to advance
    #[arg(long, default_value = "main")]
    default_branch: String,
    /// Confirmation supplied only after the human approves the bounded diff
    #[arg(long)]
    human_approved: bool,
}

/// CLI spelling of fileable record kinds.
#[derive(Clone, Copy, ValueEnum)]
enum FilingKindName {
    Issue,
    Idea,
}

/// Versioned workflow operations used by the Pi adapter.
#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Acquire a workflow for an issue, plan, and reserved work branch
    Start(BindArgs),
    /// Report transient ownership and canonical identity
    Status {
        /// Emit the versioned JSON protocol response
        #[arg(long)]
        json: bool,
    },
    /// Acquire or resume ownership with the same identity
    Bind(BindArgs),
    /// Apply one typed phase transition after checking supplied evidence
    Transition(TransitionArgs),
    /// Adopt the current committed work tip as the next SCOPE attempt baseline
    ScopeBaseline(ScopeBaselineArgs),
    /// Commit one review-ready, knowledge-only SCOPE proposal
    ScopeCheckpoint(RevisionArgs),
    /// Commit a validated BUILD block checkpoint
    Block(ProgressArgs),
    /// Record one normalized failed BUILD attempt
    Attempt(AttemptArgs),
    /// Count one failed final verification/review correction cycle
    Correction(CorrectionArgs),
    /// Commit one path-scoped implementation correction after a failed final gate
    CorrectionCheckpoint(RevisionArgs),
    /// Record isolated review or final verification evidence canonically
    Evidence(EvidenceArgs),
    /// Incorporate the expected local default tip into BUILD
    Sync(SyncArgs),
    /// Reconstruct and acquire ownership for a known workflow
    Resume(BindArgs),
    /// Record one active isolated child for cross-instance status and recovery
    ActivityStart(ActivityStartArgs),
    /// Clear the active isolated child after exit
    ActivityFinish(RevisionArgs),
    /// Pause by releasing transient ownership without changing plan phase
    Cancel(SessionArgs),
    /// Apply the human-only abandonment transition
    Abandon(AbandonArgs),
    /// Merge an accepted closure locally with `git merge --no-ff`
    Integrate(IntegrateArgs),
}

/// Stable workflow identity and Pi ownership arguments.
#[derive(Args)]
pub struct BindArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Primary issue ID
    #[arg(long)]
    issue: String,
    /// Implementing plan ID
    #[arg(long)]
    plan: String,
    /// Reserved work branch
    #[arg(long)]
    work_branch: String,
    /// Local default branch
    #[arg(long, default_value = "main")]
    default_branch: String,
}

/// Active parent and child process identity.
#[derive(Args)]
pub struct ActivityStartArgs {
    /// Owning Pi session ID.
    #[arg(long)]
    session: String,
    /// Expected current plan content revision.
    #[arg(long)]
    expected_revision: String,
    /// Isolated workflow role.
    #[arg(long)]
    role: String,
    /// Owning Pi process ID.
    #[arg(long)]
    owner_pid: u32,
    /// OS process-start identity for the owning Pi.
    #[arg(long)]
    owner_started: String,
    /// Isolated child process ID.
    #[arg(long)]
    child_pid: u32,
    /// OS process-start identity for the child.
    #[arg(long)]
    child_started: String,
}

/// Arguments common to compare-and-swap progress events.
#[derive(Args)]
pub struct ProgressArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// New plan content revision after the Rust-owned mutation
    #[arg(long)]
    revision: String,
}

/// Compare-and-swap arguments for a SCOPE attempt baseline.
#[derive(Args)]
pub struct ScopeBaselineArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected current work-branch tip
    #[arg(long)]
    expected_work: String,
}

/// Session and plan compare-and-swap arguments.
#[derive(Args)]
pub struct RevisionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
}

/// One failed BUILD command, normalized and counted by Rust.
#[derive(Args)]
pub struct AttemptArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Failed command as executed without a shell
    #[arg(long)]
    command: String,
    /// Process exit status
    #[arg(long)]
    exit_status: i32,
    /// Bounded command diagnostics
    #[arg(long)]
    diagnostics: String,
}

/// One candidate-bound failed final gate.
#[derive(Args)]
pub struct CorrectionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Candidate whose final gate failed
    #[arg(long)]
    candidate: String,
    /// Fresh isolated reviewer run
    #[arg(long)]
    review_session: String,
    /// Bounded structured finding summary
    #[arg(long)]
    summary: String,
}

/// Rust-owned canonical evidence attestation.
#[derive(Args)]
pub struct EvidenceArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Reviewed SCOPE plan revision after isolated modifying work
    #[arg(long)]
    revision: Option<String>,
    /// Evidence gate being attested
    #[arg(long, value_enum)]
    kind: EvidenceKindName,
    /// Fresh isolated reviewer session ID
    #[arg(long)]
    review_session: Option<String>,
    /// Immutable candidate for final BUILD evidence
    #[arg(long)]
    candidate: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum EvidenceKindName {
    ScopeReview,
    Verification,
    Final,
}

/// Session ownership argument.
#[derive(Args)]
pub struct SessionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
}

/// Compare-and-swap arguments for BUILD synchronization.
#[derive(Args)]
pub struct SyncArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected local default-branch tip
    #[arg(long)]
    expected_default: String,
    /// Expected local work-branch tip
    #[arg(long)]
    expected_work: String,
}

/// Typed phase transition names.
#[derive(Clone, Copy, ValueEnum)]
pub enum TransitionName {
    /// SCOPE to BUILD
    ApproveScope,
    /// BUILD to SCOPE
    ReturnToScope,
    /// ACCEPT to SCOPE
    RejectAcceptance,
    /// ACCEPT findings within approved intent to BUILD
    ReturnToBuild,
    /// ACCEPT to DONE
    Accept,
    /// Prepared DONE to BUILD after default branch drift
    RecoverStaleDefault,
}

/// Compare-and-swap transition and gate evidence.
#[derive(Args)]
pub struct TransitionArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected current phase
    #[arg(long, value_enum)]
    phase: PhaseName,
    /// Enumerated transition
    #[arg(long, value_enum)]
    transition: TransitionName,
    /// BUILD discovery or human rejection preserved verbatim on the primary issue
    #[arg(long)]
    feedback: Option<String>,
}

/// CLI spelling of durable phases.
#[derive(Clone, Copy, ValueEnum)]
pub enum PhaseName {
    Scope,
    Build,
    Accept,
    Done,
    Abandoned,
}

/// Human-only abandonment request.
#[derive(Args)]
pub struct AbandonArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Expected current plan content revision
    #[arg(long)]
    expected_revision: String,
    /// Expected current phase
    #[arg(long, value_enum)]
    phase: PhaseName,
    /// Human-approved disposition recorded on the issue
    #[arg(long)]
    reason: String,
}

/// Compare-and-swap local integration arguments.
#[derive(Args)]
pub struct IntegrateArgs {
    /// Owning Pi session ID
    #[arg(long)]
    session: String,
    /// Local default branch
    #[arg(long)]
    default_branch: String,
    /// Expected default branch tip
    #[arg(long)]
    expected_default: String,
    /// Accepted work branch
    #[arg(long)]
    work_branch: String,
    /// Expected closure commit
    #[arg(long)]
    expected_work: String,
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
- `P_init-agents-chain` [ubiquitous] `init` SHALL ensure `AGENTS.md`
  carries `@.agents/superdev.md`, appending to an existing file.
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
- `P_hook-shares-validate-default` [ubiquitous] `hook validate` SHALL
  report on the default `validate` reports on, so one rule holds whoever
  ran the check.
- `P_validate-path-replaces-defaults` [event] WHEN a `PATH` is given,
  `validate` SHALL replace both defaults with the `PATH` for what is
  reported.
- `P_validate-document-path-parity` [event] WHEN a `PATH` names a
  document, `validate` SHALL report the document with bare-run parity.
- `P_hook-validate-no-fix` [ubiquitous] `hook validate` SHALL NOT pass
  `--fix`.
- `P_workflow-versioned-json` [ubiquitous] Every successful `workflow`
  command SHALL return one JSON object carrying protocol
  `superdev-workflow/v2`.
- `P_workflow-owned-transitions` [ubiquitous] A mutating `workflow`
  command SHALL require the owning session and expected plan revision.
- `P_workflow-scope-baseline` [event] WHEN `workflow scope-baseline` succeeds,
  it SHALL require the owned SCOPE work branch, compare-and-swap the expected
  current work tip, reject uncommitted paths outside `knowledge/`, and record
  that tip as the attempt's `scope_base_revision` without granting review,
  approval, or a phase transition.
- `P_workflow-cancel-pauses` [event] WHEN `workflow cancel` succeeds,
  it SHALL release transient ownership without changing canonical phase.
- `P_workflow-local-integration` [ubiquitous] After clean-tree and expected-tip
  checks, `workflow integrate` SHALL execute shell-free local
  `git merge --no-ff` without pushing, releasing, deleting branches, stashing,
  resetting, discarding, absorbing unrelated changes, or resolving conflicts
  implicitly.
- `P_workflow-integration-bound` [ubiquitous] `workflow integrate` SHALL
  require the bound issue, plan, refs, reviewed candidate, verified default
  tip, done closure, and administrative-only descendants to agree.
- `P_file-default-branch` [ubiquitous] `file` SHALL create and validate one
  human-confirmed issue or idea in an isolated temporary worktree, commit only
  knowledge, and compare-and-swap the local default branch without changing an
  active workflow worktree.
- `P_sokf-index-rebuilds-in-full` [ubiquitous] `sokf index` SHALL
  rebuild the index in full.
- `P_sokf-index-says-lexical-only` [event] WHEN no embedding model
  loaded, `sokf index` SHALL say the index is lexical-only.
- `P_sokf-retrieval-shares-service` [ubiquitous] `sokf overview`,
  `search`, `read` and `graph` SHALL return the same rendered information as
  the corresponding shared-service and MCP operations.
- `P_sokf-direct-retrieval-skips-embeddings` [ubiquitous] `sokf read` and
  `sokf graph` SHALL parse current knowledge without loading an embedding model
  or opening the search index.
- `P_sokf-read-window-after-render` [ubiquitous] `sokf read` SHALL apply
  `offset` and `limit` after rendering concept metadata and headings.
- `P_sokf-edit-exact-atomic` [ubiquitous] `sokf edit` SHALL require every
  `oldText` to occur exactly once in the original file, reject overlapping
  edits, evaluate all edits against that original, and write nothing when any
  precondition fails.
- `P_sokf-write-whole` [ubiquitous] `sokf write` SHALL replace the complete
  target document.
- `P_sokf-write-creation-path` [event] WHEN `sokf write` creates a concept, it
  SHALL require a physical `.md` path rather than infer placement from an
  unresolved identity.
- `P_sokf-agent-safe-default` [ubiquitous] `sokf edit` and `sokf write` SHALL
  preserve an existing `id` and the exact `verified` bytes by default.
- `P_sokf-stamped-refusal` [ubiquitous] `sokf edit` and `sokf write` SHALL
  reject stamped fields under every policy.
- `P_sokf-human-override` [event] WHEN `--allow-restricted` is given, `sokf
  edit` and `sokf write` SHALL permit a deliberate identity or verification
  change.
- `P_sokf-request-policy-fixed` [ubiquitous] Mutation request JSON SHALL carry
  no policy override field.
- `P_sokf-mutation-contained` [ubiquitous] SOKF mutations SHALL refuse paths
  outside the canonical knowledge root, `..` escapes, symlink traversal,
  section-qualified virtual addresses, and direct `manifest.sokf.yaml`
  mutation; direct `index.md` mutation is permitted.
- `P_sokf-mutation-repairs` [event] WHEN a requested mutation is applied, the
  command SHALL run the same repair as `validate --fix`, validate the resulting
  repository, and return the originally resolved path, final path, requested
  and repair-generated diffs, validation state, and remaining findings.
- `P_sokf-applied-invalid-succeeds` [state] WHILE an applied mutation leaves
  validation invalid or unknown, the command SHALL retain the changes and
  return a successful applied result rather than a hard error, preventing an
  unsafe automatic retry.
- `P_hook-validate-ungoverned-path` [event] WHEN the edited path is
  outside the canonical knowledge and outside every tree the grammar
  governs, `hook validate` SHALL exit `0`.
- `P_hook-validate-leaves-tree-findings` [ubiquitous] `hook validate`
  SHALL NOT block on a finding only the whole tree settles — a broken
  body link or an `index.md` entry naming a missing file: the hook is
  handed one edited file and cannot see whether the target arrives in
  the next edit.
- `P_mcp-sokf-serves-knowledge` [ubiquitous] `mcp sokf` SHALL serve
  the canonical knowledge over stdio; what it serves is
  [contract-003-api-sokf][sokf:contract-003-api-sokf]'s to bind.
- `P_completions-man-buffer-first` [ubiquitous] `completions` and
  `man` SHALL render into a buffer before writing, so a failed write is
  an error and never partial output.

### Exit codes

The table lists `2` only where the code carries a meaning beyond a
usage error. For `hook validate`, `2` is the blocking code returned to
the invoking adapter.

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
| `superdev sokf overview` | 0 | the knowledge overview is rendered |
| `superdev sokf overview` | 2 | the knowledge is unreadable |
| `superdev sokf search` | 0 | matching knowledge is rendered |
| `superdev sokf search` | 2 | the knowledge or index is unreadable |
| `superdev sokf read` | 0 | the requested concept is rendered |
| `superdev sokf read` | 2 | the knowledge is unreadable or the target cannot be resolved |
| `superdev sokf graph` | 0 | the requested graph is rendered |
| `superdev sokf graph` | 2 | the knowledge is unreadable or the target cannot be resolved |
| `superdev sokf edit` | 0 | the mutation was applied, including an invalid or unknown resulting state |
| `superdev sokf edit` | 2 | malformed input or a failed precondition left the target unchanged |
| `superdev sokf write` | 0 | the mutation was applied, including an invalid or unknown resulting state |
| `superdev sokf write` | 2 | malformed input or a failed precondition left the target unchanged |
| `superdev file` | 0 | a confirmed issue or idea was committed on the default branch |
| `superdev file` | 2 | confirmation, validation, duplicate, worktree, or compare-and-swap checks failed |
| `superdev workflow` | 2 | no subcommand named |
| `superdev workflow start` | 0 | validated LLM-authored issue and plan records are published, their issue-derived work branch is created, and ownership is acquired |
| `superdev workflow start` | 2 | authored records, relationship, independent identities, ownership, issue-derived branch, unrelated tree state, or validation is invalid |
| `superdev workflow status` | 0 | canonical and transient state is reported |
| `superdev workflow bind` | 0 | transient ownership is acquired |
| `superdev workflow bind` | 2 | identity, revision, branch, or ownership is invalid |
| `superdev workflow transition` | 0 | the gated transition is persisted, including primary-issue discovery preservation when returning to SCOPE |
| `superdev workflow transition` | 2 | ownership, revision, phase, required feedback, or evidence is invalid |
| `superdev workflow scope-baseline` | 0 | the current committed SCOPE work-branch tip is recorded as the attempt baseline |
| `superdev workflow scope-baseline` | 2 | ownership, plan revision, expected work tip, branch, phase, or uncommitted path scope is invalid |
| `superdev workflow scope-checkpoint` | 0 | one valid knowledge-only SCOPE proposal is committed for immutable review |
| `superdev workflow scope-checkpoint` | 2 | ownership, revision, identity, branch, phase, canonical validation, or path scope is invalid |
| `superdev workflow block` | 0 | the newly completed BUILD block passes dependency and executable checks and its path-scoped checkpoint is committed |
| `superdev workflow block` | 2 | ownership, revision, identity, branch, phase, dependency, executable evidence, or path scope is invalid |
| `superdev workflow attempt` | 0 | one normalized failed BUILD attempt is durably counted |
| `superdev workflow attempt` | 2 | ownership, phase, revision, input, tree, or configured retry limit is invalid |
| `superdev workflow correction` | 0 | one candidate-bound final correction cycle is durably counted |
| `superdev workflow correction` | 2 | ownership, authority, candidate, review, phase, tree, or configured correction limit is invalid |
| `superdev workflow correction-checkpoint` | 0 | approved executable checks pass and a focused correction within SCOPE-approved Areas is committed |
| `superdev workflow correction-checkpoint` | 2 | ownership, revision, branch, phase, pending-correction, executable evidence, or path scope is invalid |
| `superdev workflow evidence` | 0 | canonical BUILD evidence is acknowledged |
| `superdev workflow evidence` | 2 | ownership, revision, identity, branch, or phase is invalid |
| `superdev workflow sync` | 0 | the expected default tip is incorporated into the BUILD branch, or was already present |
| `superdev workflow sync` | 2 | ownership, phase, revision, tree, ref, expected tip, or conflict checks failed |
| `superdev workflow resume` | 0 | a clean worktree is switched to the canonical work branch when required, canonical state is reconstructed, and ownership is acquired |
| `superdev workflow resume` | 2 | canonical identity, branch, phase, worktree, or ownership is invalid |
| `superdev workflow cancel` | 0 | transient ownership is released |
| `superdev workflow cancel` | 2 | another session owns the workflow |
| `superdev workflow abandon` | 0 | approved abandonment is persisted |
| `superdev workflow abandon` | 2 | ownership, revision, phase, or approval is invalid |
| `superdev workflow integrate` | 0 | local no-ff integration completed |
| `superdev workflow integrate` | 2 | ownership, revision, ref, tree, or expected tip is invalid |
| `superdev hook` | 2 | no subcommand named |
| `superdev hook validate` | 0 | the edited path is outside the governed trees, or the repo still validates |
| `superdev hook validate` | 2 | findings on stderr, or a payload it cannot read |
| `superdev mcp` | 2 | no subcommand named |
| `superdev mcp sokf` | 0 | the client closed stdin |
| `superdev mcp sokf` | 2 | the server could not start |
| `superdev completions` | 0 | the script is written |
| `superdev completions` | 2 | a shell it does not know |
| `superdev man` | 0 | the page is written |

### Streams

The hook invoker reads the validation hook's stderr. A closed stdout
pipe ends the command as `P_closed-stdout-exits-0` says.

- `P_report-stdout-diagnostics-stderr` [ubiquitous] Every command SHALL
  write its report to stdout and its diagnostics to stderr.
- `P_completions-man-stdout-only` [ubiquitous] `completions` and `man`
  SHALL write their generated file to stdout and nothing else, so the
  output redirects cleanly.
- `P_hook-reads-stdin` [ubiquitous] `hook validate` SHALL read its payload
  from stdin.
- `P_hook-writes-stderr` [ubiquitous] `hook validate` SHALL write findings
  to stderr.
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
- `P_sokf-read-json-shape` [event] WHEN `--json` is given to `sokf
  overview`, `search`, `read` or `graph`, the command SHALL write one JSON
  object with protocol `sokf-tools/v1`, one text content item carrying the
  ordinary command output, and an empty `details` object.
- `P_sokf-mutation-json-shape` [event] WHEN `--json` is given to `sokf edit`
  or `sokf write`, the same envelope's `details` SHALL carry `applied`,
  `validation`, `resolvedPath`, `finalPath`, `changes` (each with `path`,
  `source`, and `diff`) and `findings` (each with `severity`, optional `path`,
  and `message`).

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
[contract-004-config-superdev][sokf:contract-004-config-superdev]. Workflow
ownership and human-gated transitions receive an unpersisted Pi UI capability
through `SUPERDEV_UI_AUTHORITY`; ordinary callers cannot replace it with an
approval flag. Other commands read only the environment that `mise` and `git`
read for themselves.

- `P_hook-resolves-project-dir` [event] WHEN an adapter sets
  `CLAUDE_PROJECT_DIR`, `hook validate` SHALL resolve the repository from
  `CLAUDE_PROJECT_DIR`.
- `P_hook-resolves-working-dir` [conditional] IF `CLAUDE_PROJECT_DIR`
  is unset, `hook validate` SHALL resolve the repository from the working
  directory.
- `P_workflow-ui-authority-env` [event] WHEN Pi establishes workflow ownership or performs a human-gated transition, the command SHALL verify `SUPERDEV_UI_AUTHORITY` against the owning session's digest.

### Usage errors

Clap reports a usage error from every command alike.

- `P_usage-message-on-stderr` [event] WHEN an unknown flag, an unknown
  subcommand or a missing required value is given, every command SHALL
  exit `2` with clap's usage message on stderr.

### Side effects

`--fix` is the one way `validate` writes; SOKF `edit` and `write` are the
knowledge mutation verbs and include that repair automatically. `status`
writes nothing (`P_status-writes-nothing`). `update` is the one verb that
reaches the network unasked, to find the newest pack release.

- `P_validate-writes-only-with-fix` [event] WHEN `validate` runs
  without `--fix`, `validate` SHALL NOT write.
- `P_fix-writes-inside-knowledge` [ubiquitous] `validate --fix` SHALL
  write only inside the resolved knowledge directory.
- `P_fix-idempotent` [ubiquitous] `validate --fix` SHALL be idempotent.
- `P_sokf-mutations-write-knowledge-only` [ubiquitous] `sokf edit` and `sokf
  write` SHALL write only inside the resolved knowledge directory.
- `P_workflow-side-effects-bounded` [ubiquitous] `workflow` SHALL write only its transient cache, bound canonical records and indexes, product paths declared by the current BUILD block, and, for integration, the explicitly validated local refs and merge commit.
- `P_file-side-effects-bounded` [ubiquitous] `file` SHALL write one canonical
  record and generated index changes in a temporary worktree, then advance
  only the validated local default ref.

## Stability

Unreleased.

- `P_unreleased` [ubiquitous] Every command, argument, flag and exit
  code above MAY change without notice.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:contract-003-api-sokf]: /knowledge/contracts/public/active/contract-003-api-sokf.md
[sokf:contract-004-config-superdev]: /knowledge/contracts/public/active/contract-004-config-superdev.md
