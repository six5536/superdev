---
type: CodeReview
id: code-review-005-normative-shape-enforcement
title: Code review of feature/normative-shape-enforcement
description: Feature-wide review of plan-020 — fifteen findings, two item-parser defects and an untested guard proved by mutation among them, all applied.
---

# Code review: feature/normative-shape-enforcement

## Verdict

Sound mechanism; two correctness defects in the item parser — one an
open escape from every `item-pattern` — a guard with no test that
mutation proved, and four sweep edits that changed what a contract
binds. All applied before merge.

## Findings

### 1. An indented list escaped every item-pattern — `crates/lib/superdev-core/src/validate/schema/document.rs:770`

- Severity: major
- Category: correctness
- Problem: A top-level item was a marker at column zero, so a list
  indented one to three spaces, or a tab, yielded no items at all. The
  content-kind check accepts such a list, so the section read as
  checked and was unbound.
- Failure scenario: An author indents a promise list two spaces and
  `superdev validate` reports PASS on a contract that binds nothing —
  the whole of ADR-032, sidestepped in silence.
- Suggested fix: Take the shallowest marker in the body as the list's
  top level. Applied, with a regression test over four leading forms.

### 2. An item's text stopped dead at a nested list — `document.rs:781`

- Severity: major
- Category: correctness
- Problem: A nested item closed the parent's text and nothing reopened
  it, so every later line of the parent was dropped. The code was
  stricter than the contract it implements, which excludes the nested
  item and not the remainder.
- Failure scenario: A promise list whose items carry sub-bullets fails
  on legal markdown, with the keyword sitting in the paragraph after
  the sub-list.
- Suggested fix: Resume the parent at the first line no deeper than the
  nested marker. Applied, with a regression test.

### 3. The mis-declaration guard had no test — `document.rs:286`, `crates/lib/superdev-core/src/validate/schema/grammar.rs:618`

- Severity: major
- Category: test-coverage
- Problem: Mutation proved two claims unpinned: making
  `Requirement::admits` return `true` unconditionally, and removing the
  fenced-line skip from `items_in`, each left the whole suite green.
- Failure scenario: The `requires` cross-key check stops working and
  nothing notices; an `item-pattern` on a prose section binds nothing
  with no finding.
- Suggested fix: Unit tests for `Requirement::admits` over both
  variants, hit, miss and absent; a fenced-bullet case in the item
  tests; a `check_declarations` case for an item-pattern beside a
  non-list kind. Applied — each verified to fail against its mutation.

### 4. A sweep edit added a binding contract-002 never made — `knowledge/contracts/public/active/contract-002-cli-superdev.md:236`

- Severity: major
- Category: correctness
- Problem: "this command MUST NOT add to it" was invented so the item
  would carry a keyword. The slice's own done-check forbids changing
  what a contract binds.
- Failure scenario: A reader takes a prohibition nobody decided as
  binding on the CLI.
- Suggested fix: Restate what the command does instead. Applied:
  "`mcp sokf` MUST serve the canonical knowledge over stdio".

### 5. A sweep edit weakened a format promise — `knowledge/contracts/public/active/contract-005-file-format-pack.md:82`

- Severity: minor
- Category: correctness
- Problem: "is added without a bump" became "MAY be added without a
  bump", which under RFC 2119 makes the bump optional rather than
  forbidden.
- Failure scenario: A release bumps `format` for a
  backward-compatible addition and breaks no rule on file.
- Suggested fix: "MUST NOT require a bump". Applied.

### 6. A sweep edit turned a ceiling into a permission — `knowledge/contracts/public/active/contract-006-file-format-lock.md:56`

- Severity: minor
- Category: correctness
- Problem: "A reader may conclude two things and nothing more" is a
  prohibition; "MAY conclude" reads as an option.
- Failure scenario: A reader concludes drift from a hash, which the
  sentence exists to forbid.
- Suggested fix: "MUST NOT conclude more from a hash than two things".
  Applied.

### 7. A sweep edit stated a MUST the code did not meet — `knowledge/contracts/internal/active/contract-010-interface-document-schemas.md:145`

- Severity: minor
- Category: correctness
- Problem: The security bullet became "every schema pattern MUST
  compile through the validator's own `re` wrapper", while the path
  `validate` uses called `Regex::new` directly.
