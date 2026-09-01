---
type: FeaturePlan
id: plan-020-feature-normative-shape-enforcement
title: Normative shape enforcement — feature plan
description: Slices delivering the body-pattern vocabulary, the EARS declaration, the contract-kind declarations and the contract sweep.
lifecycle: open
links:
  - rel: implements
    to: issue-034-feature-request-normative-shapes-are-described-but-not-enforced
    note: The plan delivers the framed issue's seven criteria.
---

# Feature plan: normative shape enforcement

Request:
[issue-034][sokf:issue-034-feature-request-normative-shapes-are-described-but-not-enforced]

## Slices

### Slice 1: The body-pattern vocabulary in the engine

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `item-pattern` and `content-pattern` land in
  `validate::schema` per ADR-030 — `SectionRule` fields, the item and
  body checks, the mis-declaration findings — with unit tests. No
  schema declares either yet.
- Done-check: a probe schema declaring each pattern produces the
  ADR-030 findings on a failing document and a failing schema; the live
  tree's findings are unchanged.
- Cases:
  - unit: an item failing a declared item-pattern is an error naming
    the file, the section and the item's first line — covers 1.
  - unit: a wrapped item matches after its continuation lines join, and
    a nested item is excluded from the matched text — covers 1.
  - unit: a section body failing a declared content-pattern is an error
    naming the file and the section — covers 2.
  - unit: an unanchored pattern matches mid-text; an anchored one binds
    the ends — covers 1, 2.
  - unit: a pattern that does not compile, and an item-pattern beside a
    non-list content kind, are findings on the schema file and bind
    nothing — covers 3.
  - unit: a schema declaring neither pattern adds no finding to any
    document — covers 5.
  - unit: a schema's example is checked against the declaring schema's
    own patterns (ADR-024 path) — covers 1.

### Slice 2: EARS criteria enforced at frame time

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `schema-feature-request` declares the ADR-031 item-pattern on
  Acceptance criteria, in `knowledge/schemas/` and the pack mirror,
  with the section description updated to name the declaration.
- Done-check: a probe feature-request with an untagged criterion fails
  validate naming the criterion; the shipped knowledge validates clean.
- Cases:
  - integration: a criterion without an EARS tag or `TBD — ` fails
    validate naming the file, the section and the item — covers 4.
  - integration: I030's TBD criteria and every on-file feature-request
    pass — covers 4, 6.

### Slice 3: The contract kinds declare their promise shapes

- [ ] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: the fifteen contract-kind schemas declare the ADR-032
  item-patterns and content-patterns, in `knowledge/schemas/` and the
  pack mirror; each declaring section's description names the shape;
  each schema's `example:` block satisfies its own declarations.
- Done-check: a live-repo test enumerates the ADR-032 assignment across
  the fifteen schemas and both trees; every schema example passes its
  own declared shapes.
- Cases:
  - integration: every ADR-032 section carries its declared pattern in
    both trees, and no definitional section carries one — covers 7.
  - integration: every contract-kind schema's example passes the
    schema's own declarations — covers 6.

### Slice 4: The on-file contracts pass the declared shapes

- [ ] Done — ticked by integrate at merge.
- Depends-on: 3.
- Change: the nine active contracts are swept until the ADR-032
  declarations pass — promise items gain their keywords, promise
  sections state their promises — with no change to what any contract
  binds.
- Done-check: `superdev validate` passes on the knowledge and the pack
  mirror with every declaration live; the sweep commits touch contract
  documents only.
- Cases:
  - e2e: a full validate run over the shipped knowledge and the pack
    mirror reports zero errors with every declared shape enforced —
    covers 6.

<!-- sokf:links -->
[sokf:issue-034-feature-request-normative-shapes-are-described-but-not-enforced]: /knowledge/issues/open/issue-034-feature-request-normative-shapes-are-described-but-not-enforced.md
