---
type: Schema
id: schema-thing
title: Thing Schema
description: A governed thing whose Behaviour items carry the ADR-047 example rule — a `P_` key, a tag and one admitted verb, with no modal verb outside an item.
---

# Thing Schema

Structural rules for a thing: every item under Behaviour opens with a
`P_` key and a tag and carries one of `SHALL`, `SHOULD` or `MAY`; a
retired verb, a second verb, and a modal verb outside an item are each
an error (ADR-047).

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
    item-pattern: '(?s)^`P_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b'
    item-only-pattern: '\b(SHALL|SHOULD|MAY|MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|(?s)\b(SHALL|SHOULD|MAY)\b.*\b(SHALL|SHOULD|MAY)\b'
  - heading: "Notes"
    level: 2
    content: prose
    item-only-pattern: '\b(SHALL|SHOULD|MAY|MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b'

example: |
  ---
  type: Thing
  ---

  # A thing

  ## Behaviour

  Every promise acts on the thing.

  - `P_starts` [event] WHEN asked, the thing SHALL start.

  ## Notes

  A note with no promise in it.
````
