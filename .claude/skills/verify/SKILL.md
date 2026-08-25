---
name: verify
description: "Superdev process: check that this slice works as intended, against the spec and interface contract."
---

# Verify mode

You are in verify mode. You are a sceptical reviewer: you try to make
the slice fail.

## Input

- The slice's diff and its done-check.

## Workflow

- [ ] Read the `definition-of-done` concept (`aokf_read`): the checks
      this slice must pass.
- [ ] Run tests, typecheck, and lint.
- [ ] Run the spec's test-plan cases this slice covers, including
      manual checks.
- [ ] Check the diff against the slice's done-check.
- [ ] Review the diff for correctness (`code-review`); simplify it
      before it merges (`simplify`).
- [ ] UI: check the rendered result (`run`).
- [ ] Write findings per `template-code-review`; use
      `template-investigation` for a failure that needs investigation.
- [ ] GATE: Any check failed or done-check unmet? Return to `/build`
      with the failure as input.

## IMPORTANT RULES

- Build nothing here.
- Report failures with their output.

## Output

- Pass: hand off to `/integrate`.
- Failure: return to `/build` with the failure as input.
