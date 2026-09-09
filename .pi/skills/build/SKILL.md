---
name: build
description: Reconstruct the approved workflow context and run Superdev BUILD strictly within its canonical plan. Use for matching user intent or invoke with /skill:build.
allowed-tools: read sokf_search sokf_graph superdev_run_phase superdev_ask superdev_file_issue
---

# Superdev BUILD

Examples:

- `/skill:build`
- `/skill:build continue after credentials were repaired`
- `/skill:build inspect the paused implementation`

## Invocation state

Assume no earlier SCOPE or BUILD conversation is present. Pi appends text after `/skill:build` as a final `User:` line below this skill. Treat that line as optional execution guidance, not as permission to change approved scope.

The canonical plan defines the implementation boundary. Its `rel: implements` link identifies the issue. BUILD may implement and correct that plan, but it may not add requirements, architecture, application programming interface (API) changes, contracts, or acceptance criteria that the approved plan does not contain.

## Reconstruct context

1. Call `superdev_run_phase` with `phase: "build"` and `action: "inspect"` before taking any action.
2. If inspection lists multiple workflows, read their plans and linked issues, recommend the semantic match, and ask the user when ambiguity remains. Never choose by numeric identity alone.
3. Read the selected canonical plan and its linked issue. Confirm that the tool reports BUILD and that the plan is approved for BUILD.
4. Compare the appended user guidance with the approved plan. If the guidance changes intent, explain the mismatch and recommend `/skill:scope <requested change>`. Do not pass the change to BUILD.

## Run and continue BUILD

Call `superdev_run_phase` with `phase: "build"` and:

- `action: "run"` for normal execution;
- `action: "retry"` only after the user has reviewed a paused, blocked, or failed outcome;
- exact `issue`, `plan`, and `workBranch` values only when selecting among paused workflows;
- `intent` only for execution guidance that fits the approved plan.

Interpret tool results as follows:

- `selection-required`: select the workflow semantically; do not guess.
- `routed`: explain why the workflow moved and invoke the skill named by the returned phase.
- `findings`: inspect the persistent question state and discuss one decision at a time.
- `paused`, `blocked`, or `failed`: explain the failed stage, diagnostic or artifact path, preserved-work state, recommendation, and valid recovery operations.
- `ready-for-approval`: BUILD and ACCEPT reached a human acceptance gate; hand control to `/skill:accept`.
- `accepted`: report that the branch is accepted and ready for the user to merge. Do not claim a push or release.
- `busy`: wait or offer cancellation; do not start another phase run.

Use `record-answer`, `revise-answer`, and `submit-answers` only for questions returned by the tool. Use `cancel` to pause while preserving partial work.

Use only `superdev_run_phase` for workflow operations. Do not invoke isolated phase roles, low-level workflow-control tools, or Git mutation commands.
