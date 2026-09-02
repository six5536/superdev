---
type: FeaturePlan
id: plan-025-feature-a-contracts-behaviour-is-written-as-ears
title: A contract's behaviour is written as EARS — feature plan
description: Slices delivering I037 — the three item declarations in the validator, the sweep of nine contracts to keyed EARS promises, the contract schema in its final form with twelve examples, the tracker schemas' keyed criteria with the c<n> sweep of fifty issues, and the records.
lifecycle: open
links:
  - rel: implements
    to: issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears
    note: The framed feature whose twenty-one criteria these slices deliver.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Carries the three item declarations MUST PENDING (I037); slices 1 and 2 close them, so they run first.
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: The item form every sweep writes and every example shows.
  - rel: references
    to: adr-047-a-section-rule-declares-item-keys-and-item-bounds
    note: The three declarations slices 1 and 2 build.
---

# Feature plan: A contract's behaviour is written as EARS

Request: [issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears][sokf:issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears]

The validator first: the two slices that close
[contract-010][sokf:contract-010-interface-document-schemas]'s
PENDING promises with the three declarations of
[ADR-047][sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds],
in the order their vocabulary is needed. Then the
nine contracts are swept to the item form of
[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]
under the schema as it stands — the old Behaviour and Stability rules
ask for a paragraph and a keyword, which a section of keyed bullets
with one descriptive sentence still satisfies — so the tree validates
after every slice, and the schema switches to its final form in one
slice once every contract already conforms. The tracker's criteria
cannot be swept ahead of their schema — the old item-pattern binds the
tag to the item's start — so that slice changes the three schemas and
runs the `c<n>` script in one commit. The records close.

Owned copies and their sources, as
[plan-024][sokf:plan-024-feature-a-contract-includes-its-definition]
found them: `.agents/sokf/grammar.yaml` from
`crates/lib/superdev-core/src/validate/schema/grammar.yaml`, embedded
in the binary; every `knowledge/schemas/*.md` from
`pack/knowledge/schemas/`. A slice edits the source, runs
`scripts/superdev sync`, and commits the moved lock hashes; `superdev
status` reporting no drift is part of its done-check. The dev shim
gates the edit hook by path, so a slice that edits Rust pays no
rebuild per edit.

## Slices

### Slice 1: The validator reads item-key

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `validate::schema::document` reads `SectionRule.item_key`:
  compiles it, requires exactly one capture group, and on a rule whose
  `content` is a list kind checks every top-level item of that kind
  against it — an item with no match is a finding naming the section
  and the item's first line; the capture is the key, and a key repeated
  across the document's items under rules declaring `item-key` is a
  finding naming the key and both items. An `item-key` with no capture
  group, or on a rule whose `content` is not a list kind, is a finding
  on the schema and binds nothing. Findings are errors. The two PENDING
  sentences about `item-key` in contract-010's Behaviour lose their
  marker.
- Done-check: a probe schema declaring `item-key` on a bullet list
  reports a keyless item, a malformed key and a duplicate key, each
  naming what the criteria say; a schema with a capture-less pattern
  reports on itself; `superdev validate` passes the live tree;
  `superdev status` reports no drift.
- Cases:
  - unit: an item whose text does not match `item-key` is reported
    naming the section and the item's first line — covers AC_c2, AC_c3.
  - unit: a key captured twice in one document, across two sections
    declaring `item-key`, is reported naming the key and both items —
    covers AC_c4.
  - unit: the same key in two documents is not a finding — covers AC_c4.
  - unit: an `item-key` with no capture group, and one on a `prose`
    rule, are each a finding on the schema file and bind nothing —
    covers AC_c14.
  - unit: a matching, unique key on every item passes — covers AC_c1.
  - unit: an `item-key` finding is fatal, and `--fix` leaves the item
    unchanged — covers AC_c8, AC_c10.
  - integration: contract-010's Behaviour carries no PENDING for
    `item-key` and the live tree validates — covers AC_c14.

### Slice 2: The validator reads item-only-pattern and item-prohibited-pattern

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `item-only-pattern` — compiled; every body line outside a
  top-level item of the rule's list kind (prose, a table row, a
  heading, an item of the other list kind; fenced lines skipped) that
  matches is a finding naming the section and the line; on a rule with
  no list `content` every body line is outside. `item-prohibited-pattern`
  — compiled; every top-level item that matches is a finding naming
  the item's first line and the matched text; on a rule whose `content`
  is not a list kind it is a finding on the schema. Both share the item
  reading `item-pattern` uses. Findings are errors. The remaining
  PENDING sentences in contract-010 lose their marker.
- Done-check: a probe schema declaring both on a bullet list reports a
  modal verb in a paragraph, in a table row and in a numbered step, and
  reports a `MUST` and a two-verb item, each naming what criteria 5 to
  7 say; contract-010 carries no PENDING; the live tree validates with
  no drift.
