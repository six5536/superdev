---
type: CodeReview
id: code-review-010-filing-an-issue-without-framing-it
title: Code review — filing an issue without framing it
description: Review of plan-026's six slices on `feature/030-filing-an-issue-without-framing-it`; one major finding — the accept skill still files a gap issue as `open`, a value the shipped schema refuses — seven minor ones and two nits.
links:
  - rel: references
    to: plan-026-feature-filing-an-issue-without-framing-it
    note: The plan whose six slices this review reads.
  - rel: references
    to: issue-030-filing-an-issue-requires-framing-it
    note: The framed feature; the findings are checked against its seventeen criteria.
---

# Code review: plan-026 on `feature/030-filing-an-issue-without-framing-it` (main..9372773)

## Verdict

The six slices of
[plan-026][sokf:plan-026-feature-filing-an-issue-without-framing-it]
deliver
[I030][sokf:issue-030-filing-an-issue-requires-framing-it]:
the validator selects one rule per heading per variant, the three
tracker schemas vary by a four-state lifecycle with twelve passing
examples, the 50 issues on file are refiled and the 21 prose Expected
behaviours are keyed with their words unchanged, `/file` ships, `/frame`
frames in place, the three phases gate on `framed`, and the live tree
validates. One finding is major: the accept skill, absent from ADR-048's
follow-up list, still files a gap issue `lifecycle: open`, which the
shipped schema refuses. Seven are minor: the pack's tracker skeleton and
the maintain skill keep `open`; a literal rule and a pattern rule for
one heading escape ADR-049's disjointness check; a new schema test
anchors on LF in a file the Windows checkout converts; `/file`, ADR-048
and `AC_unframed-form` require a Motivation of kinds that have none; the
idea schema says a promoted idea leaves; I019's expected behaviours
carry a tag that misnames their pattern; and `AC_records` has no test.
Two nits: a skill count and a numbering sentence.

## Findings

### 1. The accept skill files a gap issue `lifecycle: open` — pack/knowledge/skills/accept/SKILL.md:25

- Severity: major
- Category: correctness
- Problem: FILE GAPS instructs "a BugReport concept `issue-{nnn}-bug-{slug}`
  (`lifecycle: open`)", and the bug-report schema's enum is now
  `[unframed, framed, done, wontfix]`. The filing check reports a value
  outside the enum as a fatal finding and `lifecycle::moves` skips the
  document, so `--fix` files nothing. ADR-048's follow-ups name frame,
  contract-design, feature-plan, execute-feature-plan and how-do-i;
  accept was not swept, in the pack or in `.claude/skills/accept/`.
- Failure scenario: `/accept` finds a gap on a merged feature, writes
  the issue as the step says, and runs `superdev validate`: "`lifecycle`
  value `open` is not in the schema's enum: unframed, framed, done,
  wontfix", the file stays in `knowledge/issues/`, and the phase's own
  validate gate fails on its own record. Every managed repository's
  accept skill carries the same line.
- Suggested fix: `lifecycle: unframed` — accept records the failed
  criterion and does not interview — in the pack and the synced copy,
  and a normative test that no skill writes `lifecycle: open` for an
  issue, beside `the_later_phases_refuse_an_unframed_issue`.
- Outcome: Resolved in cb73b47. FILE GAPS writes `lifecycle: unframed` and says
  `/frame` frames the gap before `/feature-plan` picks it up, whose gate
  refuses an unframed issue; pack and synced copy;
  `no_skill_or_skeleton_writes_an_issue_open` holds every skill in both
  trees to it.

### 2. The pack's tracker skeleton and the maintain skill still say `open` — pack/knowledge/concepts/issue-tracker.md:61

