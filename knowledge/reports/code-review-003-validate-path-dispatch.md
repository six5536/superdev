---
type: CodeReview
id: code-review-003-validate-path-dispatch
title: Code review of feature/validate-path-dispatch
description: Review of the validate path-dispatch feature diff against main; six confirmed correctness findings, two plausible edge findings and two reuse cleanups, all resolved before the merge.
---

# Code review: feature/validate-path-dispatch

## Verdict

Right mechanism with a rebuild owed before integration: the named
branch reconstructed the bare run's inputs piecewise, and the review
reproduced three parity breaks and a hard-fail regression against the
built binary — all ten findings resolved on the branch by running the
bare pipeline and scoping the report.

## Findings

### 1. Named runs hard-failed on unrelated broken knowledge — crates/lib/superdev-core/src/validate/mod.rs:94

- Severity: critical
- Category: correctness
- Problem: the guard change to `named || bundle.is_dir()` made
  `load_bundle` and the per-concept reads run on every named run, and
  their errors propagate past the coverage filter.
- Failure scenario: one non-UTF-8 file under `knowledge/` makes
  `validate .claude/skills/x/SKILL.md` exit 2 naming a file the user
  never named; on main the skill checks fine (probe-confirmed).
- Suggested fix: propagate knowledge read faults only when the paths
  touch the bundle; a run scoped elsewhere skips the context. Applied,
  including the out-of-scope `read(README.md)` in the same class.

### 2. The documents push over-included — crates/lib/superdev-core/src/validate/mod.rs:190

- Severity: critical
- Category: correctness
- Problem: named files joined the document candidates even where the
  bare run has no such candidate — a broken-frontmatter knowledge file,
  a typed file the dedup guard missed, a README dispatched by `type`
  instead of the bare run's glob.
- Failure scenario: a knowledge file with a duplicate frontmatter key
  gets a fatal schema finding the bare run never reports
  (probe-confirmed).
- Suggested fix: let the bare pipeline supply every candidate it has;
  the named loop adds only files the bare run never reaches. Applied.

### 3. Positive grammar claims ignored location — crates/lib/superdev-core/src/validate/mod.rs:183

- Severity: critical
- Category: correctness
- Problem: `detect_kind` claims `SKILL.md`, `*.prompt.md` and
  `superdev.md` by name wherever they sit, so a named concept with a
  unit suffix took the grammar, and a typed SKILL.md was double-checked.
- Failure scenario: `knowledge/p.prompt.md` with `type: Note` gets
  0 findings bare and 7 fatal grammar findings named — the I019 misread
  reintroduced (probe-confirmed).
- Suggested fix: apply the walk's claims only where the walk reaches —
  the roots — and give files under the bundle the knowledge's
  treatment; a positive claim keeps only files outside both. Applied.

### 4. `--fix` could not honour the findings a named run reports — crates/lib/superdev-core/src/validate/mod.rs:274

- Severity: major
- Category: correctness
- Problem: `fix_repo` kept the old coverage condition while
  `validate_repo` gained the new one, so a named run reported link
  findings whose stated remedy repairs zero files, forever.
- Failure scenario: `validate knowledge/a.md` says to run
  `validate --fix`, and `validate --fix knowledge/a.md` writes nothing
  and reports the same finding again (probe-confirmed).
- Suggested fix: cover the fix on the same condition the check reports
  knowledge findings on — a path that is the knowledge, contains it, or
  names a file inside it. Applied.

### 5. Two schema-set sources, one broken predicate — crates/lib/superdev-core/src/validate/mod.rs:141

- Severity: major
- Category: correctness
- Problem: the bare run built its schema set from the grammar-roots
  walk and the named run from `bundle.join("schemas")`, both filtered on
  the substring `/schemas/`, which fails when the bundle prefix is
  empty.
- Failure scenario: `validate --knowledge kb` exits 0 with `schemas: 0`
  while `validate --knowledge kb kb/a.md` exits 1 with `schemas: 1` — a
  direct named/bare disagreement (probe-confirmed).
- Suggested fix: one source for both runs — the knowledge's own schemas
  directory — and a bundle-relative candidate-exclusion test. Applied.

### 6. Dispatch with no schema set checked nothing — crates/lib/superdev-core/src/validate/mod.rs:183

- Severity: minor
- Category: correctness
- Problem: a typed file suppressed the fallback kind even where no
  schema set exists to resolve the type, so the run passed it clean
  with nothing validated.
- Failure scenario: in a repository without `knowledge/schemas/`,
  `validate docs-note.md` (frontmatter `type: Decision`) exits 0 with
  `files: 0, documents: 0`.
- Suggested fix: dispatch means nothing without schemas — a typed file
  outside the knowledge takes the fallback kind there, while a typed
  concept keeps the knowledge's treatment. Applied.

### 7. Coverage compared unresolved spellings — crates/lib/superdev-core/src/validate/mod.rs:245

- Severity: minor
- Category: correctness
- Problem: `covers` filters by string identity while `normalise` kept
  `..` components, so a dot-dot spelling dropped the file's SOKF and
  filing findings silently.
- Failure scenario: `validate knowledge/../knowledge/a.md` reports a
  subset of what `validate knowledge/a.md` reports.
- Suggested fix: resolve `..` lexically in `normalise`, so every
  comparison sees one spelling. Applied.

### 8. No duplicate guard on the grammar's files — crates/lib/superdev-core/src/validate/mod.rs:171

- Severity: minor
- Category: correctness
- Problem: naming a directory and a file inside it read the file twice
  and duplicated every grammar finding.
- Failure scenario: `validate .claude/skills .claude/skills/x/SKILL.md`
  reports each of the skill's findings twice and over-counts `files`.
- Suggested fix: dedupe the collected files by name, once, after every
  argument lands. Applied.

### 9. `frontmatter_type` re-implemented `fm_value` — crates/lib/superdev-core/src/validate/mod.rs:352

- Severity: nit
- Category: simplification
- Problem: the scalar-extraction semantics lived in two copies.
- Failure scenario: a change to `fm_value`'s scalar reading leaves the
  dispatch key parsed by the stale copy.
- Suggested fix: `split_frontmatter` + `fm_value(&split.fm, "type")`.
  Applied.

### 10. The candidate-exclusion predicate lived twice — crates/lib/superdev-core/src/validate/mod.rs:109

- Severity: nit
- Category: simplification
- Problem: the schemas-and-indexes exclusion was hand-copied between
  the concept loop and the named loop, held equal only by a comment.
- Failure scenario: one copy drifts and a named schema file becomes a
  document candidate on one path and not the other.
- Suggested fix: one predicate. Applied — the rebuild removed the named
  loop's copy entirely; the concept loop's bundle-relative test is the
  only one left.

## Not findings (checked and fine)

- The named-run cost of reading the whole knowledge and walking the
  roots: ADR-026 accepts it as the price of never disagreeing with the
  bare run.
- The `"no findings"` stdout assertion in the named-document CLI test
  was fragile against future warnings; tightened to the PASS summary
  line while resolving the findings.

## Notes

- Findings 2, 3 and 5 shared one root — the named branch rebuilt the
  bare run's inputs piecewise — and were resolved together: the run is
  now the bare pipeline with its report scoped by the coverage filter,
  and the named loop adds only files the bare pipeline never reaches.
- Each resolved finding carries a regression test in
  `crates/lib/superdev-core/src/validate/mod.rs` or
  `crates/lib/superdev-core/tests/fix.rs`.
