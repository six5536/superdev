---
type: Schema
id: schema-backlog
title: Backlog Schema
description: Ideas under consideration and ideas decided against, with the reasoning, in knowledge/backlog.md.
---

# Backlog Schema

Structural rules for `knowledge/backlog.md`, the canonical knowledge's Backlog concept.
Both halves are required: an idea that was rejected without its reasoning is
an idea that gets proposed again.

````yaml
target-files: "knowledge/backlog.md"
description: >
  Ideas under consideration and ideas decided against, each with the reasoning
  that put it where it is.
line-limit: 800

frontmatter:
  type:
    const: Backlog
  id:
    const: backlog
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Under consideration"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet per idea: what it is, where it came up, and why it is deferred
      rather than done.
  - heading: "Decided against"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet per idea, with the reasoning it lost to, so it is not
      re-litigated. Link the decision record if one exists.

example: |
  ---
  type: Backlog
  id: backlog
  title: Backlog & Decided Ideas
  description: Ideas under consideration and ideas decided against, with the reasoning.
  status: draft
  ---

  # Under consideration

  - Pack signature verification. Raised while writing ADR-012; deferred
    because the transport allowlist closes the same exposure at a fraction of
    the cost, and signing needs a key-distribution story first.

  # Decided against

  - A plugin API for custom pack transports. Every transport is a new
    unauthenticated channel to audit, which is exactly what ADR-012 refuses.
    See `knowledge/decisions/adr-012-pack-transport-allowlist.md`.
````
