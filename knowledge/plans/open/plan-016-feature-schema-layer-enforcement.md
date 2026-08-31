---
type: FeaturePlan
id: plan-016-feature-schema-layer-enforcement
title: Schema layer enforcement — feature plan
description: Three slices making the validator read what the schemas declare — content kinds, the frontmatter contract, and the required-key vocabulary — each landing with the reconciliation it surfaces.
lifecycle: open
---

# Feature plan: the schema layer's declarations bind

Request: [issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else][sokf:issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else]

The vocabulary and its semantics are fixed in
[contract-010-interface-document-schemas][sokf:contract-010-interface-document-schemas];
the user-facing promise in
[contract-002-cli-superdev][sokf:contract-002-cli-superdev]. Each slice
lands its check and the live findings that check surfaces in one pass, so
integrate's validate gate stays green at every merge.

## Slices

### Slice 1: Content kinds bind by presence

- [ ] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `validate::schema` reads each section rule's `content` kind and
  reports, as an error naming the document, the section and the schema, a
  matched section whose body lacks the kind's form — one bullet, one
  numbered item, one table, one fenced block, or one plain paragraph line
  for prose (ADR-023). A schema declaring a kind outside the five is
  reported on the schema file. The live findings the check surfaces are
  reconciled in the same slice — the document fixed or the schema's
  declaration corrected — in `knowledge/schemas/` and the pack mirror
  alike.
- Done-check: `cargo test` passes; `superdev validate` on a fixture with
  a bullet-less bullet-list section reports the error, and on this
  repository reports no content-kind error.
- Cases:
  - unit: a bullet-list section with no bullet anywhere is an error
    naming the document, the section and the schema — covers 1.
  - unit: a bullet-list section opening with a lead-in sentence before
    its bullets passes — covers 2.
  - unit: one pass and one fail case per remaining kind — numbered-list,
    table, code, prose — covers 1.
  - unit: a section inside a fenced code block is not parsed as content
    — a `#` or `-` line in a fence neither satisfies nor breaks a kind —
    covers 1.
  - unit: a schema declaring `content: essay` is reported on the schema
    file — covers 5.

### Slice 2: The frontmatter contract binds on present values

- [ ] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `DocSchema` parses every frontmatter key's constraint block —
  today it reads only `type` and `lifecycle` — and reports, as an error
  naming the document, the key and the schema, a present value that
  breaks its `const`, `pattern` or `enum`. A `pattern` that does not
  compile is reported on the schema file and binds nothing. A key
  declared with only a `description` is unchecked. Live findings are
  reconciled in the same slice, both trees.
- Done-check: `cargo test` passes; `superdev validate` on a fixture with
  an id breaking its schema's pattern reports the error, and on this
  repository reports no frontmatter-value error.
- Cases:
  - unit: a present value breaking its `pattern` is an error naming the
    document, the key and the schema — covers 3.
  - unit: a present value outside its `enum`, and one differing from its
    `const`, are each errors — covers 3.
  - unit: a key declared with only a `description` passes any value —
    covers 3.
  - unit: an absent key with constraints and no `required` flag is not
    reported — covers 4.
  - unit: a schema `pattern` that does not compile is reported on the
    schema file — covers 5.

### Slice 3: Required keys, declared across the schemas

- [ ] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: the per-key `required: true` flag (ADR-022) is read, and an
  absent key marked required is an error naming the document, the key
  and the schema. The 53 schemas each declare their required keys —
  `type` and `id` on filed kinds, `title` and `description` where the
  document's listing depends on them — in `knowledge/schemas/` and the
  pack mirror, byte-identical. Any absence the declarations surface in
  the live tree is fixed in the same slice.
- Done-check: `cargo test` passes; every schema in both trees declares
  its required keys; `diff -rq knowledge/schemas pack/knowledge/schemas`
  prints nothing.
- Cases:
  - unit: an absent key marked `required: true` is an error naming the
    document, the key and the schema — covers 4.
  - unit: a present key marked required passes its value checks as in
    slice 2 — covers 3, 4.
  - e2e: `superdev validate` reports PASS on this repository, every
    document against its schema's content kinds and frontmatter
    contract — covers 6.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else]: /knowledge/issues/open/issue-018-feature-request-the-schema-layer-checks-sections-and-nothing-else.md
