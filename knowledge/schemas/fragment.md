---
type: Schema
id: schema-fragment
title: Fragment Schema
description: A shared body other knowledge documents materialize through an include block — filed under knowledge/schemas/fragments/.
---

# Fragment Schema

Structural rules for fragments, filed at
`knowledge/schemas/fragments/{slug}.md`. A fragment is the authored home
of content other documents carry between include markers (SPEC §9):
`superdev validate --fix` copies its body into every including document,
so the body is written to read in place there — no headings, no title
line, nothing that assumes this file around it. The fragments directory
ships with the schema set; the grammar's schema kind claims only the
schemas beside it, never the fragments below.

````yaml
description: >
  One shared body, materialized into other documents by include blocks.
line-limit: 200

frontmatter:
  type:
    required: true
    const: Fragment
  id:
    required: true
    pattern: '^[a-z0-9][a-z0-9-]*$'
  title:
    required: true
  description:
    required: true

preamble:
  content: prose
  description: >
    The whole document: the shared content, exactly as every including
    document should carry it. No headings — an included heading would
    restructure the documents that carry it.

example: |
  ---
  type: Fragment
  id: tracker-conventions
  title: Tracker Conventions
  description: The filing rules every issue schema repeats, kept once.
  ---

  An issue is filed as `issue-{nnn}-{kind}-{slug}`, numbered after the
  highest existing issue across the tracker's folders. A duplicate
  number is an error.
````
