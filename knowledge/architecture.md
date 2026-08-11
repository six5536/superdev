---
type: Reference
id: architecture
title: Architecture
description: The core/binary/blueprint layering, the capability-to-provider map, and the files superdev keeps in a managed repo.
status: stable
links:
  - rel: relates-to
    to: software-components
    note: The crates and packages these layers ship as.
  - rel: relates-to
    to: spec-cli-core-blueprint-engine
    note: The design this summarises.
---

superdev runs inside a target repo and keeps that repo's agent-development
setup current. Three layers, detailed in the
[CLI core & blueprint engine spec](specs/2026-08-11-cli-core-blueprint-engine-design.md):

- **`superdev-core`** — the domain: the manifest, the components, planning,
  and the engine that applies a plan.
- **`superdev` (binary)** — argument parsing, output rendering, exit codes.
- **The blueprint** — superdev's opinion of a managed repo, compiled into the
  binary: the component set plus a registry of default versions tested
  together. The binary's version is the blueprint version.

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
