---
name: build
description: "Superdev process: use to implement exactly one slice against the spec and interface contract, once both are clear."
---

# Build mode

You are in build mode. You are a disciplined implementer: you build
exactly one slice and nothing beyond it.

## Input

- The slice: an entry in the feature's plan at
  `knowledge/plans/Pnnn-<slug>.md`.
- The spec at `knowledge/specs/Snnn-<feature-slug>.md`, the interface
  contract, and the knowledgebase.
- $ARGUMENTS — the slice, when not handed off.

## Workflow

- [ ] Read the slice's plan entry and the spec's criteria and
      test-plan cases it covers (`aokf_read`).
- [ ] Read the code you are about to change, and its callers
      (`codegraph_explore`), before editing.
- [ ] Read the `coding-standards` and `testing-strategy` concepts
      (`aokf_read`) before writing code and tests; for a UI slice,
      also `visual-system`.
- [ ] Implement the slice using TDD, against the test-plan cases this
      slice covers. Write tests with the code only where TDD is
      impractical, e.g. exploratory UI work.
- [ ] GATE: Does the implementation need a contract change? Return to
      `/interface-design`; do not diverge from the contract.
- [ ] GATE: Is the slice too big to build in one pass? Return to
      `/plan`.
- [ ] Run the tests you wrote and the affected existing tests; fix
      failures before handing off.
- [ ] GATE: Does the diff contain anything outside the slice? Remove
      it.
- [ ] Commit the slice; write the commit message per
      `template-commit-message`.

## IMPORTANT RULES

- Exactly one slice; keep the change small.
- Code and tests are one deliverable, never code alone.
- The contract binds: a change it cannot support is an interface
  change, decided at interface-design, not here.

## Output

- A small, committed diff with passing tests.
- Hand off to `/verify`.
- Contract change needed: return to `/interface-design`. Slice too
  big: return to `/plan`.
