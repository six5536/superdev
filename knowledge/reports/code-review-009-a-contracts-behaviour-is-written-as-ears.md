---
type: CodeReview
id: code-review-009-a-contracts-behaviour-is-written-as-ears
title: Code review — a contract's behaviour is written as EARS
description: Review of plan-025's eight slices on `feature/a-contracts-behaviour-is-written-as-ears`; three major findings — a contract with no promise passes the schema, an item swallows the heading after it, and the tree-wide PENDING guard was dropped on a false premise — seven minor ones, and five a second reviewer added; twelve resolved on the branch, two not applied, one deferred.
links:
  - rel: references
    to: plan-025-feature-a-contracts-behaviour-is-written-as-ears
    note: The plan whose eight slices this review reads.
  - rel: references
    to: issue-037-a-contracts-behaviour-is-not-written-as-ears
    note: The framed feature; the findings are checked against its criteria.
---

# Code review: plan-025 on `feature/a-contracts-behaviour-is-written-as-ears` (main..5dc4655)

## Verdict

The eight slices of
[plan-025][sokf:plan-025-feature-a-contracts-behaviour-is-written-as-ears]
deliver
[I037][sokf:issue-037-a-contracts-behaviour-is-not-written-as-ears]:
the three declarations bind, the nine contracts and fifty issues are
swept, and the live tree validates. Three findings are major — a
Behaviour with no promise passes the schema, a heading or table row
that follows a bullet is read as part of the bullet, and the test that
kept every active contract free of `PENDING` was dropped on a premise
that is false at HEAD. Seven are minor: a latent double capture, a
finding that misnames a malformed key, three documents out of step with
the item's scope, two duplications, and a test parser that diverges
from the validator's. A second reviewer's list adds five, 11 to 15: the
`RS_`/`DD_` tag left unenforced, a nested verb nobody pinned, three
two-outcome promises, a miscounted note, and an `EX_` key no schema
declares. Each finding carries its outcome: twelve are resolved on the
branch, two are not applied, and one is deferred.

## Findings

### 1. A contract with no promise passes the schema — crates/lib/superdev-core/src/validate/schema/document.rs:1656

- Severity: major
- Category: correctness
- Problem: Behaviour and Stability moved from `content-pattern` to
  `content: bullet-list`, and the kind check `body_has("bullet-list")`
  accepts any `- ` line — a thematic break `- - -`, a bullet nested
  under a numbered step — while `items_in` excludes both, so `item-key`
  and `item-pattern` bind nothing when no top-level bullet exists.
- Failure scenario: under the live contract schema a Behaviour reading
  "The probe does things." followed by a numbered flow with one nested
  note, or by `- - -`, yields no finding; `items_in` returns no item and
  no key or pattern check runs. On `main` the same body failed the
  modal-verb `content-pattern`. Verified through `findings_of("Contract",
  …)`.
- Suggested fix: the kind check requires at least one top-level item as
  `items_in` reads them, not any bullet-looking line.
- Outcome: Resolved in dbfcccb. `body_has` satisfies a list kind only by a
  top-level item as `items_in` reads one, so a Behaviour of nested bullets
  or a `- - -` break reports "carries no bullet" — `listless.md` in the
  item-bounds fixture and
  `a_section_with_no_top_level_item_fails_its_list_kind`.

### 2. An item swallows the heading or table row after it — crates/lib/superdev-core/src/validate/schema/document.rs:1406

- Severity: major
- Category: correctness
- Problem: the lazy-continuation branch of `items_in` (`open &&
  flowing`) joins an ATX heading or a table row that directly follows a
  bullet into the item's text and `lines`, so the item bounds misreport
  the line and `item-pattern` is satisfied by the heading's verb.
  CommonMark ends the paragraph at a heading or a table.
- Failure scenario: a Behaviour whose promise is followed, with no blank
  line, by `### What it MUST NOT do` reports the `MUST` under
  `item-prohibited-pattern` as the item's instead of as a line outside
  an item; a verb-less bullet followed by `### When it SHALL answer`
  produces no finding; a table row after a bullet reports a matched text
  carrying the row. The joining predates the feature; the diff's bounds
  and `Item.lines` make it observable as wrong findings.
- Suggested fix: `items_in` ends an item at a heading or a table row as
  it ends one at a blank line.
