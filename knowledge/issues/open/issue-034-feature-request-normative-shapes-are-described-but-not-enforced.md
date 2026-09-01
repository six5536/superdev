---
type: FeatureRequest
id: issue-034-feature-request-normative-shapes-are-described-but-not-enforced
title: The shapes of normative text are described to the writer and checked by nothing
description: EARS criteria and the contract style standard live in schema description prose the validator never reads, so a malformed criterion or a requirement buried in narrative passes validate.
lifecycle: open
links:
  - rel: references
    to: contract-010-interface-document-schemas
    note: Gains the item-pattern and content-pattern rows and their found-anywhere semantics (ADR-030).
---

# Feature: the shapes of normative text are described but never enforced

## Summary

The shapes normative text takes — EARS acceptance criteria in
feature-requests, RFC 2119 requirements in contracts — are fixed in
schema `description` prose the validator never reads. A criterion
missing its pattern tag and a requirement buried in a narrative
paragraph both pass `superdev validate`.

## Motivation

P019 shipped the binding-surface standard
([ADR-029][sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification])
as an included fragment: instructions to the writer, checked by nothing.
The sweep it drove passed acceptance on a keyword census while the
contracts kept their requirements inside narrative paragraphs —
[I033][sokf:issue-033-bug-two-contracts-escaped-the-modal-sweep] was the
first symptom, and the standard still fails on the swept contracts. The
tracker has the same gap: `schema-feature-request` states the EARS shape
in a section's `description`, so a malformed criterion is caught only by
whoever reads it. Where the schema engine decides a shape — content
kinds, heading patterns, table columns — the documents on file conform;
where it does not, they drift, the fault
[I011][sokf:issue-011-feature-request-index-shape-is-described-but-not-enforced]
records for indexes.

## Proposed behaviour

A schema binds the shape of normative text, and the validator enforces
the binding. Once this is done:

- A schema may declare a per-item shape on a list section, and a body
  shape on any section; an item or a section body that does not match
  is a validate error naming the file, the section and — for an item —
  the item. The schema config defines where the checks apply — the
  engine imposes nothing a schema does not declare.
- `schema-feature-request` declares the EARS opening tag on Acceptance
  criteria, so a criterion without its pattern tag fails validate at
  frame time.
- The contract-kind schemas declare shapes for their requirement-bearing
  sections; which sections and what shapes is contract-design's
  decision. Prose stays legal and valuable — it describes.
- The contracts on file pass whatever the schemas declare, restructured
  where a declared shape demands it.
- The contract-style fragment keeps the rules the validator cannot
  decide: bind only what callers rely on, link the ADR, restate no
  reasoning.

## Acceptance criteria

1. [event] WHEN a schema declares an item shape on a list section and a
   document's item does not match THE SYSTEM SHALL report a validate
   error naming the file, the section and the item.
2. [event] WHEN a schema declares a body shape on a section and the
   document's section body does not match THE SYSTEM SHALL report a
   validate error naming the file and the section.
3. [event] WHEN a schema declares a shape that does not compile, or an
   item shape on a section without a list content kind, THE SYSTEM
   SHALL report the finding on the schema file.
4. [event] WHEN a feature-request acceptance criterion does not open
   with an EARS pattern tag THE SYSTEM SHALL fail validate naming the
   criterion.
5. [state] WHILE a schema declares no shape on a section THE SYSTEM
   SHALL check nothing new about that section.
6. [ubiquitous] THE SYSTEM SHALL validate the shipped knowledge and the
   pack mirror clean with every declared shape enforced.
7. [ubiquitous] THE SYSTEM SHALL document each new declaration in the
   document-schemas contract and in each schema that carries it.

## Alternatives considered

- Keep enforcement in the writer's path — fragment instructions plus
  review; P019 ran exactly this and the contracts drifted anyway, which
  is the failure this issue records.
- A standalone contract lint outside the schema engine — a second
  constraint vocabulary beside the one the schemas already declare.
- A global keyword-placement policy compiled into the validator — the
  engine would impose checks no schema declared; the schema config is
  where a check's scope is defined.
- Full EARS grammar parsing — heavy machinery; the opening-tag shape
  catches every drift observed so far.

## Scope

- In: the schema vocabulary addition, the validator check behind it,
  the EARS declaration on `schema-feature-request`, the contract-kind
  schemas' declarations as contract-design decides them, restructuring
  the contracts on file until the declared checks pass, the
  [document-schemas contract][sokf:contract-010-interface-document-schemas]
  and the pack mirror.
- Out: the undecidable style rules, which stay in the contract-style
  fragment; checks no schema declares; index shape and index-entry
  drift (I011, I010), which the same mechanism may later serve; shapes
  for bug-report repro steps and chore done-lists.

## Comments

Acceptance withheld. Every criterion here passes on the feature head:
the two declarations land, the EARS tag is enforced, the fifteen
contract-kind schemas declare their promise shapes and the corpus
conforms. The owner read the swept CLI contract and judged the result
insufficient — the shapes make a contract's sentences bind, and leave
the contract itself unbuildable, which is what a contract is for.
[I035][sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]
carries the raised bar; this issue closes with it, so the mechanism and
the standard it serves are accepted together.

<!-- sokf:links -->
[sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]: /knowledge/adrs/deprecated/adr-029-a-contract-is-a-binding-surface-not-a-specification.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-011-feature-request-index-shape-is-described-but-not-enforced]: /knowledge/issues/open/issue-011-feature-request-index-shape-is-described-but-not-enforced.md
[sokf:issue-033-bug-two-contracts-escaped-the-modal-sweep]: /knowledge/issues/done/issue-033-bug-two-contracts-escaped-the-modal-sweep.md
[sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]: /knowledge/issues/open/issue-035-feature-request-a-contract-does-not-define-its-interface.md
