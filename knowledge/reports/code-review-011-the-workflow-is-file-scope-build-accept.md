---
type: CodeReview
id: code-review-011-the-workflow-is-file-scope-build-accept
title: Code review — I052, the workflow is scope, build, accept
description: The whole of I052 reviewed at its merge commit against a808b16 — the validator's nested items survived every attack, and sixteen findings sit in the prose and process layers, six of them filed as issues 053 to 058.
lifecycle: done
links:
  - rel: references
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: The change reviewed, at acceptance.
  - rel: references
    to: plan-027-the-workflow-is-file-scope-build-accept
    note: The plan whose seven blocks the change delivered.
---

# Code review: I052 at 2fc23e2, against a808b16

## Verdict

Sound as merged:
[I052][sokf:issue-052-the-workflow-carries-more-process-than-it-needs]'s
validator change, delivered by
[plan-027][sokf:plan-027-the-workflow-is-file-scope-build-accept],
survived every attack the review could construct, and all sixteen
findings sit in the prose and process layers, six of them filed as
issues 053 to 058.

## Findings

### 1. `item-only-pattern` admits a nested item, and three documents say it does not — `knowledge/contracts/internal/active/contract-010-interface-document-schemas.md`

- Severity: major
- Category: correctness
- Problem: with a `nested` rule declared, a line inside a nested item
  counts as inside an item, so `item-only-pattern` no longer reports
  it; contract-010, the `item_only_pattern` doc comment and the
  grammar all say the pattern matches only inside a top-level item.
- Failure scenario: a schema author declares `item-only-pattern` to
  keep modal verbs out of prose, reads the vocabulary, and expects a
  nested criterion carrying `SHALL` to be reported. It is not. The
  finding text for a real match still reads "outside a top-level
  item", which sends the author to the wrong line.
- Suggested fix: filed as issue-053. The contract decides which side
  moves; the words are likelier wrong than the behaviour.

### 2. An issue whose Behaviour is only bullets is refused — `knowledge/schemas/issue.md`

- Severity: major
- Category: correctness
- Problem: Behaviour is declared `content: prose`, which binds by the
  presence of one plain paragraph line, so an issue written as a bare
  list fails; I052's own Behaviour says the validator accepts prose,
  bullets or both.
- Failure scenario: a user files a bug as three bullets. `/file`
  writes them, `superdev validate` reports the section, and the record
  cannot be filed as the user stated it.
- Suggested fix: filed as issue-054. Widen the content kind, or hold
  the criterion to what the schema says and make `/file` open every
  section with a line of prose, which its text already claims to do.

### 3. Fifty-five "criterion N" citations resolve to nothing — `knowledge/issues/index.md`

- Severity: major
- Category: correctness
- Problem: the rewrite dropped the tracker's keys, which was intended,
  and the numbering of the criteria lists, which was not, and merged
  each feature request's proposed-behaviour bullets ahead of its
  criteria in one list.
- Failure scenario: a reader follows "I035 criterion 4" from the
  issues index and lands six bullets short of the criterion meant.
  Fifty-five citations across nineteen documents are affected.
- Suggested fix: filed as issue-055.

### 4. Three active ADRs decide in terms of retired skills — `knowledge/adrs/active/adr-028-contract-design-commits-on-approval.md`

- Severity: major
- Category: correctness
- Problem: ADR-021 says `/frame` and `/adhoc-plan` cut the branches,
  ADR-020's context sends a gate to `/frame` or `/contract-design` and
  its decision names `/execute-feature-plan`, and ADR-028 decides that
  `/contract-design` commits on approval — which the new skill refuses,
  with no superseding ADR.
- Failure scenario: an agent reads the active decision record for the
  branch convention and cuts `feature/<slug>` where the procedure says
  `feature/<nnn>-<slug>`; or reads ADR-028 and commits from
  `/contract-design`, which its own rules forbid.
- Suggested fix: filed as issue-056. The sweep test's roots omit
  `knowledge/adrs/`, so nothing catches this class.

### 5. The skills disagree on who loops and who returns — `pack/knowledge/skills/execute-plan/SKILL.md`

- Severity: major
- Category: correctness
- Problem: `/contract-design` closes with an unconditional call to
  `/scope`, whose first steps cut a branch and interview the user —
  the only cycle in the pack's skill graph; and `/execute-plan` loops
  over the blocks invoking `/build` per block, while `/build` loops
  the blocks itself and then merges and closes the plan.
