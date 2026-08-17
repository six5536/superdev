---
type: Plan
id: plan-item-list
title: Component Item List
description: Static components declare a managed-item list; one driver derives plan and owned, making their consistency structural.
status: draft
---

From the 2026-08-17 architecture review, candidate 6 (Worth exploring).
Last of the six plans — it reads best once the other refactors have
settled the component surface.

# Friction

The `Component` interface requires two methods that must agree: `plan`
emits actions whose applied lock keys must equal `owned`'s claims. The
doc comment admits the coupling (`component.rs:70`, "Derived from the
same constants plan writes from"); nothing structural enforces it, so
the 78-line `owned_matches_what_apply_locks` test (`components/
mod.rs:73`) does — with a special case for mattskills, whose claims are
lock-derived (`mod.rs:136–148`). The bodies repeat a compare-and-emit
shape (`aokf.rs:127–160`, `skillpack.rs:83–97`), and the exact-whole-
line predicate exists three times (`aokf.rs:174`, `manage.rs:391`,
`engine.rs:490`). Deletion test: mixed — each component's content is
irreducible, but the per-file diffing and plan/owned bookkeeping would
reappear in every future component.

# Design (settled by grilling, 2026-08-17)

`ManagedItem` covers the four declarative kinds — OwnedFile, Scaffold,
EnsureLine, JsonEntry — and the claim-shaped kinds embed the `Claim`
from the [managed-entry plan](2026-08-17-managed-entry-interface.md):
`owned()` is the items' claims collected, `plan()` is
read-compare-emit. EnsureLine items carry no claim (never locked).
Pins stay with the
checksum-pin planner's (landed 2026-08-17)
`planned_pin`; commands stay hand-written.

# Tasks

1. Define `ManagedItem` and one driver deriving both `plan` and
   `owned` from a component's item list (`items(ctx)` — the list is
   ctx-dependent: skillpack filters custom names).
2. Port `aokf` and `skillpack` to declarative lists — after Q1's kinds
   their `plan()` is fully derived, no hand-written remainder.
   `plugin`, `codegraph` and `mattskills` keep hand-written pairs:
   their remainders are commands or genuinely dynamic state.
3. Absorb the exact-whole-line predicate into one home the driver and
   the engine's `ensure_line` share.
4. Shrink `owned_matches_what_apply_locks` to the hand-written
   components (`plugin`, `codegraph`, `mattskills`); ported components'
   consistency is true by construction. Per-component tests assert the
   item list, not repo simulations.

# Done

A new static component is data plus one list test. The consistency test
covers only the hand-written components (`plugin`, `codegraph`,
`mattskills`). `npm test` and `npm run check:blueprint` pass
with byte-identical plans on the fixture repos.

# Sequencing

After the checksum-pin planner (landed 2026-08-17)
(the pin blocks it would otherwise duplicate are gone) and preferably
after the verb pipeline (landed 2026-08-17; the exact-whole-line
predicate copy now lives in core's pipeline.rs). Delete this
file in the commit that completes the work.
