---
type: Schema
id: schema-contract-text-format
title: Text Format Contract Schema
description: One text file others read or write — where it lives, its shape as a schema or a worked example, how a reader treats the unexpected, and the stability promise, a public contract.
---

# Text Format Contract Schema

Structural rules for one public text-format contract, filed at
`contract-{nnn}-text-format-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. A format is a public contract when someone
outside this repository writes the file by hand or reads it with their own
tools — a manifest, a lock file, an export.

One file has one contract. A file a deployer or user edits to configure a run
is not one of these: it belongs to the [configuration
contract][sokf:schema-contract-config], which defines its shape alongside the
environment variables it mirrors. What is left for this schema is every file
nobody configures anything with.

<!-- sokf:include contract-style -->
**Contract style — a contract defines its interface** (superdev
ADR-033, ADR-042, ADR-043, ADR-044):

- A contract's Definition MUST be one or more source includes of the
  regions that declare the interface, and MUST NOT carry an authored
  block; a caller reads the interface from the contract and reproduces
  it from the source the contract carries.
- A region MUST be bounded by `sokf:begin <name>` and `sokf:end <name>`
  in the source's own comment syntax. What is not marked is not
  promised.
- A doc comment inside an included region is contract text: a MUST
  there binds as a MUST in Behaviour does.
- Prose MUST describe and MUST NOT define. Behaviour MUST carry what no
  single element can say and what no include reaches — stability,
  consumers, behaviour across elements, exit codes, error semantics —
  each normative statement with an RFC 2119 modal verb, one requirement
  per sentence.
- Behaviour MUST cover what the schema's checklist names for the
  contract's kind, one `###` per item that applies.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The Definition is bound by its include. The project MUST bind each
  Behaviour promise by a test of the behaviour it promises.
- A built-from source unreadable as a surface MUST be rendered by a
  generator that writes `sokf:generated-by <what>` in the rendering's
  leading lines, and the rendering MUST be proved current by a test.
- A Behaviour or Stability statement whose behaviour is unbuilt MAY
  carry `PENDING` in uppercase beside its modal verb, naming the issue
  or plan slice in parentheses, and MUST NOT once the feature settles; a
  definition element carries none.
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
