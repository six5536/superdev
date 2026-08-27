---
type: Schema
id: schema-api-contracts
title: API Contracts Schema
description: The public surfaces, their contracts and the stability promises, in knowledge/api-contracts.md.
---

# API Contracts Schema

Structural rules for `knowledge/api-contracts.md`, the bundle's Reference
concept for the public surfaces. One heading per surface, named by the
author; `Stability` is literal and wins over the pattern.

````yaml
target-files: "knowledge/api-contracts.md"
description: >
  The public surfaces — CLI, HTTP API, library — the contract each one offers,
  and what is promised stable.
line-limit: 800

frontmatter:
  type:
    const: Reference
  id:
    const: api-contracts
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^.+$'
    level: 1
    required: true
    repeatable: true
    content: prose
    description: >
      One heading per surface, e.g. CLI, HTTP API, public library. The
      contract, one line per command, endpoint or entry point — what it takes,
      what it returns, and the error behaviour callers rely on. A fenced usage
      block where it says it faster than prose.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      What is promised stable, what may change without notice, and how breaking
      changes are signalled to callers.

example: |
  ---
  type: Reference
  id: api-contracts
  title: API Contracts
  description: The superdev CLI surface and the core library, stable from 1.0.
  status: stable
  ---

  # CLI

  Every command exits 0 on success, 1 on a usage error, and 2 when a check
  finds something to report — a finding is not a failure.

  ```
  superdev sync [--frozen]   resolve packs, write superdev.lock
  superdev check [path...]   validate the bundle, exit 2 on findings
  ```

  # Stability

  The CLI surface and the exit codes are stable from 1.0: commands and flags
  are added, never removed or repurposed, within a major version. The core
  library is unstable and may change in any release; depend on the CLI.
````
