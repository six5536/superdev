---
type: Schema
id: schema-contract-library
title: Library Contract Schema
description: One published library — what ships, its exported API in the host language, its errors and the stability promise, in knowledge/contracts/public/.
---

# Library Contract Schema

Structural rules for one public library contract, filed at
`knowledge/contracts/public/contract-{nnn}-library-{slug}.md`. The API is written in the language the compiler
enforces, so the contract and the code cannot drift apart silently.

````yaml
description: >
  One library published for others to depend on — what ships and where, the
  exported API in its own language, the errors callers handle, and what is
  promised stable.
line-limit: 400

frontmatter:
  type:
    const: LibraryContract
  id:
    pattern: '^contract-\d{3}-library-[a-z0-9-]+$'
    description: >
      contract-{nnn}-library-{slug}, the slug naming which published library. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Package"
    level: 1
    required: true
    content: prose
    description: >
      The published name, the registry it goes to, the runtimes or toolchain
      versions supported, and any feature flags that change the surface.
  - heading: "Public API"
    level: 1
    required: true
    content: code
    description: >
      The exported surface in the host language — types, functions, traits or
      interfaces, as a caller sees them. One fenced block per module, tagged
      with its language. Anything absent here is private.
  - heading: "Errors"
    level: 1
    content: prose
    description: >
      The error type callers match on, what each variant means, and what the
      library panics or throws on rather than returning. Omit where the API
      block above says it in full.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      The versioning scheme, what counts as a breaking change to this surface,
      and how long a deprecated item stays before removal.

example: |
  ---
  type: LibraryContract
  id: contract-001-library-widget-core
  title: Library Contract
  description: widget-core on crates.io — the parser and its error type, semver from 1.0.
  status: stable
  ---

  # Package

  Published as `widget-core` on crates.io. Rust 1.85 or later, edition 2024, no
  default features. The `serde` feature adds `Serialize` and `Deserialize` to
  every public type and changes nothing else.

  # Public API

  ```rust
  pub struct Widget {
      pub id: String,
      pub name: String,
  }

  pub enum ParseError {
      Empty,
      BadField { field: String },
  }

  pub fn parse(input: &str) -> Result<Widget, ParseError>;
  ```

  # Errors

  `parse` returns `ParseError` and never panics on caller input. `Empty` means
  the input held no widget; `BadField` names the first field that failed. A
  panic from this crate is a bug in the crate, not a caller error.

  # Stability

  Semver from 1.0. Adding a variant to `ParseError` is breaking, so the enum is
  `#[non_exhaustive]`; adding a field to `Widget` is not. A deprecated item
  carries `#[deprecated]` for one minor release before removal in the next
  major.
````
