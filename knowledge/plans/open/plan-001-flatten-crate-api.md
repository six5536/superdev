---
type: Plan
id: plan-001-flatten-crate-api
title: Flatten the superdev-core API
description: Apply the module flatten rule to superdev-core — private submodules, pub use re-exports at lib.rs, callers writing crate::Item.
lifecycle: open
links:
  - rel: relates-to
    to: coding-standards
    note: Implements the module rules recorded there.
---

# Plan: Flatten the superdev-core API

## Goal

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

## Contract changes

- none.

## Work blocks

### Block 1: inventory the surface

- [ ] Done — ticked by build at its commit.
- Depends-on: none.
- Change: inventory the current cross-crate surface (`grep
  superdev_core::` in the binary and tests) to get the exact re-export
  list.
- Done-check: the re-export list names every `superdev_core::` path the
  binary and the tests use.
- Cases:
  - observation: `grep superdev_core::` over the binary and the tests
    finds no path the list omits — no criterion.

### Block 2: flatten module by module

- [ ] Done — ticked by build at its commit.
- Depends-on: 1.
- Change: flatten one module at a time, keeping the suite green per step:
  `manifest`, `lock`, `capability`, `registry`, `component`, `action`,
  `pipeline`, `engine`, `orphan`, `report`, `runner`, `error`,
  `templates`, then decide `aokf` and `components`.
- Done-check: `lib.rs` carries the whole public surface as re-exports,
  and no caller writes a two-segment `superdev_core::x::Y` path except
  into deliberately-kept namespaces.
- Cases:
  - unit: the test suite passes after each module's flatten — no
    criterion.
  - observation: `grep superdev_core::` over the binary and the tests
    finds no two-segment path outside the kept namespaces — no
    criterion.

### Block 3: doc pass

- [ ] Done — ticked by build at its commit.
- Depends-on: 2.
- Change: `#![warn(missing_docs)]` stays satisfied; rustdoc links
  updated.
- Done-check: every gate (tests, clippy, rustdoc, check:aokf,
  check:blueprint) passes.
- Cases:
  - e2e: rustdoc builds with no missing-docs warning and no broken
    link — no criterion.

<!-- sokf:links -->
[sokf:coding-standards]: /knowledge/coding-standards.md
