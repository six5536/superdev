---
type: Idea
id: idea-013-workflow-commits-name-their-phase-and-plan
title: Workflow commits name their phase and plan
description: Replace generic workflow commit subjects with phase-specific subjects that identify the plan and describe the recorded change.
status: draft
---

# Idea: workflow commits name their phase and plan

Make each SOKF workflow commit subject identify the workflow phase, the plan,
and the information recorded by the commit. Prefer subjects such as
`scope(plan-077): record requirements review` over generic subjects such as
`chore(workflow): start scope`.

## Motivation

Generic workflow subjects describe an internal transition but omit the work
item and the meaningful result. Readers must inspect each commit to learn which
plan changed and whether the commit started a phase, recorded evidence, applied
feedback, or completed a gate.

## Sketch

Use the phase as the commit type and the plan ID as the scope:
`scope(plan-NNN): <information>`, `build(plan-NNN): <information>`, and
`accept(plan-NNN): <information>`. Generate the summary from the specific
workflow action and its durable effect rather than from a shared lifecycle
placeholder.

## Open questions

- Which workflow actions create commits, and what stable summary vocabulary
  should each action use?
- Should issue-filing commits follow the same pattern before a plan exists?
- Should correction and review commits use the current phase or a dedicated
  commit type?
