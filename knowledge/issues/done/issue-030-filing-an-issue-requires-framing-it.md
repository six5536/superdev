---
type: Issue
id: issue-030-filing-an-issue-requires-framing-it
title: filing an issue requires framing it, so framings go stale before the work starts
description: The workflow has no lightweight filing — /frame does the full framing at creation, but framing belongs at the point the issue is taken up, because a framing made at filing can be out of date by the time the work starts; an issue's lifecycle does not say whether it has been framed, so the schema cannot hold a framed issue to its form and let an unframed one breathe.
kind: feature
lifecycle: done
links:
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: The keyed EARS form a framed issue is held to, and that an unframed one is not.
  - rel: references
    to: adr-045-a-schema-declares-variants
    note: The mechanism by which one tracker schema varies by lifecycle.
  - rel: references
    to: issue-037-a-contracts-behaviour-is-not-written-as-ears
    note: Deferred the `EX_` key on a bug's Expected behaviour to this feature's framed state.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Gains the per-variant heading — one heading in several rules with disjoint variants — PENDING (ADR-049).
  - rel: references
    to: adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed
    note: The four states, /file, the gates, the forms per state, the sweep and the backlog's retirement.
  - rel: references
    to: adr-049-a-heading-is-declared-per-variant
    note: The schema mechanism behind criteria 2 to 5.
---

# Feature: filing an issue requires framing it

## Summary

The only path into the tracker is `/frame`, which frames the issue in
full — goal, interview, EARS criteria, branch — at creation. A user
who wants an issue recorded has no lighter path, and the tracker
cannot tell a framed issue from one filed in a hurry: both are `open`,
and the schema holds both to the framed form.

## Context

Framing captures decisions against the project as it stands. An issue
may sit in the tracker for a long time before it is worked on, and the
project moves meanwhile, so a framing made at filing is out of date
when the work starts. I028 to I030 were each filed by hand with `TBD`
criteria for this reason; I037 sat unframed for a week and was framed
against a schema landscape its filing had never seen.

[I037][sokf:issue-037-a-contracts-behaviour-is-not-written-as-ears]
then made a criterion a keyed EARS item
([ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items])
and asked how a bug's Expected behaviour could carry a key when 21 of
24 bug reports write it as a paragraph. The answer the owner gave is
this feature: the lifecycle distinguishes framed from unframed, the
schema holds a framed issue to the keyed form through a lifecycle
variant ([ADR-045][sokf:adr-045-a-schema-declares-variants]), and an
unframed issue is held only to the shape a quick record needs. The
backlog, which held "under consideration" entries beside the tracker
because filing was heavy, loses its reason to exist.

## Behaviour

An issue has four lifecycle states: `unframed`, `framed`, `done`,
`wontfix`. The folder is the state, as today; `issues/open/` becomes
`issues/unframed/` and `issues/framed/`.