- Outcome: Resolved in dbfcccb. `ends_paragraph` ends an item's lazy
  continuation at a heading, a table row or an HTML comment, as CommonMark
  ends a paragraph; the fixture's `faulty.md` carries a table row and a
  heading right under a bullet, each reported outside a top-level item —
  `a_heading_or_table_row_under_a_bullet_ends_the_item`.

### 3. The tree-wide PENDING guard was dropped on a false premise — crates/lib/superdev-core/tests/normative_shapes.rs:1027

- Severity: major
- Category: test-coverage
- Problem: the `!text.contains("PENDING")` assertion over the active
  contracts was removed at `880d77a`, when contract-010 carried I037's
  markers, and the doc comment still says "contract-010 carries I037's";
  slices 1 and 2 removed the markers without restoring the assertion,
  and `contract_010_no_longer_defers_the_item_declarations` asserts the
  opposite of the comment. No automated check now says a settled
  feature's contracts promise nothing unbuilt.
- Failure scenario: a slice integrates a promise carrying `PENDING`, the
  feature closes through integrate's last-slice edge, and `cargo test`
  and `superdev validate` both pass — `a_pending_item_passes_every_pattern`
  shows all four patterns admit the marker — leaving the accept skill's
  manual gate as the only one.
- Suggested fix: restore the assertion, or correct the comment and add a
  settled-feature gate a test can run.
- Outcome: Not applied: ADR-044 permits a PENDING promise while a feature
  runs; the accept gate judges the settled state.

### 4. Two keyed rules over one section capture every item twice — crates/lib/superdev-core/src/validate/schema/document.rs:1107

- Severity: minor
- Category: correctness
- Problem: keys are collected from every matched heading-and-rule pair,
  and a level-2 body spans its level-3 subsections, so a schema declaring
  `item-key` on a level-2 rule and on a level-3 rule beneath it captures
  each item under both and reports every one as repeating itself.
- Failure scenario: a schema with `Slices` at level 2 and `Slice n: …`
  at level 3, both `bullet-list` with the same `item-key`, over a plan
  carrying one keyed item, yields a fatal "repeats key" finding naming
  the same item twice, though no key repeats. Verified via `check_one`.
  No live schema nests two keyed rules; the feature-plan shape is the
  first candidate.
- Suggested fix: deduplicate captures by line index, or exclude a
  subsection's lines from the parent rule's item scan.
- Outcome: Resolved in dbfcccb. A captured key carries the document line its
  item opens on and counts once across the rules that capture it —
  `an_item_captured_by_two_rules_counts_once`. A keyless item under two
  rules is still reported by each, naming its own section.

### 5. A malformed key is reported as no key — knowledge/schemas/feature-request.md:93

- Severity: minor
- Category: correctness
- Problem: `item-key`'s slug grammar
  (`AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*`) is stricter than `item-pattern`'s
  key span (`AC_[a-z0-9-]+`), so a visibly keyed item whose slug opens
  with a digit is reported as "carries no key", and a keyless item is
  reported twice — once by each declaration. The contract schema's `P_`
  pair and the pack mirrors carry the same two grammars. I037 `AC_c3`
  asks that the finding name the item and the key.
- Failure scenario: `` 1. `AC_1` [event] WHEN x THE SYSTEM SHALL y. ``
  (also `AC_a--b`, `AC_-x`) yields exactly "carries no key"; the author
  sees a key and is told there is none. `1. [event] WHEN x …` yields two
  findings for one fault, which
  `a_criterion_departing_from_the_key_form_fails_naming_each_departure`
  codifies as five findings for three faults, against the module's own
  "one fault, said once" rule.
