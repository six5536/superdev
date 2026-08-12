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
                             --no-workflows, --no-frontend, --no-skills each
                             disable a capability
superdev status              report drift; exit 1 when there is work to do
superdev sync                re-apply the blueprint; --dry-run prints the plan only
superdev update [TARGET]     move pins to this binary's defaults, then sync;
                             TARGET is `<capability>[@<version>]`
superdev aokf validate [PATH]
                             check the bundle against the AOKF spec; exit 1
                             on errors. --level 0..2 (default 2), --json,
                             --repo-root <DIR> for `/`-rooted paths
superdev aokf index [PATH]   rebuild the search index from scratch
superdev aokf hook validate  the Claude Code PostToolUse hook: payload on
                             stdin, validate when the edit touched the bundle
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
  manifest, then applies the whole blueprint and the `.gitignore` lines. Skills
  the repo already has under a pack name are released into `[skills] custom`
  first, so adoption never overwrites work superdev did not write.
- **`status`** never writes. It exits `1` on any drift, missing component, or
  pin behind this binary's registry, so CI can gate on it. Each skill released
  by `[skills] custom` prints as `skills: <name> custom, unmanaged` — a
  released skill is the user's file, not drift, so it leaves the code alone.
- **`sync`** refuses to run while `workflows`, `code-index` or `skills` is
  pinned anywhere other than the registry default, and says to run
  `superdev update`. The first two are downloaded by URL and verified against a
  checksum baked into this binary beside the version, so no other version has
  provenance — or a URL; the skill pack's content is embedded in the binary, so
  the binary is its provenance. On a fresh clone it runs `mise trust` then
  `mise install` before any provider command, because the committed pins need no
  edit yet name tools this machine has never installed — and mise will not
  install from a config this machine has never trusted. That install names
  superdev's own tools, so a repo pin superdev knows nothing about can never
  fail the run.
- **`update`** rejects an explicit `workflows@<version>`,
  `code-index@<version>` or `skills@<version>` for the same reason. Every other
  capability takes an explicit version.

`aokf validate` and `aokf index` default `PATH` to `knowledge/`; the hook
always reads the bundle at `knowledge/`. The search index lives in
`.superdev/cache/aokf-index/`; `aokf index` and the server use it, `aokf
validate` never opens it.

- **`aokf validate`** prints findings as text, or as the reference validator's
  JSON under `--json` — same keys, same `bundle` key, same exit codes, so
  anything scripted against the old Python validator still works. Warnings
  alone exit `0`; only an error at or below the graded level exits `1`.
- **`aokf index`** forces a full rebuild. Nothing else needs it: the server
  syncs lazily on every tool call. It says so when no embedding model loaded
  and the index is lexical-only.
- **`aokf hook validate`** reads the PostToolUse payload from stdin and exits
  `0` unless the edited path is under the bundle. Then it validates in-process
  and, on errors, prints them to stderr and exits `2` — which Claude Code hands
  back to the agent as a blocking error. It resolves the repo from
  `CLAUDE_PROJECT_DIR` when Claude Code sets it, else the working directory.
- **`mcp aokf`** serves one stdio client and exits `0` when that client closes
  stdin. A missing bundle or an unusable index directory fails at startup
  rather than at every tool call, because a client cannot act on the latter.

# MCP tools

Four read-only tools, stdio only, no resources or prompts. Every hit carries
the locator set — bundle-relative path, concept id, heading path, line range,
snippet, score — so the next call can read exactly what matched.

- **`aokf_search`** — `query`, optional `limit` (8 by default, clamped to
  1..50), `types` and `tags`. Filters apply before fusion, so a filtered
  concept cannot re-enter through the other ranking. Results group by concept,
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
