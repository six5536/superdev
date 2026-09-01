---
type: CliContract
id: contract-002-cli-superdev
title: CLI Contract
description: The superdev command line — every command, argument, flag and exit code it offers, and what each one promises its callers.
lifecycle: active
resource: /crates/app/superdev/src/main.rs
---

# CLI contract: superdev

The superdev command line, defined: the manage verbs, the knowledge verbs
and the run verbs. The block below is the surface; a caller reproduces the
command line from it alone. The decisions behind the shape are
[ADR-033][sokf:adr-033-a-contract-defines-its-interface] and
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation-by-a-test],
and `crates/app/superdev/src/contract.rs` holds the test that binds this
contract to what the binary offers.

## Commands

```yaml
superdev:
  about: Set a repository up for agent-driven development and keep it current.
  args: []
  flags: {}
  exit: { 0: help printed }
superdev init:
  about: Set this repo up for agent-driven development.
  args: []
  flags:
    --no-frontend: { type: bool, about: Skip the frontend design workflows. }
    --no-skills: { type: bool, about: Skip the superdev skill pack. }
    --no-code-index: { type: bool, about: Skip the code index. }
    --template: { type: NAME, about: "Project template to seed from, or `none`." }
    --name: { type: NAME, about: "Project name for substitution; default the directory name." }
  exit: { 0: the repo is set up, 2: not a git repo, a re-run, an unknown template, or a failed apply }
superdev status:
  about: Report drift between the repo and its blueprint.
  args: []
  flags:
    --drift: { type: bool, about: Exit on drift alone, ignoring state a checkout never carries. }
  exit: { 0: nothing to do, 1: drift, a missing component, a planned removal, or a stale pin, 2: an orphaned lock entry it cannot read }
superdev sync:
  about: Re-apply the blueprint so the repo matches the manifest.
  args: []
  flags:
    --dry-run: { type: bool, about: Print the plan without applying it. }
  exit: { 0: the repo matches the manifest, 2: a registry-locked pin off its default, or a failed apply }
superdev update:
  about: Bring pins current, then sync.
  args: [TARGET]
  flags:
    --provider: { type: ID, about: Provider to switch the target capability to. }
  exit: { 0: the pins are current, 2: an explicit version for a registry-locked capability, an unknown capability or provider, or a pin it could not prove }
superdev validate:
  about: Check the SOKF knowledge and the files the grammar governs.
  args: [PATHS]
  flags:
    --fix: { type: bool, about: Repair links, include blocks and definition blocks before checking. }
    --json: { type: bool, about: Emit JSON instead of text. }
    --doc: { type: bool, about: Print the grammar as prose and exit. }
    --knowledge: { type: DIR, about: "SOKF knowledge directory; default `knowledge`." }
    --repo-root: { type: DIR, about: "Repository root for `/`-rooted paths; default this repo." }
  exit: { 0: no errors, 1: errors found, 2: a path it could not read }
superdev template:
  about: Inspect and render the shipped project templates.
  args: []
  flags: {}
  exit: { 0: help printed, 2: no subcommand named }
superdev template list:
  about: List the shipped project templates.
  args: []
  flags: {}
  exit: { 0: the templates are listed }
superdev template render:
  about: Write a template's token-substituted tree into a directory.
  args: [TEMPLATE]
  flags:
    --name: { type: NAME, about: Project name for substitution. }
    --dir: { type: DIR, about: "Directory to write into; created when absent." }
  exit: { 0: the tree is written, 2: an unknown template, or a directory that is not empty }
superdev sokf:
  about: SOKF knowledge commands.
  args: []
  flags: {}
  exit: { 0: help printed, 2: no subcommand named }
superdev sokf index:
  about: Rebuild the search index from scratch.
  args: [PATH]
  flags: {}
  exit: { 0: the index is rebuilt, 2: knowledge it could not read }
superdev run:
  about: Drive the state of an unattended workflow run.
  args: []
  flags: {}
  exit: { 0: help printed, 2: no subcommand named }
superdev run begin:
  about: Arm an unattended run by creating the run state exclusively.
  args: []
  flags:
    --session: { type: ID, about: The session that owns the run. }
    --next: { type: TEXT, about: The first step. }
  exit: { 0: the run is armed, 2: a run already exists }
superdev run advance:
  about: Record a step forward, resetting the watchdog.
  args: []
  flags:
    --session: { type: ID, about: The session recording the step. }
    --next: { type: TEXT, about: The next step. }
  exit: { 0: the step is recorded, 2: no run, or a session that does not own it }
superdev run end:
  about: End the run by removing its state.
  args: []
  flags: {}
  exit: { 0: no run state remains }
superdev hook:
  about: Agent hook plumbing, reading the hook payload from stdin.
  args: []
  flags: {}
  exit: { 0: help printed, 2: no subcommand named }
superdev hook validate:
  about: The Claude Code PostToolUse hook.
  args: []
  flags: {}
  exit: { 0: the edit is let through, 2: findings on stderr, or a payload it cannot read }
superdev hook run:
  about: The Claude Code Stop hook.
  args: []
  flags: {}
  exit: { 0: the turn may end, 2: the next step on stderr, or a payload it cannot read }
superdev mcp:
  about: Serve project subsystems over MCP.
  args: []
  flags: {}
  exit: { 0: help printed, 2: no subcommand named }
superdev mcp sokf:
  about: Serve the canonical knowledge to agents over MCP on stdio.
  args: []
  flags: {}
  exit: { 0: the client closed stdin, 2: the server could not start }
superdev completions:
  about: Write a completion script for the given shell to stdout.
  args: [SHELL]
  flags: {}
  exit: { 0: the script is written, 2: a shell it does not know }
superdev man:
  about: Write the man page as roff to stdout. Hidden, for packaging.
  args: []
  flags: {}
  exit: { 0: the page is written }
```

