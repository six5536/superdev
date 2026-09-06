---
type: Plan
id: plan-001-flatten-crate-api
title: Flatten the superdev-core API
description: Apply the module flatten rule to superdev-core — private submodules, pub use re-exports at lib.rs, callers writing crate::Item.
lifecycle: open
links:
- rel: implements
  to: issue-075-historical-flatten-crate-api
- rel: relates-to
  to: coding-standards
  note: Implements the module rules recorded there.
phase: scope
branch: work/075-historical-flatten-crate-api
---
# Plan: Flatten the superdev-core API

Primary issue: [issue-075-historical-flatten-crate-api][sokf:issue-075-historical-flatten-crate-api]

## Goal and boundaries

Bring `superdev-core` to the module rules in
[coding-standards][sokf:coding-standards]: submodules declared
privately, their API re-exported with `pub use`, flattened at `lib.rs` so
callers write `superdev_core::Item` instead of
`superdev_core::manifest::Manifest`.

The scope:

- `lib.rs`: change `pub mod` to `mod` per module and add the flattened
  re-export block. Keep `aokf` and `components` as public namespaces if
  flattening them would collide (`aokf::validate::Report` vs a future
  engine `Report`) — resolve collisions by renaming at the re-export, not
  by keeping deep paths.
- Rewrite every caller: the `superdev` binary, the integration tests, and
  the doctests.
- The crate is pre-1.0 and the two crates release in lockstep, so the
  public-API break costs nothing externally.

The plan is done when `lib.rs` carries the whole public surface as
re-exports, no caller writes a two-segment `superdev_core::x::Y` path
except into deliberately-kept namespaces, and every gate (tests, clippy,
rustdoc, check:aokf, check:blueprint) passes.

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

### Block 1: inventory the surface

- [ ] Done — ticked by build at its commit.
- Dependencies: none.
- Areas: unknown (legacy record; no affected area inferred).
- Outcome: inventory the current cross-crate surface (`grep
  superdev_core::` in the binary and tests) to get the exact re-export
  list.
- Verification: the re-export list names every `superdev_core::` path the
  binary and the tests use.
- Tests:
  - observation: `grep superdev_core::` over the binary and the tests
    finds no path the list omits — no criterion.
- Structural evidence: none recorded; this historical record makes no executable evidence claim.
- Documentation: none recorded; this historical record makes no documentation claim.

### Block 2: flatten module by module

- [ ] Done — ticked by build at its commit.
- Dependencies: unknown (legacy record; no dependency inferred).
- Areas: unknown (legacy record; no affected area inferred).
- Outcome: flatten one module at a time, keeping the suite green per step:
  `manifest`, `lock`, `capability`, `registry`, `component`, `action`,
  `pipeline`, `engine`, `orphan`, `report`, `runner`, `error`,
  `templates`, then decide `aokf` and `components`.
- Verification: `lib.rs` carries the whole public surface as re-exports,
  and no caller writes a two-segment `superdev_core::x::Y` path except
  into deliberately-kept namespaces.
- Tests:
  - unit: the test suite passes after each module's flatten — no
    criterion.
  - observation: `grep superdev_core::` over the binary and the tests
    finds no two-segment path outside the kept namespaces — no
    criterion.
- Structural evidence: none recorded; this historical record makes no executable evidence claim.
- Documentation: none recorded; this historical record makes no documentation claim.

### Block 3: doc pass

- [ ] Done — ticked by build at its commit.
- Dependencies: unknown (legacy record; no dependency inferred).
- Areas: unknown (legacy record; no affected area inferred).
- Outcome: `#![warn(missing_docs)]` stays satisfied; rustdoc links
  updated.
- Verification: every gate (tests, clippy, rustdoc, check:aokf,
  check:blueprint) passes.
- Tests:
  - e2e: rustdoc builds with no missing-docs warning and no broken
    link — no criterion.

- Structural evidence: none recorded; this historical record makes no executable evidence claim.

- Documentation: none recorded; this historical record makes no documentation claim.

## Build state

Current block is unset; attempts: 0; final corrections: 0; blocker: human re-scope into executable evidence is required.

## Implementation decisions

none.

## Follow-up issues

Unresolved discovery: human re-scope must replace legacy notes with approved executable evidence before BUILD.

## Completion evidence

Historical open plan migrated mechanically; scope approval and BUILD evidence are pending.

<!-- sokf:links -->
[sokf:coding-standards]: /knowledge/coding-standards.md
[sokf:issue-075-historical-flatten-crate-api]: /knowledge/issues/open/issue-075-historical-flatten-crate-api.md