- Cases:
  - unit: a match on a paragraph line, a table row and a numbered item
    under a bullet-list rule is each reported naming the section and
    the line — covers AC_c5.
  - unit: a match inside a bullet item is not an `item-only-pattern`
    finding — covers AC_c5.
  - unit: an item matching `item-prohibited-pattern` is reported naming
    the item and the matched text — covers AC_c6.
  - unit: the `(?s)` two-verb pattern of ADR-047 reports an item with
    two verbs and passes an item with `SHALL NOT` — covers AC_c7.
  - unit: an item with a tag and no verb fails the ADR-047
    `item-pattern` — covers AC_c7.
  - unit: an item carrying `PENDING` beside its verb passes all four
    patterns — covers AC_c9.
  - unit: `item-prohibited-pattern` on a `prose` rule is a finding on
    the schema — covers AC_c14.
  - integration: contract-010 carries no PENDING and the live tree
    validates — covers AC_c14.

### Slice 3: The internal contracts are swept

- [x] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: contract-007, contract-009 and contract-010 — Behaviour and
  Stability rewritten to keyed EARS bullets per ADR-046: one promise
  per bullet, `P_` key, tag, one verb from SHALL/SHOULD/MAY, the
  interface element as subject; key flows as numbered lists; prose
  carries no modal verb; no promise dropped. Each section keeps one
  descriptive sentence so the schema as it stands still finds a
  paragraph.
- Done-check: every former sentence's requirements are present as one
  item each (the reviewer counts the modal verbs before and after);
  `grep -c 'MUST' ` on the three files is zero outside fenced blocks;
  the ADR-047 patterns, run over the three files by a probe schema in
  a scratch tree, report nothing; the live tree validates.
- Cases:
  - integration: a scratch schema carrying the ADR-047 patterns on
    Behaviour and Stability passes the three swept contracts — covers
    AC_c13.
  - manual: the reviewer confirms the promise count per contract
    equals the modal-verb count before the sweep — covers AC_c13.
  - Note, at merge: contract-007 14 verbs before, 14 promises after;
    contract-009 8 and 8; contract-010 16 and 16. The scratch schema
    carrying the four ADR-047 declarations reported nothing for the
    three, and reported a keyless item, a `MUST` in prose and a
    tagless item when each was injected.

### Slice 4: The cli, api and config contracts are swept

- [x] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: contract-002, contract-003 and contract-004, as slice 3 —
  the sixty verbs of contract-002 included; the exit-code table stays
  as a table beside its promises.
- Done-check: as slice 3, for the three files.
- Cases:
  - integration: the scratch schema of slice 3 passes the three swept
    contracts — covers AC_c13.
  - manual: the reviewer confirms the promise count equals the verb
    count before the sweep — covers AC_c13.
  - Note, at merge: contract-002 60 verbs before, 58 promises after —
    the closed-stdout-pipe and the `status`-writes-nothing sentences
    each stood twice and merge into one keyed item, cited from the
    prose where the second stood; contract-003 17 and 17; contract-004
    17 and 13 — the API-key pair stood in Sources and in Secrets, and
    the load-failure pair in Validation and in Stability, each merged
    into one item cited from the other place. Every other verb is one
    item. The four config sources became a numbered list and the
    defaults a table, since a bullet under Behaviour is a promise. The
    scratch schema of slice 3 reported nothing for the three, and
    reported a keyless item, a tagless item and a `MUST` in prose when
    each was injected.

### Slice 5: The format contracts are swept

