---
type: Schema
id: schema-keyless
title: Keyless Schema
description: A schema whose item-key declarations the validator cannot read — one specimen per fault class.
---

# Keyless Schema

An `item-key` with no capture group, and one on a prose section.

````yaml
description: A mis-declared other.

frontmatter:
  type:
    const: Other

sections:
  - heading: "Items"
    level: 2
    content: bullet-list
    item-key: '^`P_[a-z-]+`'
  - heading: "Body"
    level: 2
    content: prose
    item-key: '^`(P_[a-z-]+)`'

example: |
  ---
  type: Other
  ---

  # An other

  ## Items

  - `P_one` an item.

  ## Body

  Prose.
````
