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
    pattern: '(unclosed'

example: |
  ---
  type: Thing
  ---

  # A thing
````