- Severity: minor
- Category: correctness
- Problem: slice 2 changed the skeleton's lifecycle bullet to the four
  values and left lines 31, 57, 61, 64 and 81 reading "`lifecycle:
  open` while open", "open issues", "create a new issue concept with
  `lifecycle: open`", `sokf_search` with `lifecycle: ["open"]` and "an
  `open` issue carrying no triage tag". `maintain`'s CHECK THE WORKFLOW
  RECORDS (line 34) names "an issue settled in prose but still `open`"
  and "gap issues still `open`". The live concept was rewritten; the
  skeleton, which `superdev init` writes into a new repository, was not.
- Failure scenario: an agent in a managed repository follows "when a
  skill says publish to the issue tracker" and writes `lifecycle: open`;
  validate refuses it as in finding 1. The same document says four
  values in one bullet and `open` four paragraphs later, so the reader
  cannot tell which is current.
- Suggested fix: the skeleton carries the live concept's wording for the
  filing sentence, the search filter and the triage note; maintain reads
  "still `unframed` or `framed`".
- Outcome: Resolved in cb73b47. The pack's tracker skeleton is byte-equal to the
  live concept again, as before slice 2; maintain reads "still `unframed`
  or `framed`"; the normative test covers the pack's concept skeletons
  and pins the skeleton to the live concept.

### 3. A literal rule and a pattern rule for one heading escape the disjointness check — crates/lib/superdev-core/src/validate/schema/document.rs:181

- Severity: minor
- Category: correctness
- Problem: `names_same_heading` compares two literals, or two identical
  patterns, and returns false for a `heading` beside a `heading-pattern`
  that matches it, so `heading_conflicts` sees no pair. `check_one`
  then binds both: the literal wins the document's heading, and the
  pattern rule, unmatched, reports its `required`. ADR-049 and
  contract-010 `P_heading-rules-overlap` say two rules naming one
  heading, one untagged, are a schema finding and bind nothing.
- Failure scenario: a schema declaring `heading: "Notes"`
  `variants: [framed]` and `heading-pattern: '^Notes$'` untagged, both
  required, passes `check_variants`, and a framed document carrying
  `## Notes` reports "missing required section matching /^Notes$/" —
  a section it visibly has. Verified with a scratch schema under
  `superdev validate`; the schema's own `framed` example reports the
  same.
- Suggested fix: `names_same_heading` treats a literal and a pattern as
  one heading when the literal matches the pattern at one level
  (`re::compile(p).is_some_and(|re| re.is_match(h))`), so the pair is
  reported and unbound; or contract-010 says the check is by
  declaration form and the skeleton documents it.
- Outcome: Not applied: treating a literal and a pattern it matches as one heading
  makes `superdev validate` report nine live schemas — 18 declare fixed
  headings beside a catch-all `^.+$` at one level, which `check_one`
  resolves by the literal winning and the pattern naming the rest. The
  check stays by declaration form;
  `a_literal_beside_a_pattern_it_matches_is_two_headings` pins it
  (cb73b47), and the reviewer's alternative — contract-010 and ADR-049
  say so — is in plan-026 Deferred decisions, contract-design.

### 4. A new schema test anchors on LF in a file the Windows checkout converts — crates/lib/superdev-core/tests/normative_shapes.rs:501

- Severity: minor
- Category: test-coverage
- Problem: `every_tracker_schema_varies_by_the_four_lifecycle_values`
  reads `knowledge/schemas/*.md` raw and asserts
  `contains("\nvariant-key: lifecycle\n")` and
  `contains("    enum: [unframed, framed, done, wontfix]\n")`.
  `.gitattributes` forces LF on `pack/**` and `.claude/skills/**` and
  not on `knowledge/schemas/**` (I040 rejected extending it), and the
  file's own `same` helper exists for this read; the test does not use
  it.
- Failure scenario: the Windows job checks out `knowledge/schemas/`
  with CRLF, the substring ends `lifecycle\r\n`, and the test fails. The
  job is already red on `main` (run 33677453043) at the same fault in
  `every_cited_list_declares_its_key_and_the_plan_cites_keys`, whose
  raw `plan.split("…$'\n")` at line 597 the feature keeps.
- Suggested fix: `same(&std::fs::read_to_string(…))` on both reads, as
  `the_ears_declaration_ships_to_managed_repositories` does.
- Outcome: Resolved in cb73b47. The lifecycle-values test, the plan schema read of
  the cited-list test, and the idea and wontfix reads of the backlog test
  normalise through `same`; proven by converting every `knowledge/**/*.md`
  to CRLF and running `normative_shapes` clean.

### 5. `/file` requires a Motivation of kinds that have none — pack/knowledge/skills/file/SKILL.md:26

- Severity: minor
- Category: correctness
- Problem: WRITE THE RECORD says "Summary and Motivation in the user's
  words, every other heading of the kind" for a bug, a feature request
  or a chore; the bug-report and chore schemas declare no Motivation
  heading. ADR-048's Decision (line 50) and I030 `AC_unframed-form`
  ("SHALL require its title, description, Summary and Motivation") say
  the same, and `the_file_skill_files_without_framing` pins the phrase.
- Failure scenario: `/file` on a bug writes `## Motivation`; the
  schema's ordered match ignores an undeclared heading, so validate
  passes and the record carries a section its schema does not know. An
  accept walk of `AC_unframed-form` on a chore finds no schema requiring
  Motivation, so the criterion fails as written.
- Suggested fix: the skill and the ADR read "Summary, and Motivation
  where the kind has one"; the criterion is reworded through the
  integrate-to-frame edge.
- Outcome: Deferred: plan-026 Deferred decisions, frame.

### 6. The idea schema and the ideas indexes say a promoted idea leaves — pack/knowledge/schemas/idea.md:17

