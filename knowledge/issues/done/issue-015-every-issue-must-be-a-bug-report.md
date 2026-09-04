---
type: Issue
id: issue-015-every-issue-must-be-a-bug-report
title: The Issue type has one shape, bug-report, so everything filed has to pretend to be a defect
description: One schema and one template constrain type Issue, so a feature request, a rename or a decision has to invent repro steps and an environment to be filed at all — six of the fourteen issues on file already do, and a feature request has no home but an untracked bullet in the backlog.
kind: feature
lifecycle: done
---

# Feature: the Issue type has one shape, bug-report, so everything filed has to pretend to be a defect

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

## Context

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

## Behaviour

One shape per kind of thing that can be filed, all sharing `type: Issue`,
the `issue-<nnn>-<kind>-<slug>` id, the triage tags and the lifecycle, and differing
only in their body sections. A feature request states motivation, the
proposed behaviour, the alternatives considered and the scope, and is never
asked for an error log. A scoped piece of mechanical work states the
surfaces it touches and what "done" means, and is never asked for a
regression risk it does not have.

- The tracker has three issue types — BugReport, FeatureRequest and
  Chore — each with its own schema and template.
- When an issue is filed, its filename carries its kind,
  `issue-{nnn}-{kind}-{slug}`.
- The tracker never asks a feature request for repro steps, nor a chore for
  a root cause.

## Scope

The boundary as drawn at filing:

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
  `knowledge/issues/issue-<nnn>-<kind>-<slug>.md`. This costs a one-off
  rename of the existing fourteen files and their index entries; the
  alternative, carrying the kind in `tags`, cannot be expressed in
  `target-files` at all.
- Fix `schema-bug-report`'s `target-files` glob to match the files it
  governs.
- Teach the skills to choose: `/accept` currently hardcodes
  `schema-bug-report`, and `/feature-plan` reads issues back.
- Workaround until then: file the non-bug on the bug shape and write "any"
  in the Environment section, which is what the six existing ones do.

Alternatives considered:

- Keep one shape and relax it until everything fits — a schema loose enough
  to admit a bug report and a rename requires almost nothing, so it checks
  almost nothing.
- Keep `type: Issue` and add a second field naming the kind — every concept
  would then say the same thing twice, and the tracker would have two
  dispatch mechanisms where one would do.
- File non-defects outside the tracker, in the backlog — which is what
  happens now, and it is why a feature request has no id, no triage tag and
  nothing a spec can link to.

## Resolution

P008. `type: Issue` becomes three types, one per shape, each with its own
schema and template and all sharing the id, the triage tags and the
lifecycle:

- `BugReport` — a defect: something behaves against its own specification.
  Symptom, environment, repro, root cause, regression risk.
- `FeatureRequest` — something absent that should exist. Motivation,
  proposed behaviour, alternatives considered, scope. Never asked for an
  error log.
- `Chore` — scoped mechanical work whose shape is already known. Surfaces
  and a definition of done. Never asked for a root cause it does not have.

The seventeen issues on file sort nine, six and two, and the six this issue
named as strained were rewritten into the shape they actually are — this one
among them. A feature request now has a home in the tracker rather than an
untracked bullet in the backlog.

The filename carries the kind as well, `issue-<nnn>-<kind>-<slug>.md`, which
is the rename this issue proposed above. It buys readability rather than
dispatch: P008 resolved dispatch through the frontmatter `type` (D-12), so
the kind in the path is for whoever reads the directory listing. The
mismatch this issue observed — a `target-files` glob of
`knowledge/issues/issue-*.md` against files named `Innn-<slug>.md` — is gone
with it: the glob was right and the files were wrong, and the glob itself
came off the schema when dispatch moved to `type`.

Plans took the same shape at the same time: `knowledge/feature-plans/` and
`knowledge/adhoc-plans/` merged into `knowledge/plans/` as
`plan-<nnn>-<kind>-<slug>.md`, with `<kind>` `feature` or `adhoc` and one
number series across both.

The dispatch this rests on is the wider fix: a document's `type` names the
one schema that governs it, so adding a shape is adding a type, not
widening an existing contract until it checks nothing.