- Suggested fix: `item-pattern` opens with a non-committal key span
  (`` ^`[^`]+` ``) so `item-key` alone defines the key, and the key
  finding names the malformed key.
- Outcome: Resolved in dbfcccb. `item-pattern` opens with `item-key`'s grammar
  in the contract and feature-request schemas, and each top-level item is
  checked by `item-key`, then `item-prohibited-pattern`, then
  `item-pattern`, an item one reports not checked by the next
  (contract-010 `P_item-one-finding`); the key finding reads "carries no
  key of the form `<pattern>`". Deliberately, the tracker test now expects
  three findings for three faults, not five, and the item-bounds golden
  loses `P_stops`' second finding.

### 6. A nested bullet's scope is stated three ways — crates/lib/superdev-core/src/validate/schema/document.rs:1252

- Severity: minor
- Category: correctness
- Problem: `inside` is built from top-level items only, as ADR-047
  decides, so a nested bullet under a promise is reported as "matches
  outside an item" while a nested bullet with no verb or key passes.
  `grammar.yaml`'s `item-only-pattern` doc and ADR-047's list of outside
  lines name neither nested items nor headings, and the contract schema
  (`contract.md:228`) says every bullet at any heading depth is one
  promise.
- Failure scenario: a promise with a nested `- WHEN silent, it SHALL
  NOT.` yields a finding naming a line that is visibly inside a list
  item; the same nested bullet with no verb passes as an unkeyed,
  untagged bullet the schema text calls a promise.
- Suggested fix: the finding says "outside a top-level item", and the
  grammar doc and the contract schema's prose say a nested bullet is not
  a promise and is bound as prose.
- Outcome: Resolved in dbfcccb. The finding says "matches outside a
  top-level item"; `grammar.yaml`'s `item-only-pattern` doc names prose, a
  table row, a heading, a nested item and an item of the other list kind;
  the contract schema says a nested bullet is not a promise and is bound
  as prose, in the rule and in the standard's prose.

### 7. An HTML comment line is bound by item-only-pattern — crates/lib/superdev-core/src/validate/schema/document.rs:1254

- Severity: minor
- Category: correctness
- Problem: `check_item_bounds` treats a `<!-- … -->` line as body
  content, while `is_paragraph` elsewhere in the module excludes comment
  lines from the prose kind.
- Failure scenario: a Behaviour carrying `<!-- TODO: the CLI MUST reject
  an empty path -->` between two promises fails validate naming the
  comment as a line outside an item (verified), while the same comment
  counts as nothing for the prose check.
- Suggested fix: skip comment lines in the bound as `is_paragraph` does,
  or state in the grammar doc that comments are bound.
- Outcome: Resolved in dbfcccb. `item-only-pattern` skips an HTML comment
  line, as `is_paragraph` does; the fixture's `sound.md` carries one
  between two promises and passes.

### 8. Each section's list is parsed three times — crates/lib/superdev-core/src/validate/schema/document.rs:1076

- Severity: minor
- Category: simplification
- Problem: `check_body_patterns` (1164), `check_item_keys_in` (1208) and
  `check_item_bounds` (1247) each copy the
  `rule.content.as_deref().filter(|k| LIST_KINDS.contains(k))` guard and
  call `items_in` on the same body; the `kind.is_some()` guard at 1266 is
  dead, since `items` is already empty when `kind` is `None`; and
  `Item.lines` is allocated for every caller though only the bound reads
  it.
- Failure scenario: a fourth item declaration means a fourth copy of the
  guard and a fourth walk, and a divergence in one copy makes the
  declarations disagree on what an item is.
- Suggested fix: compute `kind` and `items` once in `check_one`, which
  already hoists `fenced`, the headings and the positions, and pass
  `&[Item]` to the three checks; a `vec![false; body.len()]` mask in
  place of the `BTreeSet`, matching the `fenced` idiom.
- Outcome: Resolved in dbfcccb. `Items::read` in `check_one` reads a
  section's items once, and the three checks take the one `Items`; the
  dead `kind.is_some()` guard is gone. The `BTreeSet` stays: the mask is
  a style choice with no observable effect.

### 9. check_declarations' doc comment is false and the capture count is a fourth parse — crates/lib/superdev-core/src/validate/schema/document.rs:412

- Severity: minor
- Category: simplification
- Problem: the comment says validate reports all but the variant
  declarations through the grammar's own schema check "so it calls only
  `check_variants`", but `validate_repo` calls `check_variants` and
  `check_item_keys`, and `check_item_keys` parses and compiles every
  schema again for a capture count `check_declarations`' own
  `re::compile` at line 449 already has in hand.
- Failure scenario: a maintainer adding the next schema-level check reads
  the comment and omits it from `validate_repo`, so it runs only in the
  snapshot harness, or adds `check_declarations` to `validate_repo` and
  double-reports beside the grammar check.
- Suggested fix: correct the comment; host the capture-count check in
  `check_variants`' existing loop over the parsed `DocSchema`, or reuse
  the compiled regex in `check_declarations`.
- Outcome: Resolved in dbfcccb. The comment names `check_variants` and
  `check_item_keys`; `item_key_captures(file, rule)` is read by
  `check_declarations` from the schema it has already parsed and by
  `check_item_keys` for `validate`. The item-key golden's two schema
  findings swap order as a consequence.

### 10. The tracker sweep test carries its own item parser — crates/lib/superdev-core/tests/normative_shapes.rs:289

- Severity: minor
- Category: test-coverage
- Problem: `every_issue_on_file_carries_a_key_on_each_cited_item`
  hand-rolls a markdown item parser that diverges from the validator: it
  toggles fences on any ```` ``` ```` line, ignoring `~~~` and
  desynchronising on four-backtick nesting, and accepts only column-0
  `- ` and `N. ` markers, while `items_in` takes `* `, `+ `, `N) `, the
  shallowest indent as top level, and excludes thematic breaks.
