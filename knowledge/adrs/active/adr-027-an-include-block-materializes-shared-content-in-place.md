---
type: Decision
id: adr-027-an-include-block-materializes-shared-content-in-place
title: An include block materializes shared content in place
description: A knowledge document carries shared content between sokf:include markers — validate --fix splices the named concept's body in place and validate errors on a stale copy — so a schema stays self-contained on disk while the content has one authored home.
lifecycle: active
---

# ADR-027: An include block materializes shared content in place

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

The contract style standard (ADR-029) must reach every writer of a
contract document. The one read every writer is guaranteed to make is
the document's schema — the retrieval rule reads `schema-{type}` before
any document is opened — so the standard must be inside the 15
contract-kind schemas, yet 15 authored copies drift. The format already
has a generated-block precedent: the `<!-- sokf:links -->` block is
written by tooling and rewritten whenever a concept moves (SPEC §9).
Schema inheritance was considered and is heavier: an `extends` key
needs merge semantics for ordered section lists, a contract-010
revision, and a corpus migration.

## Decision

We will add an include block to the SOKF knowledge format. A document
authors a marker pair — `<!-- sokf:include <id> -->` …
`<!-- /sokf:include -->` — and the content between the markers is
generated, not authored: `validate --fix` splices in the body of the
concept the id names (frontmatter excluded), and `validate` reports an
error when the materialized copy is absent or differs from its source.
The markers carry the provenance; the file on disk stays self-contained
for a plain read. The block is available to any knowledge document; its
first consumers are the 15 contract-kind schemas, which include the
contract style standard. The standard's source concept ships with the
owned schema set, so managed repositories receive and refresh it with
their schemas. The SOKF SPEC is the normative home for the block's
authoring rules — the section lands beside the generated definition
block (§9), where the link rules already live — because every session
reads the SPEC at bootstrap, so format law written there needs no
other carrier.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Materialized include | One authored home; files self-contained for plain Read and grep; reuses the generated-block machinery; drift is a validate error | The content exists on disk once per including file |
| Read-time include | Files stay small; one authored home | A plain read of the file misses the content — the self-containment the schemas need is lost |
| Schema inheritance (`extends`) | Deduplicates declarations as well as prose | Merge semantics for ordered section lists, a contract-010 revision and a corpus migration — a feature, not a fix |
| A copy in each schema | No mechanism at all | 15 copies drift silently |
| A skill companion file | Ships with the writer skill | A writer editing a contract outside the skill reads only the schema and never sees the standard |

## Consequences

- Positive: shared prose gets one home with enforced copies; the issue
  schemas' repeated tracker conventions are a natural later consumer.
- Negative: including files grow by the included content, and every
  edit to the source concept rewrites them all on the next `--fix`.
- Follow-ups: contract-002's `--fix` list grows the materialization;
  `sokf_read` and search index the materialized copy as part of the
  including document, which needs no new serve-side code.
