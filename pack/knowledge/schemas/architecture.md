---
type: Schema
id: schema-architecture
title: Architecture Schema
description: The system's layers, its subsystems, and the files it reads and writes, in knowledge/architecture.md.
---

# Architecture Schema

Structural rules for `knowledge/architecture.md`, the canonical knowledge's Reference
concept for the system's shape. One named file rather than a family, and it
carries frontmatter, so the `Architecture` type names it exactly. The subsystem
headings are the author's to choose; the literal headings around them win over
the pattern.

````yaml
description: >
  The system's shape — the top-level layers and the direction of dependency
  between them, a section per subsystem that needs more than a line, and what
  the system reads and writes.
line-limit: 800

frontmatter:
  type:
    required: true
    const: Architecture
  id:
    required: true
    const: architecture
  title:
    required: true
  description:
    required: true
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: bullet-list
  description: >
    The system's shape in one short paragraph — the top-level layers or
    components and the direction of dependency between them — then one bullet
    per layer saying what it owns and what it must not know about. Link the
    decision records that set this shape.

sections-ordered: true
sections:
  - heading-pattern: '^.+$'
    level: 1
    repeatable: true
    content: prose
    description: >
      One heading per subsystem that needs more than a line: what it does, how
      it talks to the rest, and where its detail lives.
  - heading: "Files and artefacts"
    level: 1
    required: true
    content: prose
    description: >
      What the system reads and writes at runtime or install time, and who owns
      each — enough that a reader knows what is safe to touch.

example: |
  ---
  type: Architecture
  id: architecture
  title: Architecture
  description: A Rust core owning packs and the canonical knowledge, with a thin CLI over it.
  status: stable
  ---

  superdev is a Rust workspace: a core library owns pack resolution, the
  knowledge and the lockfile, and a thin CLI drives it. The dependency
  runs one way, and ADR-012 sets the transport rules the shape enforces.

  - `superdev-core` — pack resolution, manifest and lock handling, knowledge
    reads. Must not know how it was invoked.
  - `superdev-cli` — argument parsing, output formatting, exit codes. Knows
    the core; the core never knows it.

  # Pack resolution

  Resolves each manifest source to a pinned revision, refusing transports
  outside the allowlist, and records the pin in the lockfile. The interface
  detail lives in the `contract-pack-source` interface contract.

  # Files and artefacts

  The core reads `superdev.yaml` and writes `superdev.lock`; both are
  committed and both are safe to hand-edit. The pack cache under
  `.superdev/cache/` is machine-local, tool-owned, and safe to delete.
````
