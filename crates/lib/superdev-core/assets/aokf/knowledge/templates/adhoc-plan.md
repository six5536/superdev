---
type: Template
id: template-adhoc-plan
title: Ad-hoc Plan Template
description: Ad-hoc implementation plan for one-off work outside the feature workflow — context, goal, ordered steps, files affected, testing, and risks. Filed as a draft concept in knowledge/plans/, tagged done when the work lands.
status: stable
---

---
type: Plan
id: adhoc-plan-<slug>
title: <short title of the task>
description: <one line — what this plan delivers>.
status: draft
---

# Plan: <short title of the task>

## Context

<1–3 sentences: what problem this solves, why now, and any constraints that shaped the approach. Link to the request/issue if one exists.>

## Goal

<One sentence stating the observable outcome when this plan is done — what works that doesn't work today.>

Non-goals:
- <Things deliberately out of scope, so reviewers know they weren't forgotten.>

## Current state

<Brief description of the relevant existing behavior/architecture. Reference key files as `path/to/file.ts:123` so they're clickable.>

## Proposed approach

<The core idea in a short paragraph. If alternatives were considered, name them and say in one line each why they lost.>

## Steps

1. <Step name> — <what changes, in which files, and why this step comes first>
2. <Step name> — <...>
3. <Step name> — <...>

<Order steps so the codebase stays working after each one where possible. Call out any step that is hard to reverse.>

## Files affected

| File | Change |
|------|--------|
| `path/to/file` | <new / modified / deleted — one-line description> |

## Testing & verification

- <Unit/integration tests to add or update, and what they assert.>
- <Manual verification steps, exact commands to run.>

## Risks & open questions

- Risk: <what could go wrong> — <mitigation>
- Open question: <decision needed from the user, with a recommended default>

## Out-of-band notes

<Migrations, follow-up work, docs to update, anything that lands after the code does. Delete this section if empty.>

---

Notes on usage (not part of the document):

- File as `knowledge/plans/Pnnn-<slug>.md`, numbered after the highest
  existing plan; id `adhoc-plan-<slug>`.
- List it in `knowledge/plans/index.md`.
- For a feature going through the workflow, use
  `template-feature-plan` instead: the feature-plan phase produces it
  and build, verify, and integrate read it.
