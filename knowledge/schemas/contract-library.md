---
type: Schema
id: schema-contract-library
title: Library Contract Schema
description: One published library — what ships, its exported API in the host language, its errors and the stability promise, a public contract.
---

# Library Contract Schema

Structural rules for one public library contract, filed at
`contract-{nnn}-library-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. The API is written in the language the compiler
enforces, so the contract and the code cannot drift apart silently.

<!-- sokf:include contract-style -->
**Contract style — a contract is a binding surface, not a
specification** (superdev ADR-029):

- Each normative statement MUST use an RFC 2119 modal verb, one
  requirement per sentence.
- An enumerable surface — commands, flags, keys, types, error cases,
  limits — MUST be defined in the kind's native structured form: a code
  block, table or list. Prose, doc comments included, describes and
  MUST NOT define.
- A contract MUST bind only what callers rely on; behaviour a contract
  does not list is the code's to decide.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

````yaml
description: >
  One library published for others to depend on — what ships and where, the
  exported API in its own language, the errors callers handle, and what is
  promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: LibraryContract
  id:
    required: true
    pattern: '^contract-\d{3}-library-[a-z0-9-]+$'
    description: >
      contract-{nnn}-library-{slug}, the slug naming which published library. The
      number is the next free one across every contract, public and
      internal together and every lifecycle folder — a duplicate is
      an error.
  title:
    required: true
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^Library contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Package"
    level: 2
    required: true
    content: prose
    description: >
      The published name, the registry it goes to, the runtimes or toolchain
      versions supported, and any feature flags that change the surface.
  - heading: "Public API"
    level: 2
    required: true
    content: code
    description: >
      The exported surface in the host language — types, functions, traits or
      interfaces, as a caller sees them, with their full signatures, so a
      caller compiles against this block alone. One fenced block per module,
      tagged with its language. Anything absent here is private. The project
      binds these blocks to the code — by generating one from the other, or by
      a test (ADR-036).
  - heading: "Errors"
    level: 2
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      The error type callers match on, what each variant means, and what the
      library panics or throws on rather than returning. Omit where the API
      block above says it in full.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      The versioning scheme, what counts as a breaking change to this surface,
      and how long a deprecated item stays before removal.

example: |
  ---
  type: LibraryContract
  id: contract-001-library-widget-core
  title: Library Contract
  description: widget-core on crates.io — the parser and its error type, semver from 1.0.
  lifecycle: active
  ---

  # Library contract: widget-core

  widget-core on crates.io: the parser and its error type, semver from
  1.0.

  ## Package

  Published as `widget-core` on crates.io. Rust 1.85 or later, edition 2024, no
  default features. The `serde` feature adds `Serialize` and `Deserialize` to
  every public type and changes nothing else.

  ## Public API

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

  ## Errors

  `parse` returns `ParseError` and MUST NOT panic on caller input. `Empty`
  means the input held no widget; `BadField` names the first field that
  failed. A panic from this crate is a bug in the crate, not a caller error.

  ## Stability

  Semver from 1.0. Adding a variant to `ParseError` is breaking, so the enum
  MUST be `#[non_exhaustive]`; adding a field to `Widget` is not. A deprecated
  item MUST carry `#[deprecated]` for one minor release before removal in the
  next major.
````
