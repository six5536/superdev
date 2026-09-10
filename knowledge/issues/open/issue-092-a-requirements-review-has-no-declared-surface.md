---
type: Issue
id: issue-092-a-requirements-review-has-no-declared-surface
title: A requirements review has no declared surface
description: The review prompt directs a transitive closure with no fixed point, so a review reads the whole subsystem instead of the change in its context, and its cost is bounded only by the deadline that kills it.
kind: feature
lifecycle: open
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
what stops it is the deadline. Give the review a surface it can finish.

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

A requirements review knows which surface it is answerable for before it
begins, and that surface is finite.

The plan already names it. A review directed at the work blocks' declared areas,
the contracts and ADR the plan changes, and the documentation surfaces its
documentation map triggers has a definite extent, and a reviewer can report
having covered it.

Reading beyond that surface stays available, because a finding sometimes
depends on something the plan failed to name, and a plan that omits an affected
file is itself a finding. What changes is that the wider reading serves a
question rather than a survey, and that the reviewer can distinguish the two.

A review that cannot cover its declared surface within its budget should report
that rather than be killed mid-survey. What it covered and what it did not is
more useful than nothing, and it names the real problem, which is that the
surface and the budget disagree.

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

- In: the reviewer's instructions, the task string that carries the plan's
  declared surface, and whether the reviewer reports coverage of that surface.
- In: what a review returns when its surface exceeds its budget.
- Out: the deadline's value, which raising has already been shown not to fix.
- Out: keeping partial work when a role is killed, which
  [the deadline issue][sokf:issue-091-an-isolated-role-loses-everything-at-its-deadline]
  covers. That issue makes a deadline survivable; this one makes it
  unnecessary.
- Out: `code-review`, whose bounded diff surface is the design this issue asks
  requirements review to acquire.
- Out: what makes a finding worth blocking on, which is a separate question
  about the bar rather than the surface.

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
