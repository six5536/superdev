---
type: Decision
id: adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed
title: An issue's lifecycle distinguishes framed from unframed
description: An issue's lifecycle is one of unframed, framed, done and wontfix, the folder being the value; a new /file skill writes the minimum record as an unframed issue or an idea, /frame frames in place and sets framed, the later phases refuse an unframed issue, the tracker schema holds a framed or settled issue to the keyed EARS form and an unframed one to its headings alone, a bug's expected behaviour is a keyed tagged list in every state, and the backlog retires into ideas and a wontfix issue.
lifecycle: active
links:
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: The keyed EARS form a framed issue is held to; its `EX_` prefix is declared here.
  - rel: references
    to: adr-049-a-heading-is-declared-per-variant
    note: The schema mechanism that lets one heading carry an unframed rule and a framed one.
  - rel: references
    to: adr-045-a-schema-declares-variants
    note: The variant machinery, with `lifecycle` as the tracker schemas' variant key.
---

# ADR-048: An issue's lifecycle distinguishes framed from unframed

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

The only path into the tracker is `/frame`, which frames an issue in
full at creation, and every open issue is `open` whether framed or
filed in a hurry with `TBD` criteria. A framing made at filing goes
stale before the work starts; a quick record is held to the framed
form and escapes it only through the `TBD` placeholder, which the
schema admits while the issue is open.

[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]
made a criterion a keyed EARS item and could not key a bug's Expected
behaviour: 21 of 24 bug reports write it as a paragraph, and a rule
that keyed it would have failed every settled record. Framing I030,
the owner settled the shape: the lifecycle says whether an issue is
framed, the schema varies by it
([ADR-045][sokf:adr-045-a-schema-declares-variants]), and the settled
records are converted once.

## Decision

An issue's `lifecycle` is one of `unframed`, `framed`, `done` and
`wontfix`, and the folder is the value: `issues/unframed/`,
`issues/framed/`, `issues/done/`, `issues/wontfix/`. `open` is retired.

`/file` is a knowledge-carried skill and the light path in. Given a
bug, a feature request or a chore it writes the minimum record — kind,
title, description, Summary, Motivation, every other heading of the
kind with the user's words or `TBD — <the open question>` — numbered
after the highest issue, filed `unframed`, with no interview, no
branch and no criterion the user did not state. Given an idea, it
writes one per `schema-idea` into `knowledge/ideas/`; an idea is a
thought with no kind yet. Given an existing idea and a kind, it
promotes the idea to an unframed issue carrying a `references` link to
it, and leaves the idea on file. Given no kind, or one it does not
know, it asks and files nothing.

`/frame` frames an unframed issue in place and sets `framed`; run with
no issue it files and frames in one pass. Contract-design, feature-plan
and execute-feature-plan gate on `framed` and return an unframed issue
to `/frame`.

The three tracker schemas take `variant-key: lifecycle`. An unframed
issue is held to its headings and their content kinds alone: a cited
list's items — criteria, repro steps, expected behaviour, done items —
are plain sentences, `TBD`s or keyed items, unchecked beyond the list
kind. A framed, done or wontfix issue is held to the keyed form:
`AC_` key and EARS tag on every criterion, `RS_` key on every repro
step, `EX_` key and EARS tag on every expected-behaviour item, `DD_`
key on every done item, and no `TBD`. A bug's Expected behaviour is a
numbered list in every state; the settled reports that write it as a
paragraph are converted once, each paragraph one `EX_c<n>` item tagged
`[ubiquitous]`, its words unchanged. Each tracker schema carries one
example per state.

The open issues on file are swept: `unframed` where a `TBD` remains,
`framed` otherwise. The backlog retires: its three under-consideration
entries become ideas, its decided-against entry a `wontfix` chore with
the reasoning in its body, and the concept, `schema-backlog` and every
reference to the backlog in the skills, schemas and indexes go. ADRs
keep rejected design alternatives.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Four lifecycle states; `/file`; the schema varies by lifecycle; settled records converted | The state is enforceable, the light path exists, one schema per kind, the tracker is uniform | 50 files move folders once; 21 paragraphs become lists once |
| `open` kept as the unframed state, `framed` added | Fewer renames | `open` reads as the superset it no longer is |
| Framed as a triage tag | No folder change | A schema cannot vary by a tag, so the framed form is unenforced |
| A mode of `/frame` | One skill | Two behaviours a reader must tell apart, the interview skipped by argument |
| Two skills, `/file-issue` and `/file-idea` | Each narrow | Two files for one act of filing; the kind is an argument |
| A phase frames an unframed issue inline | No refusal | An unattended run frames without the interview |
| Keys on unframed criteria | Keys from the first draft | Nothing cites an unframed criterion; a key with no reader |
| Ideas retired into unframed feature requests | One record kind | An idea has no kind and may never be wanted |
| Expected behaviour left as prose in settled issues | No conversion | The content kind would vary by state, which ADR-045 has no room for; 21 paragraphs are cheaper |
| `EX_` items keyed and untagged | The sweep needs no tag | An expected behaviour is a requirement and reads as one |
| The backlog kept | Nothing migrates | An unframed issue is what an under-consideration entry was |

## Consequences

- Positive: filing costs a sentence; a framing is made when the work
  starts; the schema says what a framed issue owes and lets an
  unframed one breathe; a bug's expected behaviour is citable; one
  tracker, no backlog beside it.
- Negative: every issue moves folder once, every citation of
  `issues/open/` in prose is stale until refiled; a managed repository
  carrying a `Backlog` document loses its schema and must migrate;
  `/frame` and three phases gain a gate.
- Follow-ups: the three tracker schemas take `variant-key: lifecycle`
  and per-state rules through
  [ADR-049][sokf:adr-049-a-heading-is-declared-per-variant]; the
  `/file` skill ships in the pack; `/frame`, contract-design,
  feature-plan, execute-feature-plan and how-do-i change; the
  workflow lists `/file`; the sweep and the backlog migration run; the
  tracker concept, glossary and changelog follow.

<!-- sokf:links -->
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-049-a-heading-is-declared-per-variant]: /knowledge/adrs/active/adr-049-a-heading-is-declared-per-variant.md
