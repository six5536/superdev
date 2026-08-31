---
type: Schema
id: schema-coding-standards
title: Coding Standards Schema
description: The behavioural rules, prose rules and per-language conventions CI enforces, in knowledge/coding-standards.md.
---

# Coding Standards Schema

Structural rules for `knowledge/coding-standards.md`, the canonical knowledge's
Convention concept for how code is written here. The per-language headings
sit between literal ones, so the rule that a literal beats a pattern is what
keeps `Canonical reference` from being swallowed by the catch-all.

````yaml
description: >
  The behavioural rules for making changes, the prose rules, one section per
  language covered, and what wins when docs and code disagree.
line-limit: 800

frontmatter:
  type:
    required: true
    const: CodingStandards
  id:
    required: true
    const: coding-standards
  title:
    required: true
  description:
    required: true
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Approach"
    level: 1
    required: true
    content: prose
    description: >
      The behavioural rules for making changes — e.g. simplicity first,
      surgical diffs, success criteria before code.
  - heading: "Prose"
    level: 1
    required: true
    content: prose
    description: >
      Language and spelling, comment rules, and the register for docs and
      commit messages.
  - heading-pattern: '^.+$'
    level: 1
    required: true
    repeatable: true
    content: prose
    description: >
      One heading per language. The formatting and lint gates CI runs, then
      only the conventions that differ from language defaults or that get done
      wrong without being written down. One line each.
  - heading: "Canonical reference"
    level: 1
    required: true
    content: prose
    description: >
      What wins when docs and code disagree, and what to do about the loser.

example: |
  ---
  type: CodingStandards
  id: coding-standards
  title: Coding Standards
  description: Rust and TypeScript, gated by fmt, clippy and eslint in CI.
  status: stable
  ---

  # Approach

  Simplest thing that works, then stop. Diffs stay surgical: a change that
  touches a file for reasons unrelated to its purpose belongs in its own
  commit. Success criteria are written before the code that satisfies them.

  # Prose

  British spelling in docs and comments. Comments say why, never what — a
  comment restating the line above it is deleted on sight. Commit messages
  are imperative and lowercase after the type prefix.

  # Rust

  `cargo fmt` and `cargo clippy -- -D warnings` gate every PR. Errors are
  typed enums, never stringly-typed. `unwrap` is allowed in tests and nowhere
  else.

  # TypeScript

  `eslint` and `prettier` gate every PR. `any` needs a comment naming what
  would have to change to remove it.

  # Canonical reference

  The code wins. When a doc disagrees with it, the doc is wrong and gets
  fixed in the same PR that found the disagreement.
````
