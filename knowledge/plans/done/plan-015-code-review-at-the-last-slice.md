---
type: Plan
id: plan-015-code-review-at-the-last-slice
title: Integrate runs /code-review once, at the last slice
description: The per-slice /code-review in integrate becomes one feature-wide review at the last slice, over the whole diff, with findings returning to build as today.
lifecycle: done
links:
  - rel: relates-to
    to: plan-013-workflow-autonomy
    note: Plan-013 edits the same integrate skill; whichever lands second rebases its step list.
---

# Plan: Integrate runs /code-review once, at the last slice

## Goal

A feature's code is reviewed by `/code-review` exactly once, at the last
slice's integrate, over the whole feature diff. Integrate on any earlier
slice invokes no review and reads no code-review schema; on the last
slice it reviews the feature's whole diff against the merge target, and
its findings return to build before the merge, exactly as any failed
check does today.

Integrate runs `/code-review` on every slice today, and the review is
the most expensive step in the phase, in time and in tokens. A
thirteen-slice feature pays for thirteen reviews of overlapping code,
and the unattended loop [plan-013][sokf:plan-013-workflow-autonomy] adds
pays it with no human noticing.

The evidence the design rests on:

- `.claude/skills/integrate/SKILL.md:33` runs `/code-review` in the
  per-slice step list (step REVIEW THE DIFF), and line 35 (WRITE
  FINDINGS) records its findings per `schema-code-review`; line 14 reads
  that schema on every invocation.
  `pack/knowledge/skills/integrate/SKILL.md` is byte-identical (`diff`
  is empty).
- Nothing else invokes the review: `grep -rn 'code-review' pack/agents
  .agents/superdev.md` and the other skills return only the integrate
  lines and schema filing prose.
- `/accept` runs `/security-review` conditionally and no `/code-review`
  (`.claude/skills/accept/SKILL.md:27`), so a last-slice review
  duplicates nothing.
- Integrate already knows the last slice: step MARK THE SLICE DONE
  branches on "Last slice?" to settle the plan's lifecycle.
- The skill grammar's `when` attribute takes `if …` on steps — five
  steps in `.claude/skills/` carry one today, such as `/bootstrap`'s
  `when="if new project"` — so the condition is an attribute rather than
  prose inside the task text.
- Slice-scoped verification is the rest of the step list — cases,
  done-check, contract check, build and tests — and stays per slice.

Three decisions, all the user's: the last-slice review covers the whole
feature diff, because reviewing only the final slice's diff would leave
earlier slices unreviewed forever, and one wide review keeps every line
reviewed once; every finding returns to build, as today, rather than
splitting simplifications into issues, so findings keep one rule; and
the condition is a `when="if the last slice"` attribute.

Two risks are accepted. One wide review on a large feature may exceed
what a single `/code-review` handles well; the user weighed that against
the per-slice cost, and the effort levels remain available to the
operator. A defect an early per-slice review would have caught now
merges to the feature branch and is found at the end; nothing unattended
reaches the default branch
([ADR-021][sokf:adr-021-nothing-unattended-reaches-the-default-branch]),
so the exposure ends at the feature branch and the finding still returns
to build before the feature closes. Plan-013 edits the same integrate
skill on this branch, unmerged; the two touch different steps
(commit-at-merge against review), and whichever lands second rebases the
step list.

Out of scope: what else integrate verifies per slice, which all stays;
`/accept`, which remains the feature-level acceptance and gains no
review; the review's effort level and tooling; and any code change —
this is skill content. The unattended driver plan-013 adds inherits the
new behaviour with no edit of its own, because it invokes `/integrate`
per slice and the condition lives inside the skill.

## Contract changes

- none.

## Work blocks

### Block 1: The integrate skill

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: in `pack/knowledge/skills/integrate/SKILL.md`, REVIEW THE DIFF
  gains `when="if the last slice"` and its task widens to the whole
  feature diff against the merge target, findings still returning to
  build unapplied; the code-review half of WRITE FINDINGS moves under
  the same condition, while writing an investigation for a failure stays
  unconditional; the `knowledge/schemas/code-review.md` bootstrap read
  becomes `when="if the last slice"`. The same edit lands in
  `.claude/skills/integrate/SKILL.md`, and `CHANGELOG.md` gains an
  Unreleased line: integrate reviews once per feature, at the last
  slice, over the whole diff.
- Done-check: every remaining mention of the review sits under the
  last-slice condition, the two skill copies are identical, and
  `superdev validate` reports PASS with 0 errors.
- Cases:
  - manual: each `code-review` mention in
    `.claude/skills/integrate/SKILL.md` sits on a line carrying
    `when="if the last slice"` or inside a conditioned step's text,
    confirmed by reading the three lines — checks that a non-final
    slice invokes no review and reads no code-review schema.
  - manual: the REVIEW THE DIFF task names the whole feature diff and
    the merge target — checks that the last slice's review covers the
    feature.
  - manual: the step still says findings return to build unapplied —
    checks that review findings of every kind return to build before
    the merge.
  - manual: `diff .claude/skills/integrate/SKILL.md
    pack/knowledge/skills/integrate/SKILL.md` prints nothing — checks
    that the live skill and the pack copy stay byte-identical.
  - manual: `superdev validate` reports PASS with 0 errors on a clean
    checkout, `CHANGELOG.md` names the change under Unreleased, and
    `knowledge/plans/index.md` lists this plan with `lifecycle: done` —
    checks that the edit conforms and the records close.

<!-- sokf:links -->
[sokf:adr-021-nothing-unattended-reaches-the-default-branch]: /knowledge/adrs/active/adr-021-nothing-unattended-reaches-the-default-branch.md
[sokf:plan-013-workflow-autonomy]: /knowledge/plans/done/plan-013-workflow-autonomy.md
