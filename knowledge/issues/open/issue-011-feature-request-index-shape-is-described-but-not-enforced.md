---
type: FeatureRequest
id: issue-011-feature-request-index-shape-is-described-but-not-enforced
title: The shape SPEC §9 gives an index is described but never enforced
description: SPEC §9 fixes what an index.md looks like — no frontmatter, heading-grouped link lists, one entry per concept — but no validator checks any of it, so an index can carry frontmatter, drop its heading, or mix bullet styles and still pass.
lifecycle: open
links:
  - rel: references
    to: plan-006-adhoc-rust-format-validator
---

# Feature: the shape SPEC §9 gives an index is described but never enforced

## Summary

SPEC §9 states what an `index.md` is: it "contains no frontmatter", and its
body is "one or more heading-grouped link lists" of the form
`* [Title](file.md) - description`. None of that is checked. The AOKF
validator is not the place for it either — those rules are this project's
reading of a section the spec never put in its check list, and enforcing them
in `aokf validate` would make this repo report findings another AOKF tool
would not.

The natural home is a schema over `knowledge/**/index.md`, which is what
schemas are for. That cannot run until schemas are applied to the documents
they govern, which
[the format-validator plan][sokf:plan-006-adhoc-rust-format-validator]
lists as a non-goal.

## Motivation

Add frontmatter to `knowledge/decisions/index.md`, change its `*` bullets to
`-`, delete its `# Decisions` heading, and run the validator: it passes with
no findings — `index.md` is a reserved
file, so the per-concept checks never see it, and `check_indexes` only
resolves link targets. The format validator prints
`SKIP  [-]  knowledge/decisions/index.md`, because the grammar's
`match.except` deliberately excludes `index.md` from the `schemas` directory
so a directory listing is not mistaken for a schema.

So the file falls between the two validators: one treats it as reserved, the
other as excepted.

Nothing owns the rule. SPEC §10's warn list names one index rule — missing
targets — and §9's shape statements appear only as prose. The schema
vocabulary can express the shape, but `target-files` is not yet applied to
anything.

The eight indexes were standardised by hand against §9 and now agree: no
frontmatter, an H1, `* ` bullets, ` - ` separators, 113 entries. That
consistency is currently held by nothing but care.

## Proposed behaviour

Something reports the frontmatter, since §9 says an index has none. The bullet
style and the missing heading are worth a finding too, given every other index
in the canonical knowledge agrees on both.

## Acceptance criteria

1. `AC_c1` [event] WHEN an `index.md` carries frontmatter THE SYSTEM SHALL
   report it, since SPEC §9 says an index has none.
2. `AC_c2` [event] WHEN an index is not heading-grouped link lists in the house
   bullet style THE SYSTEM SHALL report the offending line.

## Alternatives considered

- Leave §9's shape as guidance and check nothing — the state this issue
  reports. It has already let five indexes drift without a word.
- Generate every index from the concepts rather than checking what an author
  wrote — removes the divergence outright, and gives up the heading grouping
  an author chooses. Worth its own request.
- Check the shape in the schema layer rather than the SOKF half — an index
  carries no frontmatter, so it would have to be reached by glob, and §9 is
  the specification's rule rather than a schema's.

## Scope

- In: repurpose `schema-templates-index`, which today governs
  `knowledge/templates/index.md` — a file that goes away when the schemas
  supersede the templates in the pack. Retarget it at `knowledge/**/index.md`
  as `schema-bundle-index`, describing: no frontmatter, a level-1 heading,
  optional repeatable level-2 groups, and entries as a bullet list. That turns
  a schema about to describe nothing into one covering eight live files.
- Blocked: it cannot be enforced until schema-to-document validation exists,
  so this issue is blocked behind that work rather than behind the port
  itself.

<!-- sokf:links -->
[sokf:plan-006-adhoc-rust-format-validator]: /knowledge/plans/done/plan-006-adhoc-rust-format-validator.md
