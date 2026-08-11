---
type: Reference
id: constraints-non-goals
title: Known Constraints & Non-Goals
description: Accepted limitations of the inherited machinery; product non-goals are TBD.
status: draft
---

# Non-goals

Not yet defined; record them as the [architecture](architecture.md) is decided.

# Constraints (inherited machinery)

- **Pre-1.0**: minor versions may carry breaking changes; `superdev-core`'s
  API is not stable.
- **Cross-registry publishing is not atomic**; the release pipeline is
  ordered, dry-run-gated and recoverable instead — see
  [release-procedure](release-procedure.md).
- zigbuild's musl output is non-PIE; accepted for a local CLI with no network
  input.
- **No auto-download or build-from-source fallback** in the npm launcher on
  unsupported platforms; the message points at `cargo install superdev`.
