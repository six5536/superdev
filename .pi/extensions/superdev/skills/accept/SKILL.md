---
name: accept
description: Use when the user asks for an acceptance review of a completed implementation against its approved plan.
allowed-tools: read sokf_search sokf_graph superdev_run_phase superdev_ask
---

# ACCEPT

1. Call `superdev_run_phase` with `phase: "accept", action: "inspect"`. Select the workflow by intent, not by matching numbers. Read the approved plan, its linked issue, the candidate, and the recorded verification evidence.
2. Confirm the phase is `accept`. For `accept-not-reached`, name the current phase and use `/skill:build` or `/skill:scope`. Do not force acceptance.
3. For saved work, use `resume` to reattach the persistent worker session.
4. Assess the candidate with `action: "run"` and `stage: "acceptance"`, supplying the assessment instructions in `note`. The stage resets context first, so it judges the candidate, the approved records, and the evidence rather than the implementation conversation or its verdicts. Inspect freely, but leave the checkout exactly as you found it: its Git state is compared before and after, and any change voids the assessment. For `assessment-void`, inspect and restore the checkout, then assess again.
5. Check each acceptance criterion in the approved plan against the actual candidate. Name any criterion you cannot verify, and say why. An unverifiable criterion is not a pass.
6. For findings that keep the approved intent, use `return-to-build` with the findings in `note`. This supersedes the candidate, so acceptance evidence cannot outlive the correction; the findings themselves are retained as `lastAssessment` for BUILD to correct from. Continue with `/skill:build`.
7. For findings that change the intent, use `rescope` with the requested change. Do not weaken the plan or its criteria here.
8. To close the work, use `action: "accept"`. Project policy alone decides whether a human must confirm; when it does, the service binds the human's explicit choice. Never claim acceptance that the service refused.
9. Report the accepted candidate and its branch. Acceptance does not merge, push, release, or delete the work branch; say so rather than implying the change is shipped.

Discuss one finding at a time with a recommended response. Use only `superdev_run_phase` for workflow and Git mutation. On Pause, the worker stops and progress, approvals, and the candidate remain.
