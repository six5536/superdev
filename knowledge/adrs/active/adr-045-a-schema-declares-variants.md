---
type: Decision
id: adr-045-a-schema-declares-variants
title: A schema declares variants
description: A schema names a frontmatter key as its variant discriminator, any rule in it may be tagged with the variants it applies to — untagged applies to all — and its example becomes one document per variant, so one schema governs one document shape with per-variant rules the validator enforces, in one ordered list, without inheritance.
lifecycle: active
links:
  - rel: references
    to: adr-043-one-contract-schema-and-twelve-kinds
    note: The first consumer — twelve kinds under one schema, whose per-kind sections this makes enforceable.
  - rel: references
    to: adr-027-an-include-block-materializes-shared-content-in-place
    note: Declined schema inheritance for its merge semantics; a tag on a rule in one ordered list needs none.
  - rel: references
    to: adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema
    note: The example stays self-testing — one per variant, each checked against the base and its variant.
---

# ADR-045: A schema declares variants

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

[ADR-043][sokf:adr-043-one-contract-schema-and-twelve-kinds] puts
every contract under one schema with a `kind`, and turns the sixteen
per-kind section lists into a checklist. As prose in the schema the
checklist is a nudge: a `cli` contract with no `### Exit codes` passes
validation and is caught, if at all, by the judgement step at
integration. The sixteen schemas had enforced those sections; one
schema lost the enforcement to gain the one file.

[ADR-027][sokf:adr-027-an-include-block-materializes-shared-content-in-place]
considered schema inheritance for a different need and declined it:
an `extends` key needs merge semantics for ordered section lists. What
is needed here is narrower. The shape is one; the kinds differ in
which of its sections apply. That is an attribute of a rule, not a
second tree.

## Decision

A schema MAY name a frontmatter key as its variant discriminator,
`variant-key: kind`. Any rule in the schema — a section, a frontmatter
key, a prohibited heading — MAY carry `variants: [<value>, …]`, the
values of that key it applies to. A rule with no tag applies to every
variant. A document validates against the rules its own value selects,
in the schema's one declared order, so `sections-ordered` holds on the
subsequence the document sees.

```yaml
variant-key: kind
sections:
  - heading: "Behaviour"          # untagged: every kind
    level: 2
    required: true
  - heading: "Exit codes"
    level: 3
    required: true
    variants: [cli]
  - heading: "Errors"
    level: 3
    required: true
    variants: [api, library]
  - heading: "Prompting"          # optional for the kinds it names
    level: 3
    variants: [cli]
```

A heading appears once in the list, with one requiredness across the
variants it names. A tag naming a value the discriminator's `enum`
does not carry, a `variants` tag in a schema with no `variant-key`,
and a discriminator value with no example are each a finding on the
schema, and the unreadable rule binds nothing.

When `variant-key` is set, `example` is a map keyed by variant value,
one conforming document each, every value present. Each example is
checked against the base rules and its own variant's, in place as
[ADR-024][sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema]
runs the check today, and its discriminator value MUST equal its key. A schema without
`variant-key` keeps `example` as one document.

The grammar governing schema files gains the three keys, contract-010
gains the rows, and the validator reads them in the same change.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| A `variants` tag on any rule; one example per variant | One ordered list; a variant is an attribute; no merge; every variant's rules exercised | Twelve short examples in the contract schema |
| A separate `variants:` tree appending rules per value | Rules grouped by variant | A second tree beside the first, and where an appended section sits in the order is a rule of its own |
| Schema inheritance, `extends` | Reuse across schemas | Merge semantics for ordered section lists — the cost ADR-027 declined |
| One base example plus per-variant fragments spliced in | Least example text | A splice rule — where a fragment lands — is the ordered-list merge in another form |
| Examples only for variants that add required sections | Fewer examples | An optional section's rule is never run and its description never checked |
| The checklist as prose, no mechanism (ADR-043 as first written) | Nothing to build | A nudge, not a check; the sixteen schemas' one merit given up |

## Consequences

- Positive: one schema enforces per-kind shape; the differences between
  kinds are declared in one place and read side by side; the mechanism
  is general — the tracker's three issue schemas are a natural later
  consumer.
- Negative: the contract schema carries twelve examples; a variant's
  `required` bites, so the author declares required only what always
  applies.
- Follow-ups: the grammar's schema vocabulary gains `variant-key`, the
  `variants` tag and the keyed `example`; contract-010 gains the rows;
  the contract schema's prose checklist becomes tagged rules with the
  guidance in each rule's `description`, and its example becomes
  twelve.

<!-- sokf:links -->
[sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema]: /knowledge/adrs/active/adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema.md
[sokf:adr-027-an-include-block-materializes-shared-content-in-place]: /knowledge/adrs/active/adr-027-an-include-block-materializes-shared-content-in-place.md
[sokf:adr-043-one-contract-schema-and-twelve-kinds]: /knowledge/adrs/active/adr-043-one-contract-schema-and-twelve-kinds.md
