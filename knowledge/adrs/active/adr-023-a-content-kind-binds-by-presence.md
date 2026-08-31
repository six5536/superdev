---
type: Decision
id: adr-023-a-content-kind-binds-by-presence
title: A Content Kind Binds by Presence
description: A section satisfies its declared content kind when the kind's form appears in its body — a bullet for bullet-list, a fenced block for code — with other content tolerated, so the kind names the section's substance rather than policing every line.
lifecycle: active
---

# ADR-023: A Content Kind Binds by Presence

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

I018 makes the validator read the `content` kind each section rule
declares — prose, bullet-list, numbered-list, table or code. The kind
needs a meaning every schema author shares: the live tree's list
sections mostly open with a lead-in sentence, prose sections sometimes
carry an illustrative list, and framing settled that a lead-in sentence
passes. The check must be decidable line by line and must not flood the
reconciliation with findings nobody would act on.

## Decision

We will check presence: a section satisfies its kind when the kind's
form appears in its body — at least one bullet for `bullet-list`, one
numbered item for `numbered-list`, one table for `table`, one fenced
block for `code`, and for `prose` at least one plain paragraph line
that is none of those. Content beside the declared form is tolerated.
A schema declaring a kind outside the five is itself reported.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Presence of the form | Decidable, matches the lenient framing, ~10 live findings to reconcile | A section that is mostly the wrong form but contains one right line passes |
| Exclusive: only the form | Sharpest reading | Every lead-in and every aside becomes a finding; reconciliation balloons past what anyone would fix |
| Dominant form by line count | Middle ground | A threshold to pick and defend; the finding stops being explainable in one sentence |

## Consequences

- Positive: the declared kinds start to bind at a strictness the live
  tree can meet, and a schema's `content` line means one thing
  everywhere.
- Negative: presence is a weak claim — a section can satisfy
  `bullet-list` while being mostly prose. A later tightening would be a
  breaking change to every managed repository's knowledge.
- Follow-ups: none; the reconciliation is the feature's own scope.
