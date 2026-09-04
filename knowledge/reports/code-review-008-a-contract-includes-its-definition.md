---
type: CodeReview
id: code-review-008-a-contract-includes-its-definition
title: Code review — a contract includes its definition
description: Review of plan-024's ten slices on `feature/a-contract-includes-its-definition`; two CRLF defects that fail Windows CI and corrupt a repaired block, one four-times test skeleton, and four smaller findings, all resolved before the merge.
links:
  - rel: references
    to: plan-024-feature-a-contract-includes-its-definition
    note: The plan whose ten slices this review reads.
  - rel: references
    to: issue-049-a-contract-cannot-point-at-its-definition
    note: The framed feature; the findings are checked against its criteria.
---

# Code review: plan-024 on `feature/a-contract-includes-its-definition` (main..6967e03 and the staged slice 10)

## Verdict

The ten slices of
[plan-024][sokf:plan-024-feature-a-contract-includes-its-definition]
deliver
[I049][sokf:issue-049-a-contract-cannot-point-at-its-definition].
The mechanism is sound on an LF checkout and the migration is complete;
two correctness findings block merge, both on a CRLF checkout — every
source include reports stale, and `--fix` then corrupts the block so
`validate` passes on a definition it no longer checks. Five further
findings are simplifications and test gaps that change no behaviour.
Every finding was resolved on the branch.

## Findings

### 1. A source include is stale on every CRLF checkout — crates/lib/superdev-core/src/validate/sokf.rs:493

- Severity: major
- Category: correctness
- Problem: `source::render` joins the file's lines with `\n`, and
  `check_include_blocks` compares that string byte for byte with the
  host document's block (`actual == expected`); `materialize` in
  `validate/fix.rs:180` compares the same way. A host document checked
  out with CRLF carries `\r\n` inside the block, so the two never match.
  The links block already compares a line at a time for this reason
  (`sokf.rs:607`, `fix.rs:136`; I040).
- Failure scenario: a Windows checkout with the default `core.autocrlf`,
  which is how the CI job checks out (I040). `validate` reports all 23
  source includes of the nine contracts as stale, and
  `validate_passes_the_live_repository` fails on `windows-latest`.
  Verified on Linux with a CRLF fixture: `validate` exits 1 with `the
  include block for `/src/main.rs#cli` is stale` on a block whose content
  is the region.
- Suggested fix: one predicate, `fsutil::lines(actual) ==
  fsutil::lines(&expected)`, called from the check and from
  `materialize`, as the links block does.
- Outcome: fixed. `source::carries` compares the block and the render a
  line at a time, and both sites call it; pinned by
  `a_crlf_host_or_source_agrees_with_the_render_and_a_stale_crlf_block_fails`
  and `a_crlf_host_carrying_the_concept_passes_and_a_stale_one_fails` in
  `sokf.rs`, which failed on the unfixed code.

### 2. `--fix` on a CRLF document writes the block onto the marker line — crates/lib/superdev-core/src/sokf/concept.rs:426

- Severity: major
- Category: correctness
- Problem: `include_blocks` sets `content_start` to `at + line.len()`,
  where `line` comes from the markdown parser's HTML event. The event
  text carries `\n` where the document carries `\r\n`, so on a CRLF
  document `content_start` lands on the `\r` — one byte short. The
  repair pass writes `body[..content_start]`, then the content, then
  skips to `content_end`, so the marker's line ending is dropped. This
  predates the feature; finding 1 puts every CRLF checkout on its path,
  and a source include makes the outcome silent.
- Failure scenario: a stale source include in a CRLF document under
  `validate --fix`. The file comes back as `<!-- sokf:include
  /src/main.rs#cli -->```rust`, the open marker no longer ends its line,
  the parser reads the rest as a fence, and `validate` passes with no
  finding: the definition is no longer an include block and nothing
  checks it. Verified with a CRLF fixture; a throwaway test showed
  `content_start` at the `\r` and the slice reading `"\r\nOld.\r\n"`. A
  stale concept include on CRLF is mangled the same way and at least
  fails afterwards with `` `<!-- /sokf:include -->` with no open marker``.
- Suggested fix: take `content_start` from the document's own bytes —
  the offset after the first `\n` at or past `at` — rather than from
  the event text's line length.
- Outcome: fixed. `include_blocks` walks the event's span of the
  document and takes `content_start` from `line_end(text, at)`; the
  parser hands a CRLF line back as its text and a lone `\n` with the
  `\r` in no event, which the old offset landed on. Pinned by
  `include_blocks_span_the_content_exactly_on_a_crlf_document`
  (`concept.rs`), `materialize_keeps_the_marker_line_whole_on_a_crlf_document`
  (`fix.rs`) — which reproduced the `-->```rust` corruption before the
  fix — and `validate_fix_on_a_crlf_checkout_keeps_the_include_checked`
  (`cli.rs`), which proves the repaired block is still checked.

