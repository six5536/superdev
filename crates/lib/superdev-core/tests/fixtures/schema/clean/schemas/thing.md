---
type: Schema
id: schema-fixture
title: Fixture Schema
description: A schema used only as a fixture.
---

# Fixture Schema

A schema that satisfies the document vocabulary, so the clean case has one to pass.

````yaml
target-files: "knowledge/fixtures/*.md"
description: >
  A fixture document, for the parity goldens.
line-limit: 800

sections:
  - heading: "Only"
    level: 1
    required: true
    content: prose
    description: >
      The one section this fixture declares.

example: |
  # Only

  The body of the one section.
````
