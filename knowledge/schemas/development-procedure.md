---
type: Schema
id: schema-development-procedure
title: Development Procedure Schema
description: Setup, the change workflow, and what must pass before a PR, in knowledge/development-procedure.md.
---

# Development Procedure Schema

Structural rules for `knowledge/development-procedure.md`, the canonical knowledge's
Procedure concept for how a change moves from idea to merge.

````yaml
target-files: "knowledge/development-procedure.md"
description: >
  Setup from clone to working build, the workflow a change follows, and what
  must pass before a PR.
line-limit: 800

frontmatter:
  type:
    const: Procedure
  id:
    const: development-procedure
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    Setup in one or two lines: the commands from clone to working build.

sections-ordered: true
sections:
  - heading: "Workflow"
    level: 1
    required: true
    content: numbered-list
    description: >
      The ordered steps a change follows: how it starts — where it is framed,
      specified and decided before code; how it is implemented — branching,
      commit conventions, slice size; how the canonical knowledge is kept current
      when behaviour or design changes; and what must pass before a PR, what CI
      adds on top, and the smoke test run at each merge, with the command and
      what it proves.

example: |
  ---
  type: Procedure
  id: development-procedure
  title: Development Procedure
  description: Setup, the change workflow, and what to run before a PR.
  status: stable
  ---

  Clone, then `just setup` — it installs the pinned toolchain and builds the
  workspace once so the first test run is warm.

  # Workflow

  1. A change starts as a spec under `knowledge/specs/`, and anything
     architectural is decided in an ADR before code is written.
  2. Branch from `main` as `feature/{slug}`. Commits are conventional and one
     logical change each; a slice is small enough to review in one sitting.
  3. When behaviour or design changes, the canonical knowledge concept that describes it
     changes in the same PR — a stale concept is a bug.
  4. `just check` must pass before the PR. CI adds `--all-features` and the
     `cargo deny` gate. Each merge runs `just smoke`, which resolves a real
     pack end to end and proves the lockfile round-trips.
````
