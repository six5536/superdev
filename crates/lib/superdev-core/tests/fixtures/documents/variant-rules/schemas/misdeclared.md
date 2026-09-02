---
type: Schema
id: schema-misdeclared
title: Misdeclared Schema
description: A schema whose variant declarations the validator cannot read — one specimen per fault class.
---

# Misdeclared Schema

A tag naming a value the enum does not carry, a keyed example missing a
value, and an example keyed `a` whose own `kind` says `b`.

````yaml
description: A mis-declared other.

variant-key: kind

frontmatter:
  type:
    required: true
    const: Other
  kind:
    required: true
    enum: [a, b]

sections:
  - heading: "Body"
    level: 2
    required: true
    content: prose
  - heading: "Only C"
    level: 2
    variants: [c]
    content: prose

example:
  a: |
    ---
    type: Other
    kind: b
    ---

    # An other

    ## Body

    Prose.
````
