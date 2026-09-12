---
type: Issue
id: issue-087-a-correction-pass-is-told-to-author-a-plan
title: A correction pass is told to author a plan
description: The batched correction role runs the full SCOPE authoring prompt, so it resurveys every record and source file before applying findings that already name the exact path, location, and requirement to change.
kind: feature
lifecycle: open
links:
  - rel: references
    to: issue-086-a-recommended-choice-is-prose-not-a-choice
    note: The selected choice label is the entire answer payload a correction role receives, which raises what those labels carry.
---

# Feature: a correction pass is told what to correct

## Summary

SCOPE runs its batched correction in an isolated child, which is right. That
child receives the full authoring prompt, which is not. It is told to read
both records and their relationships and to resolve requirements, contracts,
ADRs, source interfaces, and documentation surfaces, so it resurveys the
subject before applying findings that already name exactly what to change.
Give the correction pass instructions of its own.

## Context

Observed after answering three substantive findings on
`plan-077-sokf-file-tool-parity`. The correction role ran for 5 minutes 32
seconds across 20 turns: 14 reads, 1 search, and 10 edits. Several targets were
read more than once.

```text
4x  crates/lib/superdev-core/src/sokf/mcp.rs
3x  knowledge/plans/open/plan-077-sokf-file-tool-parity.md
3x  knowledge/contracts/public/active/contract-003-api-sokf.md
2x  knowledge/contracts/public/active/contract-012-api-sokf-pi-file-tools.md
1x  knowledge/adrs/active/adr-053-sokf-file-tools-delegate-to-pi.md
1x  sokf:issue-077-sokf-file-tool-parity
```

Those reads returned roughly 116,000 characters to reach 10 edits.

The findings it was applying already carried precise anchors. Each review
finding has `path`, `location`, and `requirement` fields, and all four were
populated:

```text
F4-server-instructions-name-removed-tool
  path       : crates/lib/superdev-core/src/sokf/mcp.rs
  location   : `impl ServerHandler for SokfServer` > `get_info()`
  requirement: contract-003-api-sokf P_no-read-alias
```

The role nonetheless opened that file four times.

The cause is that both passes are the same role. `isolated()` loads
`prompts/${role}.md`, and the correction pass runs as role `scope`, so it
receives `scope.md` in full. That prompt opens with "Prepare or correct SCOPE
only for the primary issue and canonical plan named by the task. Read both
records and their relationships before editing", then describes authoring a
plan from nothing. Correction appears once, as the final clause of that
paragraph: "When the task supplies one complete review finding set and
confirmed human answers, apply them together in one correction pass."

The differences between the two passes are carried entirely by the task
string, which the parent builds with the findings, the confirmed answers, and
the remaining mechanical findings as JSON. Nothing tells the role that the
anchors in that JSON are authoritative, or that a full survey is not required
when they are present. The stage is labelled `batched correction`; the
instructions describe authoring.

## Behaviour

A correction pass is told it is correcting. Its instructions treat each
finding's `path`, `location`, and `requirement` as the authoritative statement
of what to change, and direct the role to read what those anchors name rather
than resurvey the subject.

The role keeps the judgement that belongs to it. A finding's anchor says where
the problem is, not what the fix should be, and applying several findings in
one pass still requires understanding how they interact and honouring their
`dependsOn` order. Reading beyond the anchors stays available for that; what
changes is that it stops being the opening instruction.

A correction that cannot proceed from its anchors must still say so rather
than silently widen. A finding whose anchor is stale, or whose fix turns out
to reach further than the anchor suggests, is exactly the case where reading
more is correct. The role already returns `blocked` for conditions it cannot
resolve, which gives that case somewhere to go.

Isolation itself is not in question, and the record should say why. The
correction child read 116,000 characters to make its edits. In the parent
those bytes would persist for the rest of a session that must survive SCOPE,
BUILD, and ACCEPT, and would be summarized by compaction in place of workflow
state. The parent also holds the human approval gate, and a parent that
authored the plan would be advocating for its own work while brokering the
user's decision about it. Both reasons favour keeping the pen in a disposable
child. The waste is in the instructions that child receives, not in the
isolation.

Whether this becomes a separate prompt file or a correction section within
`scope.md` is an implementation question. A separate file matches how each
other role is addressed and lets the authoring prompt stop hedging between two
jobs; a section keeps one place to describe what SCOPE may and may not touch.

## Scope

What the batched correction pass is told.

- In: the instructions the correction pass receives, and whether they live in
  `scope.md` or beside it.
- In: the task string the parent composes for a correction, which is where the
  findings and answers arrive.
- Out: which model or role performs the correction. Isolation is deliberate
  and stays.
- Out: the review prompts and what findings must contain. The anchors this
  relies on are already required and already populated.
- Out: BUILD's correction pass, which applies code-review findings to source
  and has different obligations.
- Out: re-entering SCOPE at review after a failure, which concerns where a run
  starts rather than what a correction is told.

## Comments

Filed from a live session after a batched correction re-read the plan, both
contracts, the ADR, the issue, and one source file before editing. The prompt
loading was confirmed: one `scope.md` serves both authoring and correction,
and no correction-specific prompt exists.

One related observation is recorded here rather than in scope. The confirmed
answer a correction role receives is the selected choice label alone, so the
reasoning discussed at the human gate does not cross into the child. That
makes those labels load-bearing in a way
[the recommended-choice issue][sokf:issue-086-a-recommended-choice-is-prose-not-a-choice]
touches from the presentation side. Whether a correction pass should receive
more of the deciding context is a separate question from what its prompt says.

<!-- sokf:links -->
[sokf:issue-086-a-recommended-choice-is-prose-not-a-choice]: /knowledge/issues/done/issue-086-a-recommended-choice-is-prose-not-a-choice.md
