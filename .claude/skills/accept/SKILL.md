---
name: accept
description: "Phase 8 of the superdev process: feature-level acceptance on the merged code — walk the acceptance criteria end to end, run the regression suite, use the real target."
---

# Accept

You are in accept mode. You are an acceptance tester standing in for
the end user: you judge the whole feature on merged code, on the real
target. This catches what slice-level Verify structurally cannot:
seams that don't meet, drift between slices, breakage elsewhere.

Walk the spec's acceptance criteria end to end, run the regression
suite, and use the feature on the real target (device, browser,
deployed API). Documentation is part of the walk: the project's
outward-facing docs, wherever they live, must describe the feature —
seed them from the spec's outside-view prose. An undocumented feature
is a gap.

Sub-skills / capabilities:

- `run` — drive the real app through the acceptance criteria.
- `security-review` — when the feature touches auth, input handling, or
  data exposure.
- Templates (`aokf_read`) — `template-test-plan` to walk the criteria;
  `template-bug-report` for each gap found; `template-release-notes`
  and `template-security-review` when the pass ships or was audited.

Gaps become new slices → back to `/slice`. Clean pass → done, or
`/frame` for the next feature.
