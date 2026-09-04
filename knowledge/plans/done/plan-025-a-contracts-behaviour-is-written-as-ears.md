---
type: Plan
id: plan-025-a-contracts-behaviour-is-written-as-ears
title: A contract's behaviour is written as EARS
description: Blocks delivering I037 — the three item declarations in the validator, the sweep of nine contracts to keyed EARS promises, the contract schema in its final form with twelve examples, the tracker schemas' keyed criteria with the c<n> sweep of fifty issues, and the records.
lifecycle: done
links:
  - rel: implements
    to: issue-037-a-contracts-behaviour-is-not-written-as-ears
    note: The framed feature whose twenty-one criteria these blocks deliver.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Carries the three item declarations MUST PENDING (I037); blocks 1 and 2 close them, so they run first.
  - rel: references
    to: adr-046-a-promise-and-a-criterion-are-keyed-ears-items
    note: The item form every sweep writes and every example shows.
  - rel: references
    to: adr-047-a-section-rule-declares-item-keys-and-item-bounds
    note: The three declarations blocks 1 and 2 build.
---

# Plan: A contract's behaviour is written as EARS

Request: [issue-037-a-contracts-behaviour-is-not-written-as-ears][sokf:issue-037-a-contracts-behaviour-is-not-written-as-ears]

## Goal

Every promise a contract makes is a keyed EARS item the validator holds
to its form, and every criterion a plan case cites carries a key of its
own, so a promise and the criterion it comes from read the same and both
have an identity a test can name.

The validator first: the two blocks that close
[contract-010][sokf:contract-010-interface-document-schemas]'s PENDING
promises with the three declarations of
[ADR-047][sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds],
in the order their vocabulary is needed. Then the nine contracts are
swept to the item form of
[ADR-046][sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]
under the schema as it stands — the old Behaviour and Stability rules
ask for a paragraph and a keyword, which a section of keyed bullets with
one descriptive sentence still satisfies — so the tree validates after
every block, and the schema switches to its final form in one block once
every contract already conforms. The tracker's criteria cannot be swept
ahead of their schema, because the old item-pattern binds the tag to the
item's start, so that block changes the three schemas and runs the
`c<n>` script in one commit. The records close last.

Owned copies and their sources, as
[plan-024][sokf:plan-024-a-contract-includes-its-definition] found them:
`.agents/sokf/grammar.yaml` from
`crates/lib/superdev-core/src/validate/schema/grammar.yaml`, embedded in
the binary; every `knowledge/schemas/*.md` from
`pack/knowledge/schemas/`. A block edits the source, runs
`scripts/superdev sync`, and commits the moved lock hashes; `superdev
status` reporting no drift is part of its done-check. The dev shim gates
the edit hook by path, so a block that edits Rust pays no rebuild per
edit.

## Contract changes

- contract-010-interface-document-schemas: the PENDING markers leave the
  three item declarations — `item-key`, `item-only-pattern` and
  `item-prohibited-pattern` — as blocks 1 and 2 implement their checks,
  and the misdeclaration findings gain their promises; its own Behaviour
  and Stability are swept to keyed EARS bullets, 15 modal verbs before
  and 16 promises after, the one sentence reporting a misplaced
  `item-key` or `item-prohibited-pattern` "the same way" becoming
  `P_misdeclared-item-key` and `P_misdeclared-item-prohibited`.
- contract-007-interface-pack-resolution: Behaviour and Stability swept
  to keyed EARS bullets — 14 modal verbs before, 14 promises after.
- contract-009-interface-run-state: swept — 8 verbs before, 8 promises
  after.
- contract-002-cli-superdev: swept — 60 verbs before, 58 promises after;
  the closed-stdout-pipe and the `status`-writes-nothing sentences each
  stood twice and merge into one keyed item, cited from the prose where
  the second stood; the exit-code table stays a table beside its
  promises.
