---
type: Decision
id: adr-021-nothing-unattended-reaches-the-default-branch
title: Nothing Unattended Reaches the Default Branch
description: A feature runs on the branch /frame creates and an adhoc plan touching code on adhoc/<slug>; an unattended run commits and merges only there, and a human fast-forwards the default branch.
lifecycle: active
---

# ADR-021: Nothing Unattended Reaches the Default Branch

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

Today each integrated slice reaches the default branch almost
immediately, with the user present at every boundary. An unattended run
removes the user from those boundaries, so the same practice would let
unreviewed work land on the branch everyone else builds from. The
workflow also never creates a branch at all: a feature runs wherever the
user happened to be, and the 97-commit `feature/content-packs` branch
carried nine plans because nothing cut one per feature.

## Decision

We will give every feature a branch of its own: `/frame` creates
`feature/<slug>` off the default branch, `/adhoc-plan` creates
`adhoc/<slug>` when its work touches code, and a repo whose development
procedure names its own convention keeps it. An unattended run commits
and merges only on that branch. The default branch moves when a human
fast-forwards it.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Branch per feature; a human moves the default branch | Unattended work is quarantined until reviewed; one branch maps to one feature | The user gains a fast-forward step per feature |
| Merge each slice to the default branch, as today | No new step | Unattended, unreviewed work lands where everyone builds |
| A standing integration branch shared by all features | One long-lived target | Features entangle; a broken slice in one blocks the others |

## Consequences

- Positive: `git log` on the default branch shows nothing an unattended
  run produced on its own.
- Positive: a feature's whole history sits on one branch, named for it.
- Negative: today's practice changes — integrated slices wait on the
  feature branch until the user fast-forwards.
- Follow-ups: the development-procedure template and this repo's concept
  record the convention.
