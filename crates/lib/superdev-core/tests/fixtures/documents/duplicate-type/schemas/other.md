---
type: Schema
id: schema-other
title: Other Schema
description: A governed thing, for the fixture.
---

# Other Schema

Structural rules for a thing.

````yaml
description: A governed thing.
line-limit: 20

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
  x
````
