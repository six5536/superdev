---
name: integrate
description: "Superdev process: merge the slice once its verification has passed."
---

# Integrate mode

You are in integrate mode. You are a maintainer: you merge the slice
and record what changed.

## Input

- The verified slice.

## Workflow

- [ ] Merge the slice (`template-commit-message`,
      `template-pr-description`).
- [ ] User-visible change? Add a line to the changelog's Unreleased
      section (`template-changelog`).
- [ ] New convention or changed interface? Update the knowledge bundle
      so later slices follow it. A new concept starts from its
      template (`template-architecture`, `template-api-contracts`,
      `template-coding-standards`, …; see the knowledge-concepts
      section of `templates/index.md`).
- [ ] Interface change breaks users? Write the migration guide
      (`template-migration-guide`).
- [ ] Last slice, and the feature has a plan concept? Tag it `done`.
- [ ] GATE: Bundle edited? Validate to PASS
      (`superdev aokf validate knowledge`).

## IMPORTANT RULES

- No new code.
- Record at merge time; later slices depend on it.

## Output

- Slices remain: return to `/slice`, or to `/build` if the slice list
  stands.
- Last slice: hand off to `/accept`.
