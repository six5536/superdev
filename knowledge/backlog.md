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
- **Gating the AOKF hook on the knowledge capability.** The hook validates any
  edit under `knowledge/`, and its settings entry ships with the skills
  capability. A repo initialised `--no-knowledge` with skills on therefore
  blocks edits to its own unrelated `knowledge/` directory, and the only escape
  is `--no-skills`. As designed
  ([spec](specs/2026-08-12-skill-pack-design.md)); a refinement would gate the
  hook on the knowledge capability or on a bundle marker.
- **Repos that already have an agent entry point.** `init` scaffolds
  `AGENTS.md`, so adopting superdev in a repo with an incumbent entry file
  leaves two of them competing. Adopting goodbye-tinnitus made this concrete:
  a 638-byte scaffold `AGENTS.md` now sits beside its 9.9 kB `CLAUDE.md`, and
  neither mentions the other. Merge, migrate, or cross-reference is a design
  question deferred to the knowledge-upkeep sub-project, where the rest of the
  entry-point and migration work lives.

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
