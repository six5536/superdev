---
name: bootstrap
description: "Use after superdev init, when docs are stale or scattered, or when agents miss rules that live outside the knowledge bundle."
---

# Bootstrap mode

You are in bootstrap mode. You are a knowledge curator: you move the
repo's stranded facts into the bundle at `knowledge/` and interview
the owner for the rest.

A harvest relocates one durable fact from a source into the bundle,
leaving a one-line summary and a link behind in the source. Never
import-only: a fact with two full homes drifts.

## Input

- The format spec at `.agents/aokf/SPEC.md`; read it before writing
  concepts.
- $ARGUMENTS — sources or concepts to focus on, when given.

## Workflow

- [ ] Inventory every prose document in the repo: README,
      CONTRIBUTING, `docs/`, in-repo wikis, and incumbent agent entry
      points (`CLAUDE.md`, `AGENTS.md`, `GEMINI.md` and kin) that
      predate superdev. Skip generated files and the bundle itself.
      On a re-run, drop sources already summarised and cited.
- [ ] Ask the owner: sweep the code too? If yes, add the comments
      that carry repo-wide conventions, decisions, and gotchas —
      never API-describing doc comments; those are the environment.
- [ ] Harvest source by source: split each into passages. Knowledge
      (a convention, decision, reason, or gotcha an agent cannot
      derive from the environment) lands in the bundle; environment
      cache (restating `package.json` scripts, `--help` output, file
      layout, API) stays in place.
- [ ] Land knowledge skeleton-first: replace the starter concepts'
      TBD prompts; create a new concept, frontmatter per the spec,
      only when nothing fits. Rewrite the source passage to a
      one-line summary plus a link; record the source in the
      concept's `sources`.
- [ ] Passage neither knowledge nor environment cache (marketing
      prose, a stale claim, an ambiguous rule)? Ask the owner, or
      leave it with a reason in the report.
- [ ] Incumbent entry point? Ask the owner: merge and reduce (durable
      content into the bundle, always-loaded rules into `AGENTS.md`,
      the incumbent down to its `@AGENTS.md` import), or leave as
      found.
- [ ] Interview the owner (`/grill-me`) on the skeletons still
      carrying TBD prompts that are answerable now: what the project
      is and its status, constraints and non-goals, first glossary
      terms, dependency policy, intended conventions. Land each
      answer in its skeleton.
- [ ] Leave the TBDs only development can fill (architecture as
      practised, testing strategy, error-handling conventions); they
      accrete through the project's own specs and decisions.
- [ ] GATE: Validate the bundle to PASS at level 2
      (`superdev aokf validate knowledge`); fix every error.

## IMPORTANT RULES

- Ask decisions, act on mechanics. The owner decides whether to sweep
  the code, what to do with an incumbent entry point, and whether a
  marginal passage is knowledge. Clear-cut harvests run without
  asking and land as a reviewable diff.
- Leave changes uncommitted unless asked; then commit as `docs:` per
  Conventional Commits.

## Output

- The bundle, filled and PASSing.
- A report: every source harvested (naming its concepts), reduced to
  summary-and-citation, or left with a reason; every skeleton filled
  or left TBD with the reason it must accrete.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
