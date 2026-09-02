---
type: CodeReview
id: code-review-009-a-contracts-behaviour-is-written-as-ears
title: Code review — a contract's behaviour is written as EARS
description: Review of plan-025's eight slices on `feature/a-contracts-behaviour-is-written-as-ears`; three major findings — a contract with no promise passes the schema, an item swallows the heading after it, and the tree-wide PENDING guard was dropped on a false premise — and seven minor ones, recorded for the driver and not fixed here.
links:
  - rel: references
    to: plan-025-feature-a-contracts-behaviour-is-written-as-ears
    note: The plan whose eight slices this review reads.
  - rel: references
    to: issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears
    note: The framed feature; the findings are checked against its criteria.
---

# Code review: plan-025 on `feature/a-contracts-behaviour-is-written-as-ears` (main..5dc4655)

## Verdict

The eight slices of
[plan-025][sokf:plan-025-feature-a-contracts-behaviour-is-written-as-ears]
deliver
[I037][sokf:issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears]:
the three declarations bind, the nine contracts and fifty issues are
swept, and the live tree validates. Three findings are major — a
Behaviour with no promise passes the schema, a heading or table row
that follows a bullet is read as part of the bullet, and the test that
kept every active contract free of `PENDING` was dropped on a premise
that is false at HEAD. Seven are minor: a latent double capture, a
finding that misnames a malformed key, three documents out of step with
the item's scope, two duplications, and a test parser that diverges
from the validator's. Nothing is fixed on the branch; each finding
returns to build at the driver's decision.

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
  the merge; 4 to 10 can follow as a chore.
- Finding 2 predates the feature; the item bounds made it visible.
- Three grammar-rule nits in the schema prose were dropped as arguable.

<!-- sokf:links -->
[sokf:issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears]: /knowledge/issues/open/issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears.md
[sokf:plan-025-feature-a-contracts-behaviour-is-written-as-ears]: /knowledge/plans/done/plan-025-feature-a-contracts-behaviour-is-written-as-ears.md
