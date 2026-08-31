---
type: Schema
id: schema-code-review
title: Code Review Schema
description: Code review findings — verdict first, findings ranked by severity with concrete failure scenarios.
---

# Code Review Schema

Structural rules for code-review findings documents, filed at
`knowledge/reports/code-review-{nnn}-{slug}.md`, listed in that
directory's index and selected by frontmatter `type`.

````yaml
description: >
  Code review findings: verdict first, findings ranked by severity with
  concrete failure scenarios, and what was checked and found fine.
line-limit: 800

frontmatter:
  type:
    const: CodeReview
  id:
    pattern: '^code-review-\d{3}-[a-z0-9-]+$'

sections-ordered: true
sections:
  - heading-pattern: '^Code review: .+$'
    level: 1
    required: true
    description: >
      The document title, naming the branch, PR number, or path reviewed.
  - heading: "Verdict"
    level: 2
    required: true
    content: prose
    description: >
      One sentence: overall state — e.g. "Solid change; two
      correctness issues to fix before merge." Findings ranked
      most-severe first.
  - heading: "Findings"
    level: 2
    required: true
    description: >
      The ranked findings, one level-3 heading per finding.
  - heading-pattern: '^\d+\. .+ — .+$'
    level: 3
    required: true
    repeatable: true
    content: bullet-list
    description: >
      One finding: a short claim (e.g. "Race between cache write
      and invalidation") plus `path/to/file.ts:123`. Bullets for
      Severity (critical | major | minor | nit), Category
      (correctness | security | performance | simplification |
      test-coverage), Problem (one-sentence statement of the
      defect), Failure scenario (concrete inputs/state → wrong
      output or crash; if you can't construct one, it's probably
      not a finding), and Suggested fix (the smallest change that
      resolves it; a code sketch if it helps).
  - heading: "Not findings (checked and fine)"
    level: 2
    content: bullet-list
    description: >
      Optional: things that looked suspicious but were verified OK,
      so the author doesn't re-litigate them. One line each.
  - heading: "Notes"
    level: 2
    content: bullet-list
    description: >
      Non-blocking observations: follow-up ideas, style patterns
      worth adopting later. Delete if empty.

example: |
  ---
  type: CodeReview
  id: code-review-001-pack-allowlist
  title: Code review of feature/pack-allowlist
  description: Review of the pack transport allowlist change; one correctness issue to fix before merge.
  ---

  # Code review: feature/pack-allowlist

  ## Verdict

  Solid change; one correctness issue to fix before merge.

  ## Findings

  ### 1. Scheme comparison misses uppercase — `src/pack/resolve.rs:88`

  - Severity: major
  - Category: correctness
  - Problem: The transport allowlist compares schemes case-sensitively.
  - Failure scenario: A manifest source `GIT://host/repo` bypasses the
    refusal and fetches over an unauthenticated channel.
  - Suggested fix: Lowercase the scheme before the allowlist lookup.

  ## Not findings (checked and fine)

  - The file-transport path join was checked for traversal; it
    normalises before joining.
````
