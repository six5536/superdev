---
type: Issue
id: issue-085-scope-reauthors-a-plan-it-already-committed
title: SCOPE reauthors a plan it already committed when review fails
description: A requirements review that times out or fails discards a committed authoring pass, so the next SCOPE run reauthors the plan and may review a different candidate than the one that failed.
kind: bug
lifecycle: open
links:
  - rel: references
    to: idea-014-superdev-subprocesses-show-progress-and-control
    note: That idea calls the phases resumable drivers; this is one place resumption was never implemented.
---

# Bug: SCOPE reauthors a plan it already committed when review fails

## Summary

SCOPE authors a plan, commits it, then reviews it. When the review fails or
times out, the next SCOPE run starts again at authoring even though the
authored plan is already committed. The user pays for the authoring pass
twice, and the second pass may produce a different candidate from the one
whose review failed.

## Context

Reported after a requirements review ran for eighteen minutes on
`plan-077-sokf-file-tool-parity`.

[The subprocess-visibility idea][sokf:idea-014-superdev-subprocesses-show-progress-and-control]
already describes the phases as resumable drivers that carry work from their
current durable state to the next human decision. SCOPE does not resume from
its own committed authoring.

`runScopePhase` in `.pi/extensions/superdev/lib/phases.ts` is one linear pass
with no entry point after authoring:

```text
authoring → workflow commit → candidate and clean-tree check → requirements review
```

Each loop iteration chooses its task with
`correction ? <correction batch> : "Prepare issue X and plan Y from canonical
state"`. A fresh invocation has no correction, so authoring always runs first.
No branch enters at review.

An isolated-role timeout rejects with `isolated role timed out after Ns`, which
`pauseWithOutcome` turns into a paused workflow with released ownership. The
authored plan survives, because `workflow commit` already ran. What does not
survive is the knowledge that it survived.

Two related observations come from the same code.

The adapter's `WorkflowStatus.owner` type already declares
`scope_base_revision`, `candidate_revision`, and `verified_default_revision`.
No Rust code writes or emits any of them. Those are the fields a resume would
need, so the intent exists in the type and the implementation does not.

`pauseWithOutcome` contains exactly one return statement, `return false`. Its
callers assign that to `retryScope` and `retryBuild`, so
`if (retryScope) return runScopePhase(args, ctx, submitted)` and its BUILD
equivalent are unreachable. Whatever automatic retry those lines intended has
never run.

## Behaviour

A SCOPE run that follows a failed review starts at the review when the
committed candidate is still the one that failed, and starts at authoring when
it is not.

The precondition is already computed immediately before the review runs. That
code resolves the work branch to a candidate revision and refuses unless HEAD
equals it and the worktree is clean. A resume needs the same triple recorded
durably: the baseline revision, the candidate revision, and the fact that
authoring returned `complete`.

Two conditions must force authoring to run again.

- New intent. When the user supplies fresh intent, the recorded candidate no
  longer reflects what was asked, whatever the revisions say.
- A moved or dirty checkout. The recorded candidate is only usable while HEAD
  still equals it and the worktree is clean, which is the same check the
  review already performs.

Resuming does not weaken a gate. `P_scope-gate` requires explicit human
approval to leave SCOPE, and the isolated reviewer is fresh in either case: a
resumed run launches a new review child against the same immutable candidate.
Skipping authoring removes a repeated authoring pass, not a review.

The unreachable retry branches belong to the same control flow and should be
removed or connected in the same change. Leaving dead retry code beside a new
resume path invites a reader to believe retry already works.

The present behaviour is not catastrophic, and the record should say so.
`workflow commit` runs immediately after authoring succeeds, so the plan is
durable and the second pass re-reads a plan that already exists. Empty scope
snapshots are permitted, so the second pass can produce the same candidate.
The cost is a repeated model run of several minutes, and the risk is that
authoring is not idempotent: a role re-reading a complete plan may revise it,
and the review that follows then examines a different artifact from the one
that failed.

## Scope

Resuming a SCOPE run whose review did not complete.

- In: where `runScopePhase` may enter, what state records the resume point,
  and the unreachable retry branches beside it.
- In: whether the resume point belongs in the Rust transient claim. The
  adapter's declared `scope_base_revision` and `candidate_revision` suggest
  that was the original intent, and the state must survive the ownership
  release that pausing performs.
- Out: BUILD and ACCEPT resumption. BUILD carries block progress in its plan
  and deserves separate treatment.
- Out: the review's own duration, timeout configuration, and retry budgets.
  Making a review faster or more patient is a different question from not
  repeating the work before it.
- Out: what the progress panel shows while a phase runs.

## Comments

Filed from a live session, after the user observed that a failed requirements
review sends the next SCOPE run back through authoring. The control flow was
read before filing, and confirms it: authoring is unconditional on a fresh
invocation.

Two findings beyond the reported symptom are recorded above. The adapter
declares three revision fields that Rust never emits, and both automatic
retry branches are unreachable because the function feeding them always
returns false.

<!-- sokf:links -->
[sokf:idea-014-superdev-subprocesses-show-progress-and-control]: /knowledge/ideas/idea-014-superdev-subprocesses-show-progress-and-control.md
