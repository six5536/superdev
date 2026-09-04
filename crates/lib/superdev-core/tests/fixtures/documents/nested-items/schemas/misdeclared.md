---
type: Schema
id: schema-misdeclared
title: Misdeclared Schema
description: A schema whose nested declarations the validator cannot read — one specimen per fault class.
---

# Misdeclared Schema

A `nested` on a prose section, a nested `item-key` with two captures, and
`item-key-optional` with no `item-key`.

````yaml
description: A mis-declared other.

frontmatter:
  type:
    const: Other

sections:
  - heading: "Body"
    level: 2
    content: prose
    nested:
      item-key: '^`(N_[a-z-]+)`'
  - heading: "Items"
    level: 2
    content: bullet-list
    nested:
      item-key: '^`(N_)([a-z-]+)`'
  - heading: "Either"
    level: 2
    content: bullet-list
    item-key-optional: true

example: |
  ---
  type: Other
  ---

  # An other

  ## Body

  Prose.

  ## Items

  - an item.

  ## Either

  - an item.
````
