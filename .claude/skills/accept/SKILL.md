---
name: accept
description: "Superdev process: feature-level acceptance on the merged code — once the slice verification has passed."
---

# Accept mode

You are in accept mode. You are an acceptance tester standing in for
the end user: you judge the whole feature on merged code, on the real
target. This catches what slice-level Verify structurally cannot:
seams that don't meet, drift between slices, breakage elsewhere.

## Input

- The merged feature and the spec (a draft concept in
  `knowledge/specs/`): its acceptance criteria and outside-view prose.

## Workflow

- [ ] Walk the acceptance criteria end to end on the real target
      (device, browser, deployed API) — drive it with `run`, following
      the spec's test plan.
- [ ] Run the regression suite.
- [ ] Docs walk: the project's outward-facing docs, wherever they
      live, describe the feature — seed them from the spec's
      outside-view prose.
- [ ] Run `security-review` when the feature touches auth, input
      handling, or data exposure (`template-security-review` for the
      write-up).
- [ ] File each gap found (`template-bug-report`): an Issue concept at
      `knowledge/issues/<feature-slug>/Innn-<slug>.md`.
- [ ] Shipping? Write the release notes (`template-release-notes`).
- [ ] GATE: Feature undocumented? That is a gap.
- [ ] GATE: Any gap found? It becomes a new slice.
- [ ] Clean pass? Tag the spec concept `done`.
- [ ] GATE: Bundle edited? Validate to PASS
      (`superdev aokf validate knowledge`).

## IMPORTANT RULES

- Judge merged code on the real target, never the working tree.
- A gap is filed and sliced, never quietly fixed here.

## Output

- Gaps become new slices → back to `/slice`.
- Clean pass → done, or `/frame` for the next feature.
