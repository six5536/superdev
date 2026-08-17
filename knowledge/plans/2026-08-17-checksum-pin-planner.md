---
type: Plan
id: plan-checksum-pin
title: One Checksum-Pin Planner
description: One planned_pin helper replaces three copied pin blocks; the registry owns the binary-pinned flag.
status: draft
---

From the 2026-08-17 architecture review, candidate 3 (Strong). First plan
of the verb track: [verb pipeline](2026-08-17-verb-pipeline-in-core.md)
and the [runner seam](2026-08-17-runner-seam-for-verbs.md) build on it.

# Friction

The same ~25-line plan block — fetch registry default → refuse any other
version → read `.mise.toml` → `current_pin` → round-trip the desired value
through `set_pin`+`current_pin` to normalise layout → compare → emit
`SetMisePin` — is copied three times: `plugin.rs:82–120`,
`mattskills.rs:73–98`, `codegraph.rs:43–73`. `skillpack.rs:78` carries a
fourth variant of the refusal without the pin. Because the refusal is
buried in `plan()`, the binary re-derives which capabilities behave this
way as `BINARY_PINNED` (`manage.rs:35`) — the same fact in two places
with no compiler tying them. Deletion test: any copy deleted reappears
from its siblings.

# Tasks

1. Add a flag to `RegistryEntry` (working name `binary_pinned`; final
   name at execution) marking entries whose version must match the
   registry default. Set it on both `workflows` entries, `code-index`
   and `skills`. Expose a registry query for it.
2. Extract one helper — working shape
   `planned_pin(ctx, capability, tool, value) -> Result<Option<Action>>`
   — absorbing the refusal message, the `.mise.toml` read, the
   round-trip normalisation and the compare. Home: beside the existing
   mise editing seam (`components/mise.rs`) or `component.rs`; decide at
   execution.
3. Replace the three copies with calls; give `skillpack.rs` the shared
   refusal (it has no pin).
4. Replace `BINARY_PINNED` and its users (`plannable`, `parse_target`,
   `behind_pins` in `manage.rs`) with the registry query.
5. Collapse the four per-component "foreign version is rejected" tests
   and `parses_update_targets` / `plannable_resets_every_checksum_pin`
   into one helper suite plus thin per-component wiring checks.

# Done

`rg "must match the registry default" crates/lib` matches the helper
once. `BINARY_PINNED` is gone. `npm test` and `npm run check:blueprint`
pass; no behaviour change (same messages, same plans).

# Sequencing

Independent; land first in the verb track. The engine track
([transaction primitive](2026-08-17-engine-transaction-primitive.md) →
[managed entry](2026-08-17-managed-entry-interface.md)) can run in
parallel. Delete this file in the commit that completes the work.
