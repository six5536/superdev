---
name: accept
description: Reconstruct the reviewed candidate context, run Superdev ACCEPT, and explain approval or routing. Invoke explicitly with /skill:accept.
disable-model-invocation: true
allowed-tools: read sokf_search sokf_graph superdev_run_phase
---

# Superdev ACCEPT

Examples:

- `/skill:accept`
- `/skill:accept reassess the corrected candidate`
- `/skill:accept inspect the pending decision`

## Invocation state

Assume no earlier workflow discussion is present. Pi appends text after `/skill:accept` as a final `User:` line below this skill. Treat that line as optional assessment guidance, never as authority to weaken the approved plan or acceptance criteria.

ACCEPT assesses the immutable candidate produced by BUILD. The canonical plan and linked issue define what must be accepted. The deterministic tool owns candidate identity, evidence, phase transitions, trusted confirmation, and local integration.

## Reconstruct context

1. Call `superdev_run_phase` with `phase: "accept"` and `action: "inspect"` before taking any action.
2. If inspection lists multiple workflows, read their plans and linked issues, recommend the semantic match, and ask the user when ambiguity remains. Never choose by numeric identity alone.
3. Read the selected canonical plan and linked issue. Summarize the approved outcome and acceptance criteria relevant to the assessment.
4. If the tool reports SCOPE or BUILD, explain the current phase and use the corresponding skill. Do not force ACCEPT.

## Run and explain ACCEPT

Call `superdev_run_phase` with `phase: "accept"` and:

- `action: "run"` for a fresh assessment;
- `action: "retry"` only after discussing a paused, blocked, or failed outcome;
- exact `issue`, `plan`, and `workBranch` values only when selecting among paused workflows.

Interpret tool results as follows:

- `selection-required`: select the exact workflow semantically; do not guess.
- `routed` to BUILD: explain the within-scope correction that invalidated acceptance evidence, then use `/skill:build`.
- `routed` to SCOPE: explain the intent-changing finding, then use `/skill:scope`.
- `findings`: inspect persistent questions and discuss one finding at a time with a recommended response.
- `paused`, `blocked`, or `failed`: explain the stage, diagnostic or artifact path, preserved-work state, recommendation, and valid recovery operations.
- `ready-for-approval`: summarize the assessment and ask whether to accept or request one routed change.
- `accepted`: report that deterministic local integration completed. Do not claim a push, release, or branch deletion.
- `busy`: wait or offer cancellation; do not start another phase run.

At `ready-for-approval`:

- For acceptance, call `approve` only after the user explicitly chooses acceptance. Trusted Pi UI provides the authoritative confirmation when configured.
- For a within-scope change, call `record-answer` with `destination: "build"` and the exact confirmed change, then call `submit-answers`.
- For an intent-changing request, call `record-answer` with `destination: "scope"` and the exact confirmed change, then call `submit-answers`.

Use `revise-answer` to reopen a saved answer and `cancel` to pause while preserving partial work.

Use only `superdev_run_phase` for workflow operations. Do not invoke isolated phase roles, low-level workflow-control tools, or Git mutation commands.
