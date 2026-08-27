---
type: Schema
id: schema-sections-fixture
title: Sections Fixture
description: A schema whose sections break the vocabulary.
---

# Sections Fixture

A section naming both forms of heading, and another naming neither.

````yaml
target-files: "knowledge/fixtures/*.md"
description: >
  A fixture document.
line-limit: 800

sections:
  - heading: "Both"
    heading-pattern: '^Both$'
    level: 1
    required: true
    description: >
      Names a literal and a pattern.
  - level: 2
    description: >
      Names neither.

example: |
  # Both
````
