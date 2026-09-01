---
type: CodeReview
id: code-review-004-contract-design-review
title: Code review of feature/contract-design-review
description: Feature-wide review of plan-019 — ten findings, two correctness defects in the include mechanism and a false sweep claim among them, all applied.
---

# Code review: feature/contract-design-review

## Verdict

Sound feature; two correctness defects in the include mechanism, one
false completeness claim, and three self-violations of the standard the
feature ships — all ten findings applied before merge.

## Findings

### 1. Nesting detection was a raw substring test — `crates/lib/superdev-core/src/validate/sokf.rs:403`

- Severity: critical
- Category: correctness
- Problem: The check and the fix tested `contains(INCLUDE_OPEN)` while
  marker discovery is parser-based, so a source showing a marker inside
  a fenced example was falsely refused as nesting — an error `--fix`
  could never repair — and the predicate was duplicated in two files.
- Failure scenario: A fragment documenting the include mechanism with a
  fenced SPEC §9 example makes every includer permanently invalid.
- Suggested fix: One shared parser-based helper. Applied:
  `carries_include_markers` in `concept.rs`, used by check and fix, with
  a regression test on the fenced case.

### 2. One marker fault froze every include repair — `crates/lib/superdev-core/src/validate/fix.rs:138`

- Severity: critical
- Category: correctness
- Problem: `materialize` returned the body unchanged when any marker
  fault existed, while the check still told the user to run `--fix` for
  the well-formed stale blocks it reported.
- Failure scenario: A stray close marker plus a stale block: `--fix`
  changes nothing and the error naming `--fix` as the remedy persists.
- Suggested fix: Repair the well-formed blocks whatever else is wrong.
  Applied, with a regression test asserting only the marker fault
  remains after a fix.

### 3. "Nine contracts swept" was false — `CHANGELOG.md:108`

- Severity: critical
- Category: correctness
- Problem: Contracts 004 and 008 were untouched and carried zero
  RFC 2119 keywords, failing I029 criterion 5 while the CHANGELOG
  published the sweep as complete.
- Failure scenario: Acceptance walks criterion 5 and fails it on the
  feature head.
- Suggested fix: Sweep both. Applied: modal verbs on every promise in
  004's sources, secrets and stability sections and 008's compatibility
  and stability sections.

### 4. The 007 sweep dropped bound surface — `knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md:104`

- Severity: major
- Category: correctness
- Problem: The rewrite dropped the derive lines the contracted types
  promise (`Ord`/`Hash` on `Owner`/`ItemKind` back a `BTreeMap` key in
  the code) and the promise that `sync` never adds the `[[packs]]`
  entry — under the new "unlisted is the code's to decide" rule, both
  became droppable.
- Failure scenario: A later change removes `Ord` or makes `sync` write
  the manifest without violating anything on file.
- Suggested fix: Restore both. Applied: derives on every contracted
  type; "`sync` MUST NOT add the `[[packs]]` entry" in migration.

### 5. The swept 007 violated the standard it teaches — `knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md:339`

- Severity: major
- Category: correctness
- Problem: The security bullet chained five requirements into one
  semicolon-joined sentence with one modal verb; the symlink failure
  behaviour carried none.
- Failure scenario: Slice 4's done-check fails on the corpus meant to
  teach by example.
- Suggested fix: One requirement per sentence, each modal. Applied.

### 6. "nothing else SHOULD" misused the keyword — `knowledge/contracts/public/active/contract-006-text-format-lock.md:18`

- Severity: minor
- Category: correctness
- Problem: The keyword attached to a negated subject with the verb
  elided, leaving strength and direction ambiguous.
- Failure scenario: A reader cannot tell SHOULD from SHOULD NOT.
- Suggested fix: "other tools SHOULD NOT edit it". Applied.

### 7. The include-sources build was O(n²) with full-body clones — `crates/lib/superdev-core/src/validate/fix.rs:98`

- Severity: minor
- Category: performance
- Problem: A linear `find` per concept over the converted vec, cloning
  every body though only fragments are read.
- Failure scenario: Every `--fix` holds three copies of the knowledge
  tree and pays n scans of n entries.
- Suggested fix: A path index and borrowed `HashMap<&str, &str>`.
  Applied.

### 8. Every validate run gained a third full parse per document — `crates/lib/superdev-core/src/validate/sokf.rs:390`

- Severity: minor
- Category: performance
- Problem: `check_include_blocks` parsed every body though most carry
  no marker; the fragment body re-parsed once per includer.
- Failure scenario: The PostToolUse hook pays the parse on every edit
  of every document.
- Suggested fix: A substring gate before the parse, in check and fix.
  Applied; the per-source memoization was left out as not yet worth its
  machinery on a corpus this size.

### 9. A fragment travelled as a DocSchema with a slashed name — `crates/lib/superdev-core/src/content/layout.rs:95`

- Severity: nit
- Category: simplification
- Problem: The path separator inside an item name was an untyped
  convention every consumer had to silently know, and the "six
  positions" comment went stale.
- Failure scenario: A future consumer treating DocSchema names as flat
  identifiers mishandles fragments with nothing to say so.
- Suggested fix: A dedicated `ItemKind::Fragment`. Applied, with its
  own materialization arm, layout test, and contract-007 row.

### 10. The schema-check exclusion stacked path substrings — `crates/lib/superdev-core/src/validate/mod.rs:124`

- Severity: nit
- Category: simplification
- Problem: A `fragments/` carve-out on substring matching when
  `concept.kind` answers the question semantically.
- Failure scenario: Any concept whose path passes through a `schemas/`
  directory is silently dropped from schema checking.
- Suggested fix: `concept.kind == "Schema"`. Applied.

## Not findings (checked and fine)

- Blockquote and indented include markers were probed and behave as the
  parser intends: an example stays an example.
- The links-block adjacency to an include block was probed clean: the
  definition block regenerates below a trailing include correctly.
- `concept.kind` is available at the exclusion site; the semantic test
  loses no current behaviour on the live tree.
