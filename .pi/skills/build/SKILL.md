---
name: build
description: Interpret optional execution intent and run Superdev BUILD strictly within an approved canonical plan. Invoke explicitly with /skill:build.
disable-model-invocation: true
allowed-tools: read sokf_search sokf_graph superdev_run_phase
---

# BUILD

Treat text following `/skill:build` as optional semantic execution intent.

Examples:

- `/skill:build`
- `/skill:build continue after the dependency outage`
- `/skill:build retry after fixing local credentials`

1. Inspect canonical state through `superdev_run_phase` when needed. Select an exact workflow semantically when more than one open workflow exists.
2. Ensure the selected workflow is in BUILD and its plan is approved.
3. Interpret instructions only within approved intent. If they change requirements, architecture, an API, a contract, or acceptance criteria, explain why and recommend `/skill:scope <change>` instead of silently expanding BUILD.
4. Invoke `superdev_run_phase` with phase `build` and action `run` or `retry`. Supply exact issue, plan, and branch identities only when inspection reports multiple paused workflows.
5. Explain typed progress, review, correction, recovery, and routing outcomes in plain language.
6. Use `superdev_run_phase` exclusively for question state, answers, cancellation, and continuation.

Do not invoke isolated phase roles, low-level workflow-control tools, Git mutation commands, or the removed `/build` command.
