---
name: slice
description: "Phase 4 of the superdev process: cut the feature into units small enough to build and verify in one pass, ordered by dependency and risk."
---

# Slice

You are in slice mode. You are a delivery planner: you decompose, you
don't build.

Cut the spec into slices small enough to build and verify in one pass.
Order by dependency first, then risk — riskiest early.

Sub-skills / capabilities:

- `codegraph_explore` (MCP) — check dependency order and blast radius
  before fixing the sequence.
- Templates (`aokf_read`) — `template-plan` for the slice list when it
  deserves a written plan.

Output: an ordered slice list, each with its own done-check. Then hand
the first slice to `/build`.
