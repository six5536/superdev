---
type: Plan
id: plan-018-validate-path-dispatch
title: Validate path dispatch
description: Two blocks making validate check a named file as what it is — the schema half reaches a named path first, then the grammar half stops misreading documents and parity is proved end to end.
lifecycle: done
---

# Plan: Validate path dispatch

Request: [issue-019-validate-reads-a-named-file-as-a-skill][sokf:issue-019-validate-reads-a-named-file-as-a-skill].

## Goal

The bug's five expected-behaviour sentences are the criteria the cases
cite. The contract is
[ADR-026][sokf:adr-026-a-named-document-is-checked-with-bare-run-parity]:
bare-run parity, with the fallback kind only for files nothing claims.

## Contract changes

- contract-002-cli-superdev: the `validate` bullet's `PATH` sentence
  changed to promise bare-run parity for a named document (ADR-026),
  replacing "only what it names is read".

## Work blocks

### Block 1: The schema half reaches a named path

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `validate_repo` in
  `crates/lib/superdev-core/src/validate/mod.rs` builds the bare run's
  context for a named path too — the knowledge and the schema set load
  whenever a run has paths, named files join the document candidates —
  and findings are reported only for what the paths cover.
- Done-check: `validate knowledge/architecture.md` checks the document
  against `schema-architecture` and reports a non-zero schema count;
  findings name no file outside the named path.
- Cases:
  - integration: a named concept's schema findings equal the bare run's
    findings for that file — covers 1.
  - integration: a named README.md is checked against `schema-readme`'s
    glob dispatch, and CHANGELOG.md likewise — covers 2.
  - unit: a named file whose `type` names no schema gets the bare run's
    unknown-type finding — covers 3.
  - unit: a named run reports no finding about a file the paths do not
    cover — covers 1.
  - e2e: an unreadable path fails naming the path — covers 5.

### Block 2: The grammar half stops misreading a document

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: the named-path branch classifies before the grammar sees the
  file — a file dispatched as a document by `type` or glob never takes
  the grammar's fallback kind; the fallback applies only to a file no
  schema and no grammar kind claims.
- Done-check: `validate knowledge/architecture.md` exits 0 with zero
  findings, matching the bare run; `validate <a-skill-outside-the-roots>`
  still checks it as a unit.
- Cases:
  - e2e: `validate knowledge/architecture.md` reports no skill-grammar
    finding and its verdict equals the bare run's for that file —
    covers 1.
  - unit: a named file with no frontmatter that no glob and no grammar
    kind claims takes the fallback kind, so a skill outside the roots
    stays checkable — covers 4.
  - unit: a named schema file keeps its grammar kind and is checked as
    a schema, never as a document candidate — covers 1.
  - e2e: for each of a concept, README.md and a skill, the named run's
    findings equal the bare run's findings for that file — covers 1, 2.

<!-- sokf:links -->
[sokf:adr-026-a-named-document-is-checked-with-bare-run-parity]: /knowledge/adrs/active/adr-026-a-named-document-is-checked-with-bare-run-parity.md
[sokf:issue-019-validate-reads-a-named-file-as-a-skill]: /knowledge/issues/done/issue-019-validate-reads-a-named-file-as-a-skill.md
