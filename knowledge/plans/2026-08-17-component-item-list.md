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

# Tasks

1. Define `ManagedItem` — path + content source + ownership
   (Owned/Scaffold), or pin + value — and one driver deriving both
   `plan` and `owned` from a component's item list. Run design-it-twice
   on the item shape; the risk is a shape too narrow for the next real
   component.
2. Port the static components (`aokf`, `skillpack`; `plugin` and
   `codegraph` for their non-pin parts) to declarative lists.
   `mattskills` keeps its hand-written pair — its state is genuinely
   dynamic.
3. Absorb the exact-whole-line predicate into one home the driver and
   the engine's `ensure_line` share.
4. Shrink `owned_matches_what_apply_locks` to the dynamic components
   only; static components' consistency is true by construction.
   Per-component tests assert the item list, not repo simulations.

# Done

A new static component is data plus one list test. The consistency test
covers only `mattskills`. `npm test` and `npm run check:blueprint` pass
with byte-identical plans on the fixture repos.

# Sequencing

After [one checksum-pin planner](2026-08-17-checksum-pin-planner.md)
(the pin blocks it would otherwise duplicate are gone) and preferably
after the [verb pipeline](2026-08-17-verb-pipeline-in-core.md) (the
`manage.rs:391` predicate copy has moved to core by then). Delete this
file in the commit that completes the work.