- Failure scenario: an unattended run drives `/build` for block 1,
  which reaches its closing pass, runs the full suite, merges and sets
  the plan `done` — with six blocks unbuilt.
- Suggested fix: filed as issue-057.

### 6. A plan case marked manual is executed nowhere — `pack/knowledge/skills/build/SKILL.md`

- Severity: minor
- Category: test-coverage
- Problem: the retired `/integrate` ran the plan's cases, manual ones
  included; `/build` runs the tests and the suite, and its gate is
  satisfied by the label alone; `/accept` walks contract criteria, not
  plan cases, and is optional.
- Failure scenario: a plan marks a UI check manual and covers no
  contract criterion by it. Nobody runs it, and the change merges
  unverified against that case.
- Suggested fix: filed as issue-058.

### 7. The ad-hoc plans' Decisions rows have no home in `schema-plan` — `knowledge/plans/done/plan-010-links-address-ids.md`

- Severity: minor
- Category: correctness
- Problem: ten done ad-hoc plans carried a Decisions table of ninety-nine
  rows, each naming a decision, its rejected alternative and why it
  lost; `schema-plan` carries only Deferred decisions, which are open
  questions, so the rewrite had nowhere to put them.
- Failure scenario: a reader asks why the definition block is
  generated rather than author-maintained. Plan-010's D-3 answered it;
  the file now contains no mention of an alternative, and git history
  is the only record.
- Suggested fix: accept the loss, or give `schema-plan` a section for
  a decision the plan settles. Not filed: the rationale that mattered
  mostly survived as Goal prose, and ADRs carry the durable decisions.

### 8. Forty-six test doc comments cite `AC_` keys that resolve to nothing — `crates/lib/superdev-core/tests/normative_shapes.rs`

- Severity: minor
- Category: test-coverage
- Problem: the tests cite `AC_contract-criteria`, `AC_issue-schema`
  and six more — I052's pre-rewrite framed criteria, since deleted —
  while the contract schema says a test's doc comment cites a contract
  key, and a normative test now forbids an issue carrying an `AC_`
  item at all.
- Failure scenario: a reader greps for `AC_issue-schema` to find what
  a test binds and gets nothing outside `crates/`.
- Suggested fix: point them at contract-010's real keys, which several
  already cite parenthetically.

### 9. `LIVE_LIFECYCLES`'s `active` half is untested — `crates/lib/superdev-core/src/sokf/index.rs`

- Severity: minor
- Category: test-coverage
- Problem: the fixtures carry `abandoned`, `done`, `open`, `framed` and
  none, and no `lifecycle: active` document, though the doc comment
  claims both halves are covered.
- Failure scenario: a later edit drops `active` from the constant.
  Every active contract and ADR sinks below settled work in search,
  and no test says so.
- Suggested fix: add a fixture carrying `lifecycle: active`.

### 10. `item-key-optional` ships with no consumer — `crates/lib/superdev-core/src/validate/schema/document.rs`

- Severity: minor
- Category: simplification
- Problem: no schema declares it; its stated purpose in the doc
  comment is "a wontfix issue's either-form lists", which this feature
  retired, and the doc comment is materialised into contract-010's
  Definition.
- Failure scenario: a schema author reads the vocabulary and looks for
  the wontfix lists the comment names. They do not exist.
- Suggested fix: reword the doc comment, or withdraw the declaration
  under YAGNI. The user asked for the support to stay.

### 11. Three near-identical per-item loops — `crates/lib/superdev-core/src/validate/schema/document.rs`

- Severity: nit
- Category: simplification
- Problem: `check_body_patterns`, `check_item_keys_in` and
  `check_item_bounds` each repeat the same walk — take `rule.levels()`,
  skip a reported item, index the level, compile the pattern — over
  the struct's three parallel vectors, and `rule.levels()` allocates
  five times per section.
- Failure scenario: a fourth per-item check is added and the walk is
  copied a fourth time, with one copy forgetting the reported skip.
- Suggested fix: one helper taking a field accessor and a reporter.

### 12. `schema-plan` states two rules twice — `pack/knowledge/schemas/plan.md`

