---
type: Architecture
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
    note: The read-side design the sokf subsystem implements.
---

superdev runs inside a target repo and keeps that repo's agent-development
setup current. Three layers, detailed in the
[CLI core & blueprint engine spec](specs/S001-cli-core-blueprint-engine-design.md):

- **`superdev-core`** — the domain: the manifest, the components, planning,
  the engine that applies a plan, and the `sokf` subsystem that reads the
  knowledge back out.
- **`superdev` (binary)** — argument parsing, output rendering, exit codes.
- **The blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus a registry of default versions tested
  together. The binary's version is the blueprint version.

# Serving the canonical knowledge

Installing the `knowledge` capability is half of it; the other half is reading
it back. The `sokf` subsystem parses the SOKF knowledge, indexes it and
serves it to agents over MCP (`superdev mcp sokf`) — the `validate` subsystem
checks it — so an agent queries the
knowledge instead of preloading every concept — the design is in the
[SOKF MCP server spec](specs/S002-aokf-mcp-server-design.md), the tools
in [api-contracts](api-contracts.md). Freshness is lazy: every tool call
re-hashes the canonical knowledge and syncs only what changed, so there is no watcher and
no daemon state to go stale.

# Content resolves before planning

Every skill, scaffold and document template a component writes comes from one
resolved content set, built before any component plans and handed to them
through `Ctx`. Nothing is fetched or read during planning, which is what keeps
`plan` side-effect free and `status` free
([ADR-002](decisions/D002-resolve-before-plan.md)). A component asks the set
for the items it owns rather than carrying a list of them, so what superdev
ships is decided by the pack tree — see
[directory-structure](directory-structure.md) for its shape and
[ADR-003](decisions/D003-items-by-layout.md) for the rules that name an item.

Layer 0 is the pack compiled into the binary; each `[[packs]]` entry layers
over it in manifest order, and a later item of the same identity wins. A pin
naming exactly what the binary embeds resolves from it and makes no request,
whichever way the source is spelled. A local-path source is read from disk
every run, so editing it and syncing again lands the new bytes with no
rebuild.

# Capabilities and providers

A capability is a slot; the tool filling it is a swappable provider.

| Capability   | Provider          | Delivered as                    |
|--------------|-------------------|---------------------------------|

| `code-index` | `codegraph`       | checksummed release bundle (mise `http`) + `mise exec http:codegraph -- codegraph init` |
| `frontend`   | `frontend-design` | Claude Code plugin              |
| `skills`     | `superdev-skills` | owned files in the repo         |
| `bash-output-filter` | `rtk`     | checksummed release binaries in owned, platform-scoped mise config files, plus a PreToolUse rewrite hook |

The registry holds one entry per (capability, provider) pair — its version, its
checksum where it has one, and whether it is the default — and the manifest
picks among them: one `provider` field per slot, except `skills`, whose
many-provider shape is in
[configuration](configuration.md). Every capability currently has exactly one
registry entry; an id no entry matches fails with `<capability> provider must
be one of: …`. A manifest still naming the removed `workflows` capability fails at
load with a guided error
([spec](specs/S009-knowledge-carried-skills-design.md)).

`code-index` is fetched by URL and verified against a checksum this binary
carries beside the version, so superdev installs the registry version and
refuses any other — see [api-contracts](api-contracts.md). codegraph's bundles
vendor their own Node, so a managed repo needs no node of its own.

`skills` refuses any other version for a different reason: the pack's three
skill files are embedded in the binary, which makes this binary the
provenance. Nothing is installed. `sync` writes them to
`.claude/skills/<name>/SKILL.md`; Claude Code reads them natively, so a
teammate who clones the repo gets the pack without installing superdev. The
knowledge capability carries a much larger set the same way: the 17
SOKF-carried skills — the workflow phases and their support skills — each
materialised as its whole directory of owned files, and the document
templates the skills fill in as owned files under `knowledge/templates/`
([spec](specs/S009-knowledge-carried-skills-design.md)). It also merges the
validation hook's PostToolUse entry into `.claude/settings.json`, so hook and
skills exist exactly where knowledge exists
([spec](specs/S008-knowledge-owned-skills-design.md)).

# Files and artefacts

The files superdev writes into a managed repo, and what each is for.

`.superdev/config.toml` records what the repo wants and `.superdev/lock.toml`
what superdev last applied; both are committed. `.superdev/cache/` holds
machine state and is gitignored. Their shape is in
[configuration](configuration.md); the code implementing them is listed in
[software-components](software-components.md).

Three things superdev keeps are lines in files it does not own: the
`.gitignore` entries for the cache, `@AGENTS.md` in `CLAUDE.md` — the line
that makes Claude Code load the entry point at all — and
`@.agents/superdev.md` in AGENTS.md itself. All are added when missing,
never rewritten and never hashed, so none can drift or be orphaned; delete
one and the next `sync` puts it back.

AGENTS.md is the user's file: superdev's guidance sits behind that one
import, in the owned `.agents/superdev.md` — a `<superdev-system>` fence
wrapping a short prompt, the general-rules imports (`.agents/professionalism.md`,
`.agents/process.md` and `.agents/coding.md`, write-once scaffolds every
managed repo gets),
and one import per enabled capability's instruction file
(`.agents/sokf.md`, `.agents/codegraph.md`), rewritten as
the enabled set changes
([spec](specs/S010-agent-instructions-layer-design.md)). Each instruction
file is owned by its capability, so it exists exactly where the capability
does; codegraph's also comes with the `mcpServers.codegraph` registration
that serves the index over MCP.

Migrations are derived, not scripted: what the lock records minus what the
components claim is what `sync` removes, so a dropped file, a rename's old copy
and a disabled capability's pins all follow one rule.
