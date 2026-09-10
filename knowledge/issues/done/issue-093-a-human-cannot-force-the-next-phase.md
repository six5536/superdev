---
type: Issue
id: issue-093-a-human-cannot-force-the-next-phase
title: A human cannot force the next phase
description: The adapter refuses approval while review findings are open and offers no override, so a gate the contract assigns to the human is held by a reviewer that may never report clean.
kind: feature
lifecycle: done
links:
  - rel: references
    to: issue-092-a-requirements-review-has-no-declared-surface
    note: One reason a review may never clear the gate it holds.
---

# Feature: a human can force the next phase

## Summary

Superdev's phase gates are human decisions, and the service enforces them as
such. The Pi adapter adds a stricter rule: it refuses a SCOPE approval while
review findings are open, and offers no way past. A reviewer that keeps
producing findings therefore holds a gate the contract gives to the human, and
the human has no sanctioned exit.

## Context

The service treats scope approval as human authority alone. `transition.rs`
derives `human_scope_approved` from the transition being `ApproveScope` and the
owning Pi UI's capability being verified, and consults nothing about review
state. The contract agrees:

```text
P_scope-gate  WHEN SCOPE enters BUILD, the service SHALL require explicit
              human scope approval.
```

ADR-054 states the same position more broadly: gate evidence records human
authority alone, and no longer records whether a requirements review was clean,
because that judgement belongs to the reviewer.

The adapter is stricter. Approval refuses while a question queue is active:

```text
if (pending && pending.status === "active"
    && !pending.findings.every(f => f.id.startsWith("human-"))) {
    throw new Error("Resolve the active review findings before approval.");
}
```

Pausing the queue does not help. An earlier branch returns before approval is
reached whenever the queue is paused, and the only action that passes it is
`retry`, which reactivates the queue into the guard above.

The gate that ought to relieve this is unreachable on the path that needs it.
When a review returns any substantive finding, SCOPE opens a question queue and
returns before the cycle budget is consulted, so `max_scope_review_cycles` bounds
only the mechanical correct-and-re-review loop. The exhaustion finding, whose
choices are `Retry one cycle` and `Revise scope intent`, is offered only when a
review returns mechanical findings alone.

The observed run shows what that costs. Four requirements reviews ran against
one plan across roughly four hours, three of which ended at their deadline,
and the plan was never approved. A human who judged the plan good enough had no
way to say so.

The same shape appears elsewhere. ACCEPT approval requires a current assessment
in `runtime.lastOutcome`, which is parent memory rather than durable state, so
an assessment that ran before a restart no longer counts. BUILD reaches ACCEPT
only through its own completion transition. Each is defensible alone; together
they mean no phase boundary has a human override.

## Behaviour

A human can advance the workflow to the next phase at any gate the contract
assigns to them, and the workflow records that they did.

The refusal becomes a confirmation. Unresolved findings are a good reason to
stop and look, which is why the friction exists; they are not a reason the
person holding the gate may not proceed. A prompt naming what is unresolved,
and requiring explicit confirmation, keeps the friction and returns the
decision.

Forcing is recorded rather than silent. A plan approved over open findings
differs from one approved clean, and the difference belongs in the canonical
record where BUILD and ACCEPT can see it. The findings themselves already exist
in the review artifact and should be preserved with the decision rather than
discarded by it.

The authority requirement does not change. Forcing is a human act and needs the
interactive UI capability the service already demands, so a non-interactive
session cannot force anything.

What must not change is which transitions exist. Forcing means exercising a
legal transition without a precondition the adapter added, not inventing an
edge the state machine does not have. The service's transition table stays
authoritative.

One consequence should be stated rather than discovered. A workflow that can
always be forced can be forced carelessly, and the guard exists because
approving unread findings is a real mistake. The argument for the override is
that the alternative observed here is worse: a gate the contract grants the
human, held indefinitely by a process with no termination condition.

## Scope

Whether a human can advance a phase the workflow is refusing to advance.

- In: the adapter's approval guards for SCOPE and ACCEPT, the unreachable
  exhaustion gate, and how a forced advance is recorded.
- In: whether the question queue's paused state should block approval at all.
- Out: the service's transition table and its human-authority requirement, both
  of which already permit this.
- Out: why a review may never report clean, which
  [the review surface issue][sokf:issue-092-a-requirements-review-has-no-declared-surface]
  covers. That issue removes one cause; this one removes the trap whatever the
  cause.
- Out: abandonment, which is already a human-only path and already available.

## Resolution

Done in `aa59df6` on the default branch. A human forces a refused gate by
typing `/superdev-force`, with an optional reason as the command's argument.
The command names every unresolved item, requires a typed reason and an
interactive confirmation, supersedes the question queue with that reason, and
records the decision in the plan's completion evidence through the service's
new `--override-note`, which `workflow transition` accepts only at
`approve-scope` and `accept` and refuses elsewhere.

The override is a typed command rather than a tool action. `superdev_run_phase`
still refuses approval while review findings are active or an acceptance
assessment is absent, and no tool schema, skill, or prompt names the command,
so no model can invoke or learn of it. Documenting the exit where a model reads
it would have handed that model the escape the guard exists to deny it.

The unreachable exhaustion gate is reachable: SCOPE consults the review-cycle
budget on every review outcome, so a review returning substantive findings
reaches the same terminating decision as one returning mechanical findings
alone.

Bound by `P_gate-forceable`, `P_force-human-only`, `P_approval-tool-strict`,
`P_forced-gate-recorded`, `P_forced-gate-preserves-findings` and
`P_scope-review-bounded` on
[the workflow interface contract][sokf:contract-011-interface-workflow], and by
`P_workflow-override-note` and `P_workflow-override-note-refused` on
[the CLI contract][sokf:contract-002-cli-superdev]. The Rust test
`a_forced_human_gate_is_recorded_and_refused_elsewhere` proves the record is
written at the scope gate and the note refused at `complete-build`; the
extension smoke fails if the override reaches the phase tool's schema, any
skill or prompt, or if either approval refusal leaves the phase tool.

## Comments

Filed from a live session after four requirements reviews of one plan produced
no approval and no way to proceed. The service transition, the contract
promise, and the adapter guards were read before filing: the stricter rule is
the adapter's alone, and the service would have accepted the approval.

One observation is recorded rather than relied upon. Restarting the Pi session
discards the in-memory question queue, after which approval passes, so an exit
exists by accident. That it works through session state rather than through a
decision is itself an argument for an explicit override.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-011-interface-workflow]: /knowledge/contracts/internal/active/contract-011-interface-workflow.md
[sokf:issue-092-a-requirements-review-has-no-declared-surface]: /knowledge/issues/done/issue-092-a-requirements-review-has-no-declared-surface.md
