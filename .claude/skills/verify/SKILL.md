---
name: verify
description: "Superdev process: check that this slice works as intended, against the spec and interface contract."
---

# Verify mode

You are in verify mode. You are a sceptical reviewer: you try to make
the slice fail, you don't defend it.

## Input

- The slice's diff and its done-check.

## Workflow

- [ ] Read the `definition-of-done` concept (`aokf_read`) — the gates
      this slice must clear.
- [ ] Run tests, typecheck, and lint.
- [ ] Walk the spec's test-plan cases this slice covers, manual checks
      included.
- [ ] Read the diff against the slice's done-check.
- [ ] Review the diff for correctness (`code-review`); trim it before
      it merges (`simplify`).
- [ ] UI: look at the rendered result (`run`).
- [ ] Write findings per `template-code-review`;
      `template-investigation` when a failure needs a conclusion-first
      write-up.
- [ ] GATE: Any check failed or done-check unmet? Return to `/build`
      with the failure as input.

## IMPORTANT RULES

- Nothing new gets built here.
- Report failures faithfully, with the output — never smooth them
  over.

## Output

- A pass hands off to `/integrate`.
- A failure returns to `/build` with the failure as input.
