---
type: Issue
id: issue-092-a-requirements-review-has-no-declared-surface
title: A requirements review has no declared surface
description: The review prompt directs a transitive closure with no fixed point, so a review reads the whole subsystem instead of the change in its context, and its cost is bounded only by the deadline that kills it.
kind: feature
lifecycle: done
links:
  - rel: references
    to: issue-091-an-isolated-role-loses-everything-at-its-deadline
    note: The same deadline, for surviving it rather than for not needing it.
---

# Feature: a requirements review reads a declared surface

## Summary

A requirements review is told to follow the records' relationships and inspect
every affected contract, source file, test obligation, and documentation
surface. Each thing it reads names more things, so the instruction describes a
closure that terminates at the repository boundary. The review reconstructs the
subsystem rather than examining the change in the context of what exists, and
what stops it is the deadline. Point the review at what the plan names.

## Context

Two requirements reviews of `plan-077-sokf-file-tool-parity` were measured from
their artifacts. The five records under review are a constant; everything else
varies:

```text
93TqL7  complete  1117 s   37 reads   5 records   32 outside   319 KB
uQS2op  timeout   2401 s   51 reads   5 records   46 outside   434 KB
```

Doubling the deadline from 1200 to 2400 seconds produced fourteen more reads and
another 115 KB, and still no result. The extra time was spent reading, not
concluding.

The reads outside the records fall into three tiers. Seven are documentation
surfaces the plan's own Documentation changes section names, and verifying them
is the review's job. About ten are one hop from a record, such as the source
file whose declarations a contract materializes. The remaining twenty are
reached at two or more hops and are named by nothing: `main.rs`, `cli.rs`,
`sokf_cli.rs`, `contract-002-cli-superdev`, `project-overview`, `CHANGELOG.md`,
`evals/sokf/behavioral.json`, `scripts/check-docs.mjs`,
`crates/lib/superdev-core/src/validate/sokf.rs`, `scripts/sokf-eval.mjs`,
`coding-standards`, `.superdev/lock.toml`, the `pack/` mirror of an extension,
and `.pi/extensions/superdev/skills/scope/SKILL.md`.

That last one is the prompt governing the role that authored the plan. It was
read at minute 38 of a 40-minute run. The final two reads before termination
were an evaluation fixture and the coding standards, so the review was still
widening when it was killed.

The prompt directs this:

```text
Read both records, follow their relationships, and inspect every affected
contract and ADR, source/interface map, test obligation, documentation
surface, scope boundary, and internal-consistency claim.
```

"Affected" is reflexive. A contract's Definition is materialized from source
under ADR-042, so reading a contract implies reading its source file; that file
names a module; the module names its tests. Nothing in the instruction says
where to stop.

A bound already exists in the plan and is unused. Every work block declares an
`Areas:` line naming its contracts, source files, tests, and documentation
surfaces, and the plan states its primary implementation and evidence files
outright. The authoring role writes that surface into the document the reviewer
is handed, and nothing tells the reviewer it is a boundary rather than a
starting point.

The workflow already solves this for the other reviewer. `code-review` receives
`superdev_review_diff`, inventories the changed paths, pages through them, and
cannot report clean until the required paths are covered. Its surface is
enumerated before it starts. `requirements-review` receives `read` and a
closure instruction.

## Behaviour

The reviewer is pointed at what the plan names, and reads further only when a
specific finding needs it.

The plan already names that reading. It states the contracts and ADRs it
changes, its work blocks list the files each one touches, and its documentation
map names the surfaces it updates. The reviewer is handed that document and is
then told to inspect every affected thing, so what the plan names reads as a
starting point rather than as a boundary.

Reading further stays available and unrestricted. A finding sometimes depends
on something the plan failed to name, and a plan that omits an affected file is
itself a finding. What changes is that the wider reading serves a finding the
reviewer is writing rather than a survey it is conducting. The reviewer judges
what a finding needs, and no rule decides that for it.

