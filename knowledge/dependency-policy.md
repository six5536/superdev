---
type: Policy
id: dependency-policy
title: Dependency Policy
description: When a dependency may be added and how its version is chosen.
status: stable
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Contributing guide (dependencies)
---

- **Always ask before adding a dependency.** A new one needs a clear reason;
  reach for the standard library or a crate already in the tree first. The
  current set is in [technology-stack](technology-stack.md).[^contributing]
- **Always check the latest version (as of 7 days ago)** and use that, unless
  instructed otherwise.
- **Hoist to the workspace.** Dependency versions, profiles, and shared
  package metadata live in the workspace `Cargo.toml`; member crates inherit
  with `workspace = true`. The shipped project templates follow the same
  rule.
- `cargo-deny` gates licences, bans, and sources in CI; advisories run on a
  schedule and open an issue rather than failing unrelated builds.

[^contributing]: Contributing guide (dependencies)
