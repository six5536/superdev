---
type: Schema
id: schema-unparsed
title: Unparsed Schema
description: A schema whose example carries no frontmatter block.
---

# Unparsed Schema

Its documents dispatch by type, so the example owes a frontmatter block
and shows none.

````yaml
description: A governed thing.

frontmatter:
  type:
    required: true
    const: Unparsed

example: |
  # A thing

  prose
````