The instruction to discover every actionable finding in one pass goes too. It
presses in the same direction as the closure, and the correction loop already
tolerates a partial set: a finding missed in one pass is found in the next,
within the configured review-cycle budget.

The shape this needs is already in the family. `accept.md` permits one hop and
then closes it: "Assess only that candidate and its canonical records."
`build.md` closes its own with "Never broaden approved Areas merely to absorb a
change." `requirements-review.md` is the only prompt of the five that opens
such a reading and never closes it.

The relationship to the deadline is worth stating plainly. Raising
`isolated_role_timeout_seconds` from 1200 to 2400 was tried and produced a
longer failure. A review whose reading has no floor will consume whatever
ceiling it is given, so the ceiling is not the lever.

Two structural amplifiers deserve consideration but are not this issue's
subject. A plan changing two contracts pulls in both contracts and the source
their Definitions materialize from, which was the largest single read observed.
A plan triggering seven documentation surfaces pulls in all seven on every
review. Both are legitimate obligations, and both argue for what a plan may
reasonably carry rather than for how a review is directed.

## Scope

What a requirements review is told to read.

- In: the wording of `requirements-review.md`.
- Out: the task string, which already names the two records the reviewer reads
  first and needs nothing added.
- Out: whether the reviewer reports coverage of what it read. The checklist
  already requires evidence for each of its seven areas.
- Out: a rule deciding which reads are permitted. The prompt states a
  preference, and the reviewer judges what a finding needs.
- Out: what a review returns when its budget runs out. The parent holds the
  timeout and terminates the child from outside, so a review is never told its
  deadline and cannot report on its own termination.
- Out: the deadline's value, which raising has already been shown not to fix.
- Out: keeping partial work when a role is killed, which
  [the deadline issue][sokf:issue-091-an-isolated-role-loses-everything-at-its-deadline]
  covers. That issue makes a deadline survivable; this one makes it
  unnecessary.
- Out: `code-review`, whose enumerated diff surface is a stronger design than
  this issue adopts, and `scope.md` and `build.md`, whose reading instructions
  carry no unbounded closure.
- Out: what makes a finding worth blocking on, which is a separate question
  about the bar rather than the surface.

## Resolution

Done by rewording `requirements-review.md`. The closure instruction is replaced
by two sentences:

```text
Read those two records, the contracts and ADRs the plan changes, the files its
work blocks list, and the documentation it updates. Only read anything else
when a specific finding needs it.
```

The seven concerns the deleted list enumerated were already stated in the
prompt's second paragraph, which requires a completed checklist for each. The
deleted copy added no obligation and set a reading order with no floor.

`Do not fail fast. Discover every actionable finding in this pass.` is deleted
with it, for pressing in the same direction. A finding missed in one pass is
found in the next, within the configured review-cycle budget.

No rule decides which reads are permitted, and no coverage is enforced. The
reviewer judges what a finding needs; the prompt states a preference. Neither
the task string, the tool set, nor the deadline changed.

The fix is unmeasured. The evidence here is two reviews of one plan, and the
next real requirements review is the test of whether the wording holds. If a
review still runs long, the remaining lever is the structural amplifier this
issue names and excludes: a plan whose work blocks list twenty files has a
twenty-file reading whatever the prompt says.

## Comments

Filed from a live session after a review consumed a doubled deadline and
returned nothing. The read sequences of two reviews were extracted from their
artifacts and compared: both read exactly five record files, and the difference
between a completed review and a timed-out one was entirely in the surrounding
surface.

One limitation is worth recording. Two reviews of one plan establish that this
plan's review has no floor; they do not establish the distribution across
plans. The mechanism, however, is in the prompt rather than in the plan, so it
applies wherever the closure is followed.

<!-- sokf:links -->
[sokf:issue-091-an-isolated-role-loses-everything-at-its-deadline]: /knowledge/issues/open/issue-091-an-isolated-role-loses-everything-at-its-deadline.md
