---
type: Backlog
id: backlog
title: Backlog & Decided Ideas
description: Ideas under consideration and ideas decided against, with the reasoning.
status: draft
---

# Under consideration

Nothing yet.

# Decided against

- **Pinning `node` in the managed repo.** Considered because codegraph was
  pinned through mise's `npm:` backend, which cannot install without an `npm`
  in the repo's mise env and produces a `#!/usr/bin/env node` shim that cannot
  run without a node either. Rejected: pinning node writes a toolchain choice
  into the user's `.mise.toml` to work around a packaging detail. codegraph is
  now pinned through the `http` backend against its self-contained release
  bundles, which vendor their own Node, so the dependency is gone rather than
  papered over — see [architecture](architecture.md).
