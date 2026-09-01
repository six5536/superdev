---
type: CodeReview
id: code-review-006-contracts-define-their-interface
title: Code review of feature/contracts-define-their-interface
description: Feature-wide review of plan-021 — a critical defect in the flagship contract, four drift tests weaker than their ADRs claimed, and a set of stale records, all applied.
---

# Code review: feature/contracts-define-their-interface

## Verdict

The mechanism is sound and the flagship artifact was not: the CLI
contract's definition block did not parse to what it read as. Four
drift tests were weaker than the ADRs and the changelog claimed, and
five records went stale as the feature moved under them. All applied.

## Findings

### 1. The CLI definition block did not parse to what it said — `knowledge/contracts/public/active/contract-002-cli-superdev.md:37`

- Severity: critical
- Category: correctness
- Problem: The block wrote `exit` and `flags` as YAML flow mappings, so
  every comma started a new entry. Eight of 22 exit maps and two flag
  entries were cut in two: `status`'s code `1` meant only "drift", and
  prose fragments like `"an unknown template"` had become exit codes.
- Failure scenario: A generator reads `"or a failed apply"` as an exit
  code — the exact failure ADR-033 exists to prevent, in the document
  the feature holds up as its example.
- Suggested fix: Block style throughout, and tests that every `exit` key
  is an integer and a flag carries only its own keys. Applied; either
  test would have caught all ten instances.

### 2. The CLI drift test missed most of the surface — `crates/app/superdev/src/contract.rs:43`

- Severity: major
- Category: test-coverage
- Problem: A short-only flag was dropped, aliases were never read,
  requiredness and multiplicity were not modelled, possible values and
  `about` were unchecked, and a global flag declared on the root alone
  passed while reaching every subcommand.
- Failure scenario: `superdev st` works as an alias with no contract
  entry; `--drift` gains a short form and nothing notices.
- Suggested fix: Model the whole surface. Applied — `about`, aliases,
  short forms, requiredness, multiplicity, possible values and global
  propagation, with four mutations that previously passed now failing.
  Aliases needed `get_all_aliases`: the framework treats `alias` as
  hidden.

### 3. The interface binding bound almost nothing — `crates/lib/superdev-core/tests/contract_interfaces.rs:28`

- Severity: major
- Category: correctness
- Problem: The head cut treated `->` as a closing bracket, so depth went
  negative and no signature's head ever ended. Struct fields and enum
  variants were never bound at all, and the haystack was every `.rs`
  file including tests and comments, so a declaration could be satisfied
  by a comment.
- Failure scenario: A field is retyped, or an invented signature is
  planted in a comment, and the test passes.
- Suggested fix: Read the arrow as an arrow, bind fields and variants,
  and search production source only. Applied; the two mutations the
  review demonstrated now fail, and the bound count went from 30
  prefixes to sixty-odd items.

### 4. The definition-form assertion could not fail — `crates/lib/superdev-core/tests/normative_shapes.rs:375`

- Severity: major
- Category: test-coverage
- Problem: It sniffed the whole schema file for "alone" or "every",
  which the shared contract-style fragment supplies to all sixteen, so
  the assertion was true by construction.
- Failure scenario: A kind's definition section is weakened to "a few
  sample commands" and the test passes.
- Suggested fix: Read the section rule, not the prose. Applied: a
  declared block language must be one the validator reads and must
  declare keys, and a table definition must declare its columns.
  Completeness against reality is the drift test's to bind; no wording
  can decide it.

### 5. Exit-code coverage was a quarter of what was claimed — `crates/app/superdev/tests/contract_exit_codes.rs:49`

- Severity: major
- Category: test-coverage
- Problem: 14 of 55 declared pairs were probed, ten commands were never
  run, and code `1` — the one CI gates on — was never probed. An
  unconditional `code == 2` escape let a declaration be deleted
  unnoticed.
- Failure scenario: `status` returns `3` on drift and no test fails.
- Suggested fix: Drive coverage from the contract. Applied: all 37
  declared pairs are probed or named undrivable with a reason, the hooks
  and `validate`'s error code gained real probes, and the escape is
  gone. The probes found the parent commands' declared exit wrong —
  they return `2`, not `0`.

### 6. Two contracts this repository implements had no binding — `crates/lib/superdev-core/tests/contract_files.rs`

- Severity: major
- Category: test-coverage
- Problem: Slice 5 claimed drift tests bound contracts 004, 005, 006 and
  008; only 004 and 005 had any, and the pack test asserted one field.
