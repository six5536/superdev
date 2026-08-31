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
line-limit: 60

frontmatter:
  type:
    const: Thing

sections-ordered: true
sections:
  - heading: "First"
    level: 2
    required: true
    content: prose
  - heading: "Second"
    level: 2
    required: true
    content: prose
  - heading: "Rows"
    level: 2
    content: table
    columns: [ID, Name]

sections-prohibited:
  - "Overview"

example: |
  ---
  type: Thing
  ---

  # A thing

  ## First

  Prose.

  ## Second

  More prose.
````
