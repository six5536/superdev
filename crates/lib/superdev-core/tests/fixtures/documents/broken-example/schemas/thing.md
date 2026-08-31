---
type: Schema
id: schema-thing
title: Thing Schema
description: A governed thing, whose example breaks its own contract.
---

# Thing Schema

Structural rules for a thing. The example breaks the id pattern, omits a
required key, drops a required section, and starves a declared content
kind — one specimen per fault class the example check reads.

````yaml
description: A governed thing.

frontmatter:
  type:
    required: true
    const: Thing
  id:
    required: true
    pattern: '^thing-\d{3}-[a-z0-9-]+$'
  title:
    required: true
    description: The display name, required but otherwise unconstrained.

sections:
  - heading: "Context"
    level: 2
    required: true
    content: prose
  - heading: "Items"
    level: 2
    content: bullet-list

example: |
  ---
  type: Thing
  id: thing-1
  ---

  # A thing

  ## Items

  prose only
````
