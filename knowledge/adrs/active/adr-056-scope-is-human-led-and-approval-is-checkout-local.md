---
type: Decision
id: adr-056-scope-is-human-led-and-approval-is-checkout-local
title: SCOPE is human-led and approval is checkout-local
description: SCOPE runs in the controlling conversation with per-step human permission, durable progress and revision-bound approval live in Git-ignored checkout-local records, and BUILD and ACCEPT execute in one persistent worker session that resets context between stages.
lifecycle: active
links:
  - rel: supersedes
    to: adr-054-the-workflow-core-owns-state-branch-and-refusals
    note: The minimal core stands; scope moves out of child roles, and progress and approval move out of the plan document into checkout-local records.
---

# ADR-056: SCOPE is human-led and approval is checkout-local

- Date: 2026-09-11
- Deciders: superdev maintainers

## Context

[ADR-054][sokf:adr-054-the-workflow-core-owns-state-branch-and-refusals] reduced
the core to state, branch, and refusals, and left the canonical plan carrying
progress that nothing parses. Two of its consequences became problems in use.

The plan document carries progress, so it also implies authority. A plan says
`phase: build`, and that phrase travels: a clone, a copy, or a restored file
carries it without carrying the human decision that produced it. Approval was
never bound to what the human read, so an edit after approval left the record
saying the same thing.

SCOPE ran through isolated children. A child cannot ask the user a question, so
scope decisions reached the human as a review result rather than as a
conversation, and the harness then arbitrated whether that result was clean
enough to advance. The human could not repeat an interview, skip a check, or
approve over an open finding without the harness deciding on their behalf.

Execution spawned a process per role. Each spawn lost its predecessor's context
and re-derived it from documents, and a deadline killed the work rather than
pausing it. Correction budgets were counted in memory, so a restart refilled
them.

## Decision

We will make SCOPE a human-led conversation, move durable progress and approval
into checkout-local records, and execute BUILD and ACCEPT in one persistent
worker session.

SCOPE runs in the controlling conversation and invokes the grill-me and
double-check skills there. It has no child roles. Every drafting, interview, or
review step needs explicit human permission, which may name a group of steps and
belongs to the current continuation alone: a pause or a resume asks again.
Repeated and skipped steps are recorded as what they were, so a skipped check is
never reported as passed. Discussion neither advances state nor approves
anything, and the human may approve a document over open findings.

Durable progress and approval live in `.superdev/workflows/`, excluded from Git
and local to the checkout. An approval names the document's exact byte hash and
the human input that authorised it; a plan approval also names the issue
revision it implements. An unexplained edit suspends the approval until the
actual diff is assessed: a formatting-only diff keeps the original input and
records the new compatible revision, and a substantive diff clears the affected
approvals while keeping recorded progress. Neither document lifecycle, plan
prose, nor Git history confers approval, so a fresh clone needs fresh approval
before BUILD. The plan keeps its own lifecycle and its implementation content.

Each approved document is published on its own, committing only that document
and its required generated indexes. The prepared commit is recorded before the
branch moves, so an interrupted publication is completed by verifying the saved
parent, paths, bytes, and branch rather than assumed to have succeeded.

Plan approval starts nothing. BUILD begins on separate human input, and the work
branch is created then. BUILD and ACCEPT run in one persistent worker session
that keeps its identity across stages, pauses, and restarts, and resets its own
context at stage boundaries so review and acceptance do not inherit the
implementation conversation or its verdicts. The worker holds no controller
capability, so it can neither approve a document nor claim ownership; its
questions reach the human through the controller. One writer owns the checkout
at a time. Correction budgets are consumed in the durable record, so no reset,
pause, or restart grants another attempt.

The workflow CLI becomes `apply` and `status` under protocol
`superdev-workflow/v3`. The v2 mutation verbs are removed rather than aliased.
The force-gate command is retired: it existed to override a review barrier, and
a human-led SCOPE has none.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Human-led SCOPE, checkout-local approval, one persistent worker | Approval is bound to what the human read and cannot travel; the human directs their own scope; execution keeps context and budgets across resets | Approval no longer survives a clone, and the CLI surface changes again |
| Keep ADR-054's plan-carried progress | No migration | A copied or restored plan keeps stating a phase nobody approved |
| Bind approval by signing the plan document | Approval travels with the document | A signature proves authorship, not that this human approved this revision here |
| Keep isolated SCOPE children and add a question channel | Smallest change to SCOPE | A child that must ask the user for every decision is the conversation, run at a distance |
| Keep per-role spawning for BUILD | No runtime work | Each role re-derives context, and a deadline destroys work instead of pausing it |

## Consequences

- Positive: approval names an exact revision and an authenticated human input, so a later edit cannot inherit it.
- Positive: the human directs SCOPE — repeating, skipping, and approving over open findings — and the record says what actually happened.
- Positive: execution keeps one session across blocks, review, and acceptance, and a context reset costs no progress and no retry budget.
- Positive: a stage that is interrupted is reported as interrupted, because the durable record is written after the stage settles.
- Negative: a fresh clone or a linked worktree inherits no approval and must obtain it again.
- Negative: local state can be lost by deleting an ignored directory; recovery is human reconstruction rather than inference from documents.
- Negative: `contract-002-cli-superdev` loses its remaining workflow verbs, changing a public surface again.
- Negative: the current conversation cannot reset its own context, so executing there discloses that limitation instead of providing clean stage context.
- Follow-ups: [issue-086][sokf:issue-086-a-recommended-choice-is-prose-not-a-choice] and [issue-091][sokf:issue-091-an-isolated-role-loses-everything-at-its-deadline] describe defects this decision removes and should be closed against it rather than separately.

<!-- sokf:links -->
[sokf:adr-054-the-workflow-core-owns-state-branch-and-refusals]: /knowledge/adrs/active/adr-054-the-workflow-core-owns-state-branch-and-refusals.md
[sokf:issue-086-a-recommended-choice-is-prose-not-a-choice]: /knowledge/issues/done/issue-086-a-recommended-choice-is-prose-not-a-choice.md
[sokf:issue-091-an-isolated-role-loses-everything-at-its-deadline]: /knowledge/issues/done/issue-091-an-isolated-role-loses-everything-at-its-deadline.md
