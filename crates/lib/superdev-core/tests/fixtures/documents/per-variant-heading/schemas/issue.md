---
type: Schema
id: schema-issue
title: Issue Schema
description: A governed issue whose Criteria section is declared once per lifecycle state — plain while unframed, keyed once framed.
---

# Issue Schema

Structural rules for an issue. `lifecycle` selects the variant: an
`unframed` issue's Criteria is a numbered list and nothing more, a
`framed` one's is keyed and tagged (ADR-049).

````yaml
description: A governed issue in two states.

variant-key: lifecycle

frontmatter:
  type:
    required: true
    const: Issue
  lifecycle:
    required: true
    enum: [unframed, framed]

sections-ordered: true
sections:
  - heading: "Context"
    level: 2
    required: true
    content: prose
  - heading: "Criteria"
    level: 2
    required: true
    content: numbered-list
    variants: [unframed]
  - heading: "Criteria"
    level: 2
    required: true
    content: numbered-list
    item-key: '^`(AC_[a-z]+)`'
    item-pattern: '^`AC_[a-z]+` \[event\] '
    variants: [framed]
  - heading: "Tail"
    level: 2
    required: true
    content: prose

example:
  unframed: |
    ---
    type: Issue
    lifecycle: unframed
    ---

    # An unframed issue

    ## Context

    Prose.

    ## Criteria

    1. A plain sentence.

    ## Tail

    Prose.
  framed: |
    ---
    type: Issue
    lifecycle: framed
    ---

    # A framed issue

    ## Context

    Prose.

    ## Criteria

    1. `AC_one` [event] WHEN asked, it answers.

    ## Tail

    Prose.
````
