---
type: Plan
id: plan-002-agent-instructions-layer
title: Agent Instructions Layer
description: Deliver S010 — the user-owned AGENTS.md with one ensured import, the fenced superdev.md aggregator, per-capability instruction files, codegraph MCP wiring, and the code-index dogfood.
lifecycle: done
phase: done
branch: work/060-historical-agent-instructions-layer
links:
- rel: implements
  to: issue-060-historical-agent-instructions-layer
---
# Plan: Agent Instructions Layer

Primary issue: [issue-060-historical-agent-instructions-layer][sokf:issue-060-historical-agent-instructions-layer]

## Goal and boundaries

Implement the agent instructions layer (S010, retired):
AGENTS.md becomes the user's file, superdev's guidance moves behind one
ensured import, and code-index gains its agent wiring.

## Requirements

Preserve the historical plan intent and constraints recorded under Goal and boundaries.

## Contract changes

- none.

## ADR decisions

- none beyond decisions already linked or described in this historical record.

## Source and interface changes

Historical source and interface changes remain described in the work blocks.

## Knowledge changes

Preserve this record under the canonical workflow schema.

## Documentation changes

Historical documentation impact predates the documentation map; migration itself is checked as canonical-knowledge.

## Work blocks

### Block 1: prefactor — ensure-line append note

- [x] Done — ticked at merge.
- Dependencies: none.
- Areas: unknown (legacy record; no affected area inferred).
- Outcome: the engine's ensure-line
  outcome carries a note when the line was appended to a file that
  already existed, so callers can ride a report on exactly the
  migrating population.
- Verification: an engine unit test asserts the note on append-to-existing
  and its absence on create and on skip.
- Tests:
  - unit: the ensure-line outcome carries the note when the line is
    appended to an existing file — no criterion.
  - unit: the outcome carries no note on create and on skip — no
    criterion.
- Structural evidence: none recorded; this historical record makes no executable evidence claim.
- Documentation: none recorded; this historical record makes no documentation claim.

### Block 2: entry-point restructure (knowledge)

- [x] Done — ticked at merge.
- Dependencies: unknown (legacy record; no dependency inferred).
- Areas: unknown (legacy record; no affected area inferred).
- Outcome: the aokf component stops
  planning the AGENTS.md scaffold and ships `.agents/aokf.md` with the
  scaffold's content; a repo-level entry writes the fenced
  `.agents/superdev.md` aggregator with one import per enabled
  capability's instruction file; AGENTS.md gets the ensured
  `@.agents/superdev.md` line, reporting the trim hint on append.
- Verification: aokf unit tests (no AGENTS.md write, aokf.md claimed), a
  pipeline test that the aggregator's imports track the enabled set,
  and the init journey asserting the one-line AGENTS.md, the fence,
  and aokf.md.
- Tests:
  - unit: the aokf component writes no AGENTS.md and claims
    `.agents/aokf.md` — no criterion.
  - integration: the aggregator's imports track the enabled capability
    set — no criterion.
  - e2e: the init journey produces the one-line AGENTS.md, the fenced
    aggregator and `.agents/aokf.md` — no criterion.
- Structural evidence: none recorded; this historical record makes no executable evidence claim.
- Documentation: none recorded; this historical record makes no documentation claim.

### Block 3: codegraph agent wiring

- [x] Done — ticked at merge.
- Dependencies: unknown (legacy record; no dependency inferred).
- Areas: unknown (legacy record; no affected area inferred).
- Outcome: the codegraph component plans
  `.agents/codegraph.md` and the `mcpServers.codegraph` registration
  launching `codegraph serve --mcp` through mise; the aggregator
  imports it when code-index is enabled; disabling sweeps file, key
  and import.
- Verification: codegraph unit tests for plan and claims, and the
  disable-code-index journey asserting the sweep.
- Tests:
  - unit: the codegraph component plans `.agents/codegraph.md` and the
    `mcpServers.codegraph` registration, and claims both — no
    criterion.
  - e2e: disabling code-index sweeps the file, the key and the
    aggregator import — no criterion.
- Structural evidence: none recorded; this historical record makes no executable evidence claim.
- Documentation: none recorded; this historical record makes no documentation claim.

### Block 4: dogfood and knowledge upkeep

- [x] Done — ticked at merge.
- Dependencies: unknown (legacy record; no dependency inferred).
- Areas: unknown (legacy record; no affected area inferred).
- Outcome: enable `[code-index]` in this
  repo's manifest and sync; trim this repo's AGENTS.md to the new
  minimal form; update the affected concepts (architecture,
  configuration, api-contracts, development-procedure,
  directory-structure); flip the spec to `stable` and tag this plan
  `done` in the completing commit.
- Verification: `superdev status` exits 0, `superdev aokf validate
  knowledge` passes, and the full pre-PR check set is
  green.
- Tests:
  - e2e: `superdev status` exits 0 on this repository after the sync —
    no criterion.
  - e2e: `superdev aokf validate knowledge` passes — no criterion.

- Structural evidence: none recorded; this historical record makes no executable evidence claim.

- Documentation: none recorded; this historical record makes no documentation claim.

## Build state

All historical blocks are complete; blocker: none.

## Implementation decisions

none.

## Follow-up issues

none.

## Completion evidence

Historical plan migrated mechanically to the canonical plan shape. Original completion evidence remains preserved in its work blocks and Git history.

<!-- sokf:links -->
[sokf:issue-060-historical-agent-instructions-layer]: /knowledge/issues/done/issue-060-historical-agent-instructions-layer.md
