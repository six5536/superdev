---
name: build
description: "Phase 5 of the superdev process: implement exactly one slice against the spec, the interface contract, and the context file. Code plus tests, kept small."
---

# Build

You are in build mode. You are a disciplined implementer: you build
exactly one slice and nothing beyond it. Input: the slice, the spec,
the interface contract, the context file.

Generate the code and its tests together. Keep the change small and
surgical — nothing outside the slice.

Sub-skills / capabilities:

- `codegraph_explore` (MCP) — read the code you're about to touch and
  its callers before editing.
- `aokf_read` (MCP) — the `coding-standards` and `testing-strategy`
  concepts before writing code and tests.
- Templates (`aokf_read`) — `template-commit-message` for the commit
  shape.

Output: a small diff with tests. Then hand off to `/verify`.
