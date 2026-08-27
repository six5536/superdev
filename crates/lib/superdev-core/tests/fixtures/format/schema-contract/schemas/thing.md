---
type: Schema
id: schema-contract-fixture
title: Contract Fixture
description: A schema whose yaml contract breaks the vocabulary.
---

# Contract Fixture

Three backticks where four are required, an unknown key, and a missing one.

```yaml
target-files: "knowledge/fixtures/*.md"
invented-key: nonsense
sections:
  - heading: "Only"
    level: 1
    required: true
    description: >
      No line-limit and no description above.

example: |
  # Only
```
