---
type: Template
id: template-bug-report
title: Bug Report Template
description: Symptom, environment, exact repro steps, expected vs actual, root cause, and regression risk. One of the three shapes the issue tracker holds.
status: stable
---

---
type: BugReport
id: issue-nnn-<slug>
title: <one-line symptom>
description: <one line — the symptom>.
status: draft
tags: [needs-triage]
---

# Bug: <one-line symptom, e.g. "Sync fails with ETIMEDOUT on large payloads">

## Summary

<One or two sentences: what is broken and the impact — who hits it, how often, how bad.>

## Environment

- Version/commit: <e.g. v0.1.0 / 923afbc>
- Platform: <OS, runtime version, relevant config>

## Steps to reproduce

1. <Exact step>
2. <Exact step>
3. <Exact step — commands verbatim so anyone can rerun them>

## Expected behavior

<What should happen.>

## Actual behavior

<What happens instead. Paste the exact error output/logs in a code block, trimmed to the relevant lines.>

```
<error output>
```

## Root cause (if known)

<Where the defect lives (`path/to/file.ts:123`) and the mechanism: the specific input/state that takes the code down the wrong path. If not yet known, state the leading hypothesis and what would confirm it.>

## Proposed fix / workaround

- Fix: <the change that removes the defect>
- Workaround: <how users can avoid it meanwhile, if any>

## Regression risk

<What else touches this code path; which tests would catch a recurrence.>

---

Notes on usage (not part of the document):

- File as `knowledge/issues/issue-<nnn>-bug-<slug>.md`, numbered after the
  highest existing issue. Declare the feature, when there is one, with
  an `implements` or `references` link to its spec.
- The `issue-tracker` concept holds the triage labels and lifecycle:
  the role tag rides in `tags`, and a resolved issue stays, retagged
  `done` or `wontfix`.
