---
type: Decision
id: adr-031-ears-criteria-are-checked-by-item-pattern
title: EARS criteria are checked by an item pattern
description: schema-feature-request declares an item-pattern on Acceptance criteria requiring each criterion to open with an EARS tag or TBD, so a malformed criterion fails validate at frame time; the frame gate keeps forbidding TBD at close.
lifecycle: deprecated
---

# ADR-031: EARS criteria are checked by an item pattern

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

`schema-feature-request` fixes the EARS shape of acceptance criteria in
a section `description` the validator never reads, so a criterion
missing its pattern tag passes validate. The schema also permits a
criterion to read `TBD — <question>` while the request is open, and a
section rule cannot see the frontmatter `lifecycle`.

## Decision

The Acceptance criteria section declares
`item-pattern: '^\[(ubiquitous|event|state|conditional|optional|complex)\] |^TBD — '`.
Each criterion opens with one of the six EARS tags or with `TBD — `.
TBD is admitted by the pattern unconditionally; the frame skill's
existing gate — no criterion reads TBD when framing ends — remains the
process rule that retires it.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Opening-tag pattern, TBD admitted | Frame-time enforcement with no new vocabulary; zero reconciliation | A settled request could in principle carry a TBD the validator accepts; the frame gate covers it |
| Lifecycle-conditional item shapes | Exact TBD handling | Couples section rules to frontmatter values — new vocabulary for one case |
| Full EARS grammar parsing | Checks the sentence, not just the tag | Heavy machinery; the opening tag catches every drift observed |
| Leave it to the skill's instructions | No change | The mechanism I034 records as having already failed |

## Consequences

- Positive: a malformed criterion is a validate error the moment frame
  writes it, in this repository and every managed one.
- Negative: the pattern checks the opening tag, not that the sentence
  follows its tag's grammar.
- Follow-ups: none — the documents on file already conform.
