---
type: Template
id: template-code-review
title: Code Review Template
description: Verdict first, findings ranked by severity with concrete failure scenarios, and what was checked and found fine.
status: stable
---

# Code review: <branch / PR # / path reviewed>

## Verdict

<One sentence: overall state — e.g. "Solid change; two correctness issues to fix before merge." Findings ranked most-severe first.>

## Findings

### 1. <Short claim, e.g. "Race between cache write and invalidation"> — `path/to/file.ts:123`

- Severity: critical | major | minor | nit
- Category: correctness | security | performance | simplification | test-coverage
- Problem: <one-sentence statement of the defect>
- Failure scenario: <concrete inputs/state → wrong output or crash. If you can't construct one, it's probably not a finding.>
- Suggested fix: <the smallest change that resolves it; a code sketch if it helps>

### 2. <...>

## Not findings (checked and fine)

<Optional: things that looked suspicious but were verified OK, so the author doesn't re-litigate them. One line each.>

## Notes

- <Non-blocking observations: follow-up ideas, style patterns worth adopting later. Delete if empty.>
