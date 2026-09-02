---
type: FeatureRequest
id: issue-023-feature-request-a-concept-path-written-outside-the-knowledge-is-checked-by-nothing
title: A skill naming a concept by path breaks silently when that concept moves, because link checking stops at the knowledge directory
description: P010 made a link inside the SOKF knowledge survive a rename, but the eleven concept paths written in skills and agent files are checked by nothing, so the failure P010 removed from the knowledge still stands one directory away.
lifecycle: open
links:
  - rel: relates-to
    to: plan-010-adhoc-links-address-ids
---

# Feature: a concept path written outside the knowledge is checked by nothing

## Summary

Link checking reads a loaded knowledge directory, so it sees only the
documents inside it. A file under `.claude/skills/` or `.agents/` may name
a concept by path, and nothing resolves that path — before or after
[P010][sokf:plan-010-adhoc-links-address-ids], which gave a link inside the
knowledge an id that survives a rename.

## Motivation

Concept paths are still written outside the SOKF knowledge today, across
the skills' bootstrap `read_file` calls, each naming a schema:

```
.claude/skills/contract-design/SKILL.md:15   knowledge/schemas/contract-interface.md
.claude/skills/feature-plan/SKILL.md:14      knowledge/schemas/feature-plan.md
```

All eleven resolve at the time of writing. That is the whole problem: they
resolved before the schema migration too, and the only reason none broke is
that each rename since has been done by someone who happened to grep. The
schemas these skills name are the schemas P008 created and a later commit
split — `interface-contract` became fifteen `contract-*` schemas in one
change — so the paths have already survived one near miss.

A skill naming a schema that has moved does not fail loudly. The skill
opens a file that is not there, and the agent proceeds without the contract
it was told to follow. Nothing in `validate`, the hook or CI says so.

The grammar already names these trees: `.agents`, `.claude/skills` and
`knowledge/schemas` are its roots, and `validate` walks them for the schema
half of its check. Only the link half stops at the knowledge directory.

## Proposed behaviour

`validate` resolves a concept reference written anywhere the grammar
governs, and reports one that names no concept — the same finding it
already raises inside the knowledge, at the same severity, naming the file
and the path.

A file outside the SOKF knowledge is not a concept and carries no
definition block, so it names a concept by path rather than by
`[text][sokf:<id>]`; what is wanted here is that the path resolve, not that
it change form.

## Acceptance criteria

1. `AC_c1` [event] WHEN a file the grammar governs names a concept path that
   resolves to no concept THE SYSTEM SHALL report the file and the
   path, at the severity the same finding carries inside the knowledge.

## Alternatives considered

- **Let the id form reach outside the knowledge.** A skill would name
  `[schema][sokf:schema-adhoc-plan]` and gain the same rename-proofing the
  knowledge has. Rejected for now: a skill is read by an agent as text and
  by no consumer that resolves ids, so the label would render as a broken
  reference in the one place it is read. Worth reopening if skills ever
  pass through a renderer that resolves them.
- **A repository-wide grep in CI.** Cheap, and it catches a dead path. It
  also catches every path in prose that was never meant to resolve, and it
  is a second checker with its own idea of what a link is — which is what
  `validate` merging its two halves was written to stop.
- **Leave it to whoever renames a concept.** The state today, and the one
  the schema split nearly broke.

## Scope

- In: resolving concept paths in files under the grammar's roots, and
  reporting the ones that name nothing.
- Out: the form those references take. They stay paths; see the first
  alternative.
- Out: paths inside a fenced example, which is
  [I022][sokf:issue-022-feature-request-a-schemas-worked-example-is-checked-by-nothing].
- Out: the eleven references themselves. They resolve today; this is about
  the check that would say when they stop.

<!-- sokf:links -->
[sokf:issue-022-feature-request-a-schemas-worked-example-is-checked-by-nothing]: /knowledge/issues/done/issue-022-feature-request-a-schemas-worked-example-is-checked-by-nothing.md
[sokf:plan-010-adhoc-links-address-ids]: /knowledge/plans/done/plan-010-adhoc-links-address-ids.md
