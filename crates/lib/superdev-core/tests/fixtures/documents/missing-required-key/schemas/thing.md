---
type: Schema
id: schema-thing
title: Thing Schema
description: A governed thing, for the fixture.
---

# Thing Schema

Structural rules for a thing.

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
  status:
    enum: [draft, stable]

example: |
  x
````
