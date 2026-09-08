---
name: accept
description: Run and explain Superdev ACCEPT assessment, routing findings to BUILD or SCOPE when required. Invoke explicitly with /skill:accept.
disable-model-invocation: true
allowed-tools: read sokf_search sokf_graph superdev_run_phase
---

# ACCEPT

Treat text following `/skill:accept` as optional acceptance intent, never as authority to weaken the approved criteria.

Examples:

- `/skill:accept`
- `/skill:accept reassess after the corrected final review`
- `/skill:accept inspect the pending acceptance decision`

1. Inspect canonical state through `superdev_run_phase` when needed. Select an exact workflow semantically when more than one open workflow exists.
2. Invoke `superdev_run_phase` with phase `accept` and action `run` or `retry`. Supply exact issue, plan, and branch identities only when inspection reports multiple paused workflows.
3. Explain assessment and recovery outcomes in plain language.
4. Route within-scope corrections to BUILD and intent-changing findings to SCOPE. At a clean approval gate, discuss one choice at a time. Use `record-answer` with destination `build` or `scope` for a confirmed change, then submit the answer.
5. Invoke action `approve` only after the user chooses acceptance. Never claim acceptance unless the deterministic tool records it. Trusted UI confirmation remains authoritative when configured.
6. Use `superdev_run_phase` exclusively for persistent answers, cancellation, and continuation.

Do not invoke isolated phase roles, low-level workflow-control tools, Git mutation commands, or the removed `/accept` command.
