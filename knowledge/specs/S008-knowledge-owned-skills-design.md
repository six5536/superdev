---
type: Spec
id: spec-knowledge-owned-skills
title: Knowledge-Owned Skills
description: Design for aokf-carried lifecycle skills — aokf-maintain and the validation hook relocate to the knowledge capability, and a new aokf-bootstrap skill harvests a repo's stranded prose and interviews the owner to fill the seeded skeleton.
status: stable
---

# Motivation

Two pressures, one owner:

- The knowledge-lifecycle machinery ships with the wrong capability.
  `aokf-maintain` and the validation hook arrive with the skill pack, so a
  `--no-knowledge` repo carries a skill that errors and a hook that blocks
  edits to its own unrelated `knowledge/` directory — the standing backlog
  gating item.
- Adopted repos have knowledge stranded in prose with no context pointer —
  this repo's Rust rules sat unreachable in CONTRIBUTING.md until moved by
  hand. Nothing automates that move, so every adoption repeats it manually
  or not at all.

Both resolve the same way the skill-overrides spec resolved provider
content: the capability that owns the domain carries the machinery
([spec](S006-workflows-skill-overrides-design.md)).

# Decision

The aokf component carries the knowledge-lifecycle skills:

- **`aokf-maintain` and the validation hook relocate** from the skill pack
  to the aokf component. The pack drops to three skills. The relocation is
  lock-free: the skill's path and lock key are unchanged, only the owning
  claim moves, so a synced repo sees no orphan and no rewrite. A
  `--no-knowledge` repo loses the hook on its next sync through the
  ordinary orphan sweep — which closes the gating backlog item.
- **`[knowledge]` gains a `custom` list** with the standard semantics:
  releases a skill from management, `init` adoption seeds it, an unknown
  name reports `knowledge: custom names unknown skill '<name>' — no
  effect`.
- **Assets are capability-scoped** under `assets/aokf/` — the skills at
  `assets/aokf/skills/<name>/`, and the rest of the aokf install (the
  `AGENTS.md` scaffold, the `.agents` files, the knowledge seed) moved in
  beside them, so everything the component ships has one home.
  `assets/skills/` stays the pack's, per the kind-scoped assets layout.
  Relocated and new skills keep the PROJECT.md extension layer.
- **`aokf-bootstrap`** is the first new aokf-carried skill (below). It
  began as `aokf-adopt` (harvest only) and was renamed when the interview
  phase made it a fresh repo's bootstrap too, not just an adoption tool.

# The aokf-bootstrap skill

Judgement work an agent does after `superdev init`'s mechanical
scaffolding: fill the canonical knowledge from the two places a repo's knowledge
already lives — its stranded prose, and the owner's head.

- **Inventory**: every prose document — README, CONTRIBUTING, `docs/`,
  in-repo wikis, incumbent CLAUDE.md/AGENTS.md files. Code is opt-in: the
  skill first asks whether to sweep code at all; when yes, it harvests
  only comments carrying repo-wide conventions, decisions and gotchas —
  never API description, which is the environment and stays with the code.
- **Harvest** is the core move, and it is move-and-reference: the durable
  fact lands in the canonical knowledge, and the source keeps a one-line summary plus a
  link, per the no-duplication rule (AOKF §3). Import-only was rejected —
  it creates exactly the two-homes drift the format bans.
- **Skeleton-first**: harvested material fills the seeded starter concepts'
  TBDs; new concepts are created only where nothing fits. The canonical knowledge
  seed and the harvest are two halves of one design.
- **Entry points**: for an incumbent CLAUDE.md/AGENTS.md, the skill asks
  the user — merge-and-reduce (durable content into the canonical knowledge,
  always-loaded rules into AGENTS.md, CLAUDE.md down to the `@AGENTS.md`
  import) or leave as found. This closes the incumbent-entry-point backlog
  item by making the choice explicit per repo.
- **Posture**: ask decisions, act on mechanics. User-owned choices — the
  code sweep, entry-point handling, an ambiguous placement — are asked
  with a question tool; clear-cut moves run autonomously and land as a
  reviewable diff.
- **Interview**: the owner is always the inventory's last source. After
  the prose harvest — or immediately, on a fresh repo with nothing to
  harvest — the skill walks the skeletons still carrying TBD prompts and
  splits them per repo: facts that already exist (purpose, status,
  constraints, first glossary terms, the dependency policy, intended
  conventions) are interviewed and landed; knowledge only development
  produces (the real architecture, testing strategy as practised) is left
  TBD — it accretes through the project's own specs and decisions.
- **Completion criterion**: every inventoried source accounted for —
  harvested, reduced to summary-and-citation, or explicitly left with a
  reason — every skeleton filled or left TBD with the reason it must
  accrete, and the canonical knowledge validating clean. Re-running is safe: the
  inventory is whatever remains unaccounted.

# Init hint

Whenever the knowledge capability is enabled, `init` ends with
`knowledge: run /aokf-bootstrap in Claude Code to fill the canonical knowledge from
existing docs and an owner interview` — unconditional, like the workflows
setup hint. An empty repo loses nothing: with no prose to harvest, the
interview is its whole bootstrap.

# Dogfood consequence

Landing this made superdev's own repo enable `[knowledge]`: with the hook
and `aokf-maintain` owned by the knowledge capability, a skills-only
manifest would have swept both. The repo's hand-written scaffolds
(AGENTS.md, the canonical knowledge) predate the capability and are untouched; the
owned `.agents` files and the embedded assets were trued up against each
other (the stale asset took the repo's improved MATT-POCOCK-SKILLS.md,
the repo took the asset's VALIDATION.md), and `.mcp.json` now names the
bare `superdev` the dev shim resolves.

# Out of scope

- The ongoing write-side (the knowledge-capture skill in the
  [backlog](../backlog.md)) — adopt is the one-shot adoption-time bulk
  version; capturing learnings mid-task stays its own design.
- Harvesting from sources outside the repo (issue trackers, wikis, chat).
- Structured AOKF updates over MCP — adopt edits files directly, guarded
  by the validation hook like any other agent edit.