- Failure scenario: The contract is false on the day it is written.
- Suggested fix: Route the grammar's regex check through `re::compile`,
  which also memoises it; the failure path re-reads the pattern only to
  quote why. Applied.

### 8. A content-pattern read fenced blocks — `document.rs:708`

- Severity: minor
- Category: correctness
- Problem: The body was joined from raw lines, so a keyword inside a
  worked example satisfied a promise section. Every other body check
  treats a fence as non-content.
- Failure scenario: A Stability section whose only MUST sits in a code
  block passes while promising nothing.
- Suggested fix: Skip fenced lines, and say so in ADR-030 and
  contract-010, whose "raw lines" wording invited the reading. Applied.

### 9. Two findings for one fault — `document.rs:702`

- Severity: minor
- Category: simplification
- Problem: A section failing its content kind failed its
  content-pattern as well, though the columns check already has the
  precedent guard for exactly this.
- Failure scenario: An empty prose section reports twice, and a reader
  fixes one fault chasing two messages.
- Suggested fix: Skip the pattern when the kind check on the same
  section fired. Applied, with a test.

### 10. An item-pattern bound the wrong marker kind — `document.rs:769`

- Severity: minor
- Category: correctness
- Problem: Items were collected on either marker, though the contract
  and the grammar both say "the section's declared list kind".
- Failure scenario: A bullet beside a numbered list draws a finding
  against a rule that never meant to bind it.
- Suggested fix: Filter by the declared kind. Applied, with a test.

### 11. A finding on a repeatable rule could not locate itself — `document.rs:711`

- Severity: minor
- Category: correctness
- Problem: The message named the rule's pattern, not the heading that
  failed, so two occurrences were indistinguishable.
- Failure scenario: A schema pairing `heading-pattern` with a body
  pattern reports a fault the reader cannot find.
- Suggested fix: Name the occurrence's own heading. Applied, with a
  test.

### 12. A lazily continued item was truncated — `document.rs:781`

- Severity: minor
- Category: correctness
- Problem: An unindented line right under an item ended it, though
  markdown continues the item's paragraph there.
- Failure scenario: A managed repository whose contract wraps without
  indenting fails on conforming text.
- Suggested fix: Take an unindented line while the paragraph is still
  running, and close on the blank line that ends it. Applied, with a
  test covering both halves.

### 13. The changelog omitted the breaking half — `CHANGELOG.md:118`

- Severity: minor
- Category: correctness
- Problem: The vocabulary and the EARS check were recorded; the
  contract-kind declarations, which make existing contracts start
  failing after a pack update, were not.
- Failure scenario: A managed repository upgrades and reads no reason
  for its new errors.
- Suggested fix: A Changed entry naming the ADR-032 assignment and what
  to do about a flagged section. Applied.

### 14. A thematic break opened an item — `document.rs:996`

- Severity: nit
- Category: correctness
- Problem: `is_bullet` accepts `* * *`, pre-existing and made
  consequential by the item check.
- Failure scenario: A star thematic break in a bound list becomes a
  bogus item that fails any item-pattern.
- Suggested fix: Exclude thematic breaks. Applied, with a test.

### 15. Two hand-synced copies of the mis-declaration rules — `document.rs:57`

- Severity: nit
- Category: simplification
- Problem: `LIST_KINDS` and the `re::compile` guard duplicate the
  grammar's `requires` and `format: regex`; `validate` reads only the
  grammar copy.
- Failure scenario: A third list kind means two edits, and only one of
  them is enforced.
- Suggested fix: Left as it stands — the pair is two lines and the
  document-layer copy serves callers that check documents without the
  grammar pass. Recorded so the next hand touches both.

## Not findings (checked and fine)

- `check_one`'s reach: an absent or unmatched section never fires a
  pattern finding, and the example path runs both patterns with the
  `example:` prefix — verified end to end.
- One finding per mis-declaration: the grammar path reports each fault
  once and the two implementations agree on every case tried.
- The old single-value `requires` behaviour is preserved exactly by
  `Requirement::One`.
- Item boundaries on the live corpus: every item the parser finds in
  contract-002's Behaviour, contract-003's Tools and every criteria
  list was enumerated and correct.
- Shipping and performance: the pack mirror and both grammar copies are
  byte-identical, and the added work is one join and two memoised
  lookups per patterned section.
