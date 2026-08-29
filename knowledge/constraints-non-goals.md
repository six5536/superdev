---
type: Constraints
id: constraints-non-goals
title: Known Constraints & Non-Goals
description: What superdev deliberately does not do, and the accepted limitations of the inherited machinery.
status: stable
---

# Non-goals

- **Generalisation, yet.** superdev is opinionated for one stack — Claude
  Code, mise, SOKF. Other agent harnesses, tool managers and knowledge
  formats stay out of scope until the one stack is proven; the
  capability/provider split ([architecture][sokf:architecture]) is where
  alternatives would slot in later.
- **Managing toolchains.** superdev pins tools in `.mise.toml` and
  delegates installation to mise; it never downloads or manages a
  toolchain itself. (Rejecting a node pin for codegraph's sake is this
  rule applied — see [backlog][sokf:backlog].)
- **A service component.** superdev is a local CLI: no daemon, no hosted
  service, no telemetry. The MCP servers are local stdio processes the
  agent harness spawns per session.

# Constraints

Inherited machinery, and what it fixes.

- **Pre-1.0**: minor versions may carry breaking changes; `superdev-core`'s
  API is not stable.
- **Cross-registry publishing is not atomic**; the release pipeline is
  ordered, dry-run-gated and recoverable instead — see
  [release-procedure][sokf:release-procedure].
- zigbuild's musl output is non-PIE; accepted for a local CLI with no network
  input.
- **No auto-download or build-from-source fallback** in the npm launcher on
  unsupported platforms; the message points at `cargo install superdev`.

<!-- sokf:links -->
[sokf:architecture]: /knowledge/architecture.md
[sokf:backlog]: /knowledge/backlog.md
[sokf:release-procedure]: /knowledge/release-procedure.md
