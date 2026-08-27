---
type: Schema
id: schema-dependency-policy
title: Dependency Policy Schema
description: When a dependency may be added and how its version is chosen, in knowledge/dependency-policy.md.
---

# Dependency Policy Schema

Structural rules for `knowledge/dependency-policy.md`, the canonical knowledge's Policy
concept for taking on dependencies. The document is a list and nothing else,
so it declares a preamble and no sections.

````yaml
target-files: "knowledge/dependency-policy.md"
description: >
  When a dependency may be added, who approves it, how versions are chosen and
  kept current, and what gates dependencies automatically.
line-limit: 800

frontmatter:
  type:
    const: Policy
  id:
    const: dependency-policy
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: bullet-list
  description: >
    One bullet each: when a dependency may be added, who approves it, and what
    to reach for first instead; how versions are chosen and kept current; and
    what gates dependencies automatically — licence checks, advisories, bans —
    and where those run.

example: |
  ---
  type: Policy
  id: dependency-policy
  title: Dependency Policy
  description: When a dependency may be added and how its version is chosen.
  status: stable
  ---

  - A dependency is added when writing it ourselves would cost more than
    auditing it, and a maintainer approves it on the PR. Reach first for the
    standard library, then for a crate already in the tree.
  - Versions are pinned in the lockfile and floated at the caret in the
    manifest. Dependabot opens the bumps; a bump lands like any other change.
  - `cargo deny` runs in CI and gates on advisories, unmaintained crates, and
    licences outside the allowlist.
````
