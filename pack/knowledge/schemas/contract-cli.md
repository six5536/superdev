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
    description: >
      The usage block: one line per command and flag, as `--help` would print
      it. Prose describes the surface; this block defines it.
  - heading: "Behaviour"
    level: 2
    required: true
    content: bullet-list
    description: >
      One entry per command — what it reads, what it writes, what it refuses,
      and the behaviour callers rely on. Anything a script would break on
      belongs here.
  - heading: "Exit codes"
    level: 2
    content: table
    columns: [Code, Meaning]
    description: >
      The codes callers branch on. Omit the section where the project defines
      them once for every binary and the error-handling concept carries them.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
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

  ```
  widget build [--release]   compile the project into ./out
  widget check [PATH...]     validate the sources; PATH replaces the default
  widget publish --to <URL>  upload ./out to a registry
  ```

  ## Behaviour

  - **`build`** writes only under `./out` and refuses a dirty working tree.
  - **`check`** never writes. It reads every source under PATH, or the whole
    project when PATH is absent.
  - **`publish`** requires a prior `build`, and refuses a URL outside the
    configured registry allowlist.

  ## Exit codes

  | Code | Meaning |
  |------|---------|
  | 0    | success |
  | 1    | the command found something to report |
  | 2    | usage error — unknown flag or subcommand |

  ## Stability

  The commands, their flags and the exit codes are stable from 1.0: they are
  added, never removed or repurposed, within a major version. A command due for
  removal warns on stderr for one minor release first.
````