- Severity: nit
- Category: simplification
- Problem: the block-ordering rule, the no-renumbering rule and the
  case-citation rule each appear in the header prose and again in a
  section description; the retired feature-plan schema carried each
  once.
- Failure scenario: one copy is amended and the other is not.
- Suggested fix: keep the section description and cut the header copy.

### 13. "36 plans" is wrong in two live documents — `knowledge/issues/open/issue-052-the-workflow-carries-more-process-than-it-needs.md`

- Severity: nit
- Category: correctness
- Problem: the issue and ADR-050's options table both say 36 plans;
  there were 26 before the change and 27 after.
- Failure scenario: a reader checks the count against the tree and
  doubts the rest of the document.
- Suggested fix: correct both numbers.

### 14. Branches this change introduced that no test reaches — `crates/lib/superdev-core/src/validate/schema/document.rs`

- Severity: nit
- Category: test-coverage
- Problem: a nested pattern that fails to compile, an indented
  top-level list now reported without its indentation, lazy
  continuation joining the innermost open item, `required` at a second
  nested level, and two of the three mis-declaration cases in
  `a_mis_declared_nested_rule_is_a_finding_on_the_schema`, whose probe
  document is too shallow to reach them.
- Failure scenario: the mis-declaration test passes while its
  "binds nothing" half asserts over an empty finding list.
- Suggested fix: deepen the probe document and add the four cases.

### 15. A quoted panic transcript was edited — `knowledge/issues/done/issue-039-validate-fix-refuses-to-refile-under-a-symlinked-root.md`

- Severity: nit
- Category: correctness
- Problem: the id sweep rewrote a path inside a verbatim transcript of
  what the tool printed, so the quotation is no longer a quotation.
- Failure scenario: a reader reproduces the bug and compares output
  against a record that no run ever produced.
- Suggested fix: restore the transcript and leave quoted output alone
  in future sweeps.

### 16. Smaller inconsistencies — `pack/knowledge/skills/how-do-i/SKILL.md`

- Severity: nit
- Category: correctness
- Problem: how-do-i still says "once the feature has stopped
  changing" where the change moved to "work"; `check.rs` writes a
  fully-qualified `KeyTable` path though its module is imported;
  issue-023 illustrates the id form with a deleted schema's id; a
  document-snapshot doc comment describes a repeat "between a promise
  and a criterion" where the fixture repeats between two criteria; and
  the cross-level repeat message says "declares item-key" even at a
  nested level, where every other message says "a nested item-key".
- Failure scenario: each misleads one reader once.
- Suggested fix: correct in passing.

## Not findings (checked and fine)

- The nested-item reader under attack: a two-hundred-level indentation
  ladder, tab indentation, multibyte and emoji item text, a nested
  marker before any top-level one, other-kind markers at both levels,
  lazy continuation across levels, third-level markers beyond the
  declared depth, and a two-thousand-token item. No panic, no
  unbounded loop, no quadratic scan, no off-by-one, no finding naming
  the wrong item.
- `levels[item.level]` cannot index out of bounds, and the reported
  slice cannot panic: `opens` gates the level against the same depth
  the rule derives, and `beneath` is bounded by the items remaining.
- `re::compile` inside the per-item loops is memoised, so moving it
  there costs a hash lookup, not a recompile.
- The check order matches the contract: item-key, then the prohibited
  pattern, then the item pattern, then the nested requirement, which
  runs last and skips reported items.
- All 52 issues and 27 plans conform, and their `links:` blocks
  survive one for one.
- The retired types are refused, Resolution is required under `done`
  and `wontfix` and refused under `open`, and the rendered aggregator
  matches its source.
- The fifteen deleted tests all pinned behaviour this change retired;
  no live coverage was lost, and no new assertion is loose enough to
  pass against the old text.

## Notes

- The two workflow tests asserted the flow line and two of six edges,
  and a hand edit to the rendered aggregator passed them. Both now pin
  every phase, every edge and accept's entry.
- The sweep test's roots omit `knowledge/adrs/`, which is right for
  the deprecated folder and wrong for the active one. Issue-056 carries
  it.

<!-- sokf:links -->
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/done/issue-052-the-workflow-carries-more-process-than-it-needs.md
[sokf:plan-027-the-workflow-is-file-scope-build-accept]: /knowledge/plans/done/plan-027-the-workflow-is-file-scope-build-accept.md
