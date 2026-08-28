---
type: FeatureRequest
id: issue-018-the-schema-layer-checks-sections-and-nothing-else
title: A schema declares content kinds and a frontmatter contract, and the validator reads neither
description: P008 made schemas govern documents, but only their sections — the content kind under each heading and the frontmatter constraints beside it are declared on every schema and read by nothing, which is the fault P008 set out to cure, one level down.
status: draft
tags: [needs-triage]
---

# Feature: the schema layer checks sections and nothing else

## Summary

P008 made `superdev validate` check documents against the schema their `type`
names, and it checks the sections: present, ordered, not prohibited, table
columns, line limit. Two other families of constraint sit in every schema and
are read by nothing — the `content` kind declared per section, and the
`frontmatter` contract declared beside it.

## Motivation

This is the fault P008 was written to cure, one level down. `target-files` was
declared required on forty schemas and read by nothing, so no document was
ever checked; the fix left two smaller versions of the same thing in place.

Measured on this repository today:

- **Content kinds.** 33 sections declare a `content` kind their body does not
  match, across ten schemas — 16 in `spec` alone. Whether all 33 are real is
  the open question below, not a reason to leave the field unread.
- **Frontmatter contracts.** `schema-spec` declares
  `id: pattern '^spec-\d{3}-[a-z0-9-]+$'`; the fourteen specs on file carry
  ids like `spec-implements-rel`, which matches nothing. Every one of them
  would fail a check that does not exist. The `status` enums and the `title`
  and `description` constraints are in the same position.

A reader of a schema cannot tell which half of it binds. That is worse than a
schema that checks nothing, because the checked half lends the unchecked half
its authority.

## Proposed behaviour

`validate` reports a section whose body is not the kind its schema declares —
prose, bullet-list, numbered-list or table — and a frontmatter value that
breaks the constraint beside it, in the same shape as the section findings:
the document, the rule, and the schema that declares it.

## Alternatives considered

- Delete the unread declarations from the grammar instead. Cheaper, and it
  throws away the guidance the descriptions carry for an agent writing a new
  document — which is most of what a schema is for.
- Check frontmatter only, and leave content kinds. The frontmatter half is
  unambiguous and would land in an afternoon; the content half needs the
  question below settled first. A defensible order, not a different outcome.

## Scope

- In: the `content` kinds, the `frontmatter` constraints, and the
  reconciliation each will need — 33 mismatches for the first, at least the
  fourteen spec ids for the second.
- Out: whether `schema-templates-index` should exist at all. It governs
  `knowledge/templates/index.md`, and indexes carry no frontmatter and are
  deliberately excluded from the candidate list, so it is the one schema left
  that can never fire. Index shape is I011's.

## Open questions

- Does a `bullet-list` section admit a leading sentence before the bullets?
  Most such sections here have one, so the answer decides whether the count
  is 33 or nearer 10. Recommended default: yes — the kind describes the
  section's substance, not its first line.
