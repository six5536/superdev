---
type: Schema
id: schema-thing
title: Thing Schema
description: A governed thing whose Behaviour and Stability items carry a `P_` key, unique across the document.
---

# Thing Schema

Structural rules for a thing: every item under Behaviour and Stability
opens with a `P_` key, and a key is used once per thing (ADR-047).

````yaml
description: A governed thing.

frontmatter:
  type:
    const: Thing

sections:
  - heading: "Behaviour"
    level: 2
    required: true
    content: bullet-list
    item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
  - heading: "Stability"
    level: 2
    required: true
    content: bullet-list
    item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'

example: |
  ---
  type: Thing
  ---

  # A thing

  ## Behaviour

  - `P_starts` [event] WHEN asked, the thing SHALL start.

  ## Stability

  - `P_stable` [ubiquitous] The thing SHALL keep its name.
````
