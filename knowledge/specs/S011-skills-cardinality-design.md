---
type: Spec
id: spec-skills-cardinality
title: Skills Capability Cardinality
description: A capability declares whether it holds one provider or a set; skills becomes the first many-provider slot via [[skills]] entries, with the old single-table shape still parsing.
status: stable
links:
  - rel: relates-to
    to: spec-cli-core-blueprint-engine
  - rel: relates-to
    to: spec-skill-pack
---

# Summary

Every capability holds exactly one provider — the manifest has one
table per slot, and the [glossary](../glossary.md) records that every
slot currently has exactly one entry. That exclusivity is right where
providers compete (a repo wants one code indexer, one knowledge
format), but wrong for skills: skill packs are additive, not rival. A
repo owner who wants a second pack alongside superdev's has no way to
say so in `config.toml` — the slot model, introduced by the
[CLI core & blueprint engine spec](S001-cli-core-blueprint-engine-design.md),
forces a choice the domain doesn't have. Today the slot holds only
superdev's own pack, from the [skill pack spec](S003-skill-pack-design.md).

# Solution

Cardinality becomes a declared property of a capability: *single* (one
provider table, exclusive — every slot today) or *many* (a set of
provider entries). Skills is the first many slot. A many slot appears
in the manifest as an array of tables — one `[[skills]]` entry per
pack, each carrying the same fields as today's single table — and the
registry may hold several available entries for it, still with exactly
one default that `init` enables.

# Behaviour

1. A manifest may carry several `[[skills]]` entries. Each entry is
   planned as its own component — its pack files, version pin and
   `custom` releases — independently of its siblings.
2. Two entries naming the same provider fail manifest load with a
   guided error naming the duplicated provider.
3. The legacy single-table `[skills]` shape parses as a one-entry set:
   existing manifests load untouched, with no warning and no
   migration. When superdev itself rewrites `config.toml` (`update`),
   a many slot with one entry serialises as the single table; the
   array form appears only from two entries up, so no existing
   manifest changes shape.
4. An entry naming a provider the registry has no skills entry for
   fails manifest load with a guided error listing the valid
   providers for the slot.
5. A single-cardinality capability written in array form fails
   manifest load with a guided error saying the capability holds one
   provider.
6. Two enabled components claiming the same path — two packs shipping
   the same skill name, or a pack colliding with a knowledge-carried
   skill — fail the plan with a guided error naming the path and both
   owners. The resolution is releasing the skill from one side via
   `custom`; there is no silent winner.
7. `custom` stays per entry: a name is released from the pack whose
   entry lists it, and a name that entry's pack does not ship keeps
   reporting as having no effect.
8. `update skills` resets every entry's version to its provider's
   registry default; the per-provider registry-default refusal is
   unchanged and applies to each entry separately.
9. `--no-skills` and removing all entries disable the whole slot.
   Removing one entry sweeps only that pack's files as orphans;
   the other entries' files are untouched.
10. The lock records one component record per (capability, provider),
    and `status` reports each enabled pack separately, exiting 1 while
    any of them has pending work.
11. Nothing observable changes for a default repo: the registry still
    carries only `superdev-skills` for the slot, and `init` still
    enables exactly that.

# Design decisions

- Cardinality is declared in the blueprint (on the capability), not
  inferred from the manifest — a user cannot turn an exclusive slot
  plural by writing an array, which keeps mutual exclusivity a
  property superdev vouches for.
- Within a many slot the entries form a set keyed by provider, not a
  list: the same pack twice is meaningless and would double-claim the
  same files, so it is refused rather than deduplicated.
- Array-of-tables was chosen over per-provider subtables because each
  entry keeps exactly the fields of today's single table — the single
  shape is the degenerate case, which is what lets old manifests parse
  with no migration. Decided with the user.
- Serialisation keeps the single-table shape while one entry exists
  (decided with the user): flipping every managed repo's manifest to
  the array form on its next `update` would be churn that buys
  nothing; the degenerate case keeps the degenerate shape.