- Failure scenario: a chore whose Definition of done uses `* ` or an
  indented list, or a bug report whose steps use `1) `, is skipped by the
  test while the validator binds it, and the aggregate `keyed >= 200`
  still holds; a `~~~` fence holding `1. run superdev validate` fails the
  test though the validator skips it. No corpus hit today.
- Suggested fix: extend `every_feature_request_on_file_conforms`, which
  already runs `check_documents` over the tracker for one type, to
  BugReport and Chore, so the live `item-key` rule is the sweep's proof,
  and drop the second parser.
- Outcome: Not applied: the test is in the integration crate and cannot
  call `items_in`; it checks the sweep's insertion shape only.

### 11. A tagged step or done item passes the `RS_` and `DD_` rules — pack/knowledge/schemas/bug-report.md:82

- Severity: minor
- Category: correctness
- Problem: the Steps to reproduce and Definition of done rules declare
  `item-key` alone, so a step or a done item carrying an EARS tag after
  its key passes, while I037 `AC_c18` says a step carries no tag.
- Failure scenario: `` 1. `RS_c1` [event] WHEN run, the probe runs. ``
  validates clean under the live bug-report schema, and the same for a
  `DD_` item under the chore schema.
- Suggested fix: an `item-prohibited-pattern` for a tag after the key on
  both rules, in the pack and the synced copy, and a test that a tagged
  step fails.
- Outcome: Resolved in dbfcccb. Both rules declare
  `` item-prohibited-pattern: '^`RS_[a-z0-9-]+` \[(ubiquitous|…)\]' `` (`DD_`
  for the chore), synced with the lock moved, and their descriptions say a
  tagged item is an error; `a_repro_step_carries_a_key_and_no_tag` proves
  a tagged step and a tagged done item each fail naming the tag, and
  `every_cited_list_declares_its_key_and_the_plan_cites_keys` asserts the
  prohibition. The changelog carries the new error.

### 12. No test pins a nested item's modal verb as outside an item — crates/lib/superdev-core/tests/fixtures/documents/item-bounds/faulty.md:16

- Severity: minor
- Category: test-coverage
- Problem: ADR-047 reads a nested item as outside every top-level item,
  and `check_item_bounds` reports its verb so, but no fixture or unit
  test carries a nested bullet with a modal verb.
- Failure scenario: a change that folds a nested item's lines into its
  parent passes every test while a nested `SHALL NOT` silently becomes
  the parent's second verb.
- Suggested fix: a nested sub-bullet carrying a modal verb in the
  item-bounds fixture, pinned by the golden.
- Outcome: Resolved in dbfcccb. `faulty.md` carries `- WHEN silent, it SHALL
  NOT stir` nested under `P_stays`, and the golden reports it outside a
  top-level item.

### 13. Three promises carry two outcomes or a MAY-only — knowledge/contracts/public/active/contract-002-cli-superdev.md:462

- Severity: minor
- Category: correctness
- Problem: contract-002 `P_hooks-resolve-project-dir` is an `[event]`
  with an else branch, `P_hook-run-fails-open` states two outcomes with
  two exit codes, and contract-008 `P_removal-needs-notice` reads "MAY …
  only", a permission carrying a requirement — each one item with two
  requirements, which ADR-046 forbids.
- Failure scenario: a reader citing `P_hook-run-fails-open` cannot say
  which exit code the key promises, and a test covering one outcome
  claims the whole key.
