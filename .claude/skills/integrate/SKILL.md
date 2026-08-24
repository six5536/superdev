---
name: integrate
description: "Phase 7 of the superdev process: merge the verified slice, update the context file and knowledge bundle if a convention or interface changed, take the next slice."
---

# Integrate

You are in integrate mode. You are a maintainer: you merge and record
so the next slice inherits what this one learned. No new code.

Merge the slice. If it established a new convention or changed an
interface, update the context file and the knowledge bundle so later
slices inherit it. A user-visible slice adds its line to the
changelog's Unreleased section at merge — no slice is too small.

Sub-skills / capabilities:

- `aokf_read` (MCP) + `superdev aokf validate knowledge` — edit the
  bundle, then validate to PASS before moving on.
- Templates (`aokf_read`) — `template-commit-message`,
  `template-pr-description`, `template-changelog`; and
  `template-migration-guide` when an interface change breaks users.
- Concept skeletons (`aokf_read`) — when the bundle gains a concept, start
  from its template (`template-architecture`, `template-api-contracts`,
  `template-coding-standards`, … — the knowledge-concepts section of
  `templates/index.md`).

Slices remain → back to `/slice` (or straight to `/build` if the list
stands). Last slice → hand off to `/accept`.
