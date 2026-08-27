---
type: Schema
id: schema-technology-stack
title: Technology Stack Schema
description: Languages, dependencies with their reasons, and the pinned toolchain, in knowledge/technology-stack.md.
---

# Technology Stack Schema

Structural rules for `knowledge/technology-stack.md`, the bundle's Reference
concept for what the project is built with. The document has no headings, so
it declares a preamble and no sections. Whether a dependency may be added at
all belongs to `dependency-policy`, not here.

````yaml
target-files: "knowledge/technology-stack.md"
description: >
  The languages and runtime shape, where toolchain versions are pinned, and
  the dependency set with the reason behind each non-obvious choice.
line-limit: 800

frontmatter:
  type:
    const: Reference
  id:
    const: technology-stack
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: bullet-list
  description: >
    Languages and the runtime shape in one or two sentences, and where
    toolchain versions are pinned. Then a lead line noting that additions
    follow the dependency policy, and one bullet per dependency or tool: what
    it is used for, and why it was chosen when the choice was not obvious.

example: |
  ---
  type: Reference
  id: technology-stack
  title: Technology Stack
  description: Rust 2021 with a pinned toolchain, and a deliberately small dependency set.
  status: stable
  ---

  Rust 2021, one Cargo workspace, no async runtime — the tool is a
  short-lived process and the work is IO-bound in bursts, not concurrently.
  The toolchain version is pinned in `rust-toolchain.toml` and CI uses it
  unmodified.

  The dependency set (additions follow the dependency policy):

  - `serde` / `serde_yaml` — manifest and lockfile parsing.
  - `gix` — git operations, chosen over shelling out to `git` so that a
    transport refusal is a typed error rather than a parsed stderr string.
  - `regex` — schema pattern matching. Its dialect has no lookaround, which
    is a constraint the schema format is written around deliberately.
````
