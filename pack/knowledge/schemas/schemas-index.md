---
type: Schema
id: schema-schemas-index
title: Schemas Index Schema
description: The grouped listing of every schema with its one-line summary, in knowledge/schemas/index.md.
---

# Schemas Index Schema

Structural rules for `knowledge/schemas/index.md`, the grouped listing of
the schemas. Derived from the index itself, which is both the contract and
the instance — the only file in the set that is its own example. The
category headings are the author's to name, so what this schema holds is the
shape of an entry, not the taxonomy.

````yaml
target-files: "knowledge/schemas/index.md"
description: >
  The listing of every schema, grouped by the part of the process that
  produces its documents, each entry linking the file and summarising it in
  one line.
line-limit: 800

sections-ordered: true
sections:
  - heading: "Schemas"
    level: 1
    required: true
    content: prose
    description: >
      What the schemas are — the structural contract for every document the
      development process produces — and why they are checkable where a
      template could only be copied.
  - heading-pattern: '^.+$'
    level: 2
    required: true
    repeatable: true
    content: bullet-list
    description: >
      One heading per group, named for the part of the process that produces
      the documents under it. One bullet per schema: a link carrying its
      title, then a dash and a one-line summary of what the schema governs
      and where its documents are filed. The summary matches the schema's own
      frontmatter description, so a change to one is visible as a difference
      from the other.

example: |
  # Schemas

  The structural contract for every document the development process
  produces: what sections it must carry, what its frontmatter must say, and
  a worked example that satisfies the contract. A schema is checkable — the
  tool enforces it — where a template could only be copied.

  ## Planning & design

  * [Feature Plan Schema][sokf:schema-feature-plan] - the feature's slice list — per slice a done-check, its test-plan cases and a done marker — filed among the plans.
  * [ADR Schema][sokf:schema-adr] - architecture decision records — context, the decision, options considered and consequences — filed among the ADRs.

  ## Change delivery

  * [Code Review Schema][sokf:schema-code-review] - code review findings — verdict first, findings ranked by severity with concrete failure scenarios.
````
