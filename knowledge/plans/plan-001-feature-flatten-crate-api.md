---
type: FeaturePlan
id: plan-001-feature-flatten-crate-api
title: Flatten the superdev-core API
description: Apply the module flatten rule to superdev-core — private submodules, pub use re-exports at lib.rs, callers writing crate::Item.
status: draft
links:
  - rel: relates-to
    to: coding-standards
    note: Implements the module rules recorded there.
---

# Feature plan: flatten the superdev-core API

## Goal

Bring `superdev-core` to the module rules in
[coding-standards](/knowledge/coding-standards.md): submodules declared
privately, their API re-exported with `pub use`, flattened at `lib.rs` so
callers write `superdev_core::Item` instead of
`superdev_core::manifest::Manifest`.

## Scope

- `lib.rs`: change `pub mod` to `mod` per module and add the flattened
  re-export block. Keep `aokf` and `components` as public namespaces if
  flattening them would collide (`aokf::validate::Report` vs a future
  engine `Report`) — resolve collisions by renaming at the re-export, not
  by keeping deep paths.
- Rewrite every caller: the `superdev` binary, the integration tests, and
  the doctests.
- The crate is pre-1.0 and the two crates release in lockstep, so the
  public-API break costs nothing externally.

## Slices

### Slice 1: inventory the surface

Inventory the current cross-crate surface (`grep superdev_core::` in
   the binary and tests) to get the exact re-export list.

### Slice 2: flatten module by module

Flatten one module at a time, keeping the suite green per step:
   `manifest`, `lock`, `capability`, `registry`, `component`, `action`,
   `pipeline`, `engine`, `orphan`, `report`, `runner`, `error`,
   `templates`, then decide `aokf` and `components`.

### Slice 3: doc pass

 `#![warn(missing_docs)]` stays satisfied; rustdoc links
   updated.

## Done when

`lib.rs` carries the whole public surface as re-exports, no caller writes
a two-segment `superdev_core::x::Y` path except into deliberately-kept
namespaces, and every gate (tests, clippy, rustdoc, check:aokf,
check:blueprint) passes.
