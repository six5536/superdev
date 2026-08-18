---
name: aokf-adopt
description: Bring a repo's existing knowledge into the AOKF bundle at `knowledge/` — after `superdev init`, when docs feel stale or scattered, or when agents keep missing rules that live outside the bundle.
---

Bring this repo's stranded knowledge into the AOKF bundle at `knowledge/`.
The format spec is `.agents/aokf/SPEC.md`; read it before writing concepts.
Adoption is judgement work: `superdev init` scaffolded the structure, this
skill fills it from what the repo already knows.

The unit of work is the **harvest**: relocate one durable fact from a
source into the bundle, leaving a one-line summary and a link behind in
the source. Never import-only — a fact with two full homes drifts, and
the audit (`/aokf-maintain`) will flag it.

# Posture

Ask decisions, act on mechanics. A choice the repo's owner must make —
whether to sweep code, what to do with an incumbent entry point, whether
a marginal passage is knowledge — is put to the user as an explicit
question (AskUserQuestion, or your harness's equivalent). A clear-cut
harvest runs without asking and lands as a reviewable diff.

# Phase 1 — inventory

List every prose document in the repo: README, CONTRIBUTING, anything
under `docs/`, in-repo wikis, and incumbent agent entry points
(`CLAUDE.md`, `AGENTS.md`, `GEMINI.md` and kin) that predate superdev.
Skip generated files and the bundle itself.

Then ask: **should this run also sweep the code?** Only if yes, add
comments carrying repo-wide conventions, decisions, and gotchas — a
module header explaining a policy, a "we do X because Y". API-describing
doc comments are never harvested: they are the environment, reachable by
reading the code, and stale the moment the code moves.

The inventory is the work list. Re-running this skill is safe: whatever
is already summarised-and-cited is done, and the inventory is what
remains.

# Phase 2 — harvest

Work the inventory source by source:

1. Split the source into passages. For each, decide: **knowledge**
   (a convention, a decision, a reason, a gotcha — anything an agent
   needs and cannot derive from the environment) or **environment cache**
   (restates `package.json` scripts, `--help` output, file layout, API —
   leave it where it is).
2. Land each knowledge passage **skeleton-first**: the starter concepts
   (coding-standards, testing-strategy, architecture and the rest) are
   the target structure — replace their TBD prompts. Create a new concept
   only when nothing fits, with frontmatter per the spec.
3. Rewrite the source passage to a one-line summary plus a link to the
   concept. Record the source file in the concept's `sources`.
4. A passage that is neither knowledge nor cache — marketing prose, a
   stale claim, an ambiguous rule — is a decision: ask, or leave it with
   a reason noted in your report.

For an incumbent entry point, ask the owner first: **merge and reduce**
(durable content into the bundle, genuinely always-loaded rules into
`AGENTS.md`, the incumbent down to its `@AGENTS.md` import) or **leave as
found**. Never merge silently — that file is someone's curated context.

# Phase 3 — verify and report

Run the validator and fix every error before finishing:

```
superdev aokf validate knowledge
```

Finish with a report — and an empty inventory is itself the report: say
the repo had nothing to harvest. Otherwise: every inventoried source
accounted for — harvested (with its concepts named), reduced to
summary-and-citation, or explicitly left with a reason. The bundle must PASS at level 2. Leave
changes uncommitted unless asked; if asked, commit as `docs:` per
Conventional Commits.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
