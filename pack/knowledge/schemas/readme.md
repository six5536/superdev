---
type: Schema
id: schema-readme
title: README Schema
description: README.md — install, quick start, usage, configuration and the development loop.
---

# README Schema

Structural rules for `README.md` at the repository root. Install and Quick
start each require a fenced block: a front page whose first command cannot
be copy-pasted has failed at the only job it has.

````yaml
target-files: "README.md"
description: >
  The project front page: what it is and for whom, how to install it, the
  shortest path to seeing it work, the main commands, configuration, the
  contributor loop, and the licence.
line-limit: 800

sections-ordered: true
sections:
  - heading-pattern: '^.+$'
    level: 1
    required: true
    content: prose
    description: >
      The project name, then one sentence on what this is and who it is for —
      a badge row here if the project publishes CI or version badges — and one
      short paragraph on the problem it solves and the one thing that
      distinguishes it. No marketing prose.
  - heading: "Install"
    level: 2
    required: true
    content: code
    description: >
      The install command in a fenced block, and the prerequisites if any:
      runtime version, system dependencies.
  - heading: "Quick start"
    level: 2
    required: true
    content: code
    description: >
      The shortest path from install to seeing it work — a copy-pasteable
      example with its expected output, each in its own fenced block.
  - heading: "Usage"
    level: 2
    required: true
    content: prose
    description: >
      The main commands or APIs. Link to fuller docs rather than duplicating
      them here.
  - heading-pattern: '^.+$'
    level: 3
    repeatable: true
    content: code
    description: >
      One heading per common task, each with a minimal fenced example.
  - heading: "Configuration"
    level: 2
    required: true
    content: table
    columns: [Option, Default, Description]
    description: >
      One row per option: the option, its default, and what it controls.
  - heading: "Development"
    level: 2
    required: true
    content: code
    description: >
      The loop a contributor needs — clone, install dependencies, run tests —
      in a fenced block. Link CONTRIBUTING.md if one exists.
  - heading: "License"
    level: 2
    required: true
    content: prose
    description: >
      The licence name, linking to the LICENSE file.

example: |
  # superdev

  Prepares and validates the canonical knowledge your coding agent reads.

  Agent context is usually prose pasted into a prompt and trusted. superdev
  makes it a reviewed artifact: content arrives as pinned packs, and every
  structural rule is data a tool enforces rather than an instruction the
  agent is asked to remember.

  ## Install

  ```sh
  cargo install superdev
  ```

  Requires Rust 1.79 or later. No other system dependencies.

  ## Quick start

  ```sh
  superdev init && superdev sync
  ```

  ```
  resolved 2 packs, wrote superdev.lock
  ```

  ## Usage

  Two commands do the work; see `docs/` for the full reference.

  ### Sync packs

  ```sh
  superdev sync --frozen
  ```

  ### Validate the canonical knowledge

  ```sh
  superdev check knowledge/
  ```

  ## Configuration

  | Option | Default | Description |
  |--------|---------|-------------|
  | `SUPERDEV_CACHE_DIR` | `.superdev/cache` | where fetched packs are stored |
  | `--frozen` | off | fail instead of updating the lockfile |

  ## Development

  ```sh
  git clone https://github.com/acme/superdev && cd superdev
  just setup && just check
  ```

  ## License

  MIT — see [LICENSE](LICENSE).
````
