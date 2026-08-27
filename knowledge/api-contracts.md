---
type: Reference
id: api-contracts
title: API Contracts
description: The CLI surface — the manage verbs, the knowledge verbs, the four MCP tools, and the stability promises.
status: stable
resource: /crates/app/superdev/src/main.rs
---

# CLI surface

```
superdev                     print help, exit 0
superdev init                set this repo up; --no-knowledge, --no-code-index,
                             --no-bash-output-filter, --no-frontend,
                             --no-skills each disable a capability
superdev status              report drift; exit 1 when there is work to do
superdev sync                re-apply the blueprint; --dry-run prints the plan only
superdev update [TARGET]     bring pins current, then sync;
                             TARGET is `<capability>[@<version>]`;
                             --provider <ID> switches TARGET's provider
superdev validate [PATH...] check the AOKF bundle and the superdev-format
                             files; exit 1 on errors. A PATH replaces both
                             defaults. --json, --doc renders the grammar as
                             prose, --bundle <DIR>, --repo-root <DIR> for
                             `/`-rooted paths
superdev aokf validate       (hidden; the same verb under its old name)
superdev aokf index [PATH]   rebuild the search index from scratch
superdev aokf hook validate  the Claude Code PostToolUse hook: payload on
                             stdin, validate when the edit touched the bundle
                             or a tree the grammar governs
superdev mcp aokf            serve the bundle to agents over MCP on stdio
superdev completions <SHELL> write a completion script to stdout
                             (bash | zsh | fish | powershell | elvish)
superdev man                 (hidden; roff to stdout, for packaging)
-V, --version                print `superdev x.y.z` and exit
```

Every verb acts on the current directory.

- **`init`** refuses a directory that is not a git repo, and refuses a re-run
  once `.superdev/config.toml` exists (it points at `sync`). The guard is the
  manifest rather than the directory, because the knowledge verbs create
  `.superdev/cache/` in repos that were never initialised. It writes the
  manifest, then applies the whole blueprint and the `.gitignore` lines. It
  also ensures `CLAUDE.md` contains the line `@AGENTS.md`, appended to an
  existing file or created as a one-line file: Claude Code reads only
  `CLAUDE.md`, and that line is what makes it load the canonical entry point.
  AGENTS.md gets the same treatment with `@.agents/superdev.md` — the file
  is the user's; appending to a pre-existing one reports the hint that
  superdev's old sections can be trimmed.
  Skills the repo already has under a pack or knowledge-carried name are
  released into `[skills] custom` or `[knowledge] custom` first, so adoption
  never overwrites work superdev did not write.
  `--template <name>` seeds the repo from a shipped project
  template (an unknown name fails naming the shipped set), `--template none`
  declines, and `--name` sets the substitution values; on a TTY with neither
  flag, init prompts — template list first, then the project name prefilled
  with the directory name. Without a TTY there is no prompt and no template,
  so scripted init is unchanged. Template files are write-once scaffolds:
  existing files win and are reported as kept. A knowledge-enabled init ends
  with the hint to run `/aokf-bootstrap` in Claude Code — filling the bundle
  from existing docs and an owner interview is judgement work the agent does
  after the mechanical scaffolding.
- **`status`** never writes. It exits `1` on any drift, missing component,
  planned removal, or pin behind this binary's registry, so CI can gate on it.
  Each skill released by `[skills] custom` prints as
  `skills: <name> custom, unmanaged`, and each one released by
  `[knowledge] custom` the same way under its own capability name — a released
  skill is the user's file, not drift, so it leaves the code alone. Released orphans and the
  blueprint-version line print as reports and never affect the exit code.
  A `content:` line names where the content came from, because which entry
  superdev treated as layer 0 is inferred from the source and would otherwise
  be invisible ([ADR-004](decisions/D004-base-pack-identity.md)):
  `content: embedded pack <version>` when no entry replaced it,
  `content: base <source> at <rev>` when one did, `content: layer <source>`
  per pack above it, and `content: <source> not resolved` for a pin `status`
  could not satisfy — which it never fetches to satisfy. One pack hiding
  another's item prints
  `content: <winner> supersedes <loser>'s <item>`; hiding layer 0's is what a
  pack is for and prints nothing. All of these are reports and none affects
  the exit code — layering is what the manifest asked for, not drift.
- **`sync`** refuses to run while a registry-locked capability
  (`code-index`, `skills`, `bash-output-filter`) is pinned anywhere other
  than the registry default, and says to run
  `superdev update <capability>`. `code-index` and `bash-output-filter`
  are downloaded by URL and verified against a
  checksum baked into this binary beside the version, so no other version has
  provenance — or a URL; the skill pack's and the knowledge capability's
  content is embedded in the binary, so
  the binary is its provenance. On a fresh clone it runs `mise trust` then
  `mise install` before any provider command, because the committed pins need no
  edit yet name tools this machine has never installed — and mise will not
  install from a config this machine has never trusted. That install names
  superdev's own tools, so a repo pin superdev knows nothing about can never
  fail the run. Orphan removals run after every write, so a rename whose write
  fails rolls back before anything is deleted; an orphan the user has edited is
  released instead of removed. A successful run stamps this binary's version as
  the manifest's `blueprint`.
