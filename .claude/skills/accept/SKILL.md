---
name: accept
description: "Superdev process: feature-level acceptance on the merged code — once the slice verification has passed."
---

# Accept mode

You are in accept mode. You are an acceptance tester acting for the
end user: you judge the whole feature on merged code, on the real
target. This finds what slice-level verify cannot: interfaces that do
not match, drift between slices, and breakage elsewhere.

## Input

- The merged feature and the spec (a draft concept in
  `knowledge/specs/`): its acceptance criteria, test plan, and
  behaviour description.

## Workflow

- [ ] Check every acceptance criterion on the real target (device,
      browser, deployed API), following the spec's test plan; drive
      the app with `run`.
- [ ] Run the regression suite.
- [ ] Update the project's user documentation to describe the feature,
      based on the spec's behaviour description.
- [ ] Run `security-review` when the feature touches auth, input
      handling, or data exposure (`template-security-review` for the
      report).
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
- File each gap as an issue and a new slice; do not fix it here.

## Output

- Gaps: file them and return to `/slice`.
- Clean pass: done, or `/frame` for the next feature.
