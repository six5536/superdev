---
type: Schema
id: schema-migration-guide
title: Migration Guide Schema
description: Migration guides — old-to-new steps with per-step verification, behavioural differences, rollback and troubleshooting.
---

# Migration Guide Schema

Structural rules for migration guides, matched by name
(`**/*migration-guide*.md`); the source names no filing directory, and the
document is not an SOKF concept, so it carries no frontmatter.

````yaml
target-files: "**/*migration-guide*.md"
description: >
  Old-to-new migration: what changed and who must act, a before/after table,
  prerequisites, ordered steps each with its own verification, the behavioural
  differences that do not show up as errors, rollback, and troubleshooting.
line-limit: 800

sections-ordered: true
sections:
  - heading-pattern: '^Migrating from .+ to .+$'
    level: 1
    required: true
    content: prose
    description: >
      One or two sentences: what changed, why, and who needs to act. State
      clearly who does NOT need to act.
  - heading: "At a glance"
    level: 2
    required: true
    content: table
    columns: ["", Before, After]
    description: >
      One row per key aspect, with the old usage and the new one side by side,
      so a reader can find their case without reading the steps.
  - heading: "Prerequisites"
    level: 2
    required: true
    content: bullet-list
    description: >
      Minimum versions, backups to take, feature flags to check.
  - heading: "Steps"
    level: 2
    required: true
    description: >
      The ordered migration, one level-3 heading per step.
  - heading-pattern: '^\d+\. .+$'
    level: 3
    required: true
    repeatable: true
    content: code
    description: >
      One numbered step: what to do, with exact commands or a before/after
      diff where the change is mechanical, and a "Verify:" line saying how to
      confirm this step worked before moving on. A step nobody can verify is a
      step that fails silently halfway through a migration.
  - heading: "Behavioral differences"
    level: 2
    required: true
    content: bullet-list
    description: >
      Changes that do not show up as compile errors — different defaults,
      timing, error types. These are the ones that bite. One bullet per
      difference and its consequence.
  - heading: "Rollback"
    level: 2
    required: true
    content: prose
    description: >
      How to get back to the old state if something goes wrong, and until
      which step rollback stays cheap.
  - heading: "Troubleshooting"
    level: 2
    required: true
    content: bullet-list
    description: >
      One bullet per symptom or error message, with its cause and fix.

example: |
  # Migrating from superdev 0.1 to 0.2

  Pack sources must now name an allowed transport. If every source in your
  manifest already uses https, ssh or file, nothing changes and you do not
  need to act.

  ## At a glance

  | | Before | After |
  |---|--------|-------|
  | Source transport | `git://host/repo` | `https://host/repo` |
  | Refusal point | at fetch | at manifest parse |

  ## Prerequisites

  - superdev 0.1.4 or later installed, so the warning names offending
    sources before you upgrade.
  - A committed `superdev.lock`, so a bad migration is one `git checkout`
    away.

  ## Steps

  ### 1. Find the offending sources

  ```sh
  superdev check --transports
  ```

  Verify: the command lists every source that 0.2 will refuse. An empty list
  means you are already done.

  ### 2. Repoint each source

  ```diff
  - source: git://github.com/acme/pack-rust
  + source: https://github.com/acme/pack-rust
  ```

  Verify: `superdev sync` resolves to the same revision it did before —
  compare the pin in `superdev.lock`, which should not change.

  ## Behavioral differences

  - Refusal now happens at parse, so a manifest with one bad source fetches
    nothing at all rather than partially succeeding.

  ## Rollback

  Reinstall 0.1 and restore `superdev.lock` from git. Rollback stays cheap
  through step 2; once you have run 0.2's `sync`, the lockfile format has
  been rewritten and 0.1 will not read it.

  ## Troubleshooting

  - `refused transport "git"` — the source is still on the old scheme; repeat
    step 2 for it.
````
