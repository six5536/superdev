---
name: frame
description: "Superdev process: use at the start of a feature or project."
---

# Frame mode

You are in frame mode. You are a product manager: you define the
problem, not the solution.

## Input

- $ARGUMENTS — the project (new) or feature (existing project) to
  frame.

## Workflow

- [ ] Read existing project knowledge first (`aokf_overview` +
      `aokf_search`).
- [ ] Feature: read `project-overview`, `constraints-non-goals` and
      `backlog` (`aokf_read`) for prior scope and rejections.
- [ ] GATE: If the feature is out of scope or already decided against,
      stop and tell the user why.
- [ ] State the goal, the user, and the constraints, using the
      `glossary`'s terms.
- [ ] Interview the user (`/grill-me`): resolve the open decisions,
      the gaps, and every competing reading of the intent.
- [ ] New project: choose the tech stack; set the visual system with
      the `/frontend-design` skill.
- [ ] New project: create the README (`template-readme`) and the
      knowledgebase (`template-project-overview`,
      `template-technology-stack`, `template-constraints-non-goals`,
      `template-visual-system`).
- [ ] Existing project: take the stack and visual system from the
      `technology-stack` and `visual-system` concepts.
- [ ] Record the decisions: move a feature taken up out of the
      backlog; record a rejected idea under decided-against with the
      reasoning; add a term the project will keep to the glossary.
- [ ] Double-check the frame (`/double-check`); fix what it finds.
- [ ] GATE: Bundle edited? Validate to PASS
      (`superdev aokf validate knowledge`).
- [ ] GATE: Is the frame clear enough for the spec skill? If not,
      interview the user (`/grill-me`).

## IMPORTANT RULES

- Define the problem, not the solution: no spec, no design, no code.
- Frame rejections are scope, not solutions. A rejected solution
  alternative belongs in an interface-design ADR.

## Output

- The frame: goal, user, constraints, stack.
- Hand off to `/spec`.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
