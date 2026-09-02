---
type: FeatureRequest
id: issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears
title: A contract's behaviour is free prose with a modal verb, while a criterion about it must be EARS
description: Acceptance criteria are held to EARS and the contract statements they are about are not, so one kind of statement has two forms — and the contract's form buries the trigger, admits two requirements in one sentence, and gives a promise no stable identity a test can name; a criterion's number is no stable identity either.
lifecycle: done
links:
  - rel: references
    to: adr-031-ears-criteria-are-checked-by-item-pattern
    note: The item-pattern that binds a criterion's EARS tag is the mechanism that binds a contract statement's.
  - rel: references
    to: adr-043-one-contract-schema-and-twelve-kinds
    note: The one schema whose Behaviour and Stability rules this feature changes.
  - rel: references
    to: adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source
    note: PENDING sits beside the modal verb; a numbered item carries it where a sentence did.
  - rel: references
    to: issue-049-feature-request-a-contract-cannot-point-at-its-definition
    note: Gave a Definition its bound form; this gives Behaviour and Stability theirs.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Gains `item-key`, `item-only-pattern` and `item-prohibited-pattern`, declared in `SectionRule` and the grammar with their checks PENDING (ADR-047).
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: The promise form — key, tag, one verb, subject, citation — the numbered-list exemption, and the criterion keyed the same way with `c<n>` for those on file.
  - rel: references
    to: adr-047-a-section-rule-declares-item-keys-and-item-bounds
    note: The three schema declarations behind criteria 2 to 8.
---

# Feature: a contract's behaviour is not written as EARS

## Resolved

