---
type: Schema
id: schema-contract-file-format
title: File Format Contract Schema
description: One file others read or write — where it lives, its shape, how a reader treats the unexpected, and the stability promise, in knowledge/contracts/public/.
---

# File Format Contract Schema

Structural rules for one public file-format contract, filed at
`knowledge/contracts/public/contract-{nnn}-file-format-{slug}.md`. A format is a public contract when someone
outside this repository writes the file by hand or reads it with their own
tools — a manifest, a lock file, an export.

One file has one contract. A file a deployer or user edits to configure a run
is not one of these: it belongs to the [configuration
contract](contract-config.md), which defines its shape alongside the
environment variables it mirrors. What is left for this schema is every file
nobody configures anything with.

````yaml
description: >
  One file format offered to others — where the files live and who writes
  them, the shape in its own schema language, how a reader treats what it does
  not recognise, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    const: FileFormatContract
  id:
    pattern: '^contract-\d{3}-file-format-[a-z0-9-]+$'
    description: >
      contract-{nnn}-file-format-{slug}, the slug naming which file format. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Files"
    level: 1
    required: true
    content: prose
    description: >
      The paths or glob the format covers, who writes each file — the tool, the
      user, or both — and which of them is authoritative when they disagree.
  - heading: "Shape"
    level: 1
    required: true
    content: code
    description: >
      The format in its own schema language — JSON Schema, a TOML example with
      every key, a DTD, a grammar. One fenced block, tagged. Prose around it
      describes; this block defines.
  - heading: "Compatibility"
    level: 1
    required: true
    content: prose
    description: >
      What a reader does with an unknown key, a missing optional, a version
      field it does not recognise, and a file a newer release wrote. The rule
      that lets old and new tools share one file.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      Which keys are promised, how the format is versioned, and how a caller
      is migrated when it changes.

example: |
  ---
  type: FileFormatContract
  id: contract-001-file-format-manifest
  title: File Format Contract
  description: widget.toml — the hand-edited project manifest.
  status: stable
  ---

  # Files

  `widget.toml` at the repository root, hand-edited and committed. The tool
  rewrites it only on `widget init`; every other command reads it and reports
  what it would change. The file on disk is authoritative — the tool never
  merges a remote copy over it.

  # Shape

  ```toml
  # Every key the format defines. Only `name` is required.
  name = "my-project"          # string, matches ^[a-z0-9-]+$
  version = "0.1.0"            # semver; defaults to 0.0.0

  [build]
  target = "wasm32"            # one of: wasm32, native
  release = false              # boolean
  ```

  # Compatibility

  An unknown key is kept and ignored, so a file written by a newer release
  still loads. A missing optional takes the default above. A `version` the
  reader does not understand is a hard error naming the release that added it,
  because guessing at a format is worse than refusing it.

  # Stability

  Keys are added, never removed or retyped, within a major version. A key due
  for removal warns for one minor release and is ignored in the next major. A
  format change that old readers cannot ignore comes with a `widget migrate`
  step and a note in the migration guide.
````
