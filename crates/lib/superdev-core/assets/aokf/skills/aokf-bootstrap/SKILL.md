---
name: aokf-bootstrap
description: Fill the AOKF bundle at `knowledge/` from what the repo already knows — harvest its stranded prose, then interview the owner to flesh out the seeded skeleton. Run after `superdev init`, when docs feel stale or scattered, or when agents keep missing rules that live outside the bundle.
---

Fill this repo's AOKF bundle at `knowledge/` from its existing prose
and from the owner. The format spec is `.agents/aokf/SPEC.md`; read it
before writing concepts.

A **harvest** relocates one durable fact from a source into the
bundle, leaving a one-line summary and a link behind in the source.
Never import-only: a fact with two full homes drifts.

# Posture

Ask decisions, act on mechanics. The owner decides: whether to sweep
code, what to do with an incumbent entry point, whether a marginal
passage is knowledge — ask explicitly (AskUserQuestion or your
harness's equivalent). Clear-cut harvests run without asking and land
as a reviewable diff.

# Phase 1 — inventory

List every prose document in the repo: README, CONTRIBUTING, anything
under `docs/`, in-repo wikis, and incumbent agent entry points
(`CLAUDE.md`, `AGENTS.md`, `GEMINI.md` and kin) that predate superdev.
Skip generated files and the bundle itself.

Then ask: **should this run also sweep the code?** If yes, add the
comments that carry repo-wide conventions, decisions, and gotchas (a
module header explaining a policy, a "we do X because Y") — never
API-describing doc comments; those are the environment.

The owner is always the last source; the interview (phase 3) works
it, and a fresh repo with no prose starts there. The inventory is the
work list; on a re-run, a source already summarised-and-cited drops
off it.

# Phase 2 — harvest

Work the inventory source by source:

1. Split the source into passages; each is **knowledge** (a
   convention, decision, reason, or gotcha an agent cannot derive from
   the environment) or **environment cache** (restates `package.json`
   scripts, `--help` output, file layout, API — leave it in place).
2. Land knowledge **skeleton-first**: replace the starter concepts'
   TBD prompts (coding-standards, testing-strategy, architecture and
   the rest); create a new concept, frontmatter per the spec, only
   when nothing fits.
3. Rewrite the passage to a one-line summary plus a link; record the
   source file in the concept's `sources`.
4. Neither (marketing prose, a stale claim, an ambiguous rule)? Ask,
   or leave it with a reason in your report.

For an incumbent entry point, ask the owner first: **merge and reduce**
(durable content into the bundle, genuinely always-loaded rules into
`AGENTS.md`, the incumbent down to its `@AGENTS.md` import) or **leave as
found**.

# Phase 3 — interview

List the skeletons still carrying TBD prompts and split them:

- **Answerable now** — facts that already exist: what the project is
  and its status, constraints and non-goals, first glossary terms,
  dependency policy, intended conventions. Interview in rounds of a
  few questions grouped by concept; land each answer in its skeleton.
- **Accretes later** — knowledge only development produces: real
  architecture, testing strategy as practised, error-handling
  conventions. Leave the TBD in place; these fill through the
  project's own specs and decisions.

The split is per repo: a template-seeded repo, for example, already
fixes much of its technology stack.

# Phase 4 — verify and report

Run the validator and fix every error before finishing:

```
superdev aokf validate knowledge
```

Report every source as harvested (naming its concepts), reduced to
summary-and-citation, or left with a reason; every skeleton as filled
or left TBD with the reason it must accrete. The bundle must PASS at
level 2. Leave changes uncommitted unless asked; then commit as
`docs:` per Conventional Commits.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
