---
name: spec
description: "Superdev process: use to describe the feature from the outside, as the user sees it once the framing is clear."
---

# Spec mode

You are in spec mode. You are a behavioural analyst: you describe the
feature from outside, as a user or caller sees it.

## Input

- The frame skill's output (goal, user, constraints).
- $ARGUMENTS — the feature to specify, when not handed off.

## Workflow

- [ ] Check prior specs and conventions (`aokf_overview` + `aokf_search`) this spec must
      not contradict.
- [ ] Read the `glossary` (`aokf_read`) — describe behaviour in the
      project's own terms.
- [ ] Start the spec from `template-spec`: a draft concept at
      `knowledge/specs/Snnn-<feature-slug>.md`, listed in the specs
      index.
- [ ] Write observable behaviour, from outside.
- [ ] Write acceptance criteria as pass/fail checks (given/when/then).
- [ ] State the expected behaviour for bad input and failure, not just
      the happy path.
- [ ] State what is out of scope.
- [ ] UI: list the states (empty, loading, error, populated, edge
      cases) — the list is most of the spec.
- [ ] Write the test plan (`template-test-plan`), appended to the spec
      concept: automated cases, and manual checks for what automation
      cannot reach — UI is still tested.
- [ ] GATE: Can Verify and Accept walk every criterion pass/fail
      without interpretation? If not, rework it.
- [ ] GATE: Does the spec contradict a prior spec or convention?
      Surface the conflict, don't silently override.
- [ ] GATE: Validate the bundle to PASS
      (`superdev aokf validate knowledge`).

## IMPORTANT RULES

- Say what it does, never how — no implementation.
- The spec is a working document, not a record: its criteria become
  the tests, and the decisions it surfaces land in an ADR at
  interface design. It is retired — tagged `done` at accept — never
  maintained as documentation.
- The outside-view prose is the draft of the user docs — carry it to
  accept, don't discard it.

## Output

- The spec: a draft concept in `knowledge/specs/`, carrying acceptance
  criteria that Verify and Accept can walk without interpretation, and
  the outside-view prose accept seeds the docs from.
- Hand off to `/interface-design`.