Accepted 2026-09-02 on `main` at `9834972` (PR #13, plan-025). All
twenty-one criteria pass: a probe contract carrying every departure
the criteria name drew a named error each through the built binary,
`PENDING` beside a verb passed, `--fix` rewrote no statement, and the
live tree — nine swept contracts, twelve examples, fifty keyed issues
— validates. The one skill change is the frame step's key, decided by
the owner. `EX_` moved to I030's framed lifecycle.

## Summary

A feature-request's acceptance criteria are EARS sentences, one
requirement each, and the validator holds them to the tag.
The contract statements those criteria become are prose or bullets
carrying an RFC 2119 keyword, checked only for the keyword's presence
somewhere in the section. One kind of statement — a requirement — has
two forms in this repository, and only one of them can be counted,
named or checked.

## Motivation

The nine active contracts carry 175 modal verbs across their Behaviour
and Stability sections; none sits in an EARS sentence. The form buries
the trigger and admits more than one requirement per sentence:

> **`init`** MUST refuse a directory that is not a git repo, and MUST
> refuse a re-run once `.superdev/config.toml` exists.

is two requirements, each with its trigger in a subordinate clause. As
EARS items they separate, each names what sets it off, and each carries
a key a test can cite that survives the next insertion above it:

> - `P_init-outside-git` [event] WHEN `init` runs outside a git
>   repository, `init` SHALL refuse.
> - `P_init-rerun` [state] WHILE `.superdev/config.toml` exists,
>   `init` SHALL refuse a re-run.

The contract schema's `content-pattern` on Behaviour and Stability asks
only that a keyword appear once in the section, so a bullet carrying
two MUSTs, a MUST in a descriptive paragraph, and a section whose one
keyword is in its intro all pass today. The machinery to bind the form
ships already: the `item-pattern` that binds a criterion's opening tag
([ADR-031][sokf:adr-031-ears-criteria-are-checked-by-item-pattern])
binds a contract item's the same way, and a level-2 rule's body
includes its `###` subsections, so one rule on Behaviour reaches every
item beneath it.

[I049][sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]
gave the Definition a form the validator binds. Behaviour and Stability
are bound by the project's tests
([ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]),
and a test can name a promise only when the promise has a form and an
identity. A number is positional and a contract is edited whenever a
feature touches its interface, so a numbered reference rots at the rate
the contract changes; a key does not.

## Proposed behaviour

A contract's Behaviour and Stability sections state every promise as a
bulleted EARS item: a key, the pattern tag, the trigger or condition in
that pattern's words, the interface element as the subject, one modal
verb, one requirement. The verb is `SHALL` or `SHALL NOT` for a requirement,
`SHOULD` or `SHOULD NOT` for a recommendation, and `MAY` for an option;
`MUST` and its RFC 2119 siblings are retired from contracts, so a
criterion and the contract item it becomes read the same. A subject is
named as the grammar rules' active voice names one — `init`, the
validator, a caller, a served tool — never "THE SYSTEM" of a criterion,
which in a contract about one verb says less than the verb's name.

Prose stays in both sections for description — "Every verb acts on the
current directory" — and carries no modal verb. A table stays where the
kind's checklist wants one; its rows are not sentences. A `PENDING`
marker sits beside an item's verb as
[ADR-044][sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]
places it beside a sentence's.

The key is the promise's stable identity: the prefix `P_`, then a slug
of lowercase words joined by hyphens, unique within the contract across
both sections, so a test, a plan case or a comment cites
`contract-002-cli-superdev P_init-outside-git` and the citation holds
while promises are inserted around it. The prefix makes a bare key
recognisable in prose as a promise; a key of another kind's prefix is
an error. A rewording keeps the key; a
promise that no longer holds is removed with its key, which is not
reused; a new promise takes a new key.

`superdev validate` reports each departure: an item in either section
with no key or no EARS tag, a malformed key, a key used twice in one
contract, a modal verb outside an item, a retired verb, an item with
more than one verb, an item with a tag and no verb, and a `TBD` item —
a contract promises or says `PENDING`, and never defers. The finding
names the section and the item, as the criterion check's does.

A criterion carries a key the same way, before its tag or its `TBD`,
in every list a plan case cites — `AC_` on a feature-request's
Acceptance criteria, `RS_` on a bug-report's Steps to reproduce,
`DD_` on a chore's Definition of done — and a plan case names the keys
it covers, bare. A repro step or
an expected behaviour carries the key and no tag. The list stays
numbered for reading order; the key is the identity, cited bare where
the issue is the subject and after the issue's id elsewhere. Every
cited item on file takes the slug `c<n>`, `n` its number — `AC_c11` —
so the 108 test comments and 27 plans that cite a number today stay
true; a criterion written from now on takes a named slug.

The schemas are the only files that carry the form; the skills read it
there. The nine active contracts, the fifty issues on file and the
schemas' examples are swept to conform, so the live tree validates with
nothing left to convert by hand.

## Acceptance criteria

1. `AC_c1` [ubiquitous] The contract schema SHALL declare Behaviour and
   Stability as bullet lists whose every item carries a key and one of
   the six EARS tags — `[ubiquitous]`, `[event]`, `[state]`,
   `[conditional]`, `[optional]`, `[complex]` — and prose and tables
   beside the list SHALL remain permitted in both sections.
2. `AC_c2` [event] WHEN an item under Behaviour or Stability, at any heading
   depth within the section, carries no EARS tag or no key, THE SYSTEM
   SHALL report an error naming the section and the item.
3. `AC_c3` [event] WHEN an item's key is not `P_` followed by lowercase words
   joined by hyphens, THE SYSTEM SHALL report an error naming the item
   and the key.
4. `AC_c4` [event] WHEN two items in one contract carry the same key, THE
   SYSTEM SHALL report an error naming the key and both items.
5. `AC_c5` [event] WHEN a modal verb — `SHALL`, `SHOULD`, `MAY`, `MUST`,
   `REQUIRED`, `RECOMMENDED` or `OPTIONAL`, with or without `NOT` —
   appears in Behaviour or Stability outside an item, THE SYSTEM SHALL
   report an error naming the section and the line.
6. `AC_c6` [event] WHEN an item under Behaviour or Stability carries `MUST`,
   `MUST NOT`, `REQUIRED`, `RECOMMENDED` or `OPTIONAL`, THE SYSTEM
   SHALL report an error naming the item, the matched verb and the
   schema rule.
7. `AC_c7` [event] WHEN an item under Behaviour or Stability carries more than
   one modal verb — `NOT` belongs to its verb — or a tag and no modal
   verb, THE SYSTEM SHALL report an error naming the item.
8. `AC_c8` [event] WHEN an item under Behaviour or Stability opens with `TBD`,
   THE SYSTEM SHALL report an error naming the item; the criterion
   form's `TBD` is not admitted in a contract.
9. `AC_c9` [ubiquitous] An item carrying `PENDING` beside its modal verb, as
   ADR-044 places it, SHALL pass the checks above.
10. `AC_c10` [ubiquitous] A finding from criteria 2 to 8 SHALL be an error that
    fails `superdev validate`, and `superdev validate --fix` SHALL NOT
    rewrite a statement or supply a key.
11. `AC_c11` [ubiquitous] The contract schema SHALL state the citation form —
    the bare key where the contract is the subject, the contract's id
    followed by the key elsewhere — and a search for the key SHALL
    find the promise and every citation.
12. `AC_c12` [ubiquitous] The contract schema's twelve examples SHALL each carry
    Behaviour and Stability in the form, and SHALL pass the schema's
    own check.
13. `AC_c13` [ubiquitous] Every active contract's Behaviour and Stability SHALL
    conform, with each former sentence's requirements as one keyed
    item each and no promise dropped, and `superdev validate` SHALL
    pass on the live tree.
14. `AC_c14` [ubiquitous] The grammar that governs schema files SHALL accept the
    contract schema as changed, and contract-010 SHALL carry any row
    the change adds.
15. `AC_c15` [ubiquitous] No skill SHALL change for the form; the contract
    schema alone carries it.
16. `AC_c16` [ubiquitous] The glossary SHALL define the form of a contract
    promise and its key, and the changelog SHALL carry the change
    under Unreleased.
17. `AC_c17` [ubiquitous] The feature-request, bug-report and chore schemas
    SHALL declare a key on every item of the lists a plan case cites
    — `AC_` on Acceptance criteria; `RS_` on Steps to reproduce;
    `DD_` on Definition of done — of the
    form criterion 3 names with that prefix, unique within the issue,
    and a missing, malformed or duplicate key SHALL be an error as
    criteria 2 to 4 report one.
18. `AC_c18` [ubiquitous] A criterion's key SHALL precede its EARS tag or its
    `TBD`, a repro step's or an expected behaviour's key SHALL stand
    alone, the list SHALL stay numbered, and each schema SHALL state
    the citation form of criterion 11.
19. `AC_c19` [ubiquitous] The feature-plan schema SHALL have a case name the
    keys of the criteria it covers.
20. `AC_c20` [ubiquitous] Every issue on file SHALL carry a key on each cited
    item — the slug `c<n>`, `n` the item's number, for an item on file
    at the sweep — every open plan's cases SHALL cite keys, and a
    settled plan or a test comment citing a number SHALL stand as a
    citation of the `c<n>` key.
21. `AC_c21` [ubiquitous] The three tracker schemas' and the feature-plan
    schema's examples SHALL carry the form and pass their own check,
    and `superdev validate` SHALL pass on the live tree.

## Alternatives considered

- Bind the tag only where a section already lists per requirement, and
  leave prose sections prose — half the promises stay uncountable, and
  the sweep is deferred to whoever next touches each section.
- Keep `MUST` beside `SHALL` — a criterion and the item it becomes then
  differ in their verb for no reason, and the sweep must choose per
  sentence.
- `THE SYSTEM SHALL` in contracts, as the criteria say — the subject of
  a contract about one verb is the verb, and "the system refuses `init`"
  names the actor less well than "`init` refuses".
- Numbered items, the identity being the heading and the number — a
  number is positional, so every insertion renumbers the promises below
  it and rots every reference to them.
- Criteria keyed in a later feature — two sweeps of one mechanism, and
  108 test citations rot in the meantime.
- Keys on open issues only — the schema would have to vary by
  lifecycle, or settled issues fail validation.
- Bulleted items with no key, cited by heading and trigger clause — no
  identity survives a reworded trigger, and nothing a machine could
  ever match a test to.
- Convert the statements by `--fix` — a rewrite of a promise's words is
  an authoring decision, and a wrong trigger in a promise is worse than
  a missing tag.

## Scope

- In: the contract schema's Behaviour and Stability rules, the key
  and the citation form, and its twelve examples; the validator checks
  behind criteria 2 to 8; the grammar and contract-010 where a schema
  key is added; the sweep of the nine active contracts; the key on the
  tracker schemas' cited lists and the plan's case citations, with the
  sweep of the issues on file and the open plans; the glossary and the
  changelog.
- Out: acceptance criteria, which are EARS already and keep "THE SYSTEM";
  the Definition, which binds by materialisation and whose doc comments
  are the source's; the sentence beyond its tag and verb, whose grammar
  is the author's; converting statements automatically; checking that
  a plan case's keys exist in its issue, or that a test cites every key — testing a project's behaviour is a stated
  non-goal, and the key is what such a check would need if the
  non-goal were ever lifted; the grammar rules of `.agents/superdev.md`,
  whose RFC 2119 reference already treats `SHALL` as `MUST`.

## Comments

Filed without framing, at the owner's request, as an instance of the
lightweight filing
[I030][sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]
asks for. Framed 2026-09-02 against the one contract schema
([ADR-043][sokf:adr-043-one-contract-schema-and-twelve-kinds]), which
replaced the sixteen kind schemas and the keyword rule the filing
cited; the four questions the filing left open are settled in the
proposed behaviour. The key was added the same day: the owner asked
what would happen to a number cited from a test once a promise was
inserted above it, and chose a stable key over a number or a bare
bullet. Contract-design settled the form in
[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items] and
the schema vocabulary in
[ADR-047][sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds],
declared in
[contract-010][sokf:contract-010-interface-document-schemas]; criterion
6 was narrowed there to what a general declaration can say — the
matched verb and the rule, not the verb admitted in its place. The
owner then asked for the same key on acceptance criteria; criteria 17
to 21 fold it in, with the slug `c<n>` for the items on file so no
citation rots, and a prefix naming the kind of item on every key —
`P_`, `AC_`, `RS_`, `EX_`, `DD_` — so a bare key is recognisable in
prose; a prefix per section was declined for putting `B_` beside
`EB_`.

Delivered by
[plan-025][sokf:plan-025-feature-a-contracts-behaviour-is-written-as-ears]
in eight slices. The sweep counted 182 modal verbs across the nine
contracts' Behaviour and Stability at the sweep (175 at framing;
contract-010 had gained the PENDING declarations since) and wrote 174
keyed promises: contract-002 60 verbs to 58 promises, contract-004 17
to 13, contract-008 19 to 17 — each drop a sentence that stood in two
places and is now one promise cited from the other — and every other
contract one promise per verb. The tracker sweep keyed the fifty issues
on file: 141 criteria `AC_c<n>`, 72 repro steps `RS_c<n>`, 22 done
items `DD_c<n>`, no settled record reworded beyond the key; plan-025's
33 case citations were rewritten to keys. `EX_` on Expected behaviour
was deferred, since the section is prose in the bug-report schema and
in 21 of the 24 bug reports on file; the owner settled it the same
day: an issue's lifecycle will distinguish framed from unframed, and a
framed issue's behaviour and criteria are keyed EARS items — `EX_`
lands there, under
[I030][sokf:issue-030-feature-request-filing-an-issue-requires-framing-it].
`AC_c17` was narrowed to the three lists this feature keyed.

<!-- sokf:links -->
[sokf:adr-031-ears-criteria-are-checked-by-item-pattern]: /knowledge/adrs/active/adr-031-ears-criteria-are-checked-by-item-pattern.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-043-one-contract-schema-and-twelve-kinds]: /knowledge/adrs/active/adr-043-one-contract-schema-and-twelve-kinds.md
[sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]: /knowledge/adrs/active/adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source.md
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]: /knowledge/adrs/active/adr-047-a-section-rule-declares-item-keys-and-item-bounds.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]: /knowledge/issues/open/issue-030-feature-request-filing-an-issue-requires-framing-it.md
[sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]: /knowledge/issues/open/issue-049-feature-request-a-contract-cannot-point-at-its-definition.md
[sokf:plan-025-feature-a-contracts-behaviour-is-written-as-ears]: /knowledge/plans/done/plan-025-feature-a-contracts-behaviour-is-written-as-ears.md