- Suggested fix: one promise per branch and per outcome, with the notice
  as a `SHALL`; no promise lost.
- Outcome: Resolved in dbfcccb. `P_hooks-resolve-project-dir` keeps the WHEN
  branch and `P_hooks-resolve-working-dir` `[conditional]` carries the
  else; `P_hook-run-fails-open` keeps the unreadable run state at exit
  `0` and `P_hook-run-unreadable-payload` carries the unreadable payload
  at exit `2`; `P_removal-needs-notice` is an `[event]` whose requirement
  is the release-notes notice. `superdev validate` passes.

### 14. The slice-3 note miscounts contract-010 and the new tests cite criteria by number — knowledge/plans/done/plan-025-feature-a-contracts-behaviour-is-written-as-ears.md:152

- Severity: nit
- Category: simplification
- Problem: plan-025's slice-3 note records contract-010 at 16 verbs
  before the sweep, and the doc comments of the tests this feature added
  read "Covers I037 criterion n" where the schema's citation form is the
  key, `AC_cn`.
- Failure scenario: a reader checking the sweep against `git show
  cb78f13:` finds 15 verbs and doubts the count of promises; a reader
  searching the tests for `AC_c17` finds nothing.
- Suggested fix: verify the count against `cb78f13` and record it; the
  new tests cite `AC_cn`, the tests that predate the feature keep their
  wording.
- Outcome: Resolved in dbfcccb for the tests — every test this feature added
  cites `I037 AC_c<n>` — and in the records commit beside this outcome
  for the note: at `cb78f13` the Behaviour and Stability of contract-010
  carry 15 modal verbs (12 `MUST`, 2 `MUST NOT`, 1 `MAY`), and the one
  sentence reporting a misplaced `item-key` or `item-prohibited-pattern`
  "the same way" became two promises.

### 15. `AC_c17` requires an `EX_` key that no schema declares — knowledge/issues/open/issue-037-a-contracts-behaviour-is-not-written-as-ears.md:1

- Severity: major
- Category: correctness
- Problem: I037 `AC_c17` names an `EX_` key on Expected behaviour, and
  ADR-046 keys it so, but the bug-report schema keeps Expected behaviour
  as `content: prose` with no key — plan-025 slice 7 keyed the repro
  steps and left this to a deferred decision.
- Failure scenario: a bug report written with an `EX_` list validates
  no differently from one without, so the criterion is not checkable.
- Suggested fix: settle the deferred decision — the schema keys Expected
  behaviour for new reports, or `EX_` is withdrawn from ADR-046 and the
  criterion.
- Outcome: Deferred: plan-025 Deferred decisions, contract-design.

## Not findings (checked and fine)

- `check_declarations`' `per_item` loop shape and the quadratic repeat
  scan over keys: style only, no observable effect at the tree's size.
- The frame skill's example criterion carries no key: stale, and I037
  `AC_c15` and plan-025's deferred decisions defer any skill change.
- Test-fixture YAML written twice per case, and `folded` and `one_line`
  duplicated across two test files: real, below the finding bar.
- The sweep's counts: 182 modal verbs across the nine contracts became
  174 keyed promises, the eight drops each a sentence that stood in two
  places; 235 insertions against 235 deletions across the fifty issues,
  each changed line equal to its original once the key is removed.
- Clippy is clean at `-D warnings`; 815 of 816 tests pass, the one
  failure the environmental `the_hooks_return_the_codes_they_declare`;
  `superdev validate` passes and `superdev status` reports no drift.

## Notes

- Findings 1 to 3 are the ones a driver would return to build before
  the merge; 4 to 10 can follow as a chore. The driver returned 1, 2 and
  4 to 9 with 11 to 14 in one build pass; 3 and 10 stand as they are, and
  15 waits on the deferred decision.
- Finding 2 predates the feature; the item bounds made it visible.
- Three grammar-rule nits in the schema prose were dropped as arguable.

<!-- sokf:links -->
[sokf:issue-037-a-contracts-behaviour-is-not-written-as-ears]: /knowledge/issues/done/issue-037-a-contracts-behaviour-is-not-written-as-ears.md
[sokf:plan-025-feature-a-contracts-behaviour-is-written-as-ears]: /knowledge/plans/done/plan-025-feature-a-contracts-behaviour-is-written-as-ears.md
