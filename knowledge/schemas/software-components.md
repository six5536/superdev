---
type: Schema
id: schema-software-components
title: Software Components Schema
description: The deliverables and the CI/CD that builds them, in knowledge/software-components.md.
---

# Software Components Schema

Structural rules for `knowledge/software-components.md`, the bundle's
Reference concept for the deliverables. The component headings are the
author's to name; `CI/CD` is literal and wins over the pattern.

````yaml
target-files: "knowledge/software-components.md"
description: >
  The deliverables — libraries, binaries, packages — what each contains and
  where it lives, and the pipelines that build and gate them.
line-limit: 800

frontmatter:
  type:
    const: Reference
  id:
    const: software-components
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    One or two sentences: the deliverables and where each lives in the tree.
    Link the architecture concept for the design they implement.

sections-ordered: true
sections:
  - heading-pattern: '^.+$'
    level: 1
    required: true
    repeatable: true
    content: prose
    description: >
      One heading per component, e.g. the core library, the CLI, the service,
      the packages: what it contains and its role — a short list of modules or
      responsibilities where the layout is not obvious from the tree.
  - heading: "CI/CD"
    level: 1
    required: true
    content: prose
    description: >
      The pipelines: what each workflow runs, what it gates, and when it
      triggers.

example: |
  ---
  type: Reference
  id: software-components
  title: Software Components
  description: A core library, a CLI binary, and the workflows that build them.
  status: stable
  ---

  Two deliverables, both from one Cargo workspace: the library under
  `crates/lib/` and the binary under `crates/bin/`. The design they implement
  is in `knowledge/architecture.md`.

  # superdev-core

  The library, and where all behaviour lives. `pack::` resolves sources and
  writes pins, `bundle::` reads and validates the knowledge tree, and
  `lock::` owns the lockfile format — a split the directory names do not make
  obvious, since all three sit side by side under `src/`.

  # superdev-cli

  The binary. Argument parsing, output formatting and exit codes, and nothing
  else: every command is a thin call into the library.

  # CI/CD

  `ci.yml` runs fmt, clippy, tests and `cargo deny` on every push and PR, and
  gates merge. `release.yml` triggers only on a version tag, rebuilds from a
  clean checkout, publishes, and drafts the release notes.
````
