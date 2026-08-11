---
type: Backlog
id: backlog
title: Backlog & Decided Ideas
description: Ideas under consideration and ideas decided against, with the reasoning.
status: draft
---

# Under consideration

- **Pinning `node` in the managed repo.** A repo whose mise env has no node
  cannot install an `npm:` tool (mise's backend shells out to `npm`) or run
  one (the installed shim is `#!/usr/bin/env node`). Hosts with node on the
  system PATH, or in their global mise config, never see it; a host whose node
  comes from some other repo's config does. Whether superdev should pin `node`
  itself, and so write a toolchain choice into the user's `.mise.toml`, is
  open.

# Decided against

Nothing yet. Record rejected ideas here with the reasoning, so they are not
re-litigated.
