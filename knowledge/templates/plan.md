---
type: Template
id: template-plan
title: Plan Template
description: Implementation plan — context, goal, ordered steps, files affected, testing, and risks.
status: stable
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
