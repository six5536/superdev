---
type: DirectoryStructure
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
pack/                     # the content superdev ships, in pack layout
pack/knowledge/skills/    # the knowledge-carried skills (one directory each)
pack/knowledge/concepts/  # the knowledge scaffolds, mirroring the repo's knowledge/
pack/knowledge/templates/ # the document templates
pack/skills/              # the skill pack
pack/agents/              # the general-rules scaffolds
pack/projects/            # the project templates
crates/lib/superdev-core/ # all domain logic (no arg parsing)
crates/app/superdev/      # the binary: CLI parsing, wiring, output
packages/                 # npm launcher + per-platform binary packages
knowledge/                # this repository's SOKF knowledge
knowledge/specs/          # design specs (permanent decision records)
knowledge/plans/          # plan-<nnn>-feature-<slug> slice lists and
                          # plan-<nnn>-adhoc-<slug> one-off work
knowledge/decisions/      # ADRs; permanent
knowledge/contracts/      # interface contracts
knowledge/issues/         # gaps and tickets, one concept each
knowledge/schemas/        # the contract each document type is checked against
.agents/                  # the entry point, capability instructions, SOKF spec, agent rules
.claude/skills/           # committed skills: the skill pack + the knowledge-carried set
.superdev/                # superdev's own manifest and lock (this repo is self-managed)
.github/workflows/        # checks.yml (reusable), ci.yml, release.yml, audit.yml
.devcontainer/            # dev container definition
scripts/                  # version, release, and smoke-test scripts
submodules/               # read-only reference checkouts (git submodules)
```

A pack's tree is its declaration: the directory under `pack/` names the item's
owning capability and the one below it the kind, so a file's position is what
makes it an item ([ADR-003](decisions/D003-items-by-layout.md)). `pack.toml`
carries the format version and metadata, never an item list. The capability
instruction files (`pack/sokf/agents/`, `pack/codegraph/`) are not
pack content: they describe a version the binary pins or a format the compiled
validator enforces, so they move with the binary.

`crates/lib/superdev-core/assets` is a relative symlink to `pack/`: that is
what keeps the content inside the published crate while leaving it browsable at
the root, and it is why a Windows checkout needs `core.symlinks=true`
(see [CONTRIBUTING](/CONTRIBUTING.md)). Crate and package contents are detailed
in [software-components](software-components.md). Top-level docs (README,
CONTRIBUTING, CHANGELOG, SECURITY, CODE_OF_CONDUCT) are the public,
GitHub-surfaced files; AGENTS.md is the agent entry point that loads this
knowledge.[^contributing]

[^contributing]: Contributing guide (project layout)
