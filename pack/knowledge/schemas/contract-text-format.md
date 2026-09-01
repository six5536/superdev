---
type: Schema
id: schema-contract-text-format
title: Text Format Contract Schema
description: One text file others read or write — where it lives, its shape as a schema or a worked example, how a reader treats the unexpected, and the stability promise, a public contract.
---

# Text Format Contract Schema

Structural rules for one public file-format contract, filed at
`contract-{nnn}-text-format-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. A format is a public contract when someone
outside this repository writes the file by hand or reads it with their own
tools — a manifest, a lock file, an export.

One file has one contract. A file a deployer or user edits to configure a run
is not one of these: it belongs to the [configuration
contract][sokf:schema-contract-config], which defines its shape alongside the
environment variables it mirrors. What is left for this schema is every file
nobody configures anything with.

<!-- sokf:include contract-style -->
**Contract style — a contract is a binding surface, not a
specification** (superdev ADR-029):

- Each normative statement MUST use an RFC 2119 modal verb, one
  requirement per sentence.
- An enumerable surface — commands, flags, keys, types, error cases,
  limits — MUST be defined in the kind's native structured form: a code
  block, table or list. Prose, doc comments included, describes and
  MUST NOT define.
- A contract MUST bind only what callers rely on; behaviour a contract
  does not list is the code's to decide.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

````yaml
description: >
  One file format offered to others — where the files live and who writes
  them, the shape in its own schema language, how a reader treats what it does
  not recognise, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: TextFormatContract
  id:
    required: true
    pattern: '^contract-\d{3}-text-format-[a-z0-9-]+$'
    description: >
      contract-{nnn}-text-format-{slug}, the slug naming which file format. The
      number is the next free one across every contract, public and
      internal together and every lifecycle folder — a duplicate is
      an error.
  title:
    required: true
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^Text format contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Files"
    level: 2
    required: true
    content: prose
    description: >
      The paths or glob the format covers, who writes each file — the tool, the
      user, or both — and which of them is authoritative when they disagree.
  - heading: "Shape"
    level: 2
    required: true
    content: code
    description: >
      The file's shape in its own schema language — JSON Schema, a TOML or
      YAML example carrying every key, a DTD, a grammar. One fenced block,
      tagged with that language. Every key a reader may meet appears, with its
      type and its default, so a writer produces a valid file from this block
      alone; prose around it describes and never defines. A block the
      validator reads declares `block-language` here and is checked for
      completeness; any other block is bound by the contract's drift test.
  - heading: "Compatibility"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What a reader does with an unknown key, a missing optional, a version
      field it does not recognise, and a file a newer release wrote. The rule
      that lets old and new tools share one file.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which keys are promised, how the format is versioned, and how a caller
      is migrated when it changes.

example: |
  ---
  type: TextFormatContract
  id: contract-001-text-format-manifest
  title: File Format Contract
  description: widget.toml — the hand-edited project manifest.
  lifecycle: active
  ---

  # Text format contract: widget.toml

  The hand-edited project manifest: where it lives, its keys, and how a
  reader treats the unexpected.

  ## Files

  `widget.toml` at the repository root, hand-edited and committed. The tool
  rewrites it only on `widget init`; every other command reads it and reports
  what it would change. The file on disk is authoritative — the tool never
  merges a remote copy over it.

  ## Shape

  ```toml
  # Every key the format defines. Only `name` is required.
  name = "my-project"          # string, matches ^[a-z0-9-]+$
  version = "0.1.0"            # semver; defaults to 0.0.0

  [build]
  target = "wasm32"            # one of: wasm32, native
  release = false              # boolean
  ```

  ## Compatibility

  An unknown key MUST be kept and ignored, so a file written by a newer
  release still loads. A missing optional takes the default above. A `version`
  the reader does not understand MUST be a hard error naming the release that
  added it, because guessing at a format is worse than refusing it.

  ## Stability

  Within a major version keys MAY be added and MUST NOT be removed or
  retyped. A key due for removal MUST warn for one minor release and is
  ignored in the next major. A format change that old readers cannot ignore
  MUST come with a `widget migrate` step and a note in the migration guide.
````

<!-- sokf:links -->
[sokf:schema-contract-config]: /knowledge/schemas/contract-config.md
