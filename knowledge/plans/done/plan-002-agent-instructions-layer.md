---
type: Plan
id: plan-002-agent-instructions-layer
title: Agent Instructions Layer
description: Deliver S010 — the user-owned AGENTS.md with one ensured import, the fenced superdev.md aggregator, per-capability instruction files, codegraph MCP wiring, and the code-index dogfood.
lifecycle: done
---

# Plan: Agent Instructions Layer

## Goal

Implement the agent instructions layer (S010, retired):
AGENTS.md becomes the user's file, superdev's guidance moves behind one
ensured import, and code-index gains its agent wiring.

## Contract changes

- none.

## Work blocks

### Block 1: prefactor — ensure-line append note

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: the engine's ensure-line
  outcome carries a note when the line was appended to a file that
  already existed, so callers can ride a report on exactly the
  migrating population.
- Done-check: an engine unit test asserts the note on append-to-existing
  and its absence on create and on skip.
- Cases:
  - unit: the ensure-line outcome carries the note when the line is
    appended to an existing file — no criterion.
  - unit: the outcome carries no note on create and on skip — no
    criterion.

### Block 2: entry-point restructure (knowledge)

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: the aokf component stops
  planning the AGENTS.md scaffold and ships `.agents/aokf.md` with the
  scaffold's content; a repo-level entry writes the fenced
  `.agents/superdev.md` aggregator with one import per enabled
  capability's instruction file; AGENTS.md gets the ensured
  `@.agents/superdev.md` line, reporting the trim hint on append.
- Done-check: aokf unit tests (no AGENTS.md write, aokf.md claimed), a
  pipeline test that the aggregator's imports track the enabled set,
  and the init journey asserting the one-line AGENTS.md, the fence,
  and aokf.md.
- Cases:
  - unit: the aokf component writes no AGENTS.md and claims
    `.agents/aokf.md` — no criterion.
  - integration: the aggregator's imports track the enabled capability
    set — no criterion.
  - e2e: the init journey produces the one-line AGENTS.md, the fenced
    aggregator and `.agents/aokf.md` — no criterion.

### Block 3: codegraph agent wiring

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: the codegraph component plans
  `.agents/codegraph.md` and the `mcpServers.codegraph` registration
  launching `codegraph serve --mcp` through mise; the aggregator
  imports it when code-index is enabled; disabling sweeps file, key
  and import.
- Done-check: codegraph unit tests for plan and claims, and the
  disable-code-index journey asserting the sweep.
- Cases:
  - unit: the codegraph component plans `.agents/codegraph.md` and the
    `mcpServers.codegraph` registration, and claims both — no
    criterion.
  - e2e: disabling code-index sweeps the file, the key and the
    aggregator import — no criterion.

### Block 4: dogfood and knowledge upkeep

- [x] Done — ticked at merge.
- Depends-on: 3.
- Change: enable `[code-index]` in this
  repo's manifest and sync; trim this repo's AGENTS.md to the new
  minimal form; update the affected concepts (architecture,
  configuration, api-contracts, development-procedure,
  directory-structure); flip the spec to `stable` and tag this plan
  `done` in the completing commit.
- Done-check: `superdev status` exits 0, `superdev aokf validate
  knowledge` passes, and the full pre-PR check set is
  green.
- Cases:
  - e2e: `superdev status` exits 0 on this repository after the sync —
    no criterion.
  - e2e: `superdev aokf validate knowledge` passes — no criterion.
