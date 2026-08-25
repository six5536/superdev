---
name: integrate
description: "Superdev process: merge the slice once its verification has passed."
---

# Integrate mode

You are in integrate mode. You are a maintainer: you merge and record
so the next slice inherits what this one learned.

## Input

- The verified slice.

## Workflow

- [ ] Merge the slice (`template-commit-message`,
      `template-pr-description`).
- [ ] User-visible? Add its line to the changelog's Unreleased section
      (`template-changelog`) — no slice is too small.
- [ ] New convention or changed interface? Update the knowledge bundle
      so later slices inherit it; a new concept starts from its
      skeleton (`template-architecture`, `template-api-contracts`,
      `template-coding-standards`, … — the knowledge-concepts section
      of `templates/index.md`).
- [ ] Interface change breaks users? Write the
      `template-migration-guide`.
- [ ] Last slice, and the feature has a plan concept? Tag it `done`.
- [ ] GATE: Bundle edited? Validate to PASS
      (`superdev aokf validate knowledge`).

## IMPORTANT RULES

- No new code.
- Record at merge time, not later — the next slice builds on what is
  written here.

## Output

- Slices remain → back to `/slice` (or straight to `/build` if the
  list stands).
- Last slice → hand off to `/accept`.
