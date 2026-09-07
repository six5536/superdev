---
type: Idea
id: idea-014-superdev-subprocesses-show-progress-and-control
title: Superdev subprocess workflows stay visible and resumable
description: Make Superdev drive subprocess work, review corrections, interruption recovery, and the next user decision through one visible resumable flow.
status: draft
---

# Idea: Superdev subprocess workflows stay visible and resumable

Make one workflow command carry work from its current durable state to the next
human decision. Superdev must keep subprocess activity visible, feed review
findings back into bounded correction loops, and recover interrupted work
without asking the user to copy findings, compare session IDs, cancel stale
ownership, or reconstruct a sequence of internal commands.

## Motivation

`/scope` currently clears the editor and silently awaits an isolated Pi child.
The child can work for several minutes without visible progress. When a
requirements review returns a finding, the command stops and leaves SCOPE
active. Continuing can require the user to identify the owning Pi session,
repeat the finding as `/scope` input, or run a cancel-resume-command sequence.

That interface exposes orchestration mechanics instead of the next meaningful
user decision. The same problems apply to correction cycles, BUILD review,
ACCEPT assessment, and interrupted nested processes.

## Sketch

Treat `/scope`, `/build`, and `/accept` as resumable phase drivers rather than
single subprocess launchers. Each driver reconstructs canonical state, performs
all safe automatic steps, loops bounded review findings back to the appropriate
isolated role, and stops only for a human decision, a genuine requirements
question, an exhausted retry budget, or an actionable failure.

Wrap user-awaited subprocess execution in one shared progress presenter. Use
Pi's existing notification, status, widget, and prompt APIs to show the plan,
current stage, elapsed time, recent completed stage, and available controls.
Consume structured child events for bounded progress without exposing private
reasoning or flooding the transcript.

Make interruption recovery one action. A resumed Pi session should reclaim or
rebind recoverable transient ownership safely and continue from canonical
state. When automatic recovery is unsafe, present one confirmation that names
the consequence and performs the required release and resume operations. Never
require the user to discover or paste a session ID.

Make `/superdev-status` distinguish the durable phase from live activity. Show
states such as running, awaiting approval, paused after findings, timed out, and
owned by another live session. Include exactly one recommended next action.

Give each subprocess class an explicit timeout and distinguish timeout,
cancellation, child failure, malformed output, review findings, and successful
completion. Always clear or replace progress UI in a `finally` path. Keep
daemon-like transport processes quiet unless startup fails or health changes.

## Trade-offs

- Automatic correction loops spend more model time before returning control, so
  each phase needs a small explicit retry budget.
- Safe ownership recovery requires liveness and identity checks rather than an
  unconditional takeover.
- Durable progress entries improve diagnosis but can add transcript noise; the
  default view must remain compact.

## Open questions

- Which child events provide useful progress without exposing private reasoning
  or unstable model text?
- Which findings can enter an automatic correction loop, and which findings
  require a human requirements decision?
- What proves that an owning Pi instance is stale enough for safe recovery?
- What timeout and retry budget applies to each isolated role and non-model
  subprocess?
