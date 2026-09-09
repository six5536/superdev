---
type: Issue
id: issue-078-workflow-progress-repeats-plan-and-stage
title: Workflow progress repeats the plan and stage
description: superdev_run_phase displays the same SCOPE plan title and requirements-review stage in both its progress panel and status line, duplicating information during an active review.
kind: bug
lifecycle: open
---

# Bug: Workflow progress repeats the plan and stage

## Summary

`superdev_run_phase` repeats the active plan and review stage across its progress panel and status line. Present the workflow status without redundant title and stage text.

## Context

The user observed the following while reviewing `plan-077-sokf-file-tool-parity` on `work/077-sokf-file-tool-parity`:

```text
SCOPE plan-077-sokf-file-tool-parity
requirements review · 1:44
read
Esc to stop and preserve partial work
/workspaces/superdev (work/077-sokf-file-tool-parity)
[Pi usage and model footer]
SCOPE plan-077-sokf-file-tool-parity · requirements review
```

The usage footer is abbreviated; the duplicated plan and stage lines are retained.

## Behaviour

Show the plan and current stage once in the active workflow display rather than repeating both in a second status line. Preserve elapsed time, useful activity details, and the cancellation guidance. Keep useful status available when the progress panel is not visible.

## Scope

Limit the change to redundant status presentation by `superdev_run_phase`. Do not change workflow transitions, review requirements, cancellation behavior, or issue 77's SOKF file-tool work.

## Comments

The user confirmed filing this bug on the default branch. No matching duplicate was found; idea-014 addresses broader progress visibility on the work branch rather than this specific duplication.
