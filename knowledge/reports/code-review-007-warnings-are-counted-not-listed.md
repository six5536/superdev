---
type: CodeReview
id: code-review-007-warnings-are-counted-not-listed
title: Code review — a warning is counted by default and listed on request
description: Review of plan-023's three slices on `feature/decidable-findings-are-errors`; one duplication to fold, one brittle assertion, one import style.
---

# Code review: plan-023 on `feature/decidable-findings-are-errors` (b21116c..b8c9946)

## Verdict

Correct and well covered; four minor findings, none of which changes
behaviour — one duplicated rule to fold, two assertions that can fail for
the wrong reason, and one import.

## Findings

### 1. The listing rule is written twice — crates/lib/superdev-core/src/validate/sokf.rs:145

- Severity: minor
- Category: simplification
- Problem: `to_json` filters with `f.fatal || warnings == Warnings::Listed`
  and `render_human` skips with `!finding.fatal && warnings ==
  Warnings::Counted`. Two spellings of one rule, in one `impl`, which is
  the duplication the core principles refuse.
- Failure scenario: a third listing state — say a severity that lists only
  under `--json` — is added to `Warnings`. One of the two sites is updated
  and the other keeps the old reading, so the text and JSON runs disagree
  about a finding while both compile and both pass `passed()`.
- Suggested fix: one predicate, called from both.

  ```rust
  impl Warnings {
      /// Whether a report rendered this way lists `finding`.
      fn lists(self, finding: &Finding) -> bool {
          finding.fatal || self == Self::Listed
      }
  }
  ```

### 2. A substring check on the warning count — crates/app/superdev/tests/cli.rs:118

- Severity: minor
- Category: test-coverage
- Problem: `a_bare_run_counts_the_warnings_and_the_flag_lists_them` asserts
  the summary does not contain `"0 warning(s)"`, meaning to say the count
  is not zero.
- Failure scenario: the repository grows to ten warnings. The summary reads
  `PASS (0 error(s), 10 warning(s))`, which contains `0 warning(s)`, and
  the test fails while the behaviour it checks is correct.
- Suggested fix: parse the count out of the summary and compare it to zero,
  as the JSON test already does with `as_u64`.

### 3. A fully qualified path where the file imports — crates/app/superdev/src/run.rs:326

- Severity: nit
- Category: simplification
- Problem: `run.rs` writes
  `superdev_core::validate::sokf::Warnings::Counted` inline;
  `validate_cli.rs` imports `Warnings` and writes `Warnings::Counted`. The
  coding standards say to import a type directly.
- Failure scenario: none — style only.
- Suggested fix: `use superdev_core::validate::sokf::Warnings;` at the top
  of `run.rs`.

### 4. Three tests rest on this repository still carrying a warning — crates/app/superdev/tests/cli.rs:98

- Severity: minor
- Category: test-coverage
- Problem: `a_bare_run_counts_the_warnings_and_the_flag_lists_them` and
  `validate_json_states_both_counts_and_lists_what_the_text_run_listed` run
  against `REPO_ROOT` and assert a warning exists, and the parity test
  asserts the live findings list is not empty. The five warnings this
  repository carries are skill-frontmatter keys — exactly the kind someone
  eventually fixes.
- Failure scenario: the four skills drop the keys outside the Agent Skills
  spec. The repository now reports zero warnings, and three tests fail
  while the behaviour they check is still correct.
- Suggested fix: run the two listing tests against a fixture knowledge that
  deterministically carries one warning, and drop the non-empty assertion
  from the parity test, which binds bare-against-named equality and holds
  on two empty lists.
- Raised by the Copilot review on PR #11.

## Not findings (checked and fine)

- `warnings()` computes `findings.len() - errors()`; `errors()` counts a
  subset of `findings`, so the subtraction cannot underflow.
- `listing(args)` is evaluated in both branches of `run_validate`, which
  are mutually exclusive.
- The goldens moved by exactly two keys per file; no finding, severity or
  verdict changed.
- `passed()` now reads `errors() == 0`, which is the same predicate as the
  `any(fatal)` it replaced.
- The two hooks keep their verdicts: the PostToolUse hook still tests
  `f.fatal && !f.needs_the_whole_tree()`, and the Stop hook still holds on
  `!passed()`.

## Notes

- The `json` block of `contract-002` is still bound by no test
  ([I043][sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]);
  the manual key comparison at this integrate is what stands in for it.

<!-- sokf:links -->
[sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/open/issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test.md
