---
type: FeatureRequest
id: issue-010-feature-request-index-entries-are-never-checked-against-their-concept
title: An index entry may say anything about a concept, and nothing notices
description: SPEC §9 says an index entry should carry the linked concept's description, but check_indexes only tests that the target exists, so an index can drift from every concept it lists — or hold the only copy of something — and validate still passes.
status: draft
tags: [needs-triage]
---

# Feature: index entries are never checked against the concept they name

## Summary

`superdev validate` checks that an `index.md` entry points at a file that
exists, and nothing else about what the entry says. SPEC §9 requires that
"entries should carry the linked concept's `description`", so an index may
describe a concept in terms the concept itself does not use, and the SOKF
knowledge still passes.

## Motivation

The gap bites in two directions. An entry silently goes stale when the
concept is reworded, and an entry can become the only home of a fact — which
matters because §9 also says indexes may be generated, so a regeneration
would destroy it.

It is not hypothetical. A throwaway script found fourteen divergences across
five indexes in this repository, none of them visible to the validator: five
were stale wording, and nine were the `knowledge/issues/index.md` entries,
which held each issue's resolution while the concept described only the
symptom. They were fixed by hand.

The cause is that §9's requirement is stated where nothing implements it.
`check_indexes` walks each index's links and resolves them, warning only when
a target does not exist — faithful to SPEC §10, whose warn list names exactly
one index rule.

## Proposed behaviour

A warning naming the index, the entry, and the fact that its text does not
match the linked concept's `description`, in the manner of the existing
`index entry points at missing file:` warning.

Two details the implementation has to settle, both visible in the current
indexes: entries de-capitalise the concept's first letter as a house style,
so the comparison is case-insensitive on the first character; and the
comparison ignores a trailing full stop.

## Alternatives considered

- An error rather than a warning — §11 requires consumers to be permissive,
  `index.md` is a reserved file and not a concept, and §9 says "should".
  I012 argues the severities generally; this one follows whatever it decides.
- Generating indexes from the concepts instead of checking them — removes the
  divergence rather than reporting it, and gives up the heading grouping an
  author chooses. Worth its own request; it is not this one.

## Scope

- In: the comparison in `check_indexes`, the warning it emits, and the tests
  it lands with. The function already resolves the target path and the
  concepts are already loaded, so the lookup adds no IO.
- Out: index shape, which is I011. Index generation, per the alternative
  above.
- Note for the implementer: `check_indexes` has no covering tests today, so
  the change lands on untested ground and should bring its own.