- **`update`** rejects an explicit `code-index@<version>` or
  `skills@<version>` for the same reason. Every other
  capability takes an explicit version. `--provider <id>` is the only CLI path
  that switches a provider: it needs a capability target, rewrites that
  capability's provider and sets its version to the new provider's registry
  default, then syncs. Bare `update` moves versions and leaves every provider
  alone — and it alone moves the pack pin, asking the default source for its
  newest release and taking that even when it is past what this binary embeds
  ([ADR-009](decisions/D009-update-queries-default-source.md)); a targeted
  `update <capability>` makes no such request. `update workflows` is an
  unknown-capability error: the capability was
  removed, and a manifest still carrying its table fails at load with the
  guided migration error
  ([spec](specs/S009-knowledge-carried-skills-design.md)).

`validate` with no `PATH` covers the bundle at `--bundle` (default
`knowledge/`) and every tree the format grammar's `roots` names; `aokf index`
defaults `PATH` to `knowledge/`; the hook always reads the same whole set. The
search index lives in `.superdev/cache/aokf-index/`; `aokf index` and the
server use it, `validate` never opens it.

- **`validate`** runs both checks and reports once, with findings grouped by
  file, so a file both have something to say about is reported once and the
  two cannot reach different verdicts
  ([P006 D-17](adhoc-plans/P006-rust-format-validator.md)). It prints findings
  as text, or as the reference validator's JSON under `--json` — same keys,
  same `bundle` key, same exit codes, so anything scripted against the old
  Python validator still works. Warnings alone exit `0`; any error exits `1`.
  A `PATH` replaces both defaults: only what it names is read, and the bundle
  is validated only when a `PATH` is the bundle or contains it. The grammar
  comes from `.agents/format/grammar.yaml`, or from the copy inside the binary
  when the repository has none.
- **`aokf validate`** is the same verb under the name it shipped with, hidden
  from help. The hook marker and its lock entry are keyed on that spelling in
  every managed repo, so it stays.
- **`aokf index`** forces a full rebuild. Nothing else needs it: the server
  syncs lazily on every tool call. It says so when no embedding model loaded
  and the index is lexical-only.
- **`aokf hook validate`** reads the PostToolUse payload from stdin and exits
  `0` unless the edited path is under the bundle or under a tree the grammar
  governs. Then it validates the whole set in-process and, on errors, prints
  them to stderr and exits `2` — which Claude Code hands back to the agent as a
  blocking error. It resolves the repo from `CLAUDE_PROJECT_DIR` when Claude
  Code sets it, else the working directory.
- **`mcp aokf`** serves one stdio client and exits `0` when that client closes
  stdin. A missing bundle or an unusable index directory fails at startup
  rather than at every tool call, because a client cannot act on the latter.

# MCP tools

Four read-only tools, stdio only, no resources or prompts. Every hit carries
the locator set — bundle-relative path, concept id, heading path, line range,
snippet, score — so the next call can read exactly what matched.

- **`aokf_search`** — `query`, optional `limit` (8 by default, clamped to
  1..50), `types` and `tags`. Filters apply before fusion, so a filtered
  concept cannot re-enter through the other ranking. Settled work — a
  `deprecated` concept, or one tagged `done`, `resolved` or `wontfix` — is
  down-ranked after fusion, so finished plans and issues sort below live
  knowledge without leaving the results. Results group by concept,
  strongest concept first.
- **`aokf_read`** — `id` (or bundle-relative path), optional `heading`: the
  whole concept, or one section named by heading or `a > b` heading path.
  `(root)` names the frontmatter-and-preamble section.
- **`aokf_graph`** — no argument: the bundle-wide map of *declared* edges,
  grouped by source. With `id`: that concept's single-hop neighbours in both
  directions. Each group caps at 30 lines and then says how many it dropped.
- **`aokf_overview`** — the bundle name, its concept count, the directory tree
  with each concept's id and description, and every validation finding,
  warnings included, whenever there is one.

A tool failure is an MCP error payload, never a process exit: an unknown id
comes back with near-miss candidates, and a bundle that fails validation still
indexes and serves — agents need search most while fixing one. Reading a file
the parser choked on quotes the parse error instead of guessing at near
misses.

A usage error (unknown flag or subcommand) exits `2` — the npm launcher's
smoke test relies on that code. `completions` and `man` render into a buffer
before writing, because `clap_complete` panics rather than returning an error
when a write fails. Exit codes are in [error-handling](error-handling.md); the
manifest the verbs read is in [configuration](configuration.md).

# Stability

Unreleased. Everything above may change without notice; `superdev-core`'s Rust
API is not stable.
