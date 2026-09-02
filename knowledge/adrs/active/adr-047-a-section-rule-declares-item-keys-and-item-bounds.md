---
type: Decision
id: adr-047-a-section-rule-declares-item-keys-and-item-bounds
title: A section rule declares item keys and item bounds
description: A section rule may declare item-key, a one-capture regex every item matches whose capture is unique across the document; item-only-pattern, a regex that may match only inside an item; and item-prohibited-pattern, a regex no item may match — three general declarations the contract and tracker schemas compose into the keyed EARS item, each reading an item as item-pattern does.
lifecycle: active
links:
  - rel: references
    to: adr-030-a-section-rule-declares-body-patterns
    note: The two body patterns these three sit beside, and the item reading they share.
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: The consumers — the contract schema's Behaviour and Stability rules, and the tracker schemas' cited lists.
  - rel: references
    to: adr-045-a-schema-declares-variants
    note: The precedent for a general declaration a schema composes rather than a contract-specific check.
---

# ADR-047: A section rule declares item keys and item bounds

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]
makes a contract's promise a bullet with a key, a tag and one verb, and
wants five things checked: the key present and well formed, the key
unique in the contract, no modal verb outside an item, no retired verb
inside one, one verb per item. The vocabulary of
[ADR-030][sokf:adr-030-a-section-rule-declares-body-patterns] can say
two of them: `item-pattern` requires the key, the tag and a verb of
each item, and a prohibited two-verb shape is a pattern too. It cannot
say that a capture is unique, that a pattern may not appear outside an
item, or that no item may match a pattern — every declaration it has
is positive and per-item or per-body.

A check written for contracts alone would live in the validator and
not in the schema, which is what
[ADR-045][sokf:adr-045-a-schema-declares-variants] declined for the
kinds: the schemas are self-contained, and a skill or a validator
learns nothing per schema.

## Decision

A section rule MAY declare three more keys, each a regex the
validator's own wrapper compiles:

```yaml
- heading: "Behaviour"
  level: 2
  content: bullet-list
  item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
  item-pattern: '(?s)^`P_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b'
  item-only-pattern: '\b(SHALL|SHOULD|MAY|MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b'
  item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|(?s)\b(SHALL|SHOULD|MAY)\b.*\b(SHALL|SHOULD|MAY)\b'
```

`item-key` is a regex with one capture group. Every top-level item of
the section's declared list kind must match it, and the capture is the
item's key; the list's prefix is part of the pattern, so a key of
another kind's prefix is an item with no match. A key is unique across every item of the document under a
rule declaring `item-key`; a repeat is a finding naming the key and
both items, and an item with no match is a finding naming the item. An
`item-key` with no capture group, or on a section whose `content` is
not a list kind, is a finding on the schema and binds nothing.

`item-only-pattern` is a regex that may match only inside a top-level
item of the section's declared list kind. A match on any other body
line — prose, a table row, a heading, an item of the other list kind —
is a finding naming the section and the line. It needs no list
`content`: on a section declaring none it forbids the pattern
everywhere in the body.

`item-prohibited-pattern` is a regex no top-level item may match. A
match is a finding naming the item and the matched text. On a section
whose `content` is not a list kind it is a finding on the schema.

All three read an item as `item-pattern` does — its own lines with the
marker stripped and continuations joined, nested items and fenced
blocks excluded — and skip fenced blocks in the body, so one item
model serves five declarations. Each finding is an error. The grammar
gains the three keys, contract-010 the rows, and the validator reads
them in the same change.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Three general declarations on the section rule | Any schema composes them; one item model; the contract schema stays self-contained | Three keys in the vocabulary; the two-verb rule is a `(?s)` regex an author must read |
| A contract-specific check in the validator | No vocabulary change | The form lives in code, not the schema; a managed project's schema cannot vary it |
| `item-key: true` naming `item-pattern`'s first capture | One key fewer | The key's format hides inside the tag pattern, and a key with no tag pattern is impossible |
| `item-prohibited-pattern` carrying a `message` the finding quotes | The finding names the admitted verb | The first declaration to carry prose the validator prints; the rule's `description` already says why |
| A `max-matches` count for the one-verb rule | Reads as a count, not a regex | A fourth key for one rule the regex already states |

## Consequences

- Positive: the keyed EARS item is enforced by the schemas alone; the
  tracker's three schemas take `item-key` on the lists a plan cites,
  and a plan's case rule can later be checked against them.
- Negative: the vocabulary grows by three; a `(?s)` regex is the
  one-verb rule's only spelling; a key is unique per document, so two
  sections cannot reuse one.
- Follow-ups: `SectionRule` gains the three fields and the grammar the
  three keys (declared in this change); the validator reads them; the
  contract schema declares them on Behaviour and Stability, and the
  tracker schemas `item-key` on their cited lists.

<!-- sokf:links -->
[sokf:adr-030-a-section-rule-declares-body-patterns]: /knowledge/adrs/active/adr-030-a-section-rule-declares-body-patterns.md
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
