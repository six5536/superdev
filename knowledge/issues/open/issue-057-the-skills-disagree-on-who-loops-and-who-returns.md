---
type: Issue
id: issue-057-the-skills-disagree-on-who-loops-and-who-returns
title: The skills disagree on who loops and who returns, so a literal reading of the pack cycles and the unattended driver duplicates build's loop
description: contract-design ends with an unconditional call back to /scope, the only cycle in the pack's skill graph, and /execute-plan loops over the plan's blocks calling /build while /build loops over the same blocks and then merges.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: Found at acceptance of I052.
---

# Bug: the skills disagree on who loops and who returns

## Summary

An agent reads a skill and does what it says. Two pairs of shipped
skills describe control flow that cannot be followed as written: one
pair forms a cycle, and the other runs the same loop twice. The second
pair is the unattended delivery path, which runs with nobody watching.

## Context

[I052][sokf:issue-052-the-workflow-carries-more-process-than-it-needs]
made `/contract-design` a sub-skill of `/scope` and `/build` the phase
that works every block. The two skill pairs kept the flow they had as
phases.

`pack/knowledge/skills/scope/SKILL.md` calls `/contract-design` at its
DECIDE THE CONTRACT CHANGES step, "if the contract changes name a
contract", and ends with a call to `/build`.
`pack/knowledge/skills/contract-design/SKILL.md` ends with
`<skill_call name="/scope" when="always">`. `/scope` opens by cutting a
branch off the default branch and interviewing the user, so an agent
reading the call literally cuts a second branch and re-runs the
interview. Of the five `skill_call` elements in the pack, this pair is
the only cycle.

`pack/knowledge/skills/execute-plan/SKILL.md` runs a loop "until no
block is ready", picking a block, calling `superdev run advance --next`
naming it, and running `/build` for that block in a subagent.
`pack/knowledge/skills/build/SKILL.md` runs its own loop "until every
block's Done is ticked", and after the loop updates onto the merge
target, verifies the whole change, merges and sets the plan to `done`.

The two loops cannot both be right. Either the driver's loop runs once
and its `superdev run advance` bookkeeping records one step for the whole
plan, or `/build` obeys the driver and merges after the first block.
ADR-050 says `/build` "works the blocks — tests, code, the block's own
tests — with no review, and after the last block runs the full build,
tests, lint and validate once ... and merges on the branch", and that
"`/execute-plan` drives `/build` unattended". That reads as one
invocation of `/build` over the whole plan, which is not what
`/execute-plan` describes.

## Behaviour

Each skill states one flow, and the two skills of a pair agree on it.

For the first pair, `/contract-design` is a sub-skill that returns to
its caller. Its closing `skill_call` to `/scope` is deleted, and the
handover it describes — the approved edits going to `/scope` for the
plan and the commit — is stated as its output, which the skill's own
`output` attribute and its MUST NOT rule already say. The pack's skill
graph then has no cycle.

For the second pair, one skill owns the block loop. Either `/execute-plan`
invokes `/build` once and `/build` keeps its loop, in which case the
driver's PICK A BLOCK and HANDLE A RETURN steps and its per-block
`run advance` calls move into `/build`; or `/execute-plan` keeps the
loop and `/build` works exactly one block, in which case the whole-change
verification, the merge and the plan close move out of `/build` into the
driver's end-of-run step. The retry bound, the deferral rule and the
end-of-run decision queue belong to whichever skill holds the loop.

ADR-050 says which one the project chose, and its wording is the input to
that decision rather than its conclusion.

## Scope

The four skills, in the pack and in `.claude/skills`, and the decision
ADR-050 records.

- In: the closing `skill_call` in `contract-design/SKILL.md`.
- In: the block loop, the retry bound, the deferral rule and the
  `run advance` bookkeeping across `execute-plan/SKILL.md` and
  `build/SKILL.md`.
- In: the whole-change verification, the merge and the plan close, which
  follow the loop wherever it lands.
- In: ADR-050's Decision, where it describes the two skills, or a
  superseding ADR if the project decides against it.
- In: `.claude/skills` copies of all four skills, which must not diverge
  from the pack.
- Out: `/scope`'s conditional call to `/contract-design`, which is a
  sub-skill call and correct.
- Out: `/accept`'s call to `/build`, which is conditioned on the user
  asking for a fix.
- Out: the run state machine and the Stop hook, which the driver drives
  either way.

<!-- sokf:links -->
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/done/issue-052-the-workflow-carries-more-process-than-it-needs.md
