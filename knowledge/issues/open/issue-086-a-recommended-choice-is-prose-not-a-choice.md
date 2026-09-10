---
type: Issue
id: issue-086-a-recommended-choice-is-prose-not-a-choice
title: A recommended answer is prose beside the choices rather than a marked choice
description: The question UI prints its recommendation as a paragraph above an unordered choice list, so the user reads justification prose to work out which listed choice is being recommended.
kind: feature
lifecycle: open
---

# Feature: a recommended answer is a marked choice

## Summary

Workflow questions carry a recommendation, but the UI prints it as a paragraph
above the choices and never says which choice it names. The user reads several
lines of justification, then maps that prose back onto a list themselves. Put
the recommended choice first and mark it, so the recommendation is readable at
a glance.

## Context

Observed at the end of the requirements review for
`plan-077-sokf-file-tool-parity`:

```text
What text content must a successful `sokf_resolve_source` result carry
alongside its structured content?
Recommendation: Take the second choice and add the sentence to
`AC_resolve-schema` and to the plan's routed-schemas Requirements bullet. It
matches the existing `mutation_result()` convention in
`crates/lib/superdev-core/src/sokf/mcp.rs`, keeps the transport uniform for
the issue-082 mutation results that reuse it, and carries no semantic content
because the mirrored JSON is exactly the machine result.

 → No content items
   One text item mirroring the structured content
   One text item naming the canonical target
   Type another answer
   Back
   Discuss
   Pause workflow questions
```

The recommendation opens with "Take the second choice", which is a positional
reference the reader resolves by counting. The recommended option is second,
while the selection cursor rests on the first. Three lines of justification sit
between the question and the list.

Two code paths produce this shape, and both behave the same way.
`questions.ts` builds its list as
`[...(finding.choices ?? []).map((choice) => choice.label), "Type another
answer", "Back", "Discuss", "Pause workflow questions"]` and passes the
recommendation as a second prompt line. `intake.ts` does the same for
`superdev_ask`. Neither reorders the list, and neither annotates a choice.

The reason is structural rather than an oversight in presentation. A finding's
choices are `Array<{ label: string; description?: string }>` and its
`recommendation` is a separate free-text string. No field connects them, so
nothing in the payload identifies which choice is recommended, and a
recommendation may describe something no choice offers.

## Behaviour

The recommended choice appears first in the list, carries a `[recommended]`
marker, and the recommendation text stops restating which option it is. The
justification remains available, because the reasoning is why the reader can
disagree with it.

Reaching that requires the payload to identify the recommended choice rather
than describe it. A boolean on the choice, or a field naming the recommended
label, would let the UI sort and mark deterministically. Free-text prose cannot
be matched to a choice reliably, and guessing from the text would misfire on
exactly the questions where precision matters.

Two constraints bound the change.

The selected label is recorded verbatim as the answer. `questions.ts` assigns
`let answer = selected`, stores it, and the correction task interpolates
`JSON.stringify(correction.answers)` into the prompt a later SCOPE role reads.
A marker concatenated onto a label would therefore travel into the recorded
answer and into that prompt. The marker belongs to presentation only.

A recommendation need not name any choice. Some questions recommend
discussion, and some recommend an answer the reviewer did not enumerate.
Ordering must leave the list unchanged when nothing is identified as
recommended, rather than promoting an arbitrary option.

The same treatment belongs in both surfaces, since `superdev_ask` and the
review queue present the same vocabulary to the same reader.

`P_questions-ui` in contract-011 requires Pi to offer concrete choices, a
recommendation, a typed answer, and chat discussion. Today's UI satisfies that
promise, which is why this is filed as a feature rather than a defect: nothing
is broken, and what is missing is a presentation convention the promise does
not yet describe. Whether the promise should be extended to require the
marking is a question for the work, not something this issue decides.

## Scope

How a recommendation is presented among the choices it concerns.

- In: the choice ordering and marking in `questions.ts` and `intake.ts`, and
  whatever payload field identifies the recommended choice.
- Out: the wording reviewers write. This is about presenting a recommendation,
  not about making recommendations better.
- Out: the fixed trailing entries — `Type another answer`, `Back`, `Discuss`,
  and `Pause workflow questions` — which are actions rather than answers and
  belong after the answers whatever the recommendation says.
- Out: the confirmation and revision steps, which present a proposed answer
  rather than a choice list.

## Comments

Filed from a live session, from a question asked at the end of a requirements
review. The two code paths were read before filing: the choices are rendered
in the order the reviewer supplied them, and the recommendation is a prompt
line with no link to any of them.
