---
type: FeatureRequest
id: issue-022-feature-request-a-schemas-worked-example-is-checked-by-nothing
title: A schema's worked example is the thing agents copy, and it is the one part of the schema nothing checks
description: Every schema carries an `example:` block showing a conforming document, and no check reads it — five of the twenty-three example ids on file broke their own schema's id pattern, left behind by a migration that changed the pattern and not the example.
lifecycle: open
links:
  - rel: relates-to
    to: issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else
---

# Feature: a schema's worked example is checked by nothing

## Summary

Every schema carries an `example:` block showing one conforming document.
It sits inside a fenced YAML scalar, so no check reads it — not the schema
layer, which checks documents against schemas, and not the SOKF layer,
which reads links and footnotes from a body and sees a fence as opaque
text. An agent writing a new document copies the example.

## Motivation

Measured on this repository today: of the 23 example documents declared
across the schemas, **five carried an `id` its own schema's `id` pattern
refuses**.

| Schema | Example id | Its own pattern |
|--------|-----------|-----------------|
| `schema-adhoc-plan` | `adhoc-plan-002-scheme-match-cleanup` | `^plan-\d{3}-adhoc-[a-z0-9-]+$` |
| `schema-bug-report` | `issue-042-pack-sync-etimedout` | `^issue-\d{3}-bug-[a-z0-9-]+$` |
| `schema-chore` | `issue-042-drop-the-legacy-cache-directory` | `^issue-\d{3}-chore-[a-z0-9-]+$` |
| `schema-feature-plan` | `feature-plan-001-pack-source-allowlist` | `^plan-\d{3}-feature-[a-z0-9-]+$` |
| `schema-feature-request` | `issue-042-validate-reports-machine-readable-json` | `^issue-\d{3}-feature-request-[a-z0-9-]+$` |

All five were left by the commit that gave issues and plans one filename
convention: it changed each pattern and missed the example beside it, and
nothing said so for the four commits since. They were corrected by hand
alongside P010, which is the point — a hand correction is the only thing
that finds them, and only when someone happens to look.

The example is not decoration. It is the one part of a schema an agent
reads and copies verbatim, so a wrong example is a wrong document
generator. It also carries the authority of the checked half of the file:
a reader who sees the section rules enforced assumes the example beside
them was enforced too.

The same blindness covers more than the id. An example's sections, its
ordering, its content kinds and — since SOKF 0.4 — its body links are all
unread, so an example may teach a path link where the format now requires
`[text][sokf:<id>]`.

## Proposed behaviour

`validate` checks each schema's `example:` block against the schema that
declares it, and reports a failure the way it reports any other schema
finding, naming the schema and what the example broke.

The example is a document with a known governing schema — the one it sits
in — so this needs no dispatch: the check that already runs over a real
document runs over the example with the schema handed to it.

## Acceptance criteria

1. [event] WHEN a schema's `example:` block breaks the schema that
   declares it THE SYSTEM SHALL report the schema and what the example
   broke, as it reports any other schema finding.

## Alternatives considered

- **Extract examples into real files under a fixture tree.** They would
  then be checked by the existing machinery with no new code, but a schema
  whose example lives elsewhere is a schema a reader cannot read on its
  own, and the example is where it is precisely so it is read.
- **Check only the `id`.** Cheapest, and it covers all five faults found.
  Rejected: the id was wrong because nothing looked, and the sections and
  links are unlooked-at for the same reason. A check scoped to the fault
  that happened to surface leaves the next one to surface the same way.
- **Leave it and correct examples when noticed.** That is the state this
  issue describes, measured at five faults across four commits.

## Scope

- In: checking each schema's `example:` block against its own schema —
  frontmatter, sections and body — and reporting failures as schema
  findings.
- In: the check reaching every schema, including those a managed
  repository ships once
  [the schemas ship][sokf:issue-020-bug-the-schemas-do-not-ship].
- Out: the frontmatter and content-kind checks themselves. This issue
  wants the example fed to whatever checks exist;
  [I018][sokf:issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else]
  is what makes those checks read a schema's frontmatter contract at all,
  and until it lands an example-id check needs its own pattern match.
- Out: fenced examples in documents that are not schemas. A plan or a spec
  may show whatever illustrates its argument.

## Comments

2026-08-31 — A hand review of all 53 schemas found 26 examples breaking
their own frontmatter constraints: 18 carried a `type` the schema's
`const` refuses, and 8 omitted frontmatter the schema constrains.
Plan-014 fixed every instance by hand. The count strengthens the case
for the checker: the five id faults first recorded here were the
surfaced fraction of a fault class five times that size.

<!-- sokf:links -->
[sokf:issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else]: /knowledge/issues/done/issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else.md
[sokf:issue-020-bug-the-schemas-do-not-ship]: /knowledge/issues/done/issue-020-bug-the-schemas-do-not-ship.md
