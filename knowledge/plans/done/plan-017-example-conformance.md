---
type: Plan
id: plan-017-example-conformance
title: Example conformance
description: Two blocks making validate check each schema's example against the schema that declares it — the document check in place, then link form without resolution — each landing with the reconciliation it surfaces.
lifecycle: done
links:
  - rel: implements
    to: issue-022-a-schemas-worked-example-is-checked-by-nothing
---

# Plan: Example conformance

Request: [issue-022-a-schemas-worked-example-is-checked-by-nothing][sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]

## Goal

The vocabulary and its semantics are fixed in
[contract-010-interface-document-schemas][sokf:contract-010-interface-document-schemas]
per ADR-024 and ADR-025; the user-facing promise in
[contract-002-cli-superdev][sokf:contract-002-cli-superdev]. Each block
lands its check and the live findings that check surfaces in one pass, so
integrate's validate gate stays green at every merge.

## Contract changes

- contract-010-interface-document-schemas: the `example` key's obligation
  added — the example is checked in place against the schema that declares
  it, every failure a finding on the schema file (ADR-024) — and the
  link-form rule added — an example's links bind by form and never resolve
  (ADR-025).
- contract-002-cli-superdev: `validate`'s schema half grows the example
  check.

## Work blocks

### Block 1: The example is checked in place against its own schema

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `DocSchema` parses the `example:` key; `validate` reads each
  schema's example as a document and runs the existing document check
  over it with the declaring schema handed to it — no dispatch —
  reporting every failure as an error on the schema file, prefixed so a
  reader sees it is the example that broke (ADR-024). An example that
  does not parse as a document — no frontmatter block, or frontmatter
  that is not YAML — is an error on the schema file. Live findings the
  check surfaces are reconciled in the same block, in
  `knowledge/schemas/` and the pack mirror alike.
- Done-check: `cargo test` passes; `superdev validate` on a fixture
  whose schema carries a broken example reports the error on the schema
  file, and on this repository reports no example finding.
- Cases:
  - unit: an example whose `id` breaks the declaring schema's own
    pattern is an error naming the schema file — covers 1.
  - unit: an example lacking a key the declaring schema marks required
    is an error naming the schema file — covers 1.
  - unit: an example missing a required section, and one whose section
    body lacks its declared content kind, are each errors naming the
    schema file — covers 2.
  - unit: an example satisfying its schema yields no finding — covers 6.
  - unit: an example with no frontmatter block, and one whose
    frontmatter is not YAML, are each errors naming the schema file —
    covers 5.
  - snapshot: a fixture tree whose schema carries a broken example
    carries a golden of the report — covers 1, 2, 5.

### Block 2: Link form binds inside the example, without resolution

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: the example check reads the example body's markdown links and
  reports, as an error on the schema file, a link whose target is a
  path into the knowledge — the `[text][sokf:<id>]` form is the
  accepted form for a concept link (ADR-025). No id or target is
  resolved: a fictional `sokf:` label passes, and a link whose target
  is outside the knowledge — a URL, a repository path — passes in its
  ordinary markdown form. Live findings are reconciled in the same
  block, both trees.
- Done-check: `cargo test` passes; `superdev validate` on a fixture
  whose example links into the knowledge by path reports the error, and
  on this repository reports PASS.
- Cases:
  - unit: an example body link whose target is a path into the
    knowledge is an error naming the schema file — covers 3.
  - unit: a `[text][sokf:<id>]` link naming no real concept passes —
    covers 4.
  - unit: a URL link and a repository-path link outside the knowledge
    each pass in ordinary markdown form — covers 4.
  - e2e: `superdev validate` reports PASS on this repository with the
    example check live — covers 6.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]: /knowledge/issues/done/issue-022-a-schemas-worked-example-is-checked-by-nothing.md
