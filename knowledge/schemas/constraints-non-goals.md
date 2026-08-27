---
type: Schema
id: schema-constraints-non-goals
title: Constraints & Non-Goals Schema
description: What the project deliberately does not do and the limitations it accepts, in knowledge/constraints-non-goals.md.
---

# Constraints & Non-Goals Schema

Structural rules for `knowledge/constraints-non-goals.md`, the canonical knowledge's
Reference concept for the project's deliberate limits.

````yaml
target-files: "knowledge/constraints-non-goals.md"
description: >
  What the project deliberately does not do, and the limitations it has
  accepted, each with the trade-off behind it.
line-limit: 800

frontmatter:
  type:
    const: Reference
  id:
    const: constraints-non-goals
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Non-goals"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet each: what the project deliberately does not do, and why —
      written so nobody proposes it twice.
  - heading: "Constraints"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet each: an accepted limitation and the trade-off behind it —
      what was given up, and what accepting it buys.

example: |
  ---
  type: Reference
  id: constraints-non-goals
  title: Known Constraints & Non-Goals
  description: What superdev deliberately does not do, and the accepted limitations.
  status: stable
  ---

  # Non-goals

  - superdev does not run the agent. It prepares and validates the canonical knowledge the
    agent reads; anything that executes a model belongs to the host.
  - superdev does not host packs. It fetches from git over the allowed
    transports and nothing else.

  # Constraints

  - Packs resolve over git only. That gives up registry conveniences like
    yanking and semver ranges, and buys a pin that is verifiable with tools
    every contributor already has.
````
