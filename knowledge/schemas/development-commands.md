---
type: Schema
id: schema-development-commands
title: Development Commands Schema
description: The everyday command set and the traps in it, in knowledge/development-commands.md.
---

# Development Commands Schema

Structural rules for `knowledge/development-commands.md`, the canonical knowledge's
Reference concept for the everyday command set. The document has no
headings, so it declares a preamble and no sections.

````yaml
description: >
  The everyday command set, what each command runs and when to use it, and the
  commands whose local form does less than CI.
line-limit: 800

frontmatter:
  type:
    const: DevelopmentCommands
  id:
    const: development-commands
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: bullet-list
  description: >
    Where the commands are defined and which list is authoritative, then one
    bullet per command — what it runs and when to use it — and finally the
    traps: commands whose local form does less than CI, or that look
    equivalent but are not.

example: |
  ---
  type: Reference
  id: development-commands
  title: Development Commands
  description: The everyday command set and the pre-PR check list.
  status: stable
  ---

  The workspace `justfile` is authoritative; anything not in it is ad hoc.

  - `just check` — fmt, clippy and the test suite. The pre-PR gate.
  - `just test` — the test suite alone, for the inner loop.
  - `just sync` — resolves packs and rewrites `superdev.lock`.

  Traps: `cargo test` runs fewer tests than `just test`, which adds the
  `--all-features` pass CI depends on. `just sync` rewrites the lockfile even
  when nothing resolved differently, so run it before staging, not after.
````
