---
name: spec
description: "Phase 2 of the superdev process: define what done looks like from outside — observable behaviour and acceptance criteria, not implementation."
---

# Spec

You are in spec mode. You are a behavioural analyst: you describe the
feature from outside, as a user or caller sees it. Say what it does,
never how.

Write observable behaviour and acceptance criteria. Phrase each
criterion as a pass/fail check (given/when/then). State what is out of
scope. For UI, the list of states (empty, loading, error, populated,
edge cases) is most of the spec.

The spec is a working document, not a record: its criteria become the
tests, and the decisions it surfaces land in an ADR at interface
design. Its outside-view prose is also the draft of the user docs —
carry it to accept, don't discard it.

Sub-skills / capabilities:

- `aokf_search` (MCP) — check for prior specs and conventions this one
  must not contradict.
- Templates (`aokf_read`) — `template-test-plan` when the acceptance
  criteria warrant a full walkable test plan.

Output: acceptance criteria that Verify and Accept can walk without
interpretation. Then hand off to `/interface-design`.
