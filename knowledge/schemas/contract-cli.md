---
type: Schema
id: schema-contract-cli
title: CLI Contract Schema
description: One command-line surface — its commands, their behaviour, the exit codes and the stability promise, a public contract.
---

# CLI Contract Schema

Structural rules for one public command-line contract, filed at
`contract-{nnn}-cli-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. One document per binary: a project shipping two
CLIs files two, each with its own stability promise.

<!-- sokf:include contract-style -->
**Contract style — a contract defines its interface** (superdev
ADR-033, ADR-036):

- A contract MUST define every element a caller depends on in the
  structured form this schema declares, so a caller reproduces the
  interface from the contract alone.
- Prose MUST describe and MUST NOT define. Each normative statement
  outside the definition form MUST use an RFC 2119 modal verb, one
  requirement per sentence.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The project MUST bind this contract to its implementation, by
  generating the surface from it or by a test where the implementation
  is hand-written; a committed generated artifact MUST be proved
  current.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

````yaml
description: >
  One command-line surface offered to callers — the commands, what each takes
  and returns, the codes callers branch on, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: CliContract
  id:
    required: true
    pattern: '^contract-\d{3}-cli-[a-z0-9-]+$'
    description: >
      contract-{nnn}-cli-{slug}, the slug naming which command line. The
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
  - heading-pattern: '^CLI contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Commands"
    level: 2
    required: true
    content: code
    block-language: yaml
    block-entry-keys: [about, args, flags, exit]
    description: >
      The definition of the command line, keyed by the path a user types
      ("widget", "widget build"). One entry per command the binary offers,
      each carrying `about`, its positional `args` in order, its `flags` as a
      map of long form to `{type, about}` — `type: bool` for a switch, else
      the value name — and `exit`, a map of code to what it means for that
      command. A caller reproduces the command line from this block alone;
      prose around it describes and never defines. The framework's own help
      and version flags are stated once under Behaviour rather than repeated
      in every entry.
  - heading: "Behaviour"
    level: 2
    required: true
    content: bullet-list
    item-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What the definition block cannot state: what each command reads and
      writes, what it refuses, and the ordering a script depends on. One
      requirement per entry, never a restatement of the block.
  - heading: "Exit codes"
    level: 2
    required: true
    content: table
    columns: [Code, Meaning]
    description: >
      Every code the binary returns, and what it means across commands. A
      command whose code carries a narrower meaning states it in its own
      `exit` entry.
  - heading: "Streams"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      What the binary writes to stdout and to stderr, what it reads from
      stdin, and every command that departs from the rule.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which commands, flags and codes are promised, over what version range,
      and how a breaking change is signalled.

example: |
  ---
  type: CliContract
  id: contract-001-cli-widget
  title: CLI Contract
  description: The widget CLI — build, check and publish, stable from 1.0.
  lifecycle: active
  ---

  # CLI contract: widget

  The widget command line: build, check and publish, stable from 1.0.

  ## Commands

  ```yaml
  widget:
    about: Build, check and publish a widget project.
    args: []
    flags: {}
    exit: { 0: help printed }
  widget build:
    about: Compile the project into ./out.
    args: []
    flags:
      --release: { type: bool, about: Optimise, and strip debug symbols. }
    exit: { 0: built, 1: a source failed to compile, 2: usage error }
  widget check:
    about: Validate the sources without writing.
    args: [PATH]
    flags: {}
    exit: { 0: no findings, 1: a finding, 2: usage error }
  widget publish:
    about: Upload ./out to a registry.
    args: []
    flags:
      --to: { type: URL, about: The registry to upload to. }
    exit: { 0: uploaded, 2: usage error, or a URL outside the allowlist }
  ```

  ## Behaviour

  - **`build`** MUST write only under `./out`, and MUST refuse a dirty
    working tree.
  - **`check`** MUST NOT write. It reads every source under PATH, or the
    whole project when PATH is absent.
  - **`publish`** MUST refuse without a prior `build`, and MUST refuse a URL
    outside the configured registry allowlist.

  ## Exit codes

  | Code | Meaning |
  |------|---------|
  | 0    | success |
  | 1    | the command found something to report |
  | 2    | usage error — unknown flag or subcommand |

  ## Streams

  Every command MUST write its report to stdout and its diagnostics to
  stderr, and MUST read nothing from stdin. `publish` MUST write its progress
  to stderr, so a piped stdout carries the uploaded manifest alone.

  ## Stability

  The commands, their flags and the exit codes are stable from 1.0: within a
  major version they MAY be added, and MUST NOT be removed or repurposed. A
  command due for removal MUST warn on stderr for one minor release first.
````
