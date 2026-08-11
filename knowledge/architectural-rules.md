---
type: Convention
id: architectural-rules
title: Architectural Rules
description: Planning is side-effect free, the engine is the only place that applies, and capabilities are the user-facing names.
status: stable
---

The invariants behind the [architecture](architecture.md):

- **Components observe and plan; they never change anything.** `plan` reads
  the repo and the manifest and returns actions. Running it twice changes
  nothing, which is what makes `status` and `--dry-run` free.
- **The engine is the only side-effect site.** File writes, mise edits and
  external commands happen in one place, so every one of them is journalled
  and can be rolled back.
- **Capability names are the user-facing surface; provider names are not.**
  Flags, manifest keys and lock keys say `code-index`, never `codegraph`.
  Swapping a provider must not change what a user types.
- **Domain logic lives in `superdev-core`**; the binary stays a thin
  argument-parsing and wiring layer — see
  [software-components](software-components.md).