`/file` is the light path in. Given a bug, a feature request or a
chore, it writes the minimum record — kind, title, description,
Summary and, where the kind carries it, Motivation, every other
section of the kind present with the user's words or `TBD — <the open
question>` — numbered after the
highest issue, filed `unframed` by `superdev validate --fix`, with no
interview, no branch, and no criteria the user did not state. Given an
idea — a thought with no kind yet, or one the user cannot yet say is
wanted — it writes an idea per `schema-idea` into `knowledge/ideas/`.
Given an existing idea and a kind, it promotes the idea to an unframed
issue that links it. Given no kind, or one it does not know, it asks.

`/frame` frames an unframed issue in place and sets it `framed`; run
with no issue, it files and frames in one pass, as today. Framing is
what the later phases wait for: contract-design, feature-plan and
execute-feature-plan refuse an unframed issue and return to `/frame`.

The schema enforces the states. While `unframed`, a criterion, a repro
step, an expected-behaviour item or a done item is a plain sentence or
a `TBD`, with no key and no EARS tag. While `framed`, every such item
carries its key and, where the item is a requirement, its EARS tag —
`AC_` criteria, `RS_` steps, `EX_` expected behaviour, `DD_` done
items — and a `TBD` is an error. A `done` or `wontfix` issue is held
to the framed form as well; a bug's Expected behaviour is a keyed list
in every state, and the settled reports that write it as a paragraph
are converted once, each paragraph becoming one `EX_c<n>` item.

The backlog retires: its three "under consideration" entries become
ideas, its "decided against" entry a `wontfix` issue, and the concept,
its schema, and every reference to it in the skills and indexes go.
ADRs keep rejected design alternatives, as they do today.

The feature is done when the tracker meets these expectations:

- The feature-request, bug-report and chore schemas declare `lifecycle`
  as one of `unframed`, `framed`, `done` and `wontfix`, and `superdev
  validate --fix` files an issue in the folder named by its value.
- While an issue is `unframed`, the schema requires its title,
  description, Summary and, where the kind carries it, Motivation, and
  every section heading of its kind, and accepts a criterion, repro
  step, expected-behaviour item or done item that is a plain sentence
  or opens with `TBD — `, with or without a key or an EARS tag.
- While an issue is `framed`, the schema holds every criterion to the
  `AC_` key and EARS tag, every repro step to the `RS_` key, every
  expected-behaviour item to the `EX_` key and EARS tag, every done
  item to the `DD_` key, and reports an item opening with `TBD` as an
  error naming the item.
- While an issue is `done` or `wontfix`, the schema holds it to the
  framed form of criterion 3, a bug's Expected behaviour included.
- Each kind keeps one schema, varying by `lifecycle` as its variant
  key, with one example per state that passes the schema's own check.
- When the user invokes `/file` naming a bug, a feature request or a
  chore, the skill creates the issue `unframed` with the minimum record
  of the proposed behaviour, numbered after the highest issue on file,
  filed by `superdev validate --fix`, and does not interview, branch,
  or write a criterion the user did not state.
- When the user invokes `/file` naming an idea, the skill creates an
  idea per `schema-idea` in `knowledge/ideas/`, listed in its index.
- When the user invokes `/file` naming an existing idea and a kind, the
  skill creates the unframed issue from the idea's text, linking the
  idea with `references`, and leaves the idea on file.
- When `/file` is invoked with no kind, or with one that is not bug,
  feature request, chore or idea, the skill asks for the kind and does
  not file.
- When `/frame` is invoked on an `unframed` issue, the skill frames it
  in place, replaces every `TBD`, keys and tags every cited item, and
  sets `lifecycle: framed`.
- When `/frame` is invoked with no existing issue, the skill files and
  frames in one pass, ending `framed`.
- When contract-design, feature-plan or execute-feature-plan is invoked
  on an `unframed` issue, the phase refuses and returns to `/frame`.
- The workflow in `.agents/superdev.md` lists `/file` as an entry
  outside the phases, and the how-do-i skill describes it.
- `/file` ships as a knowledge-carried skill in the pack and its synced
  copy, claimed in the lock like the seventeen that exist.
- Every open issue on file is refiled by the sweep: `unframed` where a
  `TBD` remains, `framed` otherwise; every bug report whose Expected
  behaviour is prose carries it as a numbered list, one `EX_c<n>` item
  tagged `[ubiquitous]` per paragraph, its words unchanged; and
  `superdev validate` passes on the live tree.
- The backlog concept, `schema-backlog` and every reference to the
  backlog in the skills, the schemas and the indexes are removed, its
  three under-consideration entries filed as ideas and its
  decided-against entry as a `wontfix` issue.
- The issue-tracker concept, the glossary's Lifecycle and EARS entries,
  and the changelog describe the four states and `/file`.

## Scope

The work reaches the tracker's states, schemas, skills and records and
leaves the rest of the workflow as it is.

- In: the four lifecycle states and their folders; the three tracker
  schemas as lifecycle-variant schemas with per-state examples; the
  `/file` skill for issues, ideas and promotion; `/frame` framing in
  place; the refusal gates on the three later phases; the workflow and
  how-do-i entries; the sweep of the thirteen open issues and of the
  bug reports' Expected behaviour; the backlog's retirement and the
  migration of its entries; the tracker concept, glossary and
  changelog.
- Out: triage tags, which stay as they are; the plan's own lifecycle;
  `/frame`'s interview, which is unchanged; a review or reminder for
  issues that stay unframed; GitHub issues; the idea schema's shape.

Alternatives considered:

- `open` kept as the unframed state, `framed` added — `open` would
  read as the superset it no longer is.
- Framed as a triage tag rather than a lifecycle — a schema cannot vary
  by a tag, so the framed form would go unenforced.
- A mode of `/frame` instead of `/file` — one skill with two behaviours
  a reader must tell apart, and the interview steps would have to be
  skipped by argument.
- Two skills, one per kind of record — two files for one act of
  filing; the kind is an argument.
- Any phase framing an unframed issue inline — an unattended run would
  frame without the interview.
- Keys on unframed criteria — nothing cites an unframed criterion, so
  a key there is a cost with no reader.
- Ideas retired into unframed feature requests — an idea has no kind
  and may never be wanted; an issue is a request.
- The backlog kept beside the tracker — an unframed issue is what an
  "under consideration" entry was, and the decided-against half has
  homes already.
- A bug's Expected behaviour left as prose in settled issues — the
  schema would need a content kind that varies by state, which ADR-045
  has no room for, to spare a one-off conversion of 21 paragraphs.

## Resolution

Accepted 2026-09-03 on `main` at `e79673c` (PR #15, plan-026). All
seventeen criteria pass: four probe issues filed by their lifecycle
value and held to their state's form through the built binary, the
three schemas varying by `lifecycle` with four examples each, `/file`
and the gates as written, the sweep on file, the backlog gone, the
records describing the states. The two questions the run deferred were
answered by the owner before the merge.

## Comments

2026-09-02, the owner, closing a decision I037 deferred: an issue's
lifecycle distinguishes framed from unframed. Once framed, an issue's
behaviour and acceptance criteria are keyed items
([ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]),
written as EARS where the item is a requirement; a bug's Expected
behaviour becomes a keyed `EX_` list on the framed variant, and settled
records stay as they are. Candidate answer for criterion 3, and the
schema mechanism is the `lifecycle` value as a variant key
(ADR-045).

2026-08-31, the user: it might help to have dedicated skills for ideas
and for issues — with how they would interact with `/frame` an open
question. Candidate answer for criterion 1; the knowledge already holds
an `ideas/` folder and `schema-idea` with no skill that files into
them.

2026-09-01, the user: does the backlog still make sense beside issues
and ideas? Candidate answer: retire it. Once filing is lightweight, an
"under consideration" entry is an unframed open issue; "decided
against" already has homes — the `wontfix` lifecycle for rejected work,
ADRs for rejected design alternatives. The framing decides the
taxonomy and where the backlog's four current entries migrate.

2026-09-02, framed. The three earlier comments' candidate answers
stand, settled in the interview: four lifecycle states, one `/file`
skill for issues and ideas, unframed criteria with no key and no tag,
the backlog retired, an idea kept as a record with no kind. The
criteria that were `AC_c1` to `AC_c3` are replaced by the seventeen
above. The owner then struck the prose exception for a settled bug's
Expected behaviour: it is a keyed list in every state, and the settled
reports are converted. Contract-design settled the mechanism in
[ADR-049][sokf:adr-049-a-heading-is-declared-per-variant] — one heading,
a rule per disjoint variant set, declared PENDING in
[contract-010][sokf:contract-010-interface-document-schemas] — and the
whole in
[ADR-048][sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed];
an expected-behaviour item takes the EARS tag as a criterion does, and
the unframed rule checks the list kind alone.

2026-09-02, delivered by plan-026 in six slices. The sweep at `90adee1`
filed 12 open issues `framed` and one `unframed` — I042, the only one
still carrying a `TBD` item; I030 itself went `framed`, no `TBD`
remaining. The 24 bug reports' Expected behaviour is keyed: 21 were
converted from prose, one `EX_c<n>` item per paragraph, words checked
against `6bee067`; the three already lists took keys and tags. An
`EX_` item carries the EARS tag, as a criterion does, because an
expected behaviour is a requirement (ADR-048's options table). The
backlog's entries became
[idea-007][sokf:idea-007-a-knowledge-capture-skill],
[idea-008][sokf:idea-008-templates-pre-fill-knowledge-skeletons] and
[idea-009][sokf:idea-009-comment-preserving-manifest-stamping] (idea-006
was taken between the cut and the build) and the wontfix chore
[I051][sokf:issue-051-pin-node-in-the-managed-repo]. `/file` ships
as the eighteenth knowledge-carried skill; `/frame` frames in place, and
contract-design, feature-plan and execute-feature-plan gate on `framed`.

<!-- sokf:links -->
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]: /knowledge/adrs/deprecated/adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed.md
[sokf:adr-049-a-heading-is-declared-per-variant]: /knowledge/adrs/active/adr-049-a-heading-is-declared-per-variant.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:idea-007-a-knowledge-capture-skill]: /knowledge/ideas/idea-007-a-knowledge-capture-skill.md
[sokf:idea-008-templates-pre-fill-knowledge-skeletons]: /knowledge/ideas/idea-008-templates-pre-fill-knowledge-skeletons.md
[sokf:idea-009-comment-preserving-manifest-stamping]: /knowledge/ideas/idea-009-comment-preserving-manifest-stamping.md
[sokf:issue-037-a-contracts-behaviour-is-not-written-as-ears]: /knowledge/issues/done/issue-037-a-contracts-behaviour-is-not-written-as-ears.md
[sokf:issue-051-pin-node-in-the-managed-repo]: /knowledge/issues/wontfix/issue-051-pin-node-in-the-managed-repo.md
