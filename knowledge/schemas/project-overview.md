---
type: Schema
id: schema-project-overview
title: Project Overview Schema
description: What the project is, for whom, and its current status, in knowledge/project-overview.md.
---

# Project Overview Schema

Structural rules for `knowledge/project-overview.md`, the canonical knowledge's Overview
concept and the first thing a newcomer reads.

````yaml
description: >
  What the project does, for whom, the one thing that distinguishes it, and
  where it currently stands.
line-limit: 800

frontmatter:
  type:
    const: Overview
  id:
    const: project-overview
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    Two or three sentences: what the project does, for whom, and the one thing
    that distinguishes it. Written to orient a newcomer, not to sell. Link the
    architecture concept for the design.

sections-ordered: true
sections:
  - heading: "Status"
    level: 1
    required: true
    content: prose
    description: >
      Where the project stands: released or not, what is built, what is not
      yet. Keep it current — this is the first thing a newcomer reads.

example: |
  ---
  type: Overview
  id: project-overview
  title: Project Overview
  description: What superdev is and its current status.
  status: stable
  ---

  superdev prepares and validates the canonical knowledge an agent reads, and
  resolves the content packs that knowledge is assembled from. It is for teams
  who want their agent's context reviewed like code rather than pasted into a
  prompt. What distinguishes it is that every rule is data the tool enforces,
  not prose the agent is asked to remember. See `knowledge/architecture.md`.

  # Status

  Pre-1.0 and in use on this repository. Pack resolution, the lockfile and
  knowledge validation are built; schema-driven document validation is in
  progress; nothing is published to a registry yet.
````
