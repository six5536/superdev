---
type: Decision
id: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
title: A promise and a criterion are keyed EARS items
description: Every promise in a contract's Behaviour and Stability is one bullet, and every acceptance criterion one numbered item, opening with a stable key in a code span — a prefix naming the kind of item, P_ for a promise or AC_ for a criterion, an underscore and a slug — and an EARS pattern tag; a promise carries one modal verb from SHALL, SHOULD and MAY with the interface element as its subject, prose describes and carries no modal verb, a numbered list under a contract is a sequence and never a promise, and a key is cited bare where its document is the subject and after the document's id elsewhere — criteria on file take the slug c<n> so every existing citation holds.
lifecycle: active
links:
  - rel: references
    to: adr-031-ears-criteria-are-checked-by-item-pattern
    note: The criterion form this extends to contracts, and the item-pattern that binds the tag.
  - rel: references
    to: adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source
    note: PENDING keeps its place beside the modal verb; an item carries it where a sentence did.
  - rel: references
    to: adr-047-a-section-rule-declares-item-keys-and-item-bounds
    note: The schema vocabulary that makes this form checkable.
  - rel: references
    to: adr-029-a-contract-is-a-binding-surface-not-a-specification
    note: Its keyword rule — any RFC 2119 verb, one requirement per sentence — survived its deprecation in the contract schema's style; this replaces it with the EARS item.
---

# ADR-046: A promise and a criterion are keyed EARS items

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

A feature-request's acceptance criteria are EARS sentences the
validator holds to their tag
([ADR-031][sokf:adr-031-ears-criteria-are-checked-by-item-pattern]).
The contract statements those criteria become are prose or bullets
carrying an RFC 2119 keyword, and the contract schema asks only that a
keyword appear somewhere in the section — the one rule of
[ADR-029][sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]
that outlived its deprecation. A bullet carrying two MUSTs,
a MUST in a descriptive paragraph and a section whose one keyword sits
in its intro all pass. The nine active contracts carry 175 such verbs.

A promise is bound by a test the project writes
([ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]),
and a test names what it binds. A number is positional: a contract is
edited whenever a feature touches its interface, so a cited number
rots at the rate the contract changes. Framing I037 the owner asked
what would happen to a number cited from a test once a promise was
inserted above it, and chose a stable key.

One rule on a level-2 section reaches every bullet beneath it, its
`###` subsections included. An interface contract's key flows are a
list of steps under Behaviour, and steps are not promises.

A criterion is cited the same way and rots the same way: 108 test doc
comments say "I0nn criterion n", every plan case says "covers 1, 3",
and a framed issue's criteria are inserted into while it is framed —
I049 grew to 24 across three reframes, I037 from 13 to 16 in one day.
The owner asked that criteria carry the same key.

## Decision

Every promise in a contract's Behaviour and Stability is one bullet of
this form:

```markdown
- `P_init-outside-git` [event] WHEN `init` runs outside a git
  repository, `init` SHALL refuse.
```

The item opens with its key in a code span, then the EARS pattern tag,
then the sentence in that pattern's words: the trigger or condition, the
interface element as the subject, one modal verb, one requirement. The
verb is `SHALL` or `SHALL NOT` for a requirement, `SHOULD` or `SHOULD
NOT` for a recommendation, `MAY` for an option; `MUST`, `REQUIRED`,
`RECOMMENDED` and `OPTIONAL` are retired from contracts. `PENDING`
sits beside the verb as
[ADR-044][sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]
places it. A contract admits no `TBD` item.

A key is `<PREFIX>_<slug>`: an uppercase prefix naming the kind of
item, an underscore, and a slug of lowercase letters and digits joined
by hyphens, `[a-z][a-z0-9]*(-[a-z0-9]+)*`. The prefix makes a key
recognisable in prose with nothing in front of it, and a key of
another kind's prefix under a list is an error. A key is unique within
its document, so a Behaviour promise and a Stability promise share the
one prefix and never a key.

| Prefix | Item |
|--------|------|
| `P_` | A contract promise, under Behaviour or Stability |
| `AC_` | An acceptance criterion, in a feature-request |
| `RS_` | A repro step, in a bug-report |
| `EX_` | An expected outcome, in a bug-report — reserved; declared once a framed issue is a lifecycle state (I030) |
| `DD_` | A definition-of-done item, in a chore |

The key is the item's identity. A rewording keeps it; a promise that
no longer holds is removed with its key, which is not reused; a new
promise takes a new key. A citation is the bare key where the document
is already the subject — a plan case covering its issue's criteria, a
test's doc comment on the feature it tests — and the document's id
followed by the key elsewhere: `contract-002-cli-superdev
P_init-outside-git`. A search for the key finds the item and every
citation; a search for a prefix lists every item of its kind in the
tree. Nothing joins id and key: the prefix says what the key is, and
`#` stays the region separator of a source include.

Prose under Behaviour and Stability describes and carries no modal
verb. A table stays where a kind's checklist wants one; its rows are
not sentences. A numbered list is a sequence — the steps of a flow —
and the item rules do not read it; the modal-verb bound does, so a
step cannot promise. A bullet is a promise, everywhere under both
sections.

