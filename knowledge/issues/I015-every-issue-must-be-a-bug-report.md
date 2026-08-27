---
type: FeatureRequest
id: issue-015-every-issue-must-be-a-bug-report
title: The Issue type has one shape, bug-report, so everything filed has to pretend to be a defect
description: One schema and one template constrain type Issue, so a feature request, a rename or a decision has to invent repro steps and an environment to be filed at all — six of the fourteen issues on file already do, and a feature request has no home but an untracked bullet in the backlog.
status: draft
tags: [needs-triage]
---

# Bug: the Issue type has one shape, bug-report, so everything filed has to pretend to be a defect

## Summary

`type: Issue` is constrained by exactly one schema, `schema-bug-report`, and
produced by exactly one template, `template-bug-report`. Both demand a
symptom, an environment, numbered repro steps and a regression risk. Six of
the fourteen issues on file are not defects — a documentation gap, two
missing checks, a decision, two renames — and every one of them carries
invented repro steps and a "Platform: any" line to satisfy a shape it does
not fit. A feature request fares worse: it has no home in the tracker at
all, so it lands in `knowledge/backlog.md` as a bullet with no id, no triage
tag and nothing a spec can link to. bug-report is one kind of issue. It is
being used as the definition of one.

## Environment

- Version/commit: superdev 0.2.0, AOKF 0.3, grammar 2.0
- Platform: any; a gap in the schema and template sets, not in code

## Steps to reproduce

1. Run `grep -l 'const: Issue' knowledge/schemas/*.md` — one file,
   `bug-report.md`.
2. Run `ls pack/knowledge/templates/*.md | grep -cv index.md` — 40
   templates, of which one produces an Issue.
3. Read `.claude/skills/accept/SKILL.md:25`: every gap found at acceptance
   is filed "per `schema-bug-report`", whatever the gap turns out to be.
4. Read the Environment and Steps to reproduce sections of I006, I010, I011,
   I012, I013 and I014.

## Expected behaviour

One shape per kind of thing that can be filed, all sharing `type: Issue`,
the `issue-nnn-<slug>` id, the triage tags and the lifecycle, and differing
only in their body sections. A feature request states motivation, the
proposed behaviour, the alternatives considered and the scope, and is never
asked for an error log. A scoped piece of mechanical work states the
surfaces it touches and what "done" means, and is never asked for a
regression risk it does not have.

## Actual behaviour

One shape, worn by everything.

Of the fourteen issues on file, eight are defects. The other six are not,
and each is visibly strained:

- **I006** — a shipped feature missing from the documentation. Its repro
  steps are three greps that find nothing, which is a way of measuring an
  absence, not of reproducing a fault.
- **I010, I011** — checks the validator does not perform. These are feature
  requests against the validator, filed as bugs because "the tool does not
  do X" can always be restated as "X is not caught".
- **I012** — a request to change AOKF SPEC §11, which says in its own body
  that it "wants the treatment ADR-017 had rather than a quiet edit". A
  decision, filed as a defect.
- **I013, I014** — renames. I013's environment line reads "any; this is a
  naming defect on every surface, not a runtime one", which is the sentence
  a wrong template forces out of a writer.

The Environment section is dead weight in all six: "Platform: all",
"Platform: any", four times over. Nothing is learned from it, and it is
required.

A feature request cannot be filed at all. `knowledge/backlog.md` currently
holds three ideas as bullets — a knowledge-capture skill, pre-filled
knowledge skeletons, comment-preserving manifest stamping — none with an id,
a triage tag, a file of its own, or an edge a spec could declare against it.
The tracker would give all four. The price of admission is a "Steps to
reproduce" heading.

This issue is itself a request for new document kinds, filed on the
bug-report shape, for want of anywhere else to put it.

## Root cause (if known)

The tracker was specified from the acceptance path and no further. `/accept`
files the gaps it finds, gaps found at acceptance are usually defects, and
one shape covered that. `knowledge/issue-tracker.md` then wrote the
assumption down as though the two words were interchangeable: "The
`template-bug-report` template carries this shape." Nothing has added a
second shape since, so the single one now reads as the definition of an
issue.

The mechanism for more already exists. Every other schema selects its
documents with a glob over the filename — `**/*investigation*.md`,
`**/*postmortem*.md`, `**/*code-review*.md` — so a second issue kind needs
only a filename convention a glob can see. What is missing is the decision
about which kinds exist and what they are called.

One thing has to be repaired before that dispatch can be relied on:
`schema-bug-report` declares `target-files: "knowledge/issues/issue-*.md"`,
while the convention in `issue-tracker.md` and all fourteen files on disk
use `Innn-<slug>.md`. The glob matches nothing. Nobody has noticed because
nothing resolves `target-files` — `check_schema`
(`crates/lib/superdev-core/src/format/check.rs:645`) validates a schema
document's own shape, never the documents it governs, so a schema is today a
contract an agent reads rather than a check the tool runs. That defect
stands on its own and could be filed separately; it is named here because
kind-by-filename cannot work until it is fixed.

## Proposed fix / workaround

- Settle the kind set. `bug-report` exists. `feature-request` is the clear
  second — motivation, proposed behaviour, alternatives, scope. `task`
  covers scoped mechanical work with no behaviour question (I006, I013,
  I014): surfaces touched, migration and compatibility, done-when. A fourth
  for a decision (I012) is arguable, since `knowledge/decisions/` and the
  ADR schema already exist and I012 asked to be routed there; deciding that
  either way is part of this work. Stop there — a fifth kind is speculation
  until an issue arrives that fits none of the four.
- Give each kind a schema in `knowledge/schemas/` and a matching template in
  `pack/knowledge/templates/`, kept in step the way `bug-report` and
  `template-bug-report` are.
- Keep everything above the body common: `type: Issue`, the id pattern, the
  triage tags, the lifecycle, the flat directory. `issue-tracker.md` gains
  the list of kinds and the rule for choosing between them, and stops naming
  bug-report as though it were the only one.
- Dispatch by filename, because that is the only selector a schema has:
  `knowledge/issues/Innn-<kind>-<slug>.md`. This costs a one-off rename of
  the existing fourteen files and their index entries; the alternative,
  carrying the kind in `tags`, cannot be expressed in `target-files` at all.
- Fix `schema-bug-report`'s `target-files` glob to match the files it
  governs.
- Teach the skills to choose: `/accept` currently hardcodes
  `schema-bug-report`, and `/feature-plan` reads issues back.
- Workaround until then: file the non-bug on the bug shape and write "any"
  in the Environment section, which is what the six existing ones do.

## Regression risk

The skills are the risk. `/accept` and `/feature-plan` both name the tracker,
and an agent that is not told a kind set exists will keep reaching for
bug-report, which is the current behaviour by another name. The skills,
`issue-tracker.md`, the templates index and the schemas index have to land
together, or the old default silently survives the change.

Renaming the existing fourteen files breaks the links in
`knowledge/issues/index.md` and the cross-references between issues. A
broken body link is only a warning today (see
[I012](I012-five-decidable-findings-only-warn.md)), so a missed link will
not fail the run — read the report with the warnings open, or land this
after I012.

Each new schema is itself checked by the format validator for its own shape,
including the rule that its `example` instantiates every required section,
which is what keeps a schema self-testing. The duplication check does not
compare `schema|schema`, so sibling issue schemas may share a lifecycle
paragraph without tripping it; it does compare `unit|schema`, so wording
lifted from a skill into a schema will trip.
