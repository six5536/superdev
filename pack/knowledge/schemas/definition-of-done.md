---
type: Schema
id: schema-definition-of-done
title: Definition of Done Schema
description: The gates a change must satisfy before it merges, in knowledge/definition-of-done.md.
---

# Definition of Done Schema

Structural rules for `knowledge/definition-of-done.md`, the canonical knowledge's
Convention concept for the merge gate. The document has no headings — a lead
line and the gates — so it declares a preamble and no sections.

````yaml
description: >
  What a change must satisfy before it merges, as a list of gates each of
  which is checkable by someone who did not make the change.
line-limit: 800

frontmatter:
  type:
    const: DefinitionOfDone
  id:
    const: definition-of-done
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: bullet-list
  description: >
    A lead line — "A change is done when:" — then one bullet per gate: the
    checks that must pass (format, lint, tests, types); coverage or review
    requirements; documentation updated wherever behaviour changed; new
    behaviour carrying tests, and bug fixes carrying a regression test that
    fails on the unfixed code.

example: |
  ---
  type: Convention
  id: definition-of-done
  title: Definition of Done
  description: What a change must satisfy before it merges.
  status: stable
  ---

  A change is done when:

  - `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` pass.
  - One reviewer has approved it, and every review thread is resolved or
    answered.
  - Every doc that describes changed behaviour has been updated in the same
    PR, including the knowledge concepts.
  - New behaviour carries tests, and a bug fix carries a regression test that
    fails on the unfixed code — demonstrated by running it before the fix.
````
