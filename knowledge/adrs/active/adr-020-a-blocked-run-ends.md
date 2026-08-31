---
type: Decision
id: adr-020-a-blocked-run-ends
title: A Blocked Run Ends Rather Than Pauses
description: A run that hits a question only the user can answer writes it into the plan's deferred decisions and ends, releasing the run state; resuming is a fresh invocation that re-reads the plan and the answers.
lifecycle: active
---

# ADR-020: A Blocked Run Ends Rather Than Pauses

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

An unattended run hits questions it must not answer itself: a gate that
returns to `/frame` or `/contract-design` is the user's. The run needs
somewhere to put such a question and a rule for what happens to the run
while the answer is pending. Holding the run open means an idle session
owns the repo's run state for hours; the lock from ADR-019 would refuse
every other run in the meantime.

## Decision

We will end a blocked run. The question goes into the plan's deferred
decisions section, the loop continues with slices the question does not
block, and when no slice is ready the run ends: state removed, lock
released, the deferred decisions put to the user in sequence. Resuming
is a fresh `/execute-feature-plan`, which re-reads the plan and the
recorded answers.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| End the run; the plan is the durable record | No idle run holds the repo; resume works from any later session | The user must re-invoke the skill after answering |
| Hold the run open awaiting an answer | Resume is implicit | An idle session owns the run state; the watchdog would kill it anyway |
| Ask mid-run and wait at the boundary | No deferred-decision bookkeeping | Reintroduces the attended boundary the feature exists to remove, once per question |

## Consequences

- Positive: the plan carries every open question and every answer, so a
  run is resumable from a fresh session with no state but the repository.
- Positive: a stale run-state file stays exceptional rather than routine.
- Negative: answers arrive in a batch at the end rather than unblocking
  slices mid-run.
- Follow-ups: the feature-plan format gains a deferred decisions section.
