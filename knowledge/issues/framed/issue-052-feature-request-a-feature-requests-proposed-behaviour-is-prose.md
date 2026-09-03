---
type: FeatureRequest
id: issue-052-feature-request-a-feature-requests-proposed-behaviour-is-prose
title: A feature request's proposed behaviour is prose, and its criteria check nothing named
description: A framed feature request describes its proposed behaviour in paragraphs no plan, test or reader can cite, beside a criteria list whose items check behaviours the document never keys; a bug's expected behaviour became keyed EARS items under ADR-048 and a feature's proposed behaviour did not, so the two kinds of "the behaviour once done" have two forms, and a criterion's trace to the behaviour it checks is in the reader's head.
lifecycle: framed
links:
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: The keyed EARS form a proposed behaviour item takes; `PB_` is a new prefix under its one-prefix-per-kind rule.
  - rel: references
    to: adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed
    note: Keyed a bug's Expected behaviour and left a feature's Proposed behaviour prose; held wontfix to the framed form, which this feature relaxes to either form across the tracker.
  - rel: references
    to: adr-049-a-heading-is-declared-per-variant
    note: The mechanism by which Proposed behaviour is declared once per state.
---

# Feature: a feature request's proposed behaviour is prose

## Summary

A framed feature request's Proposed behaviour is paragraphs, so no plan
case, test or reader can cite a behaviour the feature promises, and its
Acceptance criteria sit in a separate list that names no behaviour, so
which criterion checks which behaviour is a reading. A bug's Expected
behaviour is a keyed EARS list in every state
([ADR-048][sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]);
a feature's Proposed behaviour is not.

## Motivation

[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]
and ADR-048 keyed every cited list in the tracker — `AC_`, `RS_`, `EX_`,
`DD_` — and left one requirement-bearing section prose: the 20 feature
requests on file describe their proposed behaviour in 1 to 8 paragraphs
each, I049's running to eight with two bullet lists inside. Accept
checks the user documentation against that section and has nothing to
tick; a plan case covers `AC_` keys and cannot say which behaviour the
criterion stands for. The section is the draft of the documentation and
reads as documentation; what it promises is held to nothing.

## Proposed behaviour

A feature request's Proposed behaviour is one numbered list. Each
top-level item is a behaviour the feature promises, written as a
criterion is: a key in a code span — `PB_` then a slug of lowercase
words joined by hyphens — then its EARS pattern tag, then the sentence
in that pattern, the subject what a user sees or a caller gets. Each
behaviour is followed by a nested numbered list of the criteria that
check it, in the `AC_` keyed form the criteria already take. The
Acceptance criteria heading goes; a criterion lives under the behaviour
it checks, so the trace from check to behaviour is the list's shape and
nothing cites anything.

```markdown
## Proposed behaviour

1. `PB_json-object` [conditional] IF `--json` is given, `validate`
   SHALL emit the report as one JSON object.
   1. `AC_json-verdict` [event] WHEN `--json` is given THE SYSTEM SHALL
      write an object carrying `verdict`, `counts` and `findings`.
   2. `AC_json-finding` [ubiquitous] THE SYSTEM SHALL give each finding
      its file, severity and message.
2. `PB_text-unchanged` [ubiquitous] `validate` SHALL leave the text
   output byte-identical when `--json` is absent.
   1. `AC_text-golden` [ubiquitous] THE SYSTEM SHALL pass the existing
      text golden unchanged.
```

The list is numbered in every state, the heading declared once per
state ([ADR-049][sokf:adr-049-a-heading-is-declared-per-variant]).
While `unframed`, a top-level item
is a plain sentence, a `TBD — <the open question>` or a keyed item, and
a nested criteria list may sit under any item; the schema checks the
list kind alone, as it does an unframed bug's Expected behaviour.
`/file` writes the user's stated behaviour as top-level items and the
criteria they state beneath them.

While `framed` or `done`, every top-level item carries its `PB_` key
and tag, every nested item its `AC_` key and tag, every behaviour has
at least one criterion beneath it, no criterion sits at the top level,
every key is unique within the issue across both prefixes, and a `TBD`
is an error. `superdev validate` reports each departure naming the
issue, the section and the item.

While `wontfix`, the section accepts either form, because a request
may be declined framed or unframed: an item carrying a key is held to
the keyed form — well formed, tagged, unique, a `PB_` with its
criteria beneath it — and an item without one is a plain sentence or a
`TBD`. Every tracker kind's cited list reads the same under `wontfix`:
a bug's Steps to reproduce and Expected behaviour, a chore's Definition
of done. ADR-048's rule that a wontfix issue is held to the framed form
is amended.

The 21 feature requests on file, this one included, are worked to fit
by judgement, every
key they carry kept: each paragraph of the prose becomes a `PB_` item,
each existing criterion moves beneath the behaviour it checks, and a
framed or done behaviour no criterion checks gets one. The numbering a
settled issue's prose cites — "criterion 12" — no longer matches the
nested reading order; the key does, and ADR-046's citation rule already
names the key.

`/frame` writes the section in the framed form — the behaviour first,
its criteria beneath it — and `/accept` walks the criteria under each
behaviour and checks the documentation against the behaviours. A plan
case still covers `AC_` keys.

