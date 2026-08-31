---
type: Schema
id: schema-configuration
title: Configuration Schema
description: The config files and stores, their shape, and what lives outside the repo, in knowledge/configuration.md.
---

# Configuration Schema

Structural rules for `knowledge/configuration.md`, the canonical knowledge's Reference
concept for configuration. One heading per config file or store, named by
the author; `Outside the repo` is literal and wins over the pattern.

````yaml
description: >
  Where configuration lives, the shape of each file or store, who writes it,
  and what a fresh machine needs that the tree does not carry.
line-limit: 800

frontmatter:
  type:
    required: true
    const: Configuration
  id:
    required: true
    const: configuration
  title:
    required: true
  description:
    required: true
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    Where configuration lives, and which files are committed, generated, or
    machine-local.

sections-ordered: true
sections:
  - heading-pattern: '^.+$'
    level: 1
    required: true
    repeatable: true
    content: code
    description: >
      One heading per config file or store: its shape with a trimmed example in
      a fenced block, who writes it — hand-edited or tool-managed — and when it
      changes.
  - heading: "Outside the repo"
    level: 1
    required: true
    content: prose
    description: >
      Environment variables, user-level caches, secrets and where they come
      from — anything a fresh machine needs that the tree does not carry.

example: |
  ---
  type: Configuration
  id: configuration
  title: Configuration & Environments
  description: superdev.yaml is hand-edited, superdev.lock is tool-written, the cache is machine-local.
  status: stable
  ---

  Two files are committed and one directory is not: `superdev.yaml` is
  hand-edited, `superdev.lock` is written by the tool, and
  `.superdev/cache/` is machine-local and gitignored.

  # superdev.yaml

  Hand-edited, and the only file a user is expected to open. It changes when
  a pack is added, removed, or repointed.

  ```yaml
  packs:
    - source: https://github.com/acme/superdev-pack-rust
      rev: v1.2.0
  ```

  # superdev.lock

  Tool-managed — written by `superdev sync`, never hand-edited. It changes
  whenever a pin resolves to a new revision.

  ```yaml
  packs:
    - source: https://github.com/acme/superdev-pack-rust
      resolved: 4127a3b9c1d0e5f6a7b8c9d0e1f2a3b4c5d6e7f8
  ```

  # Outside the repo

  `SUPERDEV_CACHE_DIR` overrides the cache location. Nothing else is read
  from the environment, and no secrets are needed: every allowed transport
  either is anonymous or uses the machine's existing git credentials.
````
