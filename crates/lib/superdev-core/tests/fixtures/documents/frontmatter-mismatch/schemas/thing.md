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
    const: Thing
  id:
    pattern: '^thing-\d{3}-[a-z0-9-]+$'
  status:
    enum: [draft, stable]
  category:
    const: reference
  title:
    description: Guidance only, binding nothing.

example: |
  x
````