## Acceptance criteria

1. `AC_framed-keyed` [state] WHILE a feature request is `framed` or
   `done` THE SYSTEM SHALL report an error naming the issue, the section
   and the item for a top-level Proposed behaviour item without a
   well-formed `PB_` key followed by an EARS tag.
2. `AC_nested-keyed` [state] WHILE a feature request is `framed` or
   `done` THE SYSTEM SHALL report an error naming the issue, the section
   and the item for a nested Proposed behaviour item without a
   well-formed `AC_` key followed by an EARS tag.
3. `AC_behaviour-checked` [state] WHILE a feature request is `framed` or
   `done` THE SYSTEM SHALL report an error naming the behaviour for a
   top-level item with no nested item beneath it.
4. `AC_criterion-placed` [state] WHILE a feature request is `framed` or
   `done` THE SYSTEM SHALL report an error naming the item for an `AC_`
   key at the top level or a `PB_` key in a nested item.
5. `AC_keys-unique` [state] WHILE a feature request is `framed`, `done`
   or `wontfix` THE SYSTEM SHALL report an error naming the key for a
   `PB_` or `AC_` key used twice in one issue.
6. `AC_no-tbd` [state] WHILE a feature request is `framed` or `done`
   THE SYSTEM SHALL report an error for a Proposed behaviour item, top
   level or nested, reading `TBD`.
7. `AC_unframed-free` [state] WHILE a feature request is `unframed` THE
   SYSTEM SHALL accept a Proposed behaviour list whose items are plain
   sentences, `TBD`s or keyed items, with or without nested lists, and
   report only a section that is not a numbered list.
8. `AC_wontfix-either` [state] WHILE an issue of any tracker kind is
   `wontfix` THE SYSTEM SHALL hold each keyed item of a cited list to
   the keyed form — a well-formed key, its tag where the list takes one,
   uniqueness, and for a `PB_` item its nested criteria — and accept an
   unkeyed item as a plain sentence or a `TBD`.
9. `AC_heading-gone` [ubiquitous] THE SYSTEM SHALL report an error for
   a feature request carrying an Acceptance criteria heading.
10. `AC_schema-examples` [ubiquitous] THE SYSTEM SHALL ship
    `schema-feature-request` with one worked example per state, each
    passing its own schema, the framed and done examples in the nested
    form.
11. `AC_sweep` [ubiquitous] THE SYSTEM SHALL carry every feature request
    on file in the form its state demands, each key it carried before
    the sweep still present, and `superdev validate` passing.
12. `AC_frame-skill` [ubiquitous] THE SYSTEM SHALL ship a `/frame` skill
    whose behaviour step writes each behaviour as a `PB_` item and whose
    criteria step nests each `AC_` under the behaviour it checks.
13. `AC_file-skill` [ubiquitous] THE SYSTEM SHALL ship a `/file` skill
    that writes a feature request's Proposed behaviour as a numbered
    list of the user's words, with any criteria the user states beneath
    the behaviour they belong to.
14. `AC_accept-skill` [ubiquitous] THE SYSTEM SHALL ship an `/accept`
    skill that walks the criteria beneath each behaviour and checks the
    user documentation against the `PB_` items.
15. `AC_adr-amended` [ubiquitous] THE SYSTEM SHALL carry ADR-048's
    wontfix rule amended to either form, the `PB_` prefix recorded under
    ADR-046's one-prefix-per-kind rule, and the tracker concept and
    glossary describing the nested section.

## Alternatives considered

- A trailing `covers PB_x` clause on each criterion, the two sections
  kept — the trace is a citation the validator must resolve, written
  twice for a criterion checking two behaviours, and read in two places.
- Proposed behaviour keyed, the criteria uncited — the two lists overlap
  in wording and nothing says which checks which.
- The `EX_` prefix shared with a bug's Expected behaviour — one prefix
  under two headings, the similar-but-not-the-same shape ADR-046
  rejected for `B_`/`EB_`.
- Prose while unframed, a list once framed — the content kind would
  differ from the tracker's other cited lists, all of which are lists in
  every state.
- Wontfix held to the framed form, as ADR-048 has it — a request
  declined unframed would have to be framed to be declined.
- A mechanical sweep, `PB_c<n>` per paragraph — cannot nest a criterion
  under a paragraph, so the framed and done requests would fail the
  form the day it landed.

## Scope

- In: the merged, nested Proposed behaviour section of
  `schema-feature-request` across its four states; the `PB_` prefix;
  the either-form wontfix rule in all three tracker schemas; the
  validator checks the nested form needs; the sweep of the 21 feature
  requests on file and of ADR-048; the `/file`, `/frame` and `/accept`
  skills; the tracker concept, glossary and changelog.
- Out: keying a chore's or a bug's sections further than ADR-048 did;
  citing `PB_` keys from a plan case; a validator check that a `PB_`
  item's sentence carries a modal verb — the frame writes it, as it
  writes a criterion's.

<!-- sokf:links -->
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]: /knowledge/adrs/active/adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed.md
[sokf:adr-049-a-heading-is-declared-per-variant]: /knowledge/adrs/active/adr-049-a-heading-is-declared-per-variant.md
