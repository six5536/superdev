---
type: Decision
id: adr-049-a-heading-is-declared-per-variant
title: A heading is declared per variant
description: A schema may declare one heading in more than one section rule when the rules' variants sets are disjoint and none is untagged, so one heading carries a different shape per variant at one place in the order; two rules for a heading that overlap, or one of them untagged, are a finding on the schema and bind nothing.
lifecycle: active
links:
  - rel: references
    to: adr-045-a-schema-declares-variants
    note: The rule this relaxes — a heading appeared once, with one requiredness across the variants it named.
  - rel: references
    to: adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed
    note: The first consumer — a tracker's cited lists unkeyed while unframed and keyed once framed.
---

# ADR-049: A heading is declared per variant

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

[ADR-045][sokf:adr-045-a-schema-declares-variants] tags a rule with
the variants it applies to and says a heading appears once in the
list, with one requiredness across the variants it names. That served
the contract schema, where a variant adds or drops a section and never
reshapes one.

[ADR-048][sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]
needs one heading in two shapes: an unframed issue's Acceptance
criteria is a numbered list and nothing more, a framed one's is the
keyed EARS list of ADR-046 with `item-key`, `item-pattern` and no
`TBD`. One rule cannot say both; a document sees one state.

## Decision

A schema MAY declare one heading in more than one section rule when
every such rule carries `variants` and the rules' sets are disjoint. A
document is checked against the one its discriminator value selects,
at that heading's one place in the declared order; requiredness,
content kind and every pattern are the selected rule's own.

```yaml
- heading: "Acceptance criteria"
  level: 2
  required: true
  content: numbered-list
  variants: [unframed]
- heading: "Acceptance criteria"
  level: 2
  required: true
  content: numbered-list
  item-key: '^`(AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'
  item-pattern: '^`AC_[a-z0-9-]+` \[(ubiquitous|event|state|conditional|optional|complex)\] '
  variants: [framed, done, wontfix]
```

Two rules naming one heading whose sets share a value, or of which
one is untagged, are a finding on the schema naming the heading and
the overlap, and both bind nothing. Two rules name one heading by
declaration form — the same `heading` literal, or the same
`heading-pattern`, at one level; a literal beside a pattern that
matches it is two headings, the literal binding its own and the
pattern the rest, which is how a schema declares fixed headings beside
a catch-all. A heading declared once keeps ADR-045's reading.
Contract-010 carries the rule.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| The same heading in several rules with disjoint variants | Uses the existing tag; a variant's shape reads whole; no merge | A heading's rules sit side by side, and disjointness is a schema check |
| Per-variant overrides inside one rule | One rule per heading | A `variants:` map of overrides is new nesting in the grammar and a merge rule per key |
| Two schemas per kind, dispatched by type and lifecycle | No vocabulary change | Doubles the tracker schemas the one-schema push just cut, and `type` alone dispatches today |
| One rule whose patterns admit both shapes | No vocabulary change | `item-key` requires a key; an unframed criterion has none |

## Consequences

- Positive: a variant can reshape a section, not only add or drop
  one; the tracker keeps one schema per kind.
- Negative: a reader of a schema sees a heading twice and must read
  the tags; the validator gains a disjointness check.
- Follow-ups: the validator checks disjointness and selects one rule
  per heading; the grammar's `variants` doc says a heading may recur
  with disjoint sets; ADR-045's "appears once" reads as "once per
  variant".

<!-- sokf:links -->
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
[sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]: /knowledge/adrs/deprecated/adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed.md
