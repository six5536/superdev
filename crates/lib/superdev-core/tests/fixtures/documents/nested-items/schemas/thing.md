---
type: Schema
id: schema-thing
title: Thing Schema
description: A governed thing whose promises carry nested criteria, keyed at every level, and whose Notes list is keyed where it chooses.
---

# Thing Schema

Structural rules for a thing: every promise under Behaviour opens with a
`P_` key and carries at least one `AC_` criterion beneath it; a note under
Notes is keyed `N_` or plain (ADR-051).

````yaml
description: A governed thing.

frontmatter:
  type:
    const: Thing

sections:
  - heading: "Behaviour"
    level: 2
    required: true
    content: bullet-list
    item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
    item-pattern: '^`P_[a-z0-9-]+` \[(ubiquitous|event|state)\] '
    nested:
      required: true
      item-key: '^`(AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
      item-pattern: '^`AC_[a-z0-9-]+` \[(ubiquitous|event|state)\] '
      item-prohibited-pattern: '\bMUST\b'
  - heading: "Notes"
    level: 2
    content: bullet-list
    item-key: '^`(N_[a-z][a-z0-9-]*)`'
    item-key-optional: true
    item-pattern: '^`N_[a-z0-9-]+` [a-z]'
    item-prohibited-pattern: '\bTBD\b'

example: |
  ---
  type: Thing
  ---

  # A thing

  ## Behaviour

  - `P_starts` [event] WHEN asked, the thing SHALL start.
    - `AC_starts-fast` [event] WHEN asked, the thing SHALL start in 1 s.

  ## Notes

  - `N_timing` measured at p99.
  - A plain note.
````
