# Plan Format

Plans are AOKF concepts under `knowledge/plans/`, one file
per plan, named `Pnnn-<feature>.md`. Scan the directory for the highest
number and increment by one. The `id` is `plan-<feature>` and never
changes.

## Template

```md
---
type: Plan
id: plan-{feature}
title: {Title}
description: {One line: what this plan delivers.}
status: draft
links:
  - rel: implements
    to: spec-{topic}
---

# Goal

{One or two sentences, with a body link to the
[spec](/knowledge/specs/Snnn-{topic}-design.md) mirroring the
`implements` edge.}

# Tasks

1. **{Task}** — {what it changes and what it delivers}.
   Verify: {the check that proves it done — a test, a command, an
   observable}.
2. …
```

## Rules

- The spec carries the decisions; tasks stay thin — a deliverable and
  its verification, never restated design.
- Every task ends on a **Verify** line the executing agent can actually
  run or observe.
- Declare link edges from the plan side only (`implements` → the spec)
  — the spec is permanent and stays free of work-item churn.
- A completed plan stays in the bundle: tag it `done` in the commit
  that completes the work, and flip the spec to `status: stable` in the
  same commit. Search down-ranks `done` concepts, so finished work
  doesn't crowd live knowledge.