### 3. Four scratch-copy tests share one skeleton — crates/app/superdev/tests/cli.rs:2587

- Severity: minor
- Category: simplification
- Problem: `a_flag_added_to_validate_args…` (2587),
  `a_field_renamed_in_the_lock_struct…` (2674),
  `a_pub_fn_renamed_in_the_resolver…` (2759) and
  `a_token_added_to_the_template_engine…` (2855) each copy a live
  contract and its sources into a temporary repository, cut the
  Definition out of the contract, run `--fix`, edit one source string,
  assert `validate` names the stale include, assert nothing was written
  without `--fix`, run `--fix` and assert the edit arrived — 70 lines a
  time, four times. Each also pins a literal line of live source (`///
  Emit JSON instead of text`, `pub digest: Option<String>,`, `pub fn
  resolve(`, the `TOKEN_PASCAL` line) and a count of includes.
- Failure scenario: the fixture format or the stale message changes and
  four tests move together; a doc comment in `validate_cli.rs` is
  reworded and a test fails whose subject is the include mechanism,
  which `validate_fix_materializes_a_source_include…` (2518) already
  proves on a pure fixture.
- Suggested fix: one helper taking the contract path, the source file to
  edit, the `(from, to)` replacement and the expected stale include, with
  each test reduced to its four arguments and its own assertions on what
  the include carries.
- Outcome: fixed. `contract_drift_in_scratch(contract, include, from, to)`
  carries the skeleton and returns the Definition's include arguments,
  the Definition and the repaired text; each test keeps its four
  arguments and its own assertions. The four tests pass with the same
  assertions as before.

### 4. An Exit codes row without backticks vanishes from the coverage check — crates/app/superdev/tests/contract_exit_codes.rs:51

- Severity: minor
- Category: test-coverage
- Problem: `declared()` keeps a table row only when its first cell opens
  with a backtick, so the header row and the rule row are skipped — and
  so is any command row written without backticks.
- Failure scenario: a row `| superdev sokf index | 3 | … |` is added to
  the contract. `declared()` drops it, and
  `every_declared_exit_code_is_probed_or_named_undrivable` passes while
  code 3 is declared and never proved.
- Suggested fix: skip the header and the `|---|` rule explicitly, and
  panic on any other row whose first cell is not a backticked command,
  as the code cell already panics when it does not parse.
- Outcome: fixed. `declared_in` skips the `Command` header and the rule
  and returns an error naming any other row it cannot read; `declared`
  panics with it. Pinned by
  `a_row_the_reader_cannot_read_is_reported_not_dropped`.

### 5. The marker rule in the code is narrower than the SPEC's — crates/lib/superdev-core/src/validate/source.rs:115

- Severity: minor
- Category: correctness
- Problem: `marks` accepts `sokf:begin <name>` only when the name ends
  at whitespace or at the end of the line, so `cli` does not open
  `cli-v2`. SPEC §9 and ADR-041 say the marker is "matched by
  substring". The code's rule is the right one; the documents do not
  state it.
- Failure scenario: a project writes `/* sokf:begin cli*/`, which the
  SPEC's wording admits, and `validate` reports `the file carries no
  region `cli``.
- Suggested fix: state in SPEC §9 (both copies) and in ADR-041's Decision
  that the name runs to whitespace or the end of the line.
- Outcome: fixed. SPEC §9 (`pack/sokf/agents/sokf/SPEC.md`, synced to
  `.agents/sokf/SPEC.md`) and ADR-041's Decision now read "matched by
  substring, the name ending at whitespace or the end of the line".

### 6. The guard against copy-comparing tests matches identifier names — crates/lib/superdev-core/tests/normative_shapes.rs:456

- Severity: minor
- Category: test-coverage
- Problem: `no_test_compares_a_fenced_block_of_an_included_contract_to_the_binary`
  fails a file only when it names a migrated contract and one of three
  helper names — `fenced_block`, `rust_lines`, `tokens_in` — the deleted
  tests happened to use.
- Failure scenario: a new test reads a fence out of `contract-002` with a
  helper named `block_of`, compares it to the clap tree, and the guard
  passes.
- Suggested fix: match the contract name together with a fence opener
  in a string literal (` ``` `), or drop the test and let criterion 23
  rest on the deletions the review confirmed.
