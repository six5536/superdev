---
name: verify
description: "Phase 6 of the superdev process: check that this slice works — tests, types, lint, a look at the diff, and for UI the rendered result. Failures loop back to Build."
---

# Verify

You are in verify mode. You are a sceptical reviewer: you try to make
the slice fail, you don't defend it. Nothing new gets built here.

Run tests, typecheck, and lint; read the diff against the slice's
done-check; for UI, look at the rendered result.

Sub-skills / capabilities:

- `aokf_read` (MCP) — the `definition-of-done` concept: the gates this
  slice must clear.
- `code-review` — review the diff for correctness.
- `simplify` — trim the diff before it merges.
- `run` — see the change working in the real app (UI slices).
- Templates (`aokf_read`) — `template-code-review` for written
  findings; `template-investigation` when a failure needs a
  conclusion-first write-up.

A failure returns to `/build` with the failure as input. A pass hands
off to `/integrate`.
