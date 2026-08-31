---
type: Schema
id: schema-architectural-rules
title: Architectural Rules Schema
description: The invariants behind the architecture, each with its reason, in knowledge/architectural-rules.md.
---

# Architectural Rules Schema

Structural rules for `knowledge/architectural-rules.md`, the canonical knowledge's
Convention concept for the invariants behind the architecture. The document
carries no headings at all — it is a lead line and a list — so it declares a
preamble and no sections.

````yaml
description: >
  The invariants behind the architecture, each stated so code review can
  enforce it, each with the reason it holds.
line-limit: 800

frontmatter:
  type:
    const: ArchitecturalRules
  id:
    const: architectural-rules
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: bullet-list
  description: >
    A lead line introducing the invariants, then one bullet per rule: the rule
    stated as an invariant that code review can enforce, followed by the reason
    it holds — what breaks or gets expensive when it is violated. A rule
    without its reason is a rule nobody can weigh against a deadline.

example: |
  ---
  type: Convention
  id: architectural-rules
  title: Architectural Rules
  description: The core never knows its caller, and the lockfile is never hand-derived.
  status: stable
  ---

  The invariants behind the architecture:

  - `superdev-core` never depends on `superdev-cli`, directly or through a
    shared crate. The core is consumed as a library by the tests and will be
    consumed by other front ends; a dependency back onto the CLI would drag
    argument parsing and exit codes into every consumer.
  - Every pin in `superdev.lock` is written by a resolution that fetched and
    verified it. A pin derived from the manifest alone records a claim rather
    than a fact, and the lockfile stops being evidence of anything.
````
