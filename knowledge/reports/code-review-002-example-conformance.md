---
type: CodeReview
id: code-review-002-example-conformance
title: Code review of feature/example-conformance
description: Review of the example-conformance feature diff against main; five correctness findings, three probe-confirmed, all resolved before the merge.
---

# Code review: feature/example-conformance

## Verdict

Sound design with a fix pass owed before integration: five correctness
findings — one making a declared constraint bind nothing, two letting
the deprecated link forms through, one producing false findings on
valid documents — every one resolved on the branch.

## Findings

### 1. An example's lifecycle bound nothing — crates/lib/superdev-core/src/validate/schema/document.rs:611

- Severity: major
- Category: correctness
- Problem: `check_frontmatter` defers `lifecycle` to the filing check
  (P011), but the filing check reads only real, filed documents — never
  an example — so the deferral left the key unread in the example check.
- Failure scenario: a schema's example writes `lifecycle: bananas`, or
  omits a required `lifecycle`, and validate passes silently
  (probe-confirmed against the ADR schema).
- Suggested fix: the deferral holds for real documents only; inside an
  example, `lifecycle` constraints bind like any other key's. Applied.

### 2. Dot-segmented knowledge paths escaped — crates/lib/superdev-core/src/validate/schema/document.rs:395

- Severity: minor
- Category: correctness
- Problem: the link-form check caught `knowledge/...` and
  `/knowledge/...` targets and missed the same path behind leading `./`
  or `../` segments.
- Failure scenario: an example writes `[plan](../knowledge/plans/x.md)`
  and the deprecated path form passes unreported (probe-confirmed).
- Suggested fix: strip leading `/`, `./` and `../` segments before the
  prefix test, in one `into_knowledge` helper. Applied.

### 3. A frontmatter comment read as a heading — crates/lib/superdev-core/src/validate/schema/document.rs:477

- Severity: minor
- Category: correctness
- Problem: `check_one` scanned the whole document for headings, so a
  YAML frontmatter comment line opening with `#` counted as one.
  Pre-existing, and newly load-bearing once examples run through the
  same check.
- Failure scenario: a frontmatter comment `# Context` satisfies a
  required "Context" section, or a comment naming a prohibited section
  draws a false fatal finding (probe-confirmed).
- Suggested fix: mask the frontmatter block alongside the fenced lines
  before the heading scan. Applied.

### 4. An inline sokf destination passed — crates/lib/superdev-core/src/validate/schema/document.rs:392

- Severity: minor
- Category: correctness
- Problem: `[text](sokf:<id>)` — the concept-link form misspelled — was
  read as an ordinary path outside the knowledge and passed.
- Failure scenario: an example teaches the inline spelling, and agents
  copy a form no consumer resolves.
- Suggested fix: a `sokf:` destination is its own finding, naming the
  `[text][sokf:<id>]` reference form. Applied.

### 5. An unreferenced definition escaped — crates/lib/superdev-core/src/validate/schema/document.rs:388

- Severity: minor
- Category: correctness
- Problem: the markdown parser emits no event for a link definition
  nothing references, so a stray definition naming a knowledge path was
  never seen.
- Failure scenario: an example's definition block carries
  `[stray]: /knowledge/x.md` with no citing link, and the deprecated
  form passes.
- Suggested fix: read definition lines directly, outside fences — which
  also gives reference links one finding at their definition instead of
  two — exempting the `sokf:`-labelled definitions of the generated
  block, whose knowledge paths are the accepted form's own plumbing.
  Applied.

## Not findings (checked and fine)

- The live schemas in `knowledge/schemas/` and the pack mirror already
  conform: `superdev validate` reports PASS on this repository with all
  five fixes live, and the two trees are byte-identical.
- A fictional `sokf:` label resolves nothing and passes, with and
  without a definition block, as ADR-025 requires.
- An image keeps its ordinary markdown form: it names a picture, never
  a concept, so the id form is not asked of one — matching the real
  document check's exemption.

## Notes

- All five findings were resolved in commit f5618ce on the feature
  branch, before the merge, each with a regression test that fails on
  the unfixed code.
- Finding 3's fix also corrects real-document checking, where the same
  false findings were reachable before this feature.
