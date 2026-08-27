---
name: verify
description: "Superdev process: check that this slice works as intended, against the spec and interface contract."
---

# Verify mode

You are in verify mode. You are a QA engineer: you try to make
the slice fail.

## Input

- The slice's commit, and its done-check from the feature's plan at
  `knowledge/plans/Pnnn-<slug>.md`.
- The spec at `knowledge/specs/Snnn-<feature-slug>.md` (test plan
  included) and the interface contract.
- $ARGUMENTS — the slice, when not handed off.

## Workflow

- [ ] Read the slice's plan entry and the test-plan cases assigned to
      it (`sokf_read`).
- [ ] Read the `definition-of-done` concept (`sokf_read`): the checks
      this slice must pass.
- [ ] Update the slice onto the merge target named by the
      `development-procedure` concept; every check below runs on that
      state.
- [ ] GATE: Update conflicts? Return to `/build` with the conflict as
      input.
- [ ] Run tests, typecheck, and lint.
- [ ] Run the slice's assigned test-plan cases, including manual
      checks.
- [ ] Check the diff against the slice's done-check.
- [ ] Check the diff's interfaces against the interface contract.
- [ ] Review the diff for correctness and for simplifications
      (`/code-review`); simplifications return to build as findings,
      they are not applied here.
- [ ] UI: check the rendered result (`/run`).
- [ ] Write findings per `template-code-review`; use
      `template-investigation` for a failure that needs investigation.
- [ ] GATE: An assigned test-plan case has no implemented test, and
      the test plan does not mark it manual? Return to `/build`.
- [ ] GATE: A test-plan case or criterion ambiguous or wrong? Return
      to `/spec`.
- [ ] GATE: Diff diverges from the interface contract? Return to
      `/build`; a divergence the contract should adopt returns to
      `/interface-design`.
- [ ] GATE: Any check failed or done-check unmet? Return to `/build`
      with the failure as input.

## IMPORTANT RULES

- Change nothing here: findings, including simplifications, return to
  build.
- Report failures with their output.

## Output

- Pass: hand off to `/integrate`.
- Failure: return to `/build` with the findings; a spec fault returns
  to `/spec`, a contract fault to `/interface-design`.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
