---
type: Issue
id: issue-058-a-plan-case-marked-manual-is-executed-nowhere
title: A plan case marked manual is executed nowhere, because the step that ran manual cases retired with integrate
description: The retired /integrate ran the slice's cases from the plan including manual ones; /build runs tests and asks only that a case be implemented or marked manual, and /accept walks the contract criteria rather than the plan's cases, so a manual case covering no criterion is run by nobody.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: Found at acceptance of I052.
---

# Bug: a plan case marked manual is executed nowhere

## Summary

A plan case labelled `manual:` is a check no test performs, written on
the understanding that a person or an agent performs it instead. No step
in the current workflow performs one. The label now exempts the case from
the gate that would otherwise demand a test for it, and nothing takes its
place.

## Context

The retired `/integrate` skill carried a RUN THE SLICE'S CASES step:
"Run the slice's cases from the plan, including manual ones, and confirm
each covers the criteria it names". Its gate then asked that every case
have an implemented test "or the plan marks it manual". The step ran the
manual cases; the gate let them through because the step had run them.

[I052][sokf:issue-052-the-workflow-carries-more-process-than-it-needs]
retired `/integrate`. `pack/knowledge/skills/build/SKILL.md` kept the
gate, word for word, and kept no step that runs a case. `/build` runs the
block's own tests and the tests its change touches, then runs the full
suite once after the last block. A manual case satisfies the gate by
carrying its label.

`pack/knowledge/skills/accept/SKILL.md` walks "the criteria of every
contract the change touched on the merged code — each promise, and each
`AC_` criterion nested under it". It reads the contracts, not the plan's
cases. It is also optional and manual: its rules forbid it running
unasked.

A manual case that covers a contract criterion is therefore reached at
acceptance, through the criterion rather than through the case, and only
when the user asks for acceptance. A manual case that covers no contract
criterion is reached by nobody. The convention is in wide use: the plans
on file carry manual cases for branch cutting, for skill text, for
`git log` on a scratch repository and for the driver's loop, none of
which a contract criterion names.

`knowledge/schemas/plan.md` names no manual marker at all. The Cases
declaration asks for "the block's test cases, one per line, each citing
the contract criteria it covers", so the label `/build`'s gate depends on
is a convention the schema does not define.

## Behaviour

Every case in a plan is executed by a named step, or the plan does not
carry it.

Giving the cases back a runner puts a step in `/build` that runs the
block's manual cases as it runs the block's tests, or one after the last
block that runs the plan's manual cases against the merged change. That
keeps the convention and closes the gap where it opened. Removing the
exemption instead requires every case to have an implemented test, which
deletes the label and forces a check no test can perform to become a
contract criterion `/accept` walks.

Whichever is chosen, `schema-plan` states it: either the Cases
declaration defines the manual marker and what runs it, or it says every
case is a test case.

## Scope

The gate, the missing step, and the schema that defines a case.

- In: `/build`'s gate on implemented tests, in the pack and in
  `.claude/skills`.
- In: a step that runs a manual case, wherever the work places it.
- In: `knowledge/schemas/plan.md` and `pack/knowledge/schemas/plan.md`,
  which must define the marker or refuse it.
- In: the manual cases in the open plans, which need a runner or a
  rewrite.
- Out: `/accept`'s walk of the contract criteria, which is correct for
  what it covers.
- Out: the manual cases in plans already `done`, which are history.
- Out: the `testing-strategy` concept's manual smoke run, which
  `knowledge/development-commands.md` already names and which no plan
  case stands in for.

<!-- sokf:links -->
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/done/issue-052-the-workflow-carries-more-process-than-it-needs.md
