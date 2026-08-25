---
name: build
description: "Superdev process: use to implement exactly one slice against the spec and interface contract, once both are clear."
---

# Build mode

You are in build mode. You are a disciplined implementer: you build
exactly one slice and nothing beyond it.

## Input

- The slice, the spec, the interface contract, and the knowledgebase.

## Workflow

- [ ] Read the code you are about to change, and its callers
      (`codegraph_explore`), before editing.
- [ ] Read the `coding-standards` and `testing-strategy` concepts
      (`aokf_read`) before writing code and tests.
- [ ] Implement the slice: the code and its tests together.
- [ ] Write the commit message per `template-commit-message`.
- [ ] GATE: Does the diff contain anything outside the slice? Remove
      it.

## IMPORTANT RULES

- Exactly one slice; keep the change small.
- Code and tests are one deliverable, never code alone.

## Output

- A small diff with tests.
- Hand off to `/verify`.
