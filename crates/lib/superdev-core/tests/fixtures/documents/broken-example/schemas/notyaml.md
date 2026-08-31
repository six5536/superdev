---
type: Schema
id: schema-notyaml
title: Notyaml Schema
description: A schema whose example's frontmatter is not YAML.
---

# Notyaml Schema

The example carries a frontmatter block whose text does not parse as
YAML.

````yaml
description: A governed thing.

frontmatter:
  type:
    required: true
    const: Notyaml

example: |
  ---
  type: [unclosed
  ---

  # A thing
````
