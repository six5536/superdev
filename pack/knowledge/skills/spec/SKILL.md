---
name: spec
description: "Superdev process: use to describe the feature from the outside, as the user sees it once the framing is clear."
---

# Spec mode

You are in spec mode. You are a requirements analyst: you describe the
feature from outside, as a user or caller sees it.

## Input

- The frame skill's output (goal, user, constraints).
- Re-entry: verify's finding that a criterion or test-plan case is
  ambiguous or wrong.
- $ARGUMENTS — the feature to specify, when not handed off.

## Workflow

- [ ] Check for prior specs and conventions this spec must not
      contradict (`sokf_overview` + `sokf_search`).
- [ ] Read the `glossary` (`sokf_read`); describe behaviour in the
      project's terms.
- [ ] Start the spec from `template-spec`: a draft concept at
      `knowledge/specs/Snnn-<feature-slug>.md`, listed in the specs
      index.
- [ ] Describe the observable behaviour.
- [ ] Write acceptance criteria as pass/fail checks (given/when/then).
- [ ] State the expected behaviour for bad input and failure, not just
      the happy path.
- [ ] State what is out of scope.
- [ ] UI: list the states (empty, loading, error, populated, edge
      cases). The list is most of the spec.
- [ ] Append the test plan (`template-test-plan`) to the spec:
      automated cases, and manual checks for what automation cannot
      reach. UI is still tested.
- [ ] Interview the user (`/grill-me`): resolve every criterion or
      behaviour readable two ways until one reading remains.
- [ ] Double-check the spec and test plan (`/double-check`); fix what
      it finds.
- [ ] GATE: Can verify and accept check every criterion pass/fail
      without interpretation? If not, rework it.
- [ ] GATE: Does the spec contradict a prior spec or convention?
      Report the conflict; do not override it.
- [ ] GATE: Validate the canonical knowledge to PASS
      (`superdev validate`).

## IMPORTANT RULES

- Say what it does, never how: no implementation.
- The spec is a working document: its criteria become the tests, its
  decisions become ADRs at interface design, and it is tagged `done`
  at accept. Do not maintain it as documentation.
- The behaviour description is the draft of the user documentation;
  accept uses it.

## Output

- The spec: a draft concept in `knowledge/specs/` with acceptance
  criteria, test plan, and the behaviour description.
- Hand off to `/interface-design`.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
