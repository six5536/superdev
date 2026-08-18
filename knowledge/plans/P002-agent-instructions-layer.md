---
type: Plan
id: plan-agent-instructions-layer
title: Agent Instructions Layer
description: Deliver S010 — the user-owned AGENTS.md with one ensured import, the fenced superdev.md aggregator, per-capability instruction files, codegraph MCP wiring, and the code-index dogfood.
status: stable
tags: [done]
links:
  - rel: implements
    to: spec-agent-instructions-layer
---

# Goal

Implement the
[agent instructions layer spec](/knowledge/specs/S010-agent-instructions-layer-design.md):
AGENTS.md becomes the user's file, superdev's guidance moves behind one
ensured import, and code-index gains its agent wiring.

# Tasks

1. **Prefactor: ensure-line append note** — the engine's ensure-line
   outcome carries a note when the line was appended to a file that
   already existed, so callers can ride a report on exactly the
   migrating population.
   Verify: an engine unit test asserts the note on append-to-existing
   and its absence on create and on skip.
2. **Entry-point restructure (knowledge)** — the aokf component stops
   planning the AGENTS.md scaffold and ships `.agents/aokf.md` with the
   scaffold's content; a repo-level entry writes the fenced
   `.agents/superdev.md` aggregator with one import per enabled
   capability's instruction file; AGENTS.md gets the ensured
   `@.agents/superdev.md` line, reporting the trim hint on append.
   Verify: aokf unit tests (no AGENTS.md write, aokf.md claimed), a
   pipeline test that the aggregator's imports track the enabled set,
   and the init journey asserting the one-line AGENTS.md, the fence,
   and aokf.md.
3. **Codegraph agent wiring** — the codegraph component plans
   `.agents/codegraph.md` and the `mcpServers.codegraph` registration
   launching `codegraph serve --mcp` through mise; the aggregator
   imports it when code-index is enabled; disabling sweeps file, key
   and import.
   Verify: codegraph unit tests for plan and claims, and the
   disable-code-index journey asserting the sweep.
4. **Dogfood and knowledge upkeep** — enable `[code-index]` in this
   repo's manifest and sync; trim this repo's AGENTS.md to the new
   minimal form; update the affected concepts (architecture,
   configuration, api-contracts, development-procedure,
   directory-structure); flip the spec to `stable` and tag this plan
   `done` in the completing commit.
   Verify: `superdev status` exits 0, `superdev aokf validate
   knowledge` passes at level 2, and the full pre-PR check set is
   green.
