---
type: Issue
id: issue-091-an-isolated-role-loses-everything-at-its-deadline
title: An isolated role loses everything at its deadline
description: A role that reaches its timeout is killed with SIGKILL and returns nothing, so twenty minutes of completed work is discarded and the only recovery is to raise the timeout and repeat the same work from the start.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-085-scope-reauthors-a-plan-it-already-committed
    note: The same wasted repetition, for a run that restarts rather than a role that is killed.
---

# Feature: an isolated role keeps the work it finished

## Summary

An isolated role that reaches its deadline is killed and returns nothing. Every
minute it spent is discarded, and the only recovery available is to raise the
timeout and run the same work again from the beginning. A role should keep what
it completed, so a deadline costs the remainder rather than the whole.

## Context

Observed across one SCOPE run on `plan-077-sokf-file-tool-parity`. Three
requirements reviews ran against the same plan:

```text
LFIg68  requirements-review  timeout   1202 s
93TqL7  requirements-review  complete  1117 s
SKvhj3  requirements-review  timeout   1201 s
```

Two of three reached the 1200-second limit and returned nothing. The one that
succeeded finished 83 seconds inside it. Between them a full authoring pass ran
again for 1151 seconds, because
[a failed review sends the next run back through authoring][sokf:issue-085-scope-reauthors-a-plan-it-already-committed].
The user waited through roughly an hour of model time for one review result.

The deadline is enforced by killing the process:

```text
setTimeout(() => { timedOut = true; stop(); }, policyTimeoutMs(policy));
stop() → killProcessTree() → process.kill(-pid, "SIGKILL")
```

SIGKILL with no grace period, so the role has no window in which to submit what
it has. The parent then rejects with `isolated role timed out after Ns`.

Nothing survives that kill. The child is spawned with `--no-session`, so no
session file exists and there is nothing to continue from. The artifact
directory holds an `events.jsonl` observation log of tools and bounded text,
which records what happened but is not a conversation a role could resume.

The absence of a session looks deliberate rather than accidental. The
originating idea states that raw Pi events, reasoning, prompts, and tool
payloads are not persisted, and that only bounded summaries reach owner-private
artifacts. Persisting a child session would reverse that, which makes
resumption a policy decision rather than a configuration change.

The workaround reached for in this session was to double the limit from 1200 to
2400 seconds. That buys time without changing the shape of the failure: a role
that exceeds 2400 seconds still returns nothing, and the raised limit applies to
every role rather than the one that needed it.

## Behaviour

A role that reaches its deadline returns the work it completed, and a later run
continues from there rather than repeating it.

Two designs reach that, and choosing between them is the substance of this
work.

The first is checkpointing. A review completes seven checklist areas, and today
it can report only when all seven are done. If each completed area were
recorded as it finished, a deadline would preserve the finished areas and cost
only the remainder. BUILD already works this way: `P_build-commits-blocks`
requires a block's verification and commit before the next block starts, so a
BUILD timeout loses one block rather than the run. Reviews have no equivalent
granularity. This design needs no session persistence and no judgement about
whether a role is progressing.

The second is resumption. A role that timed out could continue in a new process
with its prior context restored. That requires persisting a child session,
which is what the current policy declines to do, and it requires deciding
whether a role is progressing or stuck, since resuming a looping role is worse
than killing it. The artifact already carries the evidence that distinguishes
them, including turn count, tool diversity, and repeated reads of the same
path.

Whichever is chosen, the per-attempt deadline stays as configured. A budget
that resets on resumption is not a budget, and `P_bounded-retries` requires
retry and correction limits to be positive project configuration that plans,
prompts, adapters, and models cannot override. A cumulative cap belongs with
whatever allows a second attempt.

Review freshness is not weakened by either design. `P_scope-gate` requires a
fresh isolated requirements review before human approval, and the concern it
protects against is a reviewer contaminated by the authoring role's context. A
review that continues its own reading of the same immutable candidate is the
same review proceeding, and a review that resumes from its own completed
checklist areas has examined the candidate rather than inherited an opinion
about it.

The behaviour belongs to every isolated role rather than to reviews alone.
SCOPE authoring, BUILD, code review, and ACCEPT assessment all end the same way
at their deadline, and BUILD's block commits are the only partial credit any of
them earns today.

## Scope

What an isolated role leaves behind when its deadline arrives.

- In: whether a role checkpoints completed units, whether a timed-out role can
  be continued, and which of those the workflow adopts.
- In: the grace period before termination, since a role given no warning cannot
  submit partial work however it is designed to.
- In: any cumulative bound that a second attempt requires.
- Out: the deadline's value. Raising `isolated_role_timeout_seconds` is the
  workaround this issue exists to replace.
- Out: how long a review takes, which the review's own scope and the plan's
  size decide.
- Out: where a SCOPE run re-enters after a failure, which
  [the re-authoring issue][sokf:issue-085-scope-reauthors-a-plan-it-already-committed]
  covers. That issue avoids repeating a completed authoring pass; this one
  avoids discarding an incomplete review.

## Comments

Filed from a live session after the second of two requirements reviews reached
the same 1200-second limit. The spawn arguments, the timeout path, and the
artifact contents were read before filing: the child persists no session, and
termination is immediate.

One consideration is recorded rather than decided. Not persisting a child
session is a stated privacy position, so a resumption design would reverse it
deliberately. Checkpointing completed units needs no such reversal, which is
one reason it may be the better of the two rather than merely the simpler.

<!-- sokf:links -->
[sokf:issue-085-scope-reauthors-a-plan-it-already-committed]: /knowledge/issues/open/issue-085-scope-reauthors-a-plan-it-already-committed.md
