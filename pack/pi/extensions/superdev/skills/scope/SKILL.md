---
name: scope
description: Interpret a user request, establish or revise one canonical issue and linked plan, and run exhaustive Superdev SCOPE review. Invoke explicitly with /skill:scope.
disable-model-invocation: true
allowed-tools: read write edit sokf_search sokf_graph superdev_run_phase
---

# Superdev SCOPE

Examples:

- `/skill:scope add offline export with encrypted archives`
- `/skill:scope revise issue-031 retry behavior`
- `/skill:scope` and then provide the outcome when asked

## Invocation state

Assume no workflow conversation is present in context. Pi appends the text after `/skill:scope` as a final `User:` line below this skill. That line is the requested outcome, not command syntax. If the line is absent or unclear, ask what outcome the user wants before changing anything.

Use these definitions:

- **Issue**: the canonical statement of the user-visible problem or opportunity.
- **Plan**: the canonical implementation scope and acceptance criteria for exactly one issue.
- **Canonical knowledge**: SOKF records under `knowledge/`, accessed through `read`, `sokf_search`, and `sokf_graph`.
- **Workflow phase**: one of SCOPE, BUILD, or ACCEPT, reported by `superdev_run_phase`.
- **Work branch**: `work/<issue ID without the issue- prefix>`. The issue determines this branch. The plan ID is independent.

Do not assume that an issue, plan, workflow owner, branch, or prior answer already exists.

## Establish context

1. Call `superdev_run_phase` with `phase: "scope"` and `action: "inspect"`.
2. If inspection reports an active or paused workflow, read its plan and linked issue before interpreting the request. If its phase is BUILD or ACCEPT, explain that a requirement change must return through SCOPE.
3. Search SOKF semantically for related issues and plans. Read plausible matches and inspect their relationships. Never select a record from matching numbers, filenames, or words alone.
4. When multiple records or workflows could apply, compare their intent and ask the user to choose only after recommending one.

## Agree on scope

Present all of the following in ordinary language:

- your interpretation of the requested outcome;
- whether to adopt an existing issue or create a new issue;
- what is inside and outside scope;
- the selected or proposed issue title and description;
- the selected or proposed plan title, approach, and acceptance criteria.

Discuss one unresolved decision at a time. Recommend one answer. Before authoring or revising canonical records, ask the user to **Confirm**, **Revise**, or **Cancel** the complete proposal. After the user selects **Revise**, present the revised complete proposal and require a new explicit **Confirm**; do not infer confirmation from sentiment or conversational agreement.

## Author canonical records

After confirmation only:

1. Read `sokf:schema-issue` and `sokf:schema-plan`.
2. Use SOKF-aware `write` or `edit` to create or revise the exact confirmed content.
3. Choose each collision-free ID independently. Never derive a plan ID from an issue ID.
4. Ensure the open plan has `phase: scope`, exactly one `rel: implements` link to the selected issue, and the issue-derived work branch.
5. If the issue exists but no suitable linked plan exists, create a new independently identified plan.

## Run and continue SCOPE

Invoke `superdev_run_phase` with:

- `phase: "scope"`;
- `action: "run"` for a normal run or `action: "retry"` only after discussing a reported failure;
- exact `issue`, `plan`, and `workBranch` values when starting or selecting a workflow;
- `intent` containing a bounded summary of the confirmed outcome.

Interpret tool results as follows:

- `selection-required`: select the exact workflow semantically; do not guess.
- `routed`: invoke the skill named by the returned phase.
- `findings`: inspect the persistent questions, discuss one finding at a time, recommend an answer, then use `record-answer`.
- `paused`, `blocked`, or `failed`: explain the returned stage, diagnostic path, preserved-work state, recommendation, and valid recovery operations before asking what to do. Never recommend retry while the reported precondition remains unresolved. `cancel` pauses and releases ownership; it never abandons the workflow.
- `ready-for-approval`: summarize the reviewed scope and ask whether to approve or request one revision.
- `busy`: wait or offer cancellation; do not start another phase run.

Use `revise-answer` to reopen a saved answer and `submit-answers` only after all answers are confirmed. A submitted revision runs one complete correction batch followed by exhaustive re-review.

Call `approve` only after the user explicitly chooses approval. The tool then asks for authoritative confirmation in trusted Pi UI.

Use only `superdev_run_phase` for workflow status, answers, retry, approval, and cancellation. Do not invoke isolated phase roles, low-level workflow-control tools, or Git mutation commands.
