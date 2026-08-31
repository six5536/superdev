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

sections:
  - heading: "Body"
    level: 2
    required: true
    content: essay

example: |
  ---
  type: Thing
  ---

  # A thing

  ## Body

  Anything.
````
