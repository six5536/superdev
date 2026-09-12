---
name: scope
description: Use when the user asks to define or revise the scope and requirements of an issue or plan before implementation.
allowed-tools: read write edit bash sokf_overview sokf_search sokf_graph superdev_run_phase superdev_ask
---

# SCOPE

1. Call `superdev_run_phase` with `phase: "scope", action: "inspect"`. Select a local workflow by intent, not by matching numbers. Read its issue, plan when present, and saved progress. Missing local state is not prior approval.
2. For new work, find a suitable issue or propose an unused issue ID. Call `run` with `issue` only. Wait for human permission before drafting. If the service reports the wrong branch, ask the user to switch to the default branch; do not create a worktree or work branch.
3. For saved work, use `resume`. Review partial work and outstanding discussion. Ask fresh permission for the next step; do not restore unused permission from an earlier conversation. For changed intent in BUILD or ACCEPT, request `rescope` before editing.
4. Select or draft the initial issue under `select-issue` permission. Follow `../../../../skills/sokf-authoring/SKILL.md` for document authoring. Save the draft without committing it. Record the completed action with `record-step`, its exact `step`, and factual `note`.
5. Request `continue` for `interview-issue`; follow `../grill-me/SKILL.md`. Record answers and unresolved decisions. Then request `write-issue`, save the revised issue, and record that action.
6. Request `check-issue`; follow `../double-check/SKILL.md`. Record findings, including unresolved findings. A review does not give permission to correct the document.
7. Request `approve-issue`. The tool binds the human's UI choice or explicit chat approval to the named revision and publishes only that document and its required `indexes`. Do not request a second confirmation. Discussion and ambiguous agreement are not approval. The human can approve directly without a clean review; do not report omitted checks as passed.
8. Request `write-plan`. Choose the plan ID independently, then use `attach-plan` before writing it. The plan implements exactly this issue. Keep lifecycle, requirements, work blocks, verification, and implementation content in the plan—not approval, execution phase, current progress, or recovery markers.
9. Request `interview-plan`, then `update-plan`, then `check-plan`. Use the same interview, authoring, review, and result-recording steps as for the issue. Preserve findings instead of repeating reviews until they are clean.
10. Request `approve-plan`. Approval also binds the issue revision. If either document changed, inspect the actual diff. Use `assess-change` only with the previous hash and a supported formatting-only or substantive assessment. If uncertain, ask the human; do not normalise whitespace to infer equivalence.
11. At handoff, offer BUILD here, BUILD in one worker, or Exit SCOPE. Plan approval alone does not start execution or create a branch. If execution is unavailable, preserve approval and report that limitation.

At each decision, identify the saved step and recommend the next action. Offer Continue, Discuss, Do something else, and Pause. An explicit `steps` group can cover several named drafting/interview/check actions in order. Complete only that group. Honour human requests to repeat or skip actions; skipped actions are not passes. On Discuss, return to chat without reopening the same dialog. Save outstanding points with `record-discussion`. On Pause, release ownership and keep progress and valid approvals.