- Severity: minor
- Category: correctness
- Problem: the schema reads "An idea that is taken up leaves for the
  tracker — `/file` promotes it into an unframed issue — and stops
  being an idea", and `knowledge/ideas/index.md` and the pack skeleton
  `pack/knowledge/concepts/ideas/index.md` read "at which point it
  leaves". I030 `AC_promote-idea` and ADR-048 say the issue links the
  idea with `references` and the idea stays on file; `/file` says so
  too. `/file` reads `schema-idea` at bootstrap.
- Failure scenario: an agent promoting idea-007 follows the schema it
  was told to read and deletes the idea; the new issue's `references`
  link names an id no concept carries, and validate reports it.
- Suggested fix: the three sentences read "stays on file, linked from
  the issue that took it up".
- Outcome: Resolved in cb73b47. `schema-idea` reads "stays on file: `/file`
  promotes it into an unframed issue, and the issue links the idea with
  `references` (ADR-048)"; both ideas indexes read "stays on file, linked
  from the issue that took it up".

### 7. I019's expected behaviours carry a tag that misnames their pattern — knowledge/issues/done/issue-019-validate-reads-a-named-file-as-a-skill.md:54

- Severity: minor
- Category: correctness
- Problem: the sweep tagged I019's five items `[ubiquitous]`; four open
  "WHEN validate is invoked …" and one "IF the named path cannot be
  read, THEN …". ADR-046 says the tag names the pattern.
  `AC_sweep`'s `[ubiquitous]` rule covers a paragraph converted from
  prose; I019's section was a list before the sweep, and the other two
  pre-existing lists, I028 and I029, took the tag their sentences carry.
- Failure scenario: a reader or a test citing `EX_c1` as an unconditional
  requirement reads a trigger-bound one; the framed pattern admits any
  of the six tags, so nothing reports the mismatch.
- Suggested fix: `[event]` on `EX_c1` to `EX_c4`, `[conditional]` on
  `EX_c5`.
- Outcome: Resolved in the commit that files this review. `EX_c1` to `EX_c4` carry `[event]`, `EX_c5`
  `[conditional]`, words unchanged; I028 and I029 were checked and carry
  the tag their sentences take.

### 8. `AC_records` has no test though its cases say `unit` — knowledge/plans/open/plan-026-feature-filing-an-issue-without-framing-it.md:251

- Severity: minor
- Category: test-coverage
- Problem: slice 6 carries two `unit` cases covering `AC_records` —
  the tracker concept and the glossary name the four states and
  `/file`; the changelog's Unreleased names them — and no test in the
  diff reads any of the three. The slice's done note says the cases
  "were checked by reading". The plan schema's Cases are what integrate
  verifies a slice against.
- Failure scenario: a later edit drops `/file` from the tracker concept
  or the four values from the glossary's Lifecycle entry; `cargo test`
  and `superdev validate` pass, and `/accept` walking `AC_records` has
  no automated evidence.
- Suggested fix: relabel the cases `manual`, or add a test in the shape
  of `the_workflow_lists_file_outside_the_phases` that reads the three
  records for the four values and `/file`.
- Outcome: Resolved in the commit that files this review. Both cases read `manual`, with the commit they were
  read at.

### 9. The glossary counts 18 carried skills; 17 exist — knowledge/glossary.md:55

- Severity: nit
- Category: correctness
- Problem: `pack/knowledge/skills/` holds 17 directories with `/file`.
  `main` said 17 against 16 — the count dates from `784f84e`, before
  `interface-design`, `spec` and `verify` retired — and slice 3
  incremented it. I030 `AC_skill-ships`' "the seventeen that exist"
  carries the same stale figure.
- Failure scenario: a reader counting the directories finds the
  glossary wrong by one and doubts the rest of the entry.
- Suggested fix: 17, or drop the number.
- Outcome: Resolved in the commit that files this review. 17, the count of `pack/knowledge/skills/`.

### 10. `/file`'s numbering sentence names "the kind's folders" — pack/knowledge/skills/file/SKILL.md:26

- Severity: nit
- Category: correctness
- Problem: "numbered after the highest across all of the kind's
  folders", where the skill's "kind" is bug, feature request, chore or
  idea. The tracker concept says "the highest existing issue across all
  of the tracker's folders": one sequence across the three kinds — I051
  is a chore after I050, a bug. The schemas' "its kind's folders"
  predates the feature.
- Failure scenario: an agent filing a bug numbers after the highest
  bug on file and writes `issue-051-…`; `superdev validate` reports
  "duplicate `issue` number 051", so the fault costs one round trip.
- Suggested fix: "after the highest issue across the tracker's folders;
  an idea after the highest idea".
