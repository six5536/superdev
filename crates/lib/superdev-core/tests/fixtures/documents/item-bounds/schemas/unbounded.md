---
type: Schema
id: schema-unbounded
title: Unbounded Schema
description: A schema whose item-bound declarations the validator cannot read — one specimen per fault class.
---

# Unbounded Schema

An `item-prohibited-pattern` on a prose section, and an
`item-only-pattern` that does not compile.

````yaml
description: A mis-declared other.

frontmatter:
  type:
    const: Other

sections:
  - heading: "Body"
    level: 2
    content: prose
    item-prohibited-pattern: '\bMUST\b'
  - heading: "Items"
    level: 2
    content: bullet-list
    item-only-pattern: '(unclosed'

example: |
  ---
  type: Other
  ---

  # An other

  ## Body

  Prose.

  ## Items

  - an item.
````
