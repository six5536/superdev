---
type: Convention
id: coding-standards
title: Coding Standards
description: Prose rules, Rust and TypeScript conventions, and the code-is-canonical principle.
status: stable
sources:
  - id: prose
    resource: /.agents/professionalism.md
    title: Professional language rules
  - id: coding
    resource: /.agents/coding.md
    title: Coding behaviour rules
  - id: checks
    resource: /.github/workflows/checks.yml
    title: The CI gate enforcing these
---

# Approach

The behavioural rules are in [coding.md](/.agents/coding.md): think before
coding, simplicity first, surgical changes only, and verifiable success
criteria defined before executing.[^coding]

# Prose

Be concise without losing information; use plain language. British English
spelling (`behaviour`, `normalise`). The full rules are in
[professionalism.md](/.agents/professionalism.md); the core: no jargon, no
filler, no drama, no hedging, and negation only where it carries
meaning.[^prose]

# Rust

- `rustfmt` formatting, checked in CI (`cargo fmt --all -- --check`).
- Clippy clean at `-D warnings`, all targets.
- Public items in `superdev-core` need doc comments (`#![warn(missing_docs)]`);
  rustdoc builds clean under `RUSTDOCFLAGS=-D warnings`; rustdoc examples run
  as doctests.[^checks]

Module rules:

- `mod.rs` contains only `mod` declarations and `pub use` re-exports; all
  code lives in named files.
- Declare submodules privately (`mod foo;`) and expose their API via
  `pub use foo::Item;`; flatten re-exports at `lib.rs` so callers write
  `crate::Item`.
- Default to private; widen visibility via `pub(crate)` → `pub(super)` →
  `pub` only as needed.
- Group imports `std` → external → `crate`/`super`/`self`, collapse with
  nested paths, and prefer `use crate::...` over `super::super::...`.
- Import types and traits directly; import the parent module for free
  functions (`module::func()`); no glob imports except preludes, enum
  variants in `match`, and tests.

Unsafe code:

- Every `unsafe` block lives in a dedicated `*_unsafe.rs` module behind safe
  public functions.
- No unsafe code without user confirmation, and what is confirmed must be
  clearly documented.

These rules bind new code and the shipped project templates; pre-existing
files are brought into line when touched, not in bulk.

# TypeScript / JavaScript

- Only use `index.ts` when necessary; otherwise name files descriptively.

# The code is the canonical reference

README, CLI `--help`, and this knowledge all describe actual behaviour.
When a doc disagrees with the code, fix the doc — unless the code is wrong, in
which case fix the code and say so.

[^coding]: Coding behaviour rules
[^prose]: Professional language rules
[^checks]: The CI gate enforcing these