- Outcome: Resolved in cb73b47. "numbered after the highest issue across all of
  the tracker's folders", in the skill and the test that pins it.

## Not findings (checked and fine)

- `names_same_heading`'s level handling: a rule with no level names the
  heading at every depth, and two rules at two levels are two headings,
  as the third case of
  `overlapping_or_untagged_rules_for_one_heading_are_a_schema_finding_and_bind_nothing`
  pins.
- `sections-ordered` with the recurring heading: one rule per variant
  is selected, so the order check's first-appearance dedup sees one
  position; `sections-prohibited` entries are a separate list and do
  not enter `heading_conflicts`.
- The twelve examples: `check_example` reports a discriminator that
  differs from its key, and `check_examples` over the three tracker
  schemas is empty.
- The 21 converted Expected behaviours: a script under the job's tmp
  directory strips `` `EX_c<n>` [ubiquitous] `` and the three-space
  continuation indent and reproduces `main`'s paragraphs for all 21;
  keys run 1 to n, every tag is `[ubiquitous]`, and the three
  pre-existing lists differ from `main` only by the key and the tag.
- I028's `[unwanted]` to `[conditional]`: the EARS unwanted-behaviour
  form is "IF x, THEN the system SHALL y", which this project's tag
  vocabulary calls `[conditional]`; the sentences are unchanged.
- The lead-in paragraphs before the lists in I019, I028 and I029: the
  framed Expected behaviour rule declares no `item-only-pattern`, and
  the paragraphs carry no modal verb.
- The `issues/open/` refile: `target` cannot tell a retired state folder
  from an audience partition such as `contracts/public/`, and the
  changelog's migration note says to move the files up before `--fix`.
- The 50 moves: `git diff -M` shows the lifecycle line, the `EX_`
  keys and the link paths as the only changes; 13 open issues became 12
  `framed` and one `unframed` (I042, three `TBD` done items).
- The backlog: `git grep -i backlog` outside ADRs, the changelog and
  settled records finds the three ideas' provenance lines, I030,
  plan-026, the tests and one historical sentence in the tracker index's
  I015 entry.
- The changelog consolidation: the two dropped "After a pack update"
  notes concerned the per-kind contract schemas ADR-043 retired, and
  the include-marker facts survive in the `content: include` and
  source-region entries; the file stands at 800 lines, its limit.
- `contract_010_no_longer_defers_the_item_declarations` narrowed to its
  three declarations: consistent with code-review-009 finding 3's
  outcome, and `contract_010_no_longer_defers_the_per_variant_heading`
  reads the two new promises.
- `LIVE_LIFECYCLES` carries `unframed` and `framed` beside a plan's
  `open`; the search test ranks both live.
- Clippy is clean at `-D warnings`; every test passes but the
  environmental `the_hooks_return_the_codes_they_declare`; `superdev
  validate` passes and `superdev status` reports no drift; the pack and
  synced copies of the five schemas and eight skills are byte-equal.

## Notes

- Finding 1 is the one a driver would return to build before the merge;
  2 to 8 can follow in one pass.
- `sections_for` recomputes `heading_conflicts` on every document and
  example; the pairs are a property of the parsed schema and could be
  read once at parse.
- Criteria to tests: `AC_lifecycle-values`
  `every_tracker_schema_varies_by_the_four_lifecycle_values` and the
  `fix.rs` refile test; `AC_unframed-form`
  `an_unframed_issue_with_plain_tbd_and_keyed_items_passes`;
  `AC_framed-form` the two `_departing_from_the_form_` tests;
  `AC_settled-form` `a_done_and_a_wontfix_issue_are_held_to_the_framed_rules`;
  `AC_one-schema-per-kind` the four `document.rs` tests, the
  `per-variant-heading` snapshot and the contract-010 test;
  `AC_file-issue`, `AC_file-idea`, `AC_promote-idea`, `AC_file-asks`,
  `AC_frame-in-place`, `AC_frame-files`, `AC_phases-refuse`,
  `AC_workflow-lists-file` and `AC_skill-ships` the skill-text tests;
  `AC_sweep` the two `every_issue_on_file_` tests; `AC_backlog-retired`
  `nothing_names_the_backlog` and
  `the_backlog_entries_are_ideas_and_a_wontfix_chore`; `AC_records`
  none (finding 8).
- Finding 4's `main` failure predates the feature; the feature adds a
  second instance.

<!-- sokf:links -->
[sokf:issue-030-filing-an-issue-requires-framing-it]: /knowledge/issues/done/issue-030-filing-an-issue-requires-framing-it.md
[sokf:plan-026-feature-filing-an-issue-without-framing-it]: /knowledge/plans/done/plan-026-feature-filing-an-issue-without-framing-it.md
