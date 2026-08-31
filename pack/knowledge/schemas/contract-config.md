---
type: Schema
id: schema-contract-config
title: Configuration Contract Schema
description: What a deployer must supply to run the software — the settings, where they come from, which source wins, and the stability promise, a public contract.
---

# Configuration Contract Schema

Structural rules for one public configuration contract, filed at
`contract-{nnn}-config-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. Everything a deployer or user supplies to
configure a run: environment variables, the configuration file, and the
precedence between them.

Two boundaries, because configuration touches two neighbours. **A
configuration file is configuration**: its shape is defined here, in the File
section, not as a [file-format contract][sokf:schema-contract-file-format] — that schema
takes the files nobody configures anything with, a lock file or an export.
**Flags are defined by the [CLI contract][sokf:schema-contract-cli]**, which is where a
caller reads them; how a flag, a variable and a file setting resolve against
each other is defined here, because no one of the three owns the precedence.

````yaml
description: >
  What a deployer supplies to run the software — every setting with its type
  and default, the sources it may come from and which wins, the secrets among
  them, and what is promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: ConfigContract
  id:
    required: true
    pattern: '^contract-\d{3}-config-[a-z0-9-]+$'
    description: >
      contract-{nnn}-config-{slug}, the slug naming which configurable surface. The
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
  - heading-pattern: '^Config contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Settings"
    level: 2
    required: true
    content: table
    columns: [Name, Type, Default, Meaning]
    description: >
      Every setting the software reads, named as the environment spells it. A
      setting with no default is required, and the row says so. A setting the
      software reads but does not document is a setting a rebuild loses.
  - heading: "Sources and precedence"
    level: 2
    required: true
    content: bullet-list
    description: >
      Where a value may come from — flag, environment, file, built-in default —
      strongest first, and what happens when two sources disagree. Say whether
      a value is read once at startup or re-read while running.
  - heading: "File"
    level: 2
    content: code
    description: >
      The configuration file's own shape, in its own language or as an example
      carrying every key, plus the path it is read from and what an unknown key
      does. Omit the section on software configured entirely by environment.
  - heading: "Secrets"
    level: 2
    content: prose
    description: >
      Which settings carry credentials, how they are supplied, and what the
      software promises never to log or echo. Omit where none of the settings
      is sensitive.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    description: >
      Which names and defaults are promised, how a renamed setting is carried
      through a deprecation, and what a deployer must do at a major version.

example: |
  ---
  type: ConfigContract
  id: contract-001-config-widget
  title: Configuration Contract
  description: What widget reads to start — four settings, environment or file.
  lifecycle: active
  ---

  # Config contract: widget

  What widget reads to start: four settings, from the environment or
  `widget.toml`.

  ## Settings

  | Name | Type | Default | Meaning |
  |------|------|---------|---------|
  | `WIDGET_PORT` | integer | `8080` | the port the HTTP server binds |
  | `WIDGET_STORE` | url | none — required | where widgets are persisted |
  | `WIDGET_TOKEN` | string | none — required | the registry credential |
  | `WIDGET_LOG` | one of `debug`, `info`, `warn` | `info` | log verbosity |

  ## Sources and precedence

  - A command-line flag, where the command defines one.
  - The environment.
  - `widget.toml` in the working directory.
  - The default above. A setting with no default and no value is a startup
    error naming the setting, never a silent fallback.

  Every value is read once at startup. Changing one takes a restart.

  ## File

  ```toml
  # widget.toml — every key optional; each mirrors the variable of the same name.
  port = 8080
  store = "postgres://localhost/widgets"
  log = "info"
  ```

  An unknown key is a startup error naming the key and the nearest known one,
  because a typo in a config file is otherwise a silent default.

  ## Secrets

  `WIDGET_TOKEN` is a credential. It is read from the environment only — never
  from the file, so it cannot be committed — and it is redacted wherever the
  configuration is printed, including the startup banner and the error text.

  ## Stability

  Names and defaults are stable within a major version. A renamed setting is
  read under both names for one minor release, with a warning naming the new
  one, and only the new name survives the next major.
````

<!-- sokf:links -->
[sokf:schema-contract-cli]: /knowledge/schemas/contract-cli.md
[sokf:schema-contract-file-format]: /knowledge/schemas/contract-file-format.md