- contract-003-api-sokf: swept — 17 verbs before, 17 promises after.
- contract-004-config-superdev: swept — 17 verbs before, 13 promises
  after; the API-key pair stood in Sources and in Secrets, and the
  load-failure pair in Validation and in Stability, each merged into one
  item cited from the other place; the four config sources became a
  numbered list and the defaults a table, since a bullet under Behaviour
  is a promise.
- contract-005-format-pack: swept — 15 verbs before, 15 promises after.
- contract-006-format-lock: swept — 16 verbs before, 16 promises after.
- contract-008-format-template: swept — 19 verbs before, 17 promises
  after; the write-once sentence ("the engine MUST NOT hash, sync or
  revisit a seeded file") stood in Files, in Compatibility and in
  Stability and is one item, `P_seeded-file-write-once`, cited from the
  other two places; the shipped set became a numbered list.

## Work blocks

### Block 1: The validator reads item-key

- [x] Done — ticked at merge.
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
    naming the section and the item's first line — checks that a
    keyless item and a malformed key are each an error naming the item.
  - unit: a key captured twice in one document, across two sections
    declaring `item-key`, is reported naming the key and both items —
    checks that a duplicate key is an error naming the key and both
    items.
  - unit: the same key in two documents is not a finding — checks that
    uniqueness is scoped to the document.
  - unit: an `item-key` with no capture group, and one on a `prose`
    rule, are each a finding on the schema file and bind nothing —
    checks that the grammar accepts only a well-formed declaration.
  - unit: a matching, unique key on every item passes — checks that a
    conforming keyed list is accepted.
  - unit: an `item-key` finding is fatal, and `--fix` leaves the item
    unchanged — checks that the finding fails `superdev validate` and
    that `--fix` neither rewrites a statement nor supplies a key.
  - integration: contract-010's Behaviour carries no PENDING for
    `item-key` and the live tree validates — checks that contract-010
    carries the declaration the change adds.

### Block 2: The validator reads item-only-pattern and item-prohibited-pattern

- [x] Done — ticked at merge.
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
    the line — checks that a modal verb outside an item is an error.
  - unit: a match inside a bullet item is not an `item-only-pattern`
    finding — checks that a verb inside an item is admitted.
  - unit: an item matching `item-prohibited-pattern` is reported naming
    the item and the matched text — checks that a retired verb is an
    error naming the item, the verb and the rule.
  - unit: the `(?s)` two-verb pattern of ADR-047 reports an item with
    two verbs and passes an item with `SHALL NOT` — checks that an item
    carries one requirement.
  - unit: an item with a tag and no verb fails the ADR-047
    `item-pattern` — checks that a tagged item states a requirement.
  - unit: an item carrying `PENDING` beside its verb passes all four
    patterns — checks that ADR-044's marker survives the checks.
  - unit: `item-prohibited-pattern` on a `prose` rule is a finding on
    the schema — checks that the grammar accepts only a well-formed
    declaration.
  - integration: contract-010 carries no PENDING and the live tree
    validates — checks that contract-010 carries the declarations the
    change adds.

### Block 3: The internal contracts are swept

- [x] Done — ticked at merge.
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
    Behaviour and Stability passes the three swept contracts — checks
    that every active contract's Behaviour and Stability conform.
  - manual: the reviewer confirms the promise count per contract
    equals the modal-verb count before the sweep — checks that no
    promise was dropped.
  - Note, at merge: the counts and the merged sentences are in Contract
    changes above; the count of 16 verbs for contract-010 first
    recorded here was wrong, corrected against `cb78f13`, which carries
    12 `MUST`, 2 `MUST NOT` and 1 `MAY` outside fenced blocks
    (code-review-009). The scratch schema carrying the four ADR-047
    declarations reported nothing for the three contracts, and reported
    a keyless item, a `MUST` in prose and a tagless item when each was
    injected.

### Block 4: The cli, api and config contracts are swept

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: contract-002, contract-003 and contract-004, as block 3 —
  the sixty verbs of contract-002 included; the exit-code table stays
  as a table beside its promises.
- Done-check: as block 3, for the three files.
- Cases:
  - integration: the scratch schema of block 3 passes the three swept
    contracts — checks that every active contract's Behaviour and
    Stability conform.
  - manual: the reviewer confirms the promise count equals the verb
    count before the sweep — checks that no promise was dropped.
  - Note, at merge: the counts and the merged sentences are in Contract
    changes above; every other verb is one item. The scratch schema of
    block 3 reported nothing for the three, and reported a keyless
    item, a tagless item and a `MUST` in prose when each was injected.

### Block 5: The format contracts are swept

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: contract-005, contract-006 and contract-008, as block 3.
- Done-check: as block 3, for the three files.
- Cases:
  - integration: the scratch schema of block 3 passes the three swept
    contracts — checks that every active contract's Behaviour and
    Stability conform.
  - manual: the reviewer confirms the promise count equals the verb
    count before the sweep — checks that no promise was dropped.
  - Note, at merge: the counts and the merged write-once sentence are
    in Contract changes above; every other verb is one item. The
    scratch schema of block 3 reported nothing for the three, and
    reported a keyless item, a tagless item and a `MUST` in prose when
    each was injected.

### Block 6: The contract schema takes its final form

- [x] Done — ticked at merge.
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
    checks that the schema declares both sections as keyed, tagged
    bullet lists, states the citation form, and that the nine
    contracts conform.
  - integration: a scratch contract with a keyless bullet, a `MUST`
    item, a two-verb item, a `SHALL` in a paragraph and a `TBD` item
    fails validate naming each — checks the five departures the
    validator reports.
  - integration: a scratch contract whose Behaviour is a numbered flow
    beside keyed bullets passes — checks that prose, tables and
    numbered flows remain permitted beside the list.
  - unit: the schema's Behaviour and Stability rules declare the four
    patterns and say the citation form — checks that the schema alone
    carries the form.
  - unit: no skill file differs from the merge target — checks that no
    skill changes for the form.
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

### Block 7: The tracker's criteria carry keys

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: `pack/knowledge/schemas/feature-request.md`,
  `bug-report.md`, `chore.md` and `feature-plan.md`, synced — the
  Acceptance criteria, Steps to reproduce and Definition of done rules
  declare `item-key` with `AC_`, `RS_` and `DD_`, their `item-pattern`
  admits the key before the tag or `TBD` (criteria) or requires the
  key alone (steps, done items); descriptions state the citation
  form; the plan schema's case rule cites keys; examples updated. A
  one-off script under the job's scratch directory prefixes every item
  of those lists in the fifty issues with `` `<PREFIX>_c<n>` ``, `n`
  the item's number, and every open plan's "covers n" becomes "covers
  AC_cn"; I037 and this plan included.
- Done-check: `superdev validate` passes the live tree; every issue's
  cited lists carry keys and no settled issue changed outside the key
  prefix (`git diff --stat` shows one line per item); the four
  examples pass their own check; `superdev status` no drift.
- Cases:
  - integration: the live tree validates with the four schemas —
    checks that the tracker schemas declare a key on every cited item
    and that their examples pass their own check.
  - integration: a scratch feature-request with a keyless criterion,
    a `P_` key, and a duplicate `AC_` key fails naming each — checks
    that a missing, malformed or duplicate criterion key is an error.
  - integration: a scratch feature-request whose criterion reads
    `` `AC_x` TBD — … `` passes, and a bug-report step with a key and
    no tag passes — checks that a key precedes the tag or `TBD` and
    that a repro step's key stands alone.
  - unit: the plan schema's case description cites keys — checks that
    a case names the keys of the criteria it covers.
  - unit: every settled issue's diff touches only item prefixes —
    checks that every issue on file carries a `c<n>` key and nothing
    else changed.
  - Note, at merge: the sweep touched the fifty issues — 141 criteria
    `AC_c<n>`, 72 repro steps `RS_c<n>`, 22 done items `DD_c<n>` — and
    rewrote plan-025's 33 covers clauses; plan-001 has no cases. The
    diff check is not a test — a test cannot read git history — and
    integrate ran it as a shell loop over `git diff -U0` per issue: 235
    insertions against 235 deletions, each changed line equal to its
    original once the key is removed. The tracker-side proof is the
    `normative_shapes` loop over every issue's cited lists. The
    criterion `item-pattern` test of ADR-031 now expects the keyed
    pattern, deliberately.

### Block 8: The records close

- [x] Done — ticked at merge.
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
    — checks that the glossary carries the form of a promise and its
    key.
  - unit: the changelog's Unreleased names the form — checks that the
    change is recorded under Unreleased.
  - Note, at merge: no test reads the glossary or the changelog, so
    integrate checked both cases by grep — the glossary's EARS entry
    carries no "do not yet", a Promise key entry names the five
    prefixes, the slug, the `c<n>` slug and the citation form, and the
    changelog's Unreleased carries the three declarations under Added
    and the promise form, the contract sweep and the tracker sweep
    under Changed. The whole-feature review is
    [code-review-009][sokf:code-review-009-a-contracts-behaviour-is-written-as-ears]:
    three major findings and seven minor, recorded and not fixed on
    the branch.

## Deferred decisions

- Expected behaviour is `content: prose` in the bug-report schema and
  a paragraph in 21 of the 24 bug reports on file; ADR-046 keys it
  with `EX_`, and a keyed list would reword settled records, which
  ADR-046 also forbids. Block 7 keys repro steps (`RS_`) and leaves
  Expected behaviour prose. Does `EX_` stand for bug reports written
  from now on — the schema turning Expected behaviour into a keyed
  list for new reports, which needs a lifecycle variant or a sweep —
  or is `EX_` withdrawn from ADR-046? Blocks nothing; returns to
  contract-design.
  - Answer (2026-09-02, the owner): an issue's lifecycle distinguishes
    framed from unframed; once framed, its behaviour and acceptance
    criteria are keyed items, written as EARS where the item is a
    requirement. `EX_` lands there, on the framed variant, and settled
    records stay as they are. Recorded on
    [I030][sokf:issue-030-filing-an-issue-requires-framing-it],
    which owns the lifecycle; the tracker-key criterion narrows to the
    lists this feature keyed.
- The criterion that no skill changes for the form stands against the
  frame skill's "Write acceptance criteria" step, which says each
  criterion opens with its pattern tag, the key now preceding it; the
  schema governs, so the step is stale but not wrong. Amend the skill
  text in the pack and the synced copy, or leave it? Blocks nothing;
  returns to frame.
  - Answer (2026-09-02, the owner): amend the sentence.

<!-- sokf:links -->
[sokf:adr-046-a-promise-and-a-criterion-are-keyed-ears-items]: /knowledge/adrs/active/adr-046-a-promise-and-a-criterion-are-keyed-ears-items.md
[sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]: /knowledge/adrs/active/adr-047-a-section-rule-declares-item-keys-and-item-bounds.md
[sokf:code-review-009-a-contracts-behaviour-is-written-as-ears]: /knowledge/reports/code-review-009-a-contracts-behaviour-is-written-as-ears.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-030-filing-an-issue-requires-framing-it]: /knowledge/issues/done/issue-030-filing-an-issue-requires-framing-it.md
[sokf:issue-037-a-contracts-behaviour-is-not-written-as-ears]: /knowledge/issues/done/issue-037-a-contracts-behaviour-is-not-written-as-ears.md
[sokf:plan-024-a-contract-includes-its-definition]: /knowledge/plans/done/plan-024-a-contract-includes-its-definition.md
