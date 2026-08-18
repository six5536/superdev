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
  bundle mid-task — concept placement, links, sources. Judged a good idea
  during the skill-pack design
  ([spec](specs/2026-08-12-skill-pack-design.md)) and deferred to the
  knowledge-upkeep sub-project.

- **Template pre-filled knowledge skeletons.** A project template fixes
  facts about the repo it creates — the rust-npm scaffold establishes the
  Rust workspace, the npm launcher and the CI shape — so it could pre-fill
  parts of the `technology-stack` and `architecture` skeletons instead of
  leaving them TBD, shrinking `aokf-bootstrap`'s interview to the genuinely
  human questions. Raised while designing the bootstrap interview phase
  ([spec](specs/2026-08-18-knowledge-owned-skills-design.md)).

- **Comment-preserving manifest stamping.** `sync` rewrites `config.toml`
  through the whole-file `Manifest::save` when it stamps the blueprint
  version, dropping any hand-written comments — the rewrite `update` always
  did, now implicit in every post-upgrade sync. A targeted `toml_edit` edit
  of the one key would keep a hand-editable file's comments. Raised in the
  blueprint-migrations final review
  ([spec](specs/2026-08-12-blueprint-migrations-design.md)).

# Decided against

- **Pinning `node` in the managed repo.** Considered because codegraph was
  pinned through mise's `npm:` backend, which cannot install without an `npm`
  in the repo's mise env and produces a `#!/usr/bin/env node` shim that cannot
  run without a node either. Rejected: pinning node writes a toolchain choice
  into the user's `.mise.toml` to work around a packaging detail. codegraph is
  now pinned through the `http` backend against its self-contained release
  bundles, which vendor their own Node, so the dependency is gone rather than
  papered over — see [architecture](architecture.md).
