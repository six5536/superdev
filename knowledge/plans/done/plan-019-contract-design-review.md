---
type: Plan
id: plan-019-contract-design-review
title: Contract-design review and the binding-surface standard
description: Blocks delivering the include-block mechanism, the contract style standard in the schemas, the skill's explicit go-ahead, and the nine-contract sweep.
lifecycle: done
---

# Plan: contract-design review and the binding-surface standard

Request:
[issue-028-contract-design-commits-before-the-go-ahead][sokf:issue-028-contract-design-commits-before-the-go-ahead]
and
[issue-029-contract-design-writes-verbose-prose][sokf:issue-029-contract-design-writes-verbose-prose];
the decisions are ADR-027, ADR-028 and ADR-029. Case labels name the
issue and criterion they cover ("I029 c1").

## Goal

`/contract-design` presents its change set and commits only on the
user's explicit approval, and every contract it writes conforms to the
binding-surface standard of ADR-029: RFC 2119 verbs, one requirement
per sentence, enumerable surfaces in the kind's structured form, and
the reasoning left in the linked ADRs. The standard ships inside every
contract-kind schema through the include-block mechanism of ADR-027,
so the schemas carry it rather than restate it, and the nine active
contracts are swept to the form.

## Contract changes

- contract-002-cli-superdev: the `--fix` bullet carries the
  include-block materialization block 1 implements (ADR-027);
  rewritten to ADR-029 form in block 5; RFC 2119 modal verbs on every
  promise in block 6, meaning unchanged.
- contract-003-api-sokf: rewritten to ADR-029 form in block 5; modal
  verbs on every promise in block 6, meaning unchanged.
- contract-004-config-superdev: rewritten to ADR-029 form in block 5.
- contract-005-format-pack: rewritten to ADR-029 form in block 5.
- contract-006-format-lock: rewritten to ADR-029 form in block 5.
- contract-007-interface-pack-resolution: rewritten to ADR-029 form in
  block 4 — doc comments reduced to what the code enforces, each rule
  linking its ADR.
- contract-008-format-template: rewritten to ADR-029 form in block 5.
- contract-009-interface-run-state: rewritten to ADR-029 form in
  block 4.
- contract-010-interface-document-schemas: rewritten to ADR-029 form
  in block 4.

## Work blocks

### Block 1: The include mechanism

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: the SOKF SPEC gains the include-block section beside the
  generated definition block (§9); `validate --fix` materializes
  `<!-- sokf:include <id> -->` … `<!-- /sokf:include -->` from the
  named concept's body, frontmatter excluded; bare `validate` errors on
  a stale, empty or unresolvable include; CHANGELOG entry. Code in
  `crates/lib/superdev-core/src/validate/`.
- Done-check: cargo tests green with the new fixtures; a second `--fix`
  run writes nothing.
- Cases:
  - unit: `--fix` fills an empty include block with the named concept's
    body, frontmatter excluded — covers I029 c1–4 (the standard's
    carrier).
  - unit: bare `validate` errors on a stale copy, naming the file and
    the id — covers I029 c1–4 (the standard's carrier).
  - unit: an include naming no concept is an error naming the id —
    covers I029 c1–4 (the standard's carrier).
  - unit: a second `--fix` run writes nothing — covers I029 c1–4 (the
    standard's carrier).
  - integration: editing the source concept and running `--fix`
    rewrites every including file — covers I029 c1–4 (the standard's
    carrier).

### Block 2: The standard and its carriers

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: the contract style standard as a source concept shipped with
  the owned schema set, stating ADR-029's four rules; the include
  marker pair added to the 15 contract-kind schemas and materialized;
  pack backport of the schemas and the concept.
- Done-check: `superdev validate` passes; every contract-kind schema
  contains the materialized standard; the pack mirror matches.
- Cases:
  - unit: the standard concept states the four rules as RFC 2119
    sentences — covers I029 c1–4.
  - integration: each of the 15 contract-kind schemas carries the
    materialized standard after `--fix` — covers I029 c1–4 (the
    standard's carrier).
  - integration: `sokf_read` of a contract-kind schema returns the
    standard as part of the schema — covers I029 c1–4 (the standard's
    carrier).

### Block 3: The skill's explicit go-ahead

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: the contract-design skill restructured per ADR-028, in
  `.claude/skills/contract-design/SKILL.md` and its pack mirror: the
  interview binds to every ADR decision, a present-the-change-set step
  precedes the commit, and the commit is conditioned on explicit
  approval with rework re-presented.
- Done-check: the skill's process orders interview → present → approve
  → commit; `superdev validate` passes; the mirrors are identical.
- Cases:
  - inspection: the process puts the interview before ADR filing and
    the presentation before the commit — covers I028 c1, c2.
  - inspection: the commit step names rework re-presentation and the
    withheld-approval outcome — covers I028 c3, c4.
  - e2e: the next attended `/contract-design` transcript shows the
    interview and the approval before its commit — covers I028 c1–4.

### Block 4: Sweep the internal contracts

- [x] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: contracts 007, 009 and 010 rewritten to ADR-029 form —
  narrative doc comments reduced to what the code enforces with the
  reasoning left in the linked ADRs, enumerables in structured forms.
  Form only: nothing bound before the sweep is absent after.
- Done-check: `superdev validate` passes; each of the three reads as
  I029 c1–4 conformant; the diff drops nothing callers rely on.
- Cases:
  - review: contract-007's doc comments no longer restate ADR
    reasoning; each rule links its ADR — covers I029 c2, c4.
  - review: each internal contract's normative sentences carry RFC 2119
    verbs, one requirement per sentence — covers I029 c1.
  - review: everything 007, 009 and 010 bound before the sweep is still
    bound after — covers I029 c3.

### Block 5: Sweep the public contracts

- [x] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: contracts 002, 003, 004, 005, 006 and 008 rewritten to
  ADR-029 form; CHANGELOG entry for the corpus sweep.
- Done-check: `superdev validate` passes; with block 4 done, all nine
  active contracts conform.
- Cases:
  - review: each public contract's normative sentences carry RFC 2119
    verbs and its enumerable surfaces are structured — covers I029 c1,
    c2, c4.
  - review: contract-002's promises are unchanged by the rewrite —
    covers I029 c3.
  - e2e: all nine active contracts conform to the standard — covers
    I029 c5.

### Block 6: Modal verbs for the CLI and MCP contracts

- [x] Done — ticked by integrate at merge.
- Depends-on: 5.
- Change: contracts 002 and 003 gain RFC 2119 modal verbs on every
  promise, per the strict remedy chosen for I033 at acceptance; meaning
  unchanged.
- Done-check: a keyword census finds modals in both; `superdev
  validate` passes; no promise is added or dropped.
- Cases:
  - review: 002's and 003's normative sentences carry modal verbs —
    covers I029 c1, c5.
  - review: both contracts' promises are unchanged by the rewrite —
    covers I029 c3.

<!-- sokf:links -->
[sokf:issue-028-contract-design-commits-before-the-go-ahead]: /knowledge/issues/done/issue-028-contract-design-commits-before-the-go-ahead.md
[sokf:issue-029-contract-design-writes-verbose-prose]: /knowledge/issues/done/issue-029-contract-design-writes-verbose-prose.md
