---
type: BugReport
id: issue-029-bug-contract-design-writes-verbose-prose
title: contract-design writes contracts as verbose prose
description: Contract documents come out as long prose where a contract needs precision — clear normative statements and constructs that aid them, such as tables, lists and typed shapes.
lifecycle: open
---

# Bug: contract-design writes contracts as verbose prose

## Summary

`/contract-design` tends to produce contracts as long prose. A contract
must be clear and precise; prose buries the normative statements, and
the constructs that carry precision — tables, lists, typed shapes,
RFC 2119 sentences — go unused.

## Environment

- Version/commit: main / 05f8731
- Platform: any session running the SOKF-carried skill set; the defect
  is prose, in `.claude/skills/contract-design/SKILL.md` and possibly
  the contract schemas it follows

## Steps to reproduce

1. Run `/contract-design` on a framed issue that touches a contract.
2. Read the contract document it writes or updates.

## Expected behaviour

Contracts are clear and precise: normative statements use RFC 2119
modal verbs, and the document leans on constructs that aid precision —
tables, bullet lists, typed shapes, code blocks — over paragraphs.
Precise criteria: TBD at framing.

## Actual behaviour

Contract sections arrive as extended paragraphs; the requirements are
embedded in narrative and take effort to extract.

## Root cause (if known)

TBD. Candidate causes: the skill's prose sets no style requirement for
contract text; the contract schemas mark most sections `content: prose`,
which invites paragraphs. May share a cause with
issue-028 (the phase settles its output without the user's review, so
verbosity is never pushed back on).

## Proposed fix / workaround

- Fix: TBD — settled in CONTRACT-DESIGN; likely the skill's prose, the
  contract schemas, or both.
- Workaround: ask for a rewrite of the contract after the phase runs.

## Regression risk

TBD. The contract schemas govern every existing contract on file; a
schema change may make settled contracts fail validation.
