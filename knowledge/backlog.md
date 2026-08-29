---
type: Backlog
id: backlog
title: Backlog & Decided Ideas
description: Ideas under consideration and ideas decided against, with the reasoning.
status: draft
---

# Under consideration

- **A knowledge-capture skill.** The write-side complement to the search-first
  AGENTS.md: teach agents when and how to record durable learnings in the
  knowledge mid-task — concept placement, links, sources. Judged a good idea
  during the skill-pack design
  ([spec][sokf:spec-003-skill-pack]) and deferred to the
  knowledge-upkeep sub-project.

- **Template pre-filled knowledge skeletons.** A project template fixes
  facts about the repo it creates — the rust-npm scaffold establishes the
  Rust workspace, the npm launcher and the CI shape — so it could pre-fill
  parts of the `technology-stack` and `architecture` skeletons instead of
  leaving them TBD, shrinking `bootstrap`'s interview to the genuinely
  human questions. Raised while designing the bootstrap interview phase
  ([spec][sokf:spec-008-knowledge-owned-skills]).

- **Comment-preserving manifest stamping.** `sync` rewrites `config.toml`
  through the whole-file `Manifest::save` when it stamps the blueprint
  version, dropping any hand-written comments — the rewrite `update` always
  did, now implicit in every post-upgrade sync. A targeted `toml_edit` edit
  of the one key would keep a hand-editable file's comments. Raised in the
  blueprint-migrations final review
  ([spec][sokf:spec-004-blueprint-migrations]).

# Decided against

- **Pinning `node` in the managed repo.** Considered because codegraph was
  pinned through mise's `npm:` backend, which cannot install without an `npm`
  in the repo's mise env and produces a `#!/usr/bin/env node` shim that cannot
  run without a node either. Rejected: pinning node writes a toolchain choice
  into the user's `.mise.toml` to work around a packaging detail. codegraph is
  now pinned through the `http` backend against its self-contained release
  bundles, which vendor their own Node, so the dependency is gone rather than
  papered over — see [architecture][sokf:architecture].

<!-- sokf:links -->
[sokf:architecture]: /knowledge/architecture.md
[sokf:spec-003-skill-pack]: /knowledge/specs/spec-003-skill-pack.md
[sokf:spec-004-blueprint-migrations]: /knowledge/specs/spec-004-blueprint-migrations.md
[sokf:spec-008-knowledge-owned-skills]: /knowledge/specs/spec-008-knowledge-owned-skills.md
