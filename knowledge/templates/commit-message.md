---
type: Template
id: template-commit-message
title: Commit Message Template
description: Conventional-commit shape — typed summary line, why-not-what body, and breaking-change footer.
status: stable
---

<type>(<scope>): <imperative summary, ≤72 chars, no trailing period>

<Optional body: why the change was made, not what the diff shows.
Wrap at ~72 chars. Mention behavior changes, trade-offs, and anything
a future `git log` reader needs that the diff can't tell them.>

<Optional footer:>
BREAKING CHANGE: <what breaks and how to migrate>
Fixes #<issue>

---

Notes on usage (not part of the message):
- type: feat | fix | docs | style | refactor | perf | test | chore | build | ci
- scope: the module/package touched, e.g. feat(core), fix(npm)
- Follow the repository's existing convention if it differs from this.
- One logical change per commit; if the body needs "and also", split it.
