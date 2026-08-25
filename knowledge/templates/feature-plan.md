---
type: Template
id: template-feature-plan
title: Feature Plan Template
description: The feature's slice list — per slice a done-check, the assigned test-plan cases, and a done marker. Produced by the feature-plan phase; read by build, verify, and integrate.
status: stable
---

---
type: Plan
id: feature-plan-<slug>
title: <feature title> — feature plan
description: <one line — the feature this plan delivers>.
status: draft
---

# Feature plan: <feature title>

Spec: <link to the spec at knowledge/specs/Snnn-<feature-slug>.md>

## Slices

<Ordered by dependency first, then risk: riskiest early.>

### Slice 1: <name>

- [ ] Done — ticked by integrate at merge.
- Change: <what this slice changes, and where>
- Done-check: <the pass/fail check verify runs against this slice>
- Cases: <the test-plan case numbers assigned to this slice; an
  integration case belongs to the slice that completes its boundary>

### Slice 2: <name>

- [ ] Done — ticked by integrate at merge.
- Change: <...>
- Done-check: <...>
- Cases: <...>

---

Notes on usage (not part of the document):

- File as `knowledge/plans/Pnnn-<slug>.md`, numbered after the highest
  existing plan; id `feature-plan-<slug>`.
- List it in `knowledge/plans/index.md`.
- Every test-plan case in the spec appears in exactly one slice's
  Cases line.
- Integrate ticks each slice's Done at merge and tags the concept
  `done` after the last slice.
- For one-off work outside the feature workflow, use
  `template-adhoc-plan`.
