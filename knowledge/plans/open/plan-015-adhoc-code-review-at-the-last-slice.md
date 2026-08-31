---
type: AdhocPlan
id: plan-015-adhoc-code-review-at-the-last-slice
title: Integrate runs /code-review once, at the last slice
description: The per-slice /code-review in integrate becomes one feature-wide review at the last slice, over the whole diff, with findings returning to build as today.
lifecycle: open
links:
  - rel: relates-to
    to: plan-013-feature-workflow-autonomy
    note: Plan-013 edits the same integrate skill; whichever lands second rebases its step list.
---

# Plan: integrate runs /code-review once, at the last slice

## Context

Integrate runs `/code-review` on every slice, and the review is the most
expensive step in the phase — in time and in tokens. A thirteen-slice
feature pays for thirteen reviews of overlapping code, and the unattended
loop [plan-013][sokf:plan-013-feature-workflow-autonomy] adds will pay it
without a human noticing. The review moves to the last slice and widens to the whole
feature, so every line is still reviewed exactly once.

## Facts

- `.claude/skills/integrate/SKILL.md:33` runs `/code-review` in the
  per-slice step list (step REVIEW THE DIFF), and line 35 (WRITE FINDINGS)
  records its findings per `schema-code-review`; line 14 reads that schema
  on every invocation. `pack/knowledge/skills/integrate/SKILL.md` is
  byte-identical (`diff` is empty).
- Nothing else invokes the review: `grep -rn 'code-review' pack/agents
  .agents/superdev.md` and the other skills return only the integrate
  lines and schema filing prose.
- `/accept` runs `/security-review` conditionally and no `/code-review`
  (`.claude/skills/accept/SKILL.md:27`), so a last-slice review duplicates
  nothing.
- Integrate already knows the last slice: step MARK THE SLICE DONE branches
  on "Last slice?" to settle the plan's lifecycle.
- The skill grammar's `when` attribute takes `if …` on steps — five steps
  in `.claude/skills/` carry one today (e.g. `/bootstrap`'s
  `when="if new project"`).
- Slice-scoped verification is the rest of the step list — cases,
  done-check, contract check, build and tests — and stays per slice.
- Plan-013's slices edit the same integrate skill (the commit-at-merge
  step), on this branch, unmerged.

## Goal

A feature's code is reviewed by `/code-review` exactly once, at the last
slice's integrate, over the whole feature diff.

## Outcomes

- O1 — integrate on a non-final slice invokes no `/code-review` and reads
  no code-review schema.
- O2 — integrate on the last slice reviews the feature's whole diff
  against the merge target, and its findings return to build before the
  merge, exactly as any failed check does today.

## Non-goals

- Changing what else integrate verifies per slice: cases, done-check,
  contract check, build, tests and UI check all stay.
- Touching `/accept` — it stays the feature-level acceptance and gains no
  review.
- The review's effort level or its tooling; `/code-review` is invoked as
  today, only less often and over a wider diff.
- Any code change; this is skill content.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | The REVIEW THE DIFF step carries `when="if the last slice"` and reviews the whole feature diff against the merge target | O1, O2 |
| FR-2 | The code-review half of WRITE FINDINGS carries the same condition; the investigation half stays per slice | O1, O2 |
| FR-3 | The `schema-code-review` bootstrap read carries `when="if the last slice"` | O1 |
| FR-4 | Review findings, all kinds, return to build before the last slice merges | O2 |
| FR-5 | The live skill and the pack copy stay byte-identical | O1, O2 |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | The last-slice review covers the whole feature diff | review only the final slice's diff | earlier slices would merge unreviewed forever; one wide review keeps every line reviewed once — chosen by the user |
| D-2 | Every finding returns to build, as today | file simplifications as issues and block on correctness only | one rule for findings, no new vocabulary — chosen by the user |
| D-3 | The condition is a `when="if the last slice"` attribute | conditional phrasing inside the task text, as MARK THE SLICE DONE does | the grammar has one place for conditions and five skills already use it on steps |

## Workstreams

### W1: The integrate skill

Depends on: none.

1. Condition the review — in `pack/knowledge/skills/integrate/SKILL.md`,
   add `when="if the last slice"` to REVIEW THE DIFF and widen its task to
   the whole feature diff against the merge target; findings keep
   returning to build unapplied.
2. Split WRITE FINDINGS — the code-review findings sentence moves under
   the same condition; writing an investigation for a failure stays
   unconditional.
3. Condition the bootstrap — the `knowledge/schemas/code-review.md` read
   becomes `when="if the last slice"`.
4. Mirror — apply the same edit to `.claude/skills/integrate/SKILL.md` and
   confirm the two files are identical.
5. Record — a CHANGELOG.md Unreleased line: integrate reviews once per
   feature, at the last slice, over the whole diff.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `pack/knowledge/skills/integrate/SKILL.md` | modified — conditioned review, split findings, conditioned bootstrap | W1 |
| `.claude/skills/integrate/SKILL.md` | modified — mirror of the pack edit | W1 |
| `CHANGELOG.md` | modified — Unreleased entry | W1 |

## Acceptance

| Check | Verifies |
|-------|----------|
| `grep -c 'code-review' .claude/skills/integrate/SKILL.md` finds each remaining mention on a line carrying `when="if the last slice"` or inside the conditioned steps' text, confirmed by reading the three lines | FR-1, FR-2, FR-3 |
| The REVIEW THE DIFF task names the whole feature diff and the merge target | FR-1 |
| The step still says findings return to build unapplied | FR-4 |
| `diff .claude/skills/integrate/SKILL.md pack/knowledge/skills/integrate/SKILL.md` prints nothing | FR-5 |
| `superdev validate` reports PASS with 0 errors | FR-1, FR-2, FR-3 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- `knowledge/plans/index.md` lists this plan, and its `lifecycle` reads
  done.
- `CHANGELOG.md` names the change under Unreleased.

## Risks

- Risk: plan-013's integrate edits and this one collide on the same file.
  Mitigation: whichever lands second rebases the step list; the two touch
  different steps (commit-at-merge against review). Early signal: a merge
  conflict in `integrate/SKILL.md`.
- Risk: one wide review on a large feature exceeds what a single
  `/code-review` handles well. Accepted: the user weighed it against
  per-slice cost; `/code-review`'s effort levels remain available to the
  operator.
- Risk: a defect an early per-slice review would have caught now merges to
  the feature branch and is found at the end. Accepted deliberately —
  nothing unattended reaches the default branch
  ([ADR-021][sokf:adr-021-nothing-unattended-reaches-the-default-branch]),
  so the exposure ends at the feature branch, and the finding still
  returns to build before the feature closes.

## Out-of-band notes

The unattended driver plan-013 adds will inherit this behaviour with no
edit of its own: it invokes `/integrate` per slice, and the condition
lives inside the skill.

<!-- sokf:links -->
[sokf:adr-021-nothing-unattended-reaches-the-default-branch]: /knowledge/adrs/active/adr-021-nothing-unattended-reaches-the-default-branch.md
[sokf:plan-013-feature-workflow-autonomy]: /knowledge/plans/done/plan-013-feature-workflow-autonomy.md