- Failure scenario: A key is added to the lock and its contract never
  moves.
- Suggested fix: Bind the lock. Applied, with an independently built
  fixture — a fixture read from the contract only agrees with itself,
  which is why the first attempt caught nothing.

### 7. The CLI rewrite dropped surface the old contract bound — `contract-002-cli-superdev.md`

- Severity: major
- Category: correctness
- Problem: `--json`'s output keys, `completions`'s accepted shells,
  `update TARGET`'s grammar, and the `hook validate`, `sokf index` and
  `mcp sokf` promises were lost with no replacement.
- Failure scenario: A caller parsing `--json` has nothing to build
  against — the failure ADR-033 targets.
- Suggested fix: Restore them to the block and to Behaviour. Applied,
  with `arg-values`, `arg-grammar` and a `json` key now declared.

### 8. The config key check compared bare words — `contract_files.rs:97`

- Severity: minor
- Category: correctness
- Problem: `declared.contains(key)` matched a bare word anywhere, so
  `template.version` was satisfied by `version` under another table.
- Failure scenario: A key superdev writes is deleted from its contract
  and nothing notices.
- Suggested fix: Compare key paths, and populate the fixture fully.
  Applied; the mutation now fails.

### 9. A definition block's decoy was read instead of the block — `crates/lib/superdev-core/src/validate/schema/document.rs:857`

- Severity: minor
- Category: correctness
- Problem: Only the first fence in a section was read, so an
  illustration before the definition drew the finding.
- Failure scenario: A contract carrying an example above its definition
  is reported against the example.
- Suggested fix: Read the block whose tag the rule declares, and name
  the tags the section carries when none matches. Applied, with tests.

### 10. A language alone silently demanded a mapping — `document.rs:888`

- Severity: minor
- Category: correctness
- Problem: `block-language` with no keys reported "declares keys for
  it" when none were declared — a rule stated nowhere.
- Suggested fix: A language alone demands only the language. Applied.

### 11. The rename rewrote two historical records — `knowledge/reports/`

- Severity: minor
- Category: correctness
- Problem: The blanket sweep rewrote code-review-004 and 005 to cite
  contract names that did not exist when those reviews were written.
- Suggested fix: Restore them. Applied, and the retired-kind test's own
  exemption for reports is what made the sweep's reach visible.

### 12. Five records went stale as the feature moved — `adr-032`, `adr-033`, `adr-034`, `adr-035`, `adr-029`, `CHANGELOG.md`

- Severity: minor
- Category: correctness
- Problem: ADR-032 still listed `mcp Tools` and `deployment Runtime` as
  promise sections and `file-format` as a kind; three ADRs said
  "fifteen" of what the same feature made sixteen; ADR-033 claimed nine
  contracts were rewritten when four were; ADR-029's note claimed all
  four style rules survived when one was replaced.
- Suggested fix: Correct each, and say in ADR-032 why a section leaves
  its assignment. Applied.

### 13. Stale kind prose the retired-kind test did not hunt — `contract-text-format.md:10`

- Severity: minor
- Category: correctness
- Problem: The test searched four id tokens, so "one public file-format
  contract" and two link texts read clean.
- Suggested fix: Hunt the kind's name too, scoped to what a writer
  builds against — the records that say what changed may name it.
  Applied.

### 14. The grammar's language enum was unpinned — `document.rs:79`

- Severity: minor
- Category: test-coverage
- Problem: `BLOCK_LANGUAGES` and the grammar's enum are two lists; a
  third language added to one would accept a block nothing reads.
- Suggested fix: A test pinning them together. Applied; adding `toml` to
  the grammar alone now fails.

## Not findings (checked and fine)

- The definition block's parsing: each language is read by its own
  parser, and sequences, scalars, non-mapping entries, duplicate keys,
  indented fences and tilde fences all behave.
- The MCP drift test is both-directional over name, argument,
  requiredness and type, and unwraps the optional union correctly.
- Both grammar copies are byte-identical, and every count pin was
  updated correctly.

## Left as it stands

- **Six fenced-block readers across the validator and the drift tests.**
  The duplication is real and the DRY rule names it. Consolidating means
  a shared reader in `superdev-core` that the binary crate's test and
  four test files all call, which is a refactor of its own rather than a
  review fix; the readers agree today, and the grammar's language pin
  now catches the one divergence that would matter.
- **A non-string block key reports `(unnamed)`.** The integer keys in an
  `exit` map are the only case on file, and the two new shape tests
  report those by name instead.