The subject is the interface element, named in the active voice —
`init`, the validator, a caller, a served tool — never the criteria's
"THE SYSTEM": in a contract about one verb, the verb's name says more.

A criterion carries the key the same way, before its tag or its
`TBD`, in every list a plan case cites: a feature-request's
Acceptance criteria, a bug-report's Steps to reproduce and Expected
behaviour, a chore's Definition of done. A repro step or an expected
behaviour carries the key and no tag: a step is not a requirement.
The list stays numbered; the key is the identity and the number the
reading order. A plan case names the keys it covers, bare.

```markdown
1. `AC_stale-include` [event] WHEN an include is stale THE SYSTEM SHALL
   report an error.
2. `AC_region-scope` TBD — whether a region may span two files.
```

The tracker's fifty issues are swept once: every cited item on file
takes the slug `c<n>`, `n` its number — `AC_c11`, `RS_c2` — so "I049
criterion 11" and "covers 1, 3" stay true as citations of `AC_c11`,
`AC_c1` and `AC_c3`, and no settled record is reworded. A criterion
written from then on takes a named slug. Open plans cite keys; settled
plans stand.

The contract and tracker schemas carry the form as declarations
([ADR-047][sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]);
no skill changes for it. The nine active contracts, the fifty issues
and the schemas' examples are swept to conform in one feature.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| A bullet with a code-span key, tag, one verb from SHALL/SHOULD/MAY | Same form as a criterion; a stable identity a citation survives insertion on; one rule per level-2 section; a grep finds promise and citations | Every promise in nine contracts rewritten once |
| Numbered items, cited by heading and number | No new syntax | A number is positional; every insertion renumbers the promises below it and rots every citation |
| Bullets with no key, cited by heading and trigger clause | No new syntax | No identity survives a reworded trigger, and nothing a machine could match a test to |
| The key in bold, or after the tag with a colon | Reads as a label | A code span is what a grep for the key already finds; the tag-first form hides the key mid-line |
| An unprefixed slug, cited as `<id>#<key>` | Shorter keys | A bare slug in prose is not recognisable as a key, so every citation needs the id; `#` is already the region separator |
| A prefix per section — `B_`, `S_`, `EB_` | The section readable from the key | Two rules for six prefixes, and `B_` beside `EB_`; the document already says which section a promise is in |
| A prose shorthand — `I049#c11` — as the citation | Short | `I049` is a habit of the prose, defined nowhere and resolved by nothing |
| `MUST` kept beside `SHALL` | No verb sweep | A criterion and the item it becomes differ in their verb for no reason |
| `THE SYSTEM SHALL`, as the criteria say | One subject everywhere | A contract about one verb names its actor less well than the verb's name does |
| Item rules on every level-3 rule instead of the level-2 one | Key flows untouched | Some forty rules carrying the same patterns, and bullets directly under `## Behaviour` unchecked |
| Key flows as prose so every list is a promise | One list kind | Steps read worse as paragraphs; a numbered list is what a sequence already is |
| Convert the statements by `--fix` | No hand sweep | A promise's words are an authoring decision; a wrong trigger is worse than a missing tag |
| Criteria keyed in the same feature, the slug `c<n>` for those on file | One mechanism, one sweep; every existing citation still resolves | Fifty issues touched by a script; two slug styles coexist on file |
| Criteria in a later feature | I037 stays contracts-only | Two sweeps of the same mechanism, and citations rot in the meantime |
| Named slugs for open issues, `c<n>` for settled | Open issues read better | I049's 24 criteria are cited from tests today; every citation rewritten by hand |
| Keys on open issues only | No sweep of settled records | The schema must vary by lifecycle, or settled issues fail validation |

## Consequences

- Positive: a promise and the criterion it satisfies read the same
  and are cited the same; each is countable and citable, and the
  citation holds while items are inserted around it; a modal verb in
  prose is caught, so a requirement cannot hide in a description.
- Negative: 175 statements are rewritten once, each keyed by hand; an
  author writes a key per promise and per criterion from now on; the
  numbered-list exemption under a contract is a rule a reader must
  know; `c<n>` slugs sit beside named ones on file.
- Follow-ups: the contract schema's Behaviour and Stability rules take
  `item-key`, `item-pattern`, `item-only-pattern` and
  `item-prohibited-pattern`, and its twelve examples take the form;
  the three tracker schemas take `item-key` on their cited lists and
  the plan schema's case rule cites keys; contract-002 to contract-010
  and the fifty issues are swept; the glossary defines the form and
  the key; the grammar rules' RFC 2119 reference already treats
  `SHALL` as `MUST`, so `.agents/superdev.md` stands.

<!-- sokf:links -->
[sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]: /knowledge/adrs/deprecated/adr-029-a-contract-is-a-binding-surface-not-a-specification.md
[sokf:adr-031-ears-criteria-are-checked-by-item-pattern]: /knowledge/adrs/active/adr-031-ears-criteria-are-checked-by-item-pattern.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]: /knowledge/adrs/active/adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source.md
[sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]: /knowledge/adrs/active/adr-047-a-section-rule-declares-item-keys-and-item-bounds.md
