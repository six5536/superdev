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
- **A mise-pinned tool is invoked through `mise exec`.** `mise install` puts
  the tool on no PATH the running process can see, so a bare spawn fails
  wherever mise is activated rather than shimmed. Tools superdev does not pin
  — `claude`, from the user's own install — are spawned directly.
- **Every mise command names the tools superdev manages.** `mise install` and
  `mise exec` default to the repo's whole toolchain, which would tie a
  superdev run to pins it knows nothing about: adopting superdev in a repo
  whose `cargo:` pin cannot build on this machine failed the entire apply.
  Naming the tools keeps the blast radius at superdev's own.
- **Capability names are the user-facing surface; provider names are not.**
  Flags, manifest keys and lock keys say `code-index`, never `codegraph`.
  Swapping a provider must not change what a user types.
- **Domain logic lives in `superdev-core`**; the binary stays a thin
  argument-parsing and wiring layer — see
  [software-components](software-components.md).
