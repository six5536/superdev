---
type: Plan
id: plan-checksum-pin
title: One Checksum-Pin Planner
description: One planned_pin helper replaces three copied pin blocks; every registry pin carries its provenance.
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

# Design (settled by grilling, 2026-08-17)

[Provenance](../glossary.md) is the domain term: why a pinned version is
locked to the registry default. Every version the registry pins carries
one — a pin without provenance is unrepresentable.

# Tasks

1. Registry: fuse version and provenance —
   `version: Option<Pinned>` with
   `Pinned { version: &'static str, provenance: Provenance }`,
   `Provenance::{Checksum, Embedded}`. Checksum for both `workflows`
   entries and `code-index`; Embedded for `skills`. Delete `is_behind`
   and its `behind_pins` else-branch: unreachable today (every
   versioned entry is locked) and structurally dead once fused.
2. New `components/pin.rs`, using `components/mise.rs` underneath:
   `require_registry_default(ctx, capability, provider) ->
   Result<&'static str>` (the refusal; message derives from the
   provenance variant) and
   `planned_pin(ctx, capability, provider, tool, value_toml) ->
   Result<Option<Action>>` (calls it, then read → round-trip
   normalisation → compare → `SetMisePin`).
3. Replace the three pin blocks (`plugin.rs:82–120`,
   `mattskills.rs:73–98`, `codegraph.rs:43–73`) with `planned_pin`
   calls; `skillpack.rs` calls `require_registry_default`.
4. Unified refusal message, all four sites: `{capability} version must
   match the registry default {default} — the {pinned checksum |
   embedded content} is the provenance — run `superdev update
   {capability}``. This rewords skillpack's prefix and adds the update
   hint everywhere; update the affected assertions.
5. Replace `BINARY_PINNED` and its users (`plannable`, `parse_target`,
   `behind_pins`, `checksum_pin_mismatch`) with a registry query
   against the manifest's selected provider's entry.
6. Collapse the four per-component "foreign version is rejected" tests
   and `parses_update_targets` / `plannable_resets_every_checksum_pin`
   into one helper suite plus thin per-component wiring checks.

# Done

The refusal message template exists in one function; no component
carries its own copy. `BINARY_PINNED` and `is_behind` are gone. `npm test` and `npm run check:blueprint`
pass. Only intended behaviour change: the reworded refusal messages
(task 4); plans byte-identical on the fixture repos.

# Sequencing

Independent; land first in the verb track. The engine track
([transaction primitive](2026-08-17-engine-transaction-primitive.md) →
[managed entry](2026-08-17-managed-entry-interface.md)) can run in
parallel. Delete this file in the commit that completes the work.