- Path collisions between enabled components are refused at plan
  time, never resolved by precedence (decided with the user): a
  silent winner would make pack order load-bearing, and claims are
  where overlap is already visible. The rule is cross-capability —
  the knowledge capability writes into the same skills directory —
  which restores the very guarantee S009 cited when it removed the
  colliding workflows capability.
- The change is implemented now, without waiting for a second pack
  (decided with the user): the manifest surface and lock granularity
  are the contract later packs build on, and landing them while the
  slot has one entry means no migration when the first real pack
  arrives.
- The registry keeps exactly one default per slot even for a many
  slot: `init`'s baseline stays opinionated, and further packs are
  deliberate manifest edits.
- No second pack ships with this change. Re-registering superpowers —
  de-registered by the
  [knowledge-carried skills spec](S009-knowledge-carried-skills-design.md)
  — or adding any other pack is a product decision for its own spec;
  this one is the model and the manifest surface.
- The lock mirrors the manifest's per-entry granularity so the orphan
  pass works per pack: dropping one entry derives that pack's sweep
  and nothing else's.

# Testing

Seams, all existing, as confirmed with the user: manifest unit tests
(multi-entry parse, duplicate-provider refusal, legacy single-table
acceptance, unknown-provider and array-on-single guided errors, array
serialisation); registry unit tests (several entries per many slot,
exactly one default); the pipeline test (one planned component per
enabled entry, and the guided refusal when two enabled components
claim the same path); and the manage journeys (a two-entry manifest
plans both packs; removing one entry sweeps only its files). With one
registered pack, the plans-both journey cannot yet run end-to-end —
as shipped the journey exercises the guided refusals (unknown pack,
duplicate, array-on-exclusive) and the one-entry-array equivalence,
and the plans-both case is covered at the engine seam; the full
journey lands with the first registered second pack. Good tests
assert manifest text in and reported plans out, not parser internals.
Prior art: the existing manifest parse tests, the registry
one-default-per-capability test, and the S009 provider-switch sweep
tests' descendants.

# Acceptance criteria

1. The behaviour described below holds, as proved by the automated cases in
   the test plan. This spec shipped before the contract asked for acceptance
   criteria, and none were written at the time; the tests are the record of
   what was actually accepted.

# Edge cases & errors

- Not recorded separately when this spec was written. What the code does at
  the edges is in the tests named in the test plan, which is the only
  contemporaneous record.

# Out of scope

- Shipping or registering a second skill pack (including superpowers).
- Making knowledge, code-index, frontend or bash-output-filter
  many-cardinality — nothing competes additively in those slots today.
- New `init` flags for choosing extra packs; extra entries are
  manifest edits.

# Open questions

How `status` labels a many slot's per-pack report lines, and the
serialised order of entries, are implementation judgement; the
per-entry planning and sweeping above are not.

# Test plan: skills capability cardinality

## Scope

- Manifest parsing of the many slot, and the collision refusal.
- Out: everything the sections above place out of scope.

## Risks driving this plan

1. Recorded after the fact. This plan was written when the spec was
   conformed to its contract, not when the feature was built, so it names
   the risks the tests actually cover rather than the ones weighed at the
   time.

## Test cases

### Automated

| # | Case | Type | Inputs / setup | Expected result |
|---|------|------|----------------|-----------------|
| 1 | A multi-entry slot parses | unit | the array form | one config per entry, in order |
| 2 | A duplicated provider is refused | unit | the same provider twice | the guided error |
| 3 | The array form on a single slot | unit | an exclusive capability | the guided error naming the single-table form |

### Manual verification

1. None recorded. The feature shipped under the automated cases above; no
   manual step was written down at the time, and inventing one now would
   claim a check nobody made.

## Exit criteria

- The automated cases above pass.
- `superdev validate` reports no error for this document.
