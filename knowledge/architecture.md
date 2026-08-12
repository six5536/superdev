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
| `code-index` | `codegraph`       | checksummed release bundle (mise `http`) + `mise exec -- codegraph init` |
| `workflows`  | `superpowers`     | mise pin + Claude Code plugin   |
| `frontend`   | `frontend-design` | Claude Code plugin              |
| `skills`     | `superdev-plugin` | slot only; no provider yet      |

`workflows` and `code-index` are fetched by URL and verified against a
checksum this binary carries beside the version, so superdev installs the
registry version of those two and refuses any other — see
[api-contracts](api-contracts.md). codegraph's bundles vendor their own Node,
so a managed repo needs no node of its own.

# Files in a managed repo

`.superdev/config.toml` records what the repo wants and `.superdev/lock.toml`
what superdev last applied; both are committed. `.superdev/cache/` holds
machine state and is gitignored. Their shape is in
[configuration](configuration.md); the code implementing them is listed in
[software-components](software-components.md).
