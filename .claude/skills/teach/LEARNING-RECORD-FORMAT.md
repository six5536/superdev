# Learning Record Format

Learning records are concepts under `knowledge/learning-records/`, named `Lnnn-<slug>.md` (`type: LearningRecord`, `id: lr-<slug>`). Create the directory lazily — only when the first record is written — and list each new record in the bundle's `index.md`.

They are the teaching equivalent of Decision concepts: they capture non-obvious lessons, key insights, and stated prior knowledge that will steer future sessions. They are used to calculate the zone of proximal development.

## Template

```md
---
type: LearningRecord
id: lr-{slug}
title: {Short title of what was learned or established}
description: {One line: the insight.}
---

{1-3 sentences: what was learned (or what prior knowledge was established), and why it matters for future sessions.}
```

That is the whole format. A learning record can be a single paragraph. The value is recording _that_ this is now known and _why_ it changes what to teach next — not in filling out sections.

## Optional sections

Only include these when they add genuine value. Most records won't need them.

- **Supersession links** — when an earlier understanding is replaced, the new record declares `supersedes` (with the mirroring body link) and the old one gets `status: deprecated`.
- **Evidence** — how the user demonstrated the understanding (a question answered, an exercise completed, prior experience cited). Useful when the claim might be revisited.
- **Implications** — what this unlocks or rules out for future sessions. Worth recording when non-obvious.

## Numbering

Scan `knowledge/learning-records/` for the highest existing number and increment by one. The `id` never changes, even when the file is renamed.

## When to write a learning record

Write one when any of these is true:

1. **The user demonstrated genuine understanding of something non-trivial** — not just exposure, but evidence they can use the concept correctly. This sets a new floor for what to teach next.
2. **The user disclosed prior knowledge** — "I already know X." Record it so future sessions don't re-teach it. Also record the _depth_ claimed.
3. **A misconception was corrected** — the user previously believed something wrong and now sees why. These are high-value: they predict future stumbling blocks for related topics.
4. **The mission shifted in response to learning** — the user discovered they cared about something different than they thought. Link to the mission concept and update it.

### What does _not_ qualify

- Material that was merely covered. Coverage is not learning. Wait for evidence.
- Anything already captured tersely in the glossary concept as a term definition. Don't duplicate.
- Session-by-session activity logs. Learning records are not a journal — they are decision-grade insights.

## Supersession

When a later record contradicts an earlier one (the user's understanding deepened or corrected), the new record declares a `supersedes` link and the old one gets `status: deprecated` rather than being deleted. The history of how understanding evolved is itself useful signal.
