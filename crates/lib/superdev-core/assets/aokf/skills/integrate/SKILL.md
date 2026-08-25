---
name: integrate
description: "Superdev process: merge the slice once its verification has passed."
---

# Integrate mode

You are in integrate mode. You are the integration manager: you merge
verified work into the shared branch and keep the project's records
current.

## Input

- The verified slice's commit, and its entry in the feature's plan at
  `knowledge/plans/Pnnn-<slug>.md`.
- $ARGUMENTS — the slice, when not handed off.

## Workflow

- [ ] Read the slice's plan entry (`aokf_read`) and the
      `development-procedure` concept: the merge convention (target
      branch, PR or direct, required checks).
- [ ] The merge target moved since verify? Update the slice onto it
      again.
- [ ] Run the full build, the linter, all integration tests, and a
      smoke test.
- [ ] GATE: Conflict, or a check failed? Return to `/build` with the
      failure as input.
- [ ] Merge the slice per the convention (`template-commit-message`,
      `template-pr-description`).
- [ ] User-visible change? Add a line to the changelog's Unreleased
      section (`template-changelog`).
- [ ] New convention, changed interface, or new term? Update the
      knowledge bundle so later slices follow it: the glossary for
      terms; a new concept starts from its template
      (`template-architecture`, `template-api-contracts`,
      `template-coding-standards`, …; see the knowledge-concepts
      section of `templates/index.md`).
- [ ] Interface change breaks users? Write the migration guide
      (`template-migration-guide`).
- [ ] Mark the slice done in the feature's plan (`knowledge/plans/`).
      Last slice? Tag the plan concept `done`.
- [ ] GATE: Bundle edited? Validate to PASS
      (`superdev aokf validate knowledge`).

## IMPORTANT RULES

- No new code.
- Record at merge time; later slices depend on it.

## Output

- Slices remain: hand the next slice to `/build`; return to
  `/feature-plan` when the slice list needs re-cutting.
- Last slice: done, or `/frame` for the next feature. `/accept` runs
  when the user asks for it.
- Merge conflict or failed check: return to `/build`.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