## Behaviour

Every verb acts on the current directory. Every command carries `-h` and
`--help`, and the root carries `-V` and `--version`; the framework adds
them, so the block above states them once here rather than in each entry.

- **`init`** MUST refuse a directory that is not a git repo, and MUST refuse
  a re-run once `.superdev/config.toml` exists. It MUST write the manifest
  before applying, so a failed run leaves the file the retry resumes from.
  It MUST ensure `CLAUDE.md` carries `@AGENTS.md` and `AGENTS.md` carries
  `@.agents/superdev.md`, appending to an existing file. A skill the repo
  already has under a managed name MUST be released into `custom` before
  anything is written. A template file MUST NOT overwrite an existing file;
  the existing one is kept and reported. On a TTY with neither `--template`
  nor `--name`, `init` prompts; without a TTY it MUST NOT prompt.
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
- **`validate`** MUST report both halves once, findings grouped by file. A
  `PATH` MUST replace both defaults for what is reported, and a `PATH`
  naming a document MUST be reported with bare-run parity. `--fix` is the
  one way `validate` writes: without it `validate` MUST NOT write. `--fix`
  MUST write only inside the resolved knowledge directory and MUST be
  idempotent. `hook validate` MUST NOT pass it.
- **`run`** MUST NOT touch git, the network, or any file outside
  `.superdev/cache/`. `begin` MUST name the owning session and `run end`
  when it refuses.
- **`hook run`** MUST fail open: an unreadable run state is reported and
  exits `0`, while an unreadable payload is a loud `2`.
- **`completions`** and **`man`** MUST render into a buffer before writing,
  so a failed write is an error and never partial output.

## Exit codes

Every command MUST exit `2` on a usage error — an unknown flag, an unknown
subcommand, a missing required value — so the block above lists `2` only
where the code carries a meaning beyond that.

| Code | Meaning |
|------|---------|
| 0 | success — including a check that found nothing, and a closed stdout pipe |
| 1 | a check found something: `status` found work to do, or `validate` found errors |
| 2 | a usage error, a hard failure, or an I/O failure, as `error: <message>` on stderr; for `hook validate` and `hook run`, the blocking code Claude Code hands back to the agent |

## Streams

Every command MUST write its report to stdout and its diagnostics to stderr.
`completions` and `man` MUST write their generated file to stdout and
nothing else, so the output redirects cleanly. `hook validate` and
`hook run` MUST read their payload from stdin and MUST write their findings
and their next step to stderr, which is where Claude Code reads them.
`mcp sokf` MUST speak the MCP protocol over stdin and stdout, so nothing
else MAY be written to stdout for the life of the process. A closed stdout
pipe MUST end the run at `0`, silently.

## Stability

Unreleased. Every command, argument, flag and exit code above MAY change
without notice.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation-by-a-test]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation-by-a-test.md
