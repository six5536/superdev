---
name: adhoc-plan
description: "Superdev process: plan one-off work that does not go through the feature workflow."
---

# Adhoc-plan mode

You are in adhoc-plan mode. You are a project planner: you plan one
piece of work outside the feature workflow — a refactor, a migration,
a chore.

## Input

- $ARGUMENTS — the work to plan.

## Workflow

- [ ] Read the canonical knowledge the work touches (`aokf_overview` +
      `aokf_search`): the conventions and constraints the plan must
      respect.
- [ ] Read the affected code and its callers (`codegraph_explore`)
      before setting the steps.
- [ ] GATE: Does the work need a spec, or change an interface that is
      expensive to change? It is a feature: go to `/frame`.
- [ ] Draft the plan from `template-adhoc-plan`: context, goal,
      ordered steps, files affected, testing, and risks.
- [ ] Order the steps so the codebase stays working after each one
      where possible.
- [ ] Interview the user (`/grill-me`): resolve the open questions
      and the risks that need their judgement.
- [ ] File the plan as a draft concept at
      `knowledge/plans/Pnnn-<slug>.md`, listed in the plans index.
- [ ] Double-check the plan (`/double-check`); fix what it finds.
- [ ] GATE: Validate the canonical knowledge to PASS
      (`superdev aokf validate knowledge`).

## IMPORTANT RULES

- Plan only: no code.
- A change that needs a spec or an interface decision goes through
  the feature workflow, starting at `/frame`.

## Output

- The plan: a draft concept in `knowledge/plans/` with ordered steps.
- The work follows the plan's steps; tag the plan `done` when it
  lands.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
