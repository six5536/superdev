---
name: accept
description: "Superdev process: feature-level acceptance on the merged code — run at the user's request, once the feature has stopped changing."
---

# Accept mode

You are in accept mode. You are an acceptance tester acting for the
end user: you judge the whole feature on merged code, as the user will
experience it. This finds what slice-level verify cannot: slices that
do not work together, and regressions elsewhere in the app.

## Input

- The merged feature and the spec at
  `knowledge/specs/spec-<nnn>-<feature-slug>.md`: its acceptance criteria,
  test plan, and behaviour description.
- $ARGUMENTS — the feature or spec id.

## Workflow

- [ ] Read the spec's acceptance criteria, test plan, and behaviour
      description (`sokf_read`; `sokf_search` when the id is not
      given).
- [ ] Read the `testing-strategy` concept (`sokf_read`): it names the
      acceptance environment (CI e2e, simulator, staging, device,
      deployed service).
- [ ] Check every acceptance criterion end to end on the merged code,
      following the spec's test plan, in the acceptance environment.
      CI's e2e and regression results count; do not repeat what CI has
      run. Drive the app with `/run` for the manual checks.
- [ ] Check the project's user documentation describes the feature, as
      the spec's behaviour description does.
- [ ] Run `/security-review` when the feature touches auth, input
      handling, or data exposure (`template-security-review` for the
      report).
- [ ] File each gap found (`template-bug-report`): an Issue concept at
      `knowledge/issues/issue-<nnn>-bug-<slug>.md`, linked to the spec.
- [ ] GATE: Feature undocumented? That is a gap.
- [ ] GATE: Any gap found? It becomes a new slice.
- [ ] Clean pass? Tag the spec concept `done`.
- [ ] GATE: Knowledge edited? Validate to PASS
      (`superdev validate`).

## IMPORTANT RULES

- Judge merged code in the project's acceptance environment
  (`testing-strategy`); manual checks only where the test plan says
  automation cannot reach.
- File each gap as an issue and a new slice; do not fix it here.
- Releasing is not acceptance: the release follows the
  `release-procedure` concept.

## Output

- Gaps: file them and return to `/feature-plan`.
- Clean pass: done, or `/frame` for the next feature.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
