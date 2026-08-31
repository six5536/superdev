---
type: Chore
id: issue-026-chore-rehearse-the-driver-on-a-real-feature
title: Run the next real feature through /execute-feature-plan and record the outcome
description: The loop's machinery is tested and was rehearsed with shell steps, but the driver's prose — slice picking, the retry bound, deferral, the end-of-run queue — has not driven a real multi-slice plan; the next feature is the rehearsal.
lifecycle: open
links:
  - rel: references
    to: issue-024-feature-request-the-workflow-cannot-run-unattended
    note: Covers the manual cases of criteria 5, 7, 8 and 9 that acceptance could not run in place.
---

# Chore: rehearse the driver on a real feature

## Summary

Acceptance for
[I024][sokf:issue-024-feature-request-the-workflow-cannot-run-unattended]
proved the run machinery live — a session held at its turn boundary
executed a two-step chain with an `advance` between and released the run
— but substituted shell steps for build and integrate. The driver skill's
own prose has not yet taken a real multi-slice plan through the loop, so
its manual cases (a run to completion, a twice-failing slice deferred,
the deferred decisions put in sequence) stand unexecuted.

## Surfaces

- The next feature delivered in this repository, driven end to end by
  `/execute-feature-plan`.
- `pack/knowledge/skills/execute-feature-plan/SKILL.md`, amended with
  whatever the rehearsal teaches.
- This issue, recording the outcome.

## Definition of done

- One real feature's plan runs through the driver: every ready slice
  built and integrated with no turn boundary stopping to ask, on the
  feature's branch, with the records committed at each integrate.
- A deferral path is observed — a genuine user gate, or a deliberately
  planted one — and the run ends by putting the queue to the user.
- The default branch shows nothing from the run.
- Findings are folded into the skill and this issue is refiled done with
  the outcome.

<!-- sokf:links -->
[sokf:issue-024-feature-request-the-workflow-cannot-run-unattended]: /knowledge/issues/done/issue-024-feature-request-the-workflow-cannot-run-unattended.md