- [x] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: contract-005, contract-006 and contract-008, as slice 3.
- Done-check: as slice 3, for the three files.
- Cases:
  - integration: the scratch schema of slice 3 passes the three swept
    contracts — covers AC_c13.
  - manual: the reviewer confirms the promise count equals the verb
    count before the sweep — covers AC_c13.
  - Note, at merge: contract-005 15 verbs before, 15 promises after;
    contract-006 16 and 16; contract-008 19 and 17 — the write-once
    sentence ("the engine MUST NOT hash, sync or revisit a seeded
    file") stood in Files, in Compatibility and in Stability and is
    one item, `P_seeded-file-write-once`, cited from the other two
    places. Every other verb is one item. The shipped set of
    contract-008 became a numbered list. The scratch schema of slice 3
    reported nothing for the three, and reported a keyless item, a
    tagless item and a `MUST` in prose when each was injected.

### Slice 6: The contract schema takes its final form

- [x] Done — ticked by integrate at merge.
- Depends-on: 3, 4, 5.
- Change: `pack/knowledge/schemas/contract.md`, synced — Behaviour and
  Stability become `content: bullet-list` with the four ADR-047
  declarations (the `P_` key, the tag-and-verb `item-pattern`, the
  modal-verb `item-only-pattern`, the retired-verb and two-verb
  `item-prohibited-pattern`), no `content-pattern`; the descriptions
  and the contract-style prose say the form, the retired verbs, the
  subject, the numbered-list rule, `PENDING`, no `TBD`, and the
  citation — bare key where the contract is the subject, id then key
  elsewhere; the twelve examples carry Behaviour and Stability in the
  form. The `normative_shapes` tests that read the schema follow it.
- Done-check: `superdev validate` passes the live tree, every example
  included; a contract with a numbered item under Behaviour and no
  keyed bullet fails; no file under `.claude/skills/` or
  `pack/knowledge/skills/` changed; `superdev status` no drift.
- Cases:
  - integration: the live tree validates with the final schema —
    covers AC_c1, AC_c11, AC_c12, AC_c13.
  - integration: a scratch contract with a keyless bullet, a `MUST`
    item, a two-verb item, a `SHALL` in a paragraph and a `TBD` item
    fails validate naming each — covers AC_c2, AC_c5, AC_c6, AC_c7, AC_c8.
  - integration: a scratch contract whose Behaviour is a numbered flow
    beside keyed bullets passes — covers AC_c1.
  - unit: the schema's Behaviour and Stability rules declare the four
    patterns and say the citation form — covers AC_c1, AC_c11.
  - unit: no skill file differs from the merge target — covers AC_c15.
  - Note, at merge: the final schema reported nothing on the nine
    swept contracts; its one finding was on the schema's own
    `deployment` example, a key opening with a digit, renamed before
    the commit. The two scratch cases are tests in `normative_shapes`
    against the live schema set, and a scratch tree under the job's
    directory carrying the real schema set confirmed them through the
    CLI, with a Behaviour of numbered items and no bullet failing as
    the done-check asks. The skill check is not a test — a test cannot
    read git history — and integrate ran `git diff --stat main --
    .claude/skills pack/knowledge/skills`, which was empty.

### Slice 7: The tracker's criteria carry keys

- [ ] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `pack/knowledge/schemas/feature-request.md`,
  `bug-report.md`, `chore.md` and `feature-plan.md`, synced — the
  Acceptance criteria, Steps to reproduce and Definition of done rules
  declare `item-key` with `AC_`, `RS_` and `DD_`, their `item-pattern`
  admits the key before the tag or `TBD` (criteria) or requires the
  key alone (steps, done items); descriptions state the citation
  form; the plan schema's case rule cites keys ("covers AC_c1,
  AC_stale-include"); examples updated. A one-off script under the
  job's scratch directory prefixes every item of those lists in the
  fifty issues with `` `<PREFIX>_c<n>` ``, `n` the item's number, and
  every open plan's "covers n" becomes "covers AC_cn"; I037 and this
  plan included.
- Done-check: `superdev validate` passes the live tree; every issue's
  cited lists carry keys and no settled issue changed outside the key
  prefix (`git diff --stat` shows one line per item); the four
  examples pass their own check; `superdev status` no drift.
- Cases:
  - integration: the live tree validates with the four schemas —
    covers AC_c17, AC_c20, AC_c21.
  - integration: a scratch feature-request with a keyless criterion,
    a `P_` key, and a duplicate `AC_` key fails naming each — covers
    AC_c17.
  - integration: a scratch feature-request whose criterion reads
    `` `AC_x` TBD — … `` passes, and a bug-report step with a key and
    no tag passes — covers AC_c18.
  - unit: the plan schema's case description cites keys — covers AC_c19.
  - unit: every settled issue's diff touches only item prefixes —
    covers AC_c20.

### Slice 8: The records close

- [ ] Done — ticked by integrate at merge.
- Depends-on: 6, 7.
- Change: the glossary defines the promise form and the key (both
  prefixes tables, the `c<n>` slugs, the citation); the changelog's
  Unreleased carries the form, the three declarations and the sweep;
  `constraints-non-goals` notes the key beside the behaviour-testing
  non-goal; I037's Comments record the sweep counts.
- Done-check: `superdev validate` passes; the glossary's EARS entry no
  longer says "do not yet".
- Cases:
  - unit: the glossary defines the key, its prefixes and the citation
    — covers AC_c16.
  - unit: the changelog's Unreleased names the form — covers AC_c16.

## Deferred decisions

- Expected behaviour is `content: prose` in the bug-report schema and
  a paragraph in 21 of the 24 bug reports on file; ADR-046 keys it
  with `EX_`, and a keyed list would reword settled records, which
  ADR-046 also forbids. Slice 7 keys repro steps (`RS_`) and leaves
  Expected behaviour prose. Does `EX_` stand for bug reports written
  from now on — the schema turning Expected behaviour into a keyed
  list for new reports, which needs a lifecycle variant or a sweep —
  or is `EX_` withdrawn from ADR-046? Blocks nothing; returns to
  contract-design.
- Criterion 15 says no skill changes for the form. The frame skill's
  "Write acceptance criteria" step says each criterion opens with its
  pattern tag, which the key now precedes; the schema governs, so the
  step is stale but not wrong. Amend the skill text in the pack and
  the synced copy, or leave it? Blocks nothing; returns to frame.

<!-- sokf:links -->
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]: /knowledge/adrs/active/adr-047-a-section-rule-declares-item-keys-and-item-bounds.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears]: /knowledge/issues/open/issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears.md
[sokf:plan-024-feature-a-contract-includes-its-definition]: /knowledge/plans/done/plan-024-feature-a-contract-includes-its-definition.md
