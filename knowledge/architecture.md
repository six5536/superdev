---
type: Reference
id: architecture
title: Architecture
description: The core/binary/blueprint layering, the capability-to-provider map, the knowledge-serving side, and the files superdev keeps in a managed repo.
status: stable
links:
  - rel: relates-to
    to: software-components
    note: The crates and packages these layers ship as.
  - rel: relates-to
    to: spec-cli-core-blueprint-engine
    note: The design this summarises.
  - rel: relates-to
    to: spec-aokf-mcp-server
    note: The read-side design the aokf subsystem implements.
---

superdev runs inside a target repo and keeps that repo's agent-development
setup current. Three layers, detailed in the
[CLI core & blueprint engine spec](specs/2026-08-11-cli-core-blueprint-engine-design.md):

- **`superdev-core`** — the domain: the manifest, the components, planning,
  the engine that applies a plan, and the `aokf` subsystem that reads the
  knowledge bundle back out.
- **`superdev` (binary)** — argument parsing, output rendering, exit codes.
- **The blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus a registry of default versions tested
  together. The binary's version is the blueprint version.

# Serving the knowledge bundle

Installing the `knowledge` capability is half of it; the other half is reading
it back. The `aokf` subsystem parses the bundle, validates it, indexes it, and
serves it to agents over MCP (`superdev mcp aokf`), so an agent queries the
knowledgebase instead of preloading every concept — the design is in the
[AOKF MCP server spec](specs/2026-08-11-aokf-mcp-server-design.md), the tools
in [api-contracts](api-contracts.md). Freshness is lazy: every tool call
re-hashes the bundle and syncs only what changed, so there is no watcher and
no daemon state to go stale.

# Capabilities and providers

A capability is a slot; the tool filling it is a swappable provider.

| Capability   | Provider          | Delivered as                    |
|--------------|-------------------|---------------------------------|
| `knowledge`  | `aokf`            | files embedded in the binary    |
| `code-index` | `codegraph`       | checksummed release bundle (mise `http`) + `mise exec http:codegraph -- codegraph init` |
| `workflows`  | `mattpocock-skills` (default) | mise pin + owned files under `.claude/skills/` |
| `workflows`  | `superpowers`     | mise pin + Claude Code plugin   |
| `frontend`   | `frontend-design` | Claude Code plugin              |
| `skills`     | `superdev-skills` | owned files in the repo         |

The registry holds one entry per (capability, provider) pair — its version, its
checksum where it has one, and whether it is the default — and the manifest's
`provider` field picks among them. `workflows` is the only capability with a
choice; an id no entry matches fails with `workflows provider must be one of:
superpowers, mattpocock-skills`.

The default is repo-owned deliberately. `mattpocock-skills` copies the pinned
checkout's skills into `.claude/skills/` as committed files, so a collaborator
who clones the repo has them from git alone; `superpowers` needs a per-user
`claude plugin install` on every machine, which is what makes it the secondary.

`workflows` and `code-index` are fetched by URL and verified against a
checksum this binary carries beside the version, so superdev installs the
registry version of those two and refuses any other — see
[api-contracts](api-contracts.md). codegraph's bundles vendor their own Node,
so a managed repo needs no node of its own.

`skills` refuses any other version for a different reason: the four skill
files are embedded in the binary, which makes this binary the provenance.
Nothing is installed. `sync` writes them to `.claude/skills/<name>/SKILL.md`
and merges one PostToolUse entry into `.claude/settings.json`; Claude Code
reads both natively, so a teammate who clones the repo gets the pack and the
validation hook without installing superdev.

# Files in a managed repo

`.superdev/config.toml` records what the repo wants and `.superdev/lock.toml`
what superdev last applied; both are committed. `.superdev/cache/` holds
machine state and is gitignored. Their shape is in
[configuration](configuration.md); the code implementing them is listed in
[software-components](software-components.md).

Two things superdev keeps are lines in files it does not own: the `.gitignore`
entries for the cache, and `@AGENTS.md` in `CLAUDE.md` — the line that makes
Claude Code load the canonical entry point at all. Both are added when missing,
never rewritten and never hashed, so neither can drift or be orphaned; delete
one and the next `sync` puts it back.

Migrations are derived, not scripted: what the lock records minus what the
components claim is what `sync` removes, so a dropped file, a rename's old copy
and a disabled capability's pins all follow one rule.
