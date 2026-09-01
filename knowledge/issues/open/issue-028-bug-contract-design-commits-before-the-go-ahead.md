---
type: BugReport
id: issue-028-bug-contract-design-commits-before-the-go-ahead
title: contract-design commits contract edits before the user has seen them
description: A /contract-design session writes the contracts and ADRs, commits them, and only then asks whether to continue with /feature-plan, so the review its go-ahead gate promises never happens.
lifecycle: open
---

# Bug: contract-design commits contract edits before the user has seen them

## Summary

A `/contract-design` session edits the contracts, records the ADRs,
commits, and only then asks its first question — whether to continue
with `/feature-plan`. The user's first sight of the changes is a commit
already on the branch.

## Environment

- Version/commit: main / 05f8731
- Platform: any session running the SOKF-carried skill set; the defect
  is prose, in `.claude/skills/contract-design/SKILL.md`

## Steps to reproduce

1. Run `/contract-design` on a framed issue.
2. Let the session run without interrupting it.
3. Read the transcript from the first contract edit to the first
   question put to the user.

## Expected behaviour

The phase interviews the user on each decision as it is made and
commits only after the user has approved the complete change set. As
criteria:

1. [event] WHEN a decision is to be recorded as an ADR, THE SYSTEM
   SHALL put the decision and its alternatives to the user before
   filing it, whether or not the session sees the decision contested.
2. [event] WHEN the contract and ADR edits are written, THE SYSTEM
   SHALL present the complete change set to the user and request
   approval before committing.
3. [unwanted] IF the user names rework, THEN THE SYSTEM SHALL apply
   the rework and present the revised change set again before
   committing.
4. [unwanted] IF the user withholds approval, THEN THE SYSTEM SHALL
   leave the edits uncommitted on the feature branch.

## Actual behaviour

The session updates the contracts, records the ADRs, commits, and then
asks one question:

```text
The contracts and ADRs are committed. Continue with /feature-plan?
```

## Root cause (if known)

Confirmed at framing. The skill's prose orders the go-ahead gate
before the commit step, but a gate reads as a self-check the session
satisfies by seeing no objection, and the interview step binds to
"each decision and its alternatives" — a session that sees no
contested decision skips it. Nothing in the prose marks either as a
mandatory interaction.

## Proposed fix / workaround

- Fix: settled in CONTRACT-DESIGN; the surface is the skill's prose in
  `.claude/skills/contract-design/SKILL.md` and its pack mirror,
  making the interview and the pre-commit approval explicit
  interactions.
- Workaround: interrupt before the commit, or review it afterwards with
  `git show` and ask for rework; the edits stay on the feature branch.

## Regression risk

`/execute-feature-plan` runs contract-design before a run begins,
attended, so a blocking question adds no unattended stall; the
driver's contracts-settled gate stands unchanged. `/frame` and
`/integrate` keep their commit-their-own-records pattern — out of
scope by decision.

## Comments

2026-09-01, framed. The review form is interview plus final approval:
the phase puts each ADR decision to the user as it is made, then
presents the complete change set and commits only on explicit
approval. Scope is `/contract-design` alone — frame's records are
co-written in conversation and integrate commits inside unattended
runs by design, so neither has the unseen-changes problem.

2026-09-01, contract-design. The fix is decided in ADR-028:
restructured process steps — the interview binds to every ADR
decision, a present-the-change-set step precedes the commit, and the
commit is conditioned on explicit approval. Hook enforcement and a
gate-owner vocabulary were rejected there. No contract changes for
this half; the fix is the skill's prose.
