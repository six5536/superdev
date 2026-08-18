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

Ask decisions, act on mechanics. Some choices belong to the repo's
owner: whether to sweep code, what to do with an incumbent entry point,
whether a marginal passage is knowledge. Put those to the user as an
explicit question (AskUserQuestion, or your harness's equivalent). A
clear-cut harvest runs without asking and lands as a reviewable diff.

# Phase 1 — inventory

List every prose document in the repo: README, CONTRIBUTING, anything
under `docs/`, in-repo wikis, and incumbent agent entry points
(`CLAUDE.md`, `AGENTS.md`, `GEMINI.md` and kin) that predate superdev.
Skip generated files and the bundle itself.

Then ask: **should this run also sweep the code?** Only if yes, add
the comments that carry repo-wide conventions, decisions, and gotchas:
a module header explaining a policy, a "we do X because Y". API-describing
doc comments are never harvested; they are the environment.

The owner is always the inventory's last source; the interview
(phase 3) works it. A fresh repo with no prose to list starts there.

The inventory is the work list; on a re-run, a source already
summarised-and-cited is done and drops off it.

# Phase 2 — harvest

Work the inventory source by source:

1. Split the source into passages. For each, decide: **knowledge**
   (a convention, a decision, a reason, a gotcha: anything an agent
   needs and cannot derive from the environment) or **environment cache**
   (restates `package.json` scripts, `--help` output, file layout, API —
   leave it where it is).
2. Land each knowledge passage **skeleton-first**: the starter concepts
   (coding-standards, testing-strategy, architecture and the rest) are
   the target structure; replace their TBD prompts. Create a new concept
   only when nothing fits, with frontmatter per the spec.
3. Rewrite the source passage to a one-line summary plus a link to the
   concept. Record the source file in the concept's `sources`.
4. A passage that is neither knowledge nor cache (marketing prose, a
   stale claim, an ambiguous rule) is a decision: ask, or leave it with
   a reason noted in your report.

For an incumbent entry point, ask the owner first: **merge and reduce**
(durable content into the bundle, genuinely always-loaded rules into
`AGENTS.md`, the incumbent down to its `@AGENTS.md` import) or **leave as
found**.

# Phase 3 — interview

Work the last source: the owner. List the skeleton concepts still
carrying TBD prompts and split them in two:

- **Answerable now**: facts that already exist, in the owner's head or
  the repo's shape. What the project is and its status, constraints and
  non-goals, the first glossary terms, the dependency policy, intended
  conventions. Interview for these in rounds of a few questions, grouped
  by concept, and land each answer in its skeleton.
- **Accretes later**: knowledge only development produces — the real
  architecture, the testing strategy as practised, error-handling
  conventions. Leave the TBD prompt in place; these fill through
  the project's own specs and decisions.

The split is per repo, not a fixed list: a repo seeded from a project
template, for example, already fixes much of its technology stack.

# Phase 4 — verify and report

Run the validator and fix every error before finishing:

```
superdev aokf validate knowledge
```

Finish with a report that accounts for every inventoried source:
harvested (with its concepts named), reduced to summary-and-citation,
or explicitly left with a reason. Account for every skeleton the same
way: filled, or left TBD with the reason it must accrete. The bundle
must PASS at level 2. Leave changes uncommitted unless
asked; if asked, commit as `docs:` per Conventional Commits.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
