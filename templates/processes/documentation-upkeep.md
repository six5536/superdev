# Process: Documentation upkeep

Docs work is part of the change that invalidates them, not a separate task for later.

## 1. Sweep for invalidated docs on every change

- After a behavioral change, search for what the change made stale: README examples, CLAUDE.md commands and architecture notes, doc comments on changed functions, config option tables, CHANGELOG.
- Error messages and CLI help text are documentation too — update them with the behavior they describe.

## 2. Update in the same change

- Ship the doc fix in the same commit/PR as the code change so they can never drift apart in history.
- Verify examples by running them — a copy-pasted example that no longer works is worse than no example.

## 3. Fix or flag, never ignore

- Docs found already-stale while working: fix in passing if it's a line or two; otherwise note it to the user rather than silently leaving known-wrong docs.
- When code and docs disagree, the code is the fact — but the disagreement itself is a finding to surface, since the docs may record the *intended* behavior.

## 4. Write for the reader the doc serves

- README: a newcomer evaluating or starting out. CLAUDE.md: an agent needing commands and non-obvious structure. Doc comments: the caller, stating contracts (inputs, outputs, errors, invariants) — not narrating the implementation.
- Delete rather than accumulate: outdated sections, docs for removed features, commented-out examples. Wrong docs cost more than missing ones.

## 5. Report

- List doc updates alongside code changes in the summary; call out any stale docs found and deliberately left for a follow-up.
