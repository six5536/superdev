---
type: ArchitecturalRules
id: architectural-rules
title: Architectural Rules
description: Planning is side-effect free, the engine is the only place that applies, and capabilities are the user-facing names.
status: stable
---

The invariants behind the [architecture][sokf:architecture]:

- **Components observe and plan; they never change anything.** `plan` reads
  the repo and the manifest and returns actions. Running it twice changes
  nothing, which is what makes `status` and `--dry-run` free.
- **The engine is the only side-effect site for the repo.** Every write to a
  managed file, every mise edit and every provisioning command happens in one
  place, so each is journalled and can be rolled back.
  Resolving content is the one exception, and sits outside the repo: it reads
  local paths, may spawn `git` for a pinned pack, and populates the machine's
  own cache under `.superdev/cache/packs/`
  ([ADR-002][sokf:adr-002-resolve-before-plan],
  [ADR-005][sokf:adr-005-pack-cache-and-fetch]). It runs to completion
  before any plan exists, so there is nothing yet to roll back — and putting
  it after planning would mean planning read the network, which is what keeps
  `status` free of it.
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
  [software-components][sokf:software-components].

<!-- sokf:links -->
[sokf:adr-002-resolve-before-plan]: /knowledge/decisions/adr-002-resolve-before-plan.md
[sokf:adr-005-pack-cache-and-fetch]: /knowledge/decisions/adr-005-pack-cache-and-fetch.md
[sokf:architecture]: /knowledge/architecture.md
[sokf:software-components]: /knowledge/software-components.md
