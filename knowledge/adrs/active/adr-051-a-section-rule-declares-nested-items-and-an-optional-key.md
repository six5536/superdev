---
type: Decision
id: adr-051-a-section-rule-declares-nested-items-and-an-optional-key
title: A section rule declares nested items and an optional key
description: A section rule carries `nested`, the rule for the items one level below its own — `item-key`, `item-pattern`, `item-prohibited-pattern`, `required` and its own `nested`, to any depth — and `item-key-optional`, under which an item matching `item-key` is held to the keyed form and one not matching it is a plain item; keys are unique across every level of a document.
lifecycle: active
links:
  - rel: references
    to: adr-047-a-section-rule-declares-item-keys-and-item-bounds
    note: The item declarations a nested level repeats.
  - rel: references
    to: adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept
    note: The first consumer — a contract's criteria nested under the promise they check.
---

# ADR-051: A section rule declares nested items and an optional key

- Date: 2026-09-03
- Deciders: superdev maintainers

## Context

[ADR-047][sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]
binds a list section's top-level items and drops a nested item's
lines from the item above.
[ADR-050][sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]
puts a contract's criteria one level below the promise each checks,
keyed and tagged. That is not declarable: the vocabulary binds one
level. An optional key — an item held to the keyed form only where it
carries one — is the other shape a list may need, declared here with
it.

## Decision

A section rule MAY carry `nested`, the rule for the items one level
below its own. A nested rule carries `item-key`, `item-pattern`,
`item-prohibited-pattern`, `required` and its own `nested`, so a
schema declares as many levels as the document has. A nested item is a
marker of the section's list kind indented past the marker of the item
above it; a marker deeper than the deepest declared level, or of the
other list kind, is text of the item it sits in. `required` makes an
item of the level above with no item of this level an error naming it.
A key captured at any level is unique with every other key of the
document.

```yaml
- heading: "Behaviour"
  level: 2
  required: true
  content: bullet-list
  item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
  item-pattern: '^`P_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\] '
  nested:
    item-key: '^`(AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
    item-pattern: '^`AC_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\] '
```

A section rule MAY set `item-key-optional` beside `item-key`. An item
matching `item-key` is checked as it would be without the flag — its
pattern, its prohibited pattern, its nested rule; an item not matching
it is checked by `item-prohibited-pattern` alone. The flag on a rule
with no `item-key`, a `nested` on a section whose content is not a list
kind, and a nested `item-key` without exactly one capture are findings
on the schema.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| A recursive `nested` rule on the section rule | One shape per level, any depth, the item declarations reused | The section rule and the nested rule carry the same three keys |
| Flat `nested-item-key`, `nested-item-pattern`, `nested-item-required` | No new nesting in the grammar | One level only; a third level is three more keys |
| A per-depth list, `levels: [{...}, {...}]` | Any depth | The top level moves out of the section rule, and every schema on file changes |
| A code-span opening as the keyed test under the optional flag | Catches a malformed key | Two definitions of "keyed", one of them a character; the key pattern is already the definition |
| A separate keyed rule per list an item may or may not carry a key in | No flag | Two rules for one heading, differing in one flag |

## Consequences

- Positive: a document's list can carry structure the schema binds —
  a promise and its checks, a step and its sub-steps — without a
  second section; a list keyed where it chooses is declarable.
- Negative: the item reader learns depth; a nested key shares the
  document's key space, so a schema author cannot reuse a key across
  levels.
- Follow-ups: contract-010 carries the promises; the grammar documents
  `nested` and `item-key-optional`; `schema-contract` declares the
  nested criteria per ADR-050.

<!-- sokf:links -->
[sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]: /knowledge/adrs/active/adr-047-a-section-rule-declares-item-keys-and-item-bounds.md
[sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]: /knowledge/adrs/active/adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept.md
