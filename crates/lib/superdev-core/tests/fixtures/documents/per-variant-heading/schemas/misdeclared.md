---
type: Schema
id: schema-misdeclared
title: Misdeclared Schema
description: A schema declaring one heading twice with overlapping variants, and another twice with one rule untagged — neither pair binds.
---

# Misdeclared Schema

"Criteria" is declared by two rules whose sets share `framed`; "Notes"
by a tagged rule and an untagged one. Both pairs are reported on this
file and bind nothing, so the `other.md` document passes.

````yaml
description: A mis-declared other.

variant-key: lifecycle

frontmatter:
  type:
    required: true
    const: Other
  lifecycle:
    required: true
    enum: [unframed, framed]

sections:
  - heading: "Criteria"
    level: 2
    required: true
    content: numbered-list
    variants: [unframed, framed]
  - heading: "Criteria"
    level: 2
    required: true
    content: numbered-list
    item-key: '^`(AC_[a-z]+)`'
    variants: [framed]
  - heading: "Notes"
    level: 2
    required: true
    content: prose
    variants: [framed]
  - heading: "Notes"
    level: 2
    required: true
    content: bullet-list

example:
  unframed: |
    ---
    type: Other
    lifecycle: unframed
    ---

    # An other
  framed: |
    ---
    type: Other
    lifecycle: framed
    ---

    # An other
````
