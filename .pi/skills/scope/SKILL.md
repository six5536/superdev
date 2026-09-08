---
name: scope
description: Interpret a free-form product request, establish one canonical issue and linked plan, and run exhaustive Superdev SCOPE review. Invoke explicitly with /skill:scope.
disable-model-invocation: true
allowed-tools: read write edit sokf_search sokf_graph superdev_run_phase
---

# SCOPE

Treat all text following `/skill:scope` as semantic user intent, not deterministic syntax. Valid input includes a feature request, an issue or plan reference, a requested scope correction, or no argument.

Examples:

- `/skill:scope add offline export with encrypted archives`
- `/skill:scope revisit the retry policy in issue-031`
- `/skill:scope` and then describe the desired outcome when asked

1. Ask for the desired outcome when intent is absent.
2. Search canonical SOKF knowledge for related issues and plans. Read likely matches; do not select by number or filename coincidence.
3. Recommend adopting one existing open issue, creating one issue, or revising the active SCOPE proposal.
4. Read `sokf:schema-issue` and `sokf:schema-plan` before authoring either record. Present the interpreted intent, recommendation, scope boundary, selected or proposed issue, and selected or proposed linked plan. Discuss one question at a time. Offer Confirm, Revise, or Cancel and state a recommendation.
5. Only after explicit confirmation, use SOKF-aware `write` or `edit` to author either missing record. If no suitable linked plan exists, author a new independently identified plan. The LLM chooses each valid, collision-free ID and all semantic content. Keep issue and plan IDs independent.
6. Ensure the selected open plan has exactly one canonical `rel: implements` link to the selected issue, `phase: scope`, and an issue-derived `work/...` branch. Never infer a plan ID from the issue ID.
7. Invoke `superdev_run_phase` with action `run`, phase `scope`, exact issue/plan/branch identity, and the bounded confirmed intent.
8. Use only `superdev_run_phase` for phase status, findings, answers, retry, approval, and cancellation. Explain typed outcomes in plain language. Discuss findings one at a time and recommend an answer before recording it.
9. At a clean approval gate, invoke action `approve` only after the user chooses approval. For a requested revision, record and submit the confirmed answer so the complete SCOPE correction and exhaustive re-review run.

Do not invoke `superdev_isolated_role`, `superdev_workflow_control`, `superdev_workflow_questions`, Git mutation commands, or the removed `/scope` command. Deterministic code validates and commits; it does not author canonical knowledge.
