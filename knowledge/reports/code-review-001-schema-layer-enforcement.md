---
type: CodeReview
id: code-review-001-schema-layer-enforcement
title: Code review of feature/schema-layer-enforcement
description: Review of the schema-layer-enforcement feature diff against main; nine correctness findings and one cleanup, all resolved before the merge.
---

# Code review: feature/schema-layer-enforcement

## Verdict

Sound design with a fix pass owed before integration: nine correctness
findings — three producing false fatal findings on valid documents,
three making declared constraints silently bind nothing — and one
cleanup, every one resolved on the branch.

## Findings

### 1. A failing contract dropped silently — crates/lib/superdev-core/src/validate/schema/document.rs:157

- Severity: major
- Category: correctness
- Problem: `DocSchema::parse` swallowed serde errors, so a schema whose
  contract did not deserialize dropped out of the set with no finding.
- Failure scenario: a schema writes a non-scalar `enum` entry; the typed
  parse fails, the schema governs nothing, and its documents report only
  a misleading "type X names no schema".
- Suggested fix: `parse` returns the serde error and `SchemaSet::load`
  reports it on the schema file. Applied.

### 2. YAML comments compared as value text — crates/lib/superdev-core/src/validate/schema/document.rs:520

- Severity: major
- Category: correctness
- Problem: constraints compared against the raw scalar, which kept
  trailing `# …` comments and read full-line comments as a block, while
  dispatch used real YAML.
- Failure scenario: `type: Decision # accepted 2026` dispatches
  correctly and then draws a false fatal const mismatch; a comment line
  under a key forces "is not a scalar".
- Suggested fix: read the compared value as YAML reads it — comments
  stripped, quotes removed, comment-only blocks ignored. Applied.

### 3. Nested fences double-toggled — crates/lib/superdev-core/src/validate/schema/document.rs:625

- Severity: major
- Category: correctness
- Problem: `body_has` toggled on any three-backtick prefix, ignoring
  marker length and tilde fences, unlike `read::fence_map`.
- Failure scenario: a four-backtick example containing three-backtick
  lines flips the reading, so fence-interior lines count as content; a
  `~~~sh` block fails the `code` kind outright.
- Suggested fix: drive `body_has`, `heading_positions` and
  `table_header` from one `read::fence_map` reading. Applied.

### 4. The grammar documented the wrong default — crates/lib/superdev-core/src/validate/schema/grammar.yaml:621

- Severity: minor
- Category: correctness
- Problem: the frontmatter-constraint `required` doc said "Defaults to
  true; false marks an optional key", while ADR-022 and the code make
  absence optional.
- Failure scenario: an author omits the flag expecting
  required-by-default, and documents missing the key pass silently.
- Suggested fix: state ADR-022's semantics in both grammar copies.
  Applied.

### 5. A second frontmatter splitter — crates/lib/superdev-core/src/validate/schema/document.rs:559

- Severity: minor
- Category: correctness
- Problem: `frontmatter_block` duplicated `read::split_frontmatter` with
  divergent behaviour — CRLF documents read as having no frontmatter,
  and any line starting with `---` closed the block.
- Failure scenario: a CRLF document reports every required key absent
  while its section checks pass.
- Suggested fix: strip `\r` once in `check_one` and split through
  `read::split_frontmatter`. Applied.

### 6. lifecycle skipped wholesale — crates/lib/superdev-core/src/validate/schema/document.rs:505

- Severity: minor
- Category: correctness
- Problem: `check_frontmatter` skipped the `lifecycle` key entirely,
  while the filing check activates only on a non-empty enum.
- Failure scenario: `lifecycle: {required: true}` with no enum binds
  nothing in either layer.
- Suggested fix: skip only where the filing check owns the key — a
  declared enum; otherwise its constraints bind normally. Applied.

### 7. Columns rule blind to subsections — crates/lib/superdev-core/src/validate/schema/document.rs:702

- Severity: minor
- Category: correctness
- Problem: `table_header` stopped at the next heading of any level,
  while contract-010 counts a subsection's content.
- Failure scenario: `## Section` → lead-in → `### Sub` → table draws a
  false "carries no table" under a columns rule.
- Suggested fix: seek the table in the same level-bounded body range the
  content check reads. Applied.

### 8. Anything counted as prose — crates/lib/superdev-core/src/validate/schema/document.rs:655

- Severity: minor
- Category: correctness
- Problem: the prose arm accepted any non-empty, non-list, non-pipe line
  — deeper headings, `<!-- sokf:links -->`, `[sokf:x]:` definitions,
  dividers — so a trailing-section prose rule could never fail.
- Failure scenario: a prose section holding only the sokf links block
  passes without a paragraph line.
- Suggested fix: a paragraph line has words and is none of those forms.
  Applied.

### 9. One fault, two findings — crates/lib/superdev-core/src/validate/schema/document.rs:238

- Severity: minor
- Category: simplification
- Problem: the load-time unknown-content-kind and uncompilable-pattern
  findings duplicated what `check_schema` reports on the same file in
  the same `validate` run.
- Failure scenario: a schema with `content: essay` earns two fatal
  findings for one fault.
- Suggested fix: `validate` reports through the grammar's schema check;
  `check_declarations` carries the document-layer report for callers
  without that pass. Applied.

### 10. The content vocabulary in three copies — crates/lib/superdev-core/src/validate/schema/document.rs:53

- Severity: nit
- Category: simplification
- Problem: `CONTENT_KINDS` restates the grammar's section-content enum,
  and the finding message restated it a third time.
- Failure scenario: a kind added to the grammar desynchronises the
  copies — accepted by one check, rejected by the other.
- Suggested fix: the finding message derives from `CONTENT_KINDS`; the
  grammar's enum remains the authority the shipped flow checks against.
  Applied for the message copy; the code-side array stays, as `body_has`
  dispatches on the literal kinds.

## Not findings (checked and fine)

- `knowledge/schemas/` and `pack/knowledge/schemas/` are byte-identical,
  and all 54 shipped schemas deserialize into the typed contract.
- The repository's own knowledge passes the new checks; the golden
  fixtures match the emitted messages.
- Frontmatter `pattern` matches unanchored by design: contract-010
  places anchoring on the author.

## Notes

- All ten findings were resolved in commit 99e0364 on the feature
  branch, before the merge.
- Below the cap: contract-010's "the vocabulary's newest rows" is
  change-relative wording; README duplicates CONTRIBUTING's setup block.