- Outcome: fixed. The guard splits each source file into its top-level
  items and fails when an item names a migrated contract and either that
  item or any non-test item carries a fence reader — a literal ` ``` `
  or an identifier carrying `fenced` — whatever the helper is called.
  Verified to fail on a probe test that split `contract-002` on a fence
  through a helper named `block_of`.

### 7. The include target is resolved twice — crates/lib/superdev-core/src/validate/sokf.rs:461

- Severity: nit
- Category: simplification
- Problem: `check_include_blocks` (sokf.rs:461–490) and `materialize`
  (fix.rs:161–178) carry the same `match` on `IncludeTarget`: look the
  concept up, refuse one that nests, or render the source. One pushes a
  finding where the other skips.
- Failure scenario: a third target form is added to `IncludeTarget` and
  one site is updated; the check reports a block the repair cannot fill,
  or the reverse.
- Suggested fix: one `fn expected(target, lookup: impl Fn(&str) ->
  Option<&str>, repo_root) -> Result<String, String>`, whose `Err` is the
  finding's text and the repair's reason to skip.
- Outcome: fixed. `source::expected` resolves the target for both the
  check and the repair; `check_include_blocks` pushes its `Err` as the
  finding and `materialize` skips on it.

## Not findings (checked and fine)

- Containment: `render` canonicalises the file and the root before a
  component-wise `starts_with`, so `..`, a symlink out of the tree and,
  on Windows, an absolute `C:/` argument that `join` would adopt are all
  refused before any read; `--fix` reads source and writes only under
  the bundle root.
- Fence widening: one backtick more than the longest run opening a body
  line, never fewer than three; a region carrying ```` ``` ```` renders
  inside a four-backtick fence.
- An empty file or an empty region renders ```` ```tag\n\n``` ````, which
  the check accepts once written, so `--fix` converges.
- `marks` slices at ASCII marker offsets and cannot split a character;
  `line_starts` and `fsutil::lines` have the same length, so
  `starts[start..end]` cannot panic.
- A document with no `kind`, or one outside the enum, sees the untagged
  rules and the frontmatter check names the value; `id_kind` on a
  two-segment id is `None` and the schema's `id` pattern reports it.
- The deleted tests each compared a copy: `contract.rs`'s surface and
  key-shape tests, `mcp.rs`'s tool comparison, `contract_files.rs`,
  `contract_interfaces.rs` and `contract_template.rs`.
  `every_command_carries_the_help_flag` and `contract_exit_codes.rs`,
  the two that test behaviour, stay.
- The app crate's `serde_yaml_ng` dev-dependency, the `fenced_block`
  helpers and `BLOCK_LANGUAGES` are gone; clippy is clean at
  `-D warnings` and 789 tests pass.
- The schema examples' fictional include paths are never rendered: the
  example check reads presence only, and the SOKF check does not run on
  an example.

## Notes

- All seven findings were resolved on the feature branch in the commit
  after the review, before the merge; findings 1, 2 and 4 each carry a
  regression test that fails on the unfixed code, and the full suite
  grew from 789 to 795 tests.
- No test parses the shipped `pack/pack.toml` with `PackManifest::parse`
  now that `contract_files.rs` is gone; the deleted test parsed the
  contract's copy, so nothing was lost, but plan-024 slice 7's caveat
  was not acted on. Whether `init`'s sync path parses the embedded
  manifest through `read_pack` was not traced.
- `manage.rs:131` opens its second `cli` region on the line before a
  blank one, so the block in `contract-002` opens with an empty line.
- `knowledge/reports/index.md` does not exist; none of the eight reviews
  is listed anywhere.
- A document that includes its own path can never settle: each `--fix`
  changes the file the next render reads.

<!-- sokf:links -->
[sokf:issue-049-a-contract-cannot-point-at-its-definition]: /knowledge/issues/open/issue-049-a-contract-cannot-point-at-its-definition.md
[sokf:plan-024-feature-a-contract-includes-its-definition]: /knowledge/plans/done/plan-024-feature-a-contract-includes-its-definition.md
