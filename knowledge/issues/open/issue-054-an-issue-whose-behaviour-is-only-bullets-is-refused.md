---
type: Issue
id: issue-054-an-issue-whose-behaviour-is-only-bullets-is-refused
title: An issue whose Behaviour is only bullets is refused, and I052's criterion says the validator accepts one
description: schema-issue declares Behaviour and Scope as prose, which binds by one plain paragraph line, so an issue written as a bare list fails validation naming the section — while I052 says the validator accepts an issue whose Behaviour is prose, bullets or both.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: Found at acceptance of I052.
---

# Bug: an issue whose Behaviour is only bullets is refused

## Summary

`schema-issue` declares Behaviour and Scope with `content: prose`, and
the prose kind is satisfied only by a plain paragraph line. An issue
whose Behaviour is a bare bullet list therefore fails validation naming
the section, against the criterion the schema was written to keep.

## Context

`is_paragraph` accepts a line that is not blank, not a bullet, not a
numbered item, not a table row, not a heading, not an HTML comment and
not a link definition, and that carries at least one alphanumeric
character. A section of bullets alone offers no such line, so the
section check reports it.

[I052][sokf:issue-052-the-workflow-carries-more-process-than-it-needs]
states in its Behaviour that the validator accepts an issue whose
Behaviour is "prose, bullets or both, with no key and no tag". Its test,
`an_open_issue_passes_with_prose_bullets_or_both` in
`crates/lib/superdev-core/tests/normative_shapes.rs`, exercises three
Behaviour bodies: a paragraph, a paragraph followed by bullets, and a
paragraph with bullets between two paragraphs. A bare list is not among
them, so the criterion's middle case has never been run.

Nothing on file breaks today. All 52 issues in the tracker carry a
paragraph line under Behaviour, and under Scope where the section is
present. `/file`'s WRITE THE RECORD step already tells the agent to open
every section with a line of prose, so an issue filed through the skill
conforms. An issue written by hand, or a user's words taken down as a
bare list, does not.

## Behaviour

The schema and the criterion state the same rule. The decision is which
one moves.

Widening the content kind lets a bare list stand: Behaviour and Scope
take a kind that accepts a paragraph, a bullet list or both, and the
test gains a bare-list case. Requiring the paragraph keeps the schema as
it is: I052's Behaviour is corrected to say that every section opens
with a plain paragraph line, `/file`'s existing instruction becomes the
statement of record, and the test's three cases are the whole of the
rule.

Scope carries the same declaration and the same defect, so whichever way
Behaviour is settled, Scope follows it.

## Scope

The two section declarations, the criterion they answer to, and the test
that pins them.

- In: `Behaviour` and `Scope` in `knowledge/schemas/issue.md` and
  `pack/knowledge/schemas/issue.md`.
- In: I052's Behaviour statement about what the validator accepts.
- In: `an_open_issue_passes_with_prose_bullets_or_both`, which must
  exercise whichever rule is chosen.
- In: `/file`'s WRITE THE RECORD step, where the two are reconciled by
  requiring the paragraph.
- Out: the other four sections of `schema-issue`, whose bodies no issue
  on file writes as a bare list.
- Out: the `prose` content kind itself, which every other schema uses as
  it stands.

<!-- sokf:links -->
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/done/issue-052-the-workflow-carries-more-process-than-it-needs.md
