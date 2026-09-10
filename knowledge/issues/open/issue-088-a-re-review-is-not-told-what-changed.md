---
type: Issue
id: issue-088-a-re-review-is-not-told-what-changed
title: A re-review is not told what changed
description: Every requirements review receives the same task string, so a review following a correction cannot tell which findings were corrected or what the human decided, and rediscovers the plan from nothing.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-087-a-correction-pass-is-told-to-author-a-plan
    note: The same correction cycle, for the role that applies findings rather than the one that re-reviews them.
---

# Feature: a re-review is told what changed

## Summary

SCOPE reviews a candidate, the user answers its findings, a correction applies
them, and a second review runs. That second review receives the same task
string as the first. Nothing tells it which findings were corrected, what the
human decided, or that it is a re-review at all, so it rediscovers the plan
from nothing. Tell it what changed, so it can verify the corrections first and
treat the remainder as confirmation.

## Context

Observed across three requirements reviews of `plan-077-sokf-file-tool-parity`
in one session. The third ran after a correction applied ten edits across the
plan, both contracts, the ADR, and the issue. Its opening reads were the same
survey the first review performed:

```text
sokf:issue-077 · sokf:plan-077 · contract-012 · adr-053 · contract-003
sokf:issue-082 · package.json · sokf-pi-adapter.test.mjs · mcp.rs · sokf:issue-083
```

The parent composes one task string for every pass:

```text
Review issue {issue} and plan {plan} in the complete immutable SCOPE candidate
{candidate} against baseline {firstBase}.
```

Nothing in it varies between the first review and the third. The parent knows
what varied: it holds the finding set it just corrected, the confirmed human
answers, and a `cycle` counter it increments after each correction. It passes
none of them to the reviewer.

The reviewer's prompt says nothing about re-review either. Its only mention of
correction concerns its own submission payload.

The revisions are supplied and unusable. `firstBase` is resolved once per
SCOPE run and every review receives it, so the child is launched with a base
and a candidate spanning the correction:

```text
LFIg68  base b0ba06c8 -> candidate c0e75692
93TqL7  base d6565c49 -> candidate 188286b5
```

But `requirements-review` has no tool that reads a diff. Its toolset is
`read, sokf_search, sokf_graph, superdev_submit_result`, and
`superdev_review_diff` is registered only when the child role is
`code-review`. The delta is named in the task and cannot be inspected.

## Behaviour

A review that follows a correction knows it is one. It receives the findings
that were corrected, the confirmed human answers that directed them, and which
cycle it is, so it can verify those corrections first and spend the rest of its
pass confirming that nothing else broke.

The seven-area checklist stays exhaustive, and this is the part worth being
explicit about. A correction edits several records at once, and a requirements
review asks whether the plan is coherent and complete rather than whether one
edit is correct. Editing a contract to satisfy one finding can contradict a
plan block nobody touched, and a review scoped to the changed lines would not
look there. What changes is where the reviewer starts and what it can conclude
quickly, not what it is answerable for.

The contrast with code review is deliberate and should stay. `code-review`
pages a bounded immutable diff and cannot report clean until every required
path is covered, because code correctness is mostly local and the surrounding
tree was reviewed before. Requirements review has no such guarantee, so it
keeps its whole-candidate obligation.

Giving `requirements-review` the ability to read its diff would help it orient,
and the base and candidate it already receives make that a small step. It
belongs as an aid rather than a scope: read the delta to know where to look,
then complete the checklist as now.

One thing should be measured before any of this is optimised for speed. The
re-review observed here reached its findings in less time than the first
review took, so the cost of a second pass may already be lower than it appears.
The argument for telling it what changed is that a reviewer which cannot
distinguish a first pass from a third cannot report on the corrections it was
convened to check, which holds whatever the timings say.

## Scope

What a requirements review is told when it follows a correction.

- In: the task string the parent composes for a review, and whatever carries
  the corrected findings, confirmed answers, and cycle number into it.
- In: whether `requirements-review` gains diff access as an orientation aid.
- Out: the seven-area checklist and the exhaustiveness obligation. Narrowing a
  requirements review to the changed lines is the alternative this issue
  rejects.
- Out: what the correction pass itself is told, which
  [the correction-prompt issue][sokf:issue-087-a-correction-pass-is-told-to-author-a-plan]
  covers.
- Out: `code-review`, whose diff-scoped design is correct for what it reviews.
- Out: the cycle budget and the exhaustion gate, which decide how many
  corrections may run rather than what each review knows.

## Comments

Filed from a live session while a third requirements review was running. The
task string, the reviewer's toolset, and the launch diagnostics were read
before filing: the base and candidate spanning the correction are supplied to
every review, and no review has a tool that can read between them.

The framing was considered and rejected once. Narrowing a re-review to the
changes would leave a correction free to break something outside its own diff,
because a requirements review judges the whole candidate rather than an edit.
Telling the reviewer what changed achieves the orientation without giving up
that guarantee.

<!-- sokf:links -->
[sokf:issue-087-a-correction-pass-is-told-to-author-a-plan]: /knowledge/issues/open/issue-087-a-correction-pass-is-told-to-author-a-plan.md
