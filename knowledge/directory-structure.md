---
type: Reference
id: directory-structure
title: Directory Structure
description: What lives where in the repository.
status: stable
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Contributing guide (project layout)
---

```
crates/lib/superdev-core/ # all domain logic (no arg parsing)
crates/app/superdev/      # the binary: CLI parsing, wiring, output
packages/                 # npm launcher + per-platform binary packages
knowledge/                # this AOKF bundle — canonical project knowledge
knowledge/specs/          # design specs (permanent decision records)
knowledge/plans/          # implementation plans (tagged done on landing)
.agents/                  # AOKF spec and agent-facing rules (PROSE.md, MATT-POCOCK-SKILLS.md)
.claude/skills/           # committed skills: the four-skill pack + the materialised workflows set
.superdev/                # superdev's own manifest and lock (this repo is self-managed)
.github/workflows/        # checks.yml (reusable), ci.yml, release.yml, audit.yml
.devcontainer/            # dev container definition
scripts/                  # version, release, and smoke-test scripts
submodules/               # read-only reference checkouts (git submodules)
```

Crate and package contents are detailed in
[software-components](software-components.md). Top-level docs (README,
CONTRIBUTING, CHANGELOG, SECURITY, CODE_OF_CONDUCT) are the public,
GitHub-surfaced files; AGENTS.md is the agent entry point that loads this
bundle.[^contributing]

[^contributing]: Contributing guide (project layout)
