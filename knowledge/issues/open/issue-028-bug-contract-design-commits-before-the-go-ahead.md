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

The phase works through its changes with the user — or at minimum
presents them for review — and commits only after the go-ahead.
Precise criteria: TBD at framing.

## Actual behaviour

The session updates the contracts, records the ADRs, commits, and then
asks one question:

```text
The contracts and ADRs are committed. Continue with /feature-plan?
```

## Root cause (if known)

TBD. Leading hypothesis: the go-ahead gate sits among gates the session
checks for itself, so nothing marks it as an interaction, and the
interview step binds to contested decisions rather than to the written
changes — a session that sees none skips straight to the commit.

## Proposed fix / workaround

- Fix: TBD — settled in CONTRACT-DESIGN.
- Workaround: interrupt before the commit, or review it afterwards with
  `git show` and ask for rework; the edits stay on the feature branch.

## Regression risk

TBD. `/frame` and `/integrate` share the commit-their-own-records
pattern; `/execute-feature-plan` must keep driving its phases without
new blocking questions.
