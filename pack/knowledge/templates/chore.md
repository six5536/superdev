---
type: Template
id: template-chore
title: Chore Template
description: The surfaces scoped mechanical work reaches and the check that says it is done. One of the three shapes the issue tracker holds.
status: stable
---

---
type: Chore
id: issue-nnn-<slug>
title: <one line — the work>
description: <one line — what changes, and why it is worth doing>.
status: draft
tags: [needs-triage]
---

# Chore: <the work>

## Summary

<One or two sentences: what changes, and why it is worth doing. No
symptom — nothing is broken, or this would be a bug report.>

## Surfaces

- <Where the work reaches> — <the count or the command that measured it>.
- <Where the work reaches> — <the count or the command that measured it>.

## Definition of done

- <A command, with the result that counts as a pass.>
- <Something checkable by someone who did not do the work.>

---

Notes on usage (not part of the document):

- File as `knowledge/issues/issue-<nnn>-chore-<slug>.md`, numbered after the
  highest existing issue. Declare the feature, when there is one, with
  an `implements` or `references` link to its spec.
- The `issue-tracker` concept holds the triage labels and lifecycle:
  the role tag rides in `tags`, and a resolved issue stays, retagged
  `done` or `wontfix`.
