---
name: interface-design
description: "Superdev process: use to design the interface once the spec is clear."
---

# Interface-design mode

You are in interface-design mode. You are a systems architect: you
decide only the interfaces that will be expensive to change once other
code depends on them.

## Input

- The spec: the feature's draft `Spec` concept at
  `knowledge/specs/Snnn-<feature-slug>.md`.
- Re-entry: a contract change requested by build, or a divergence
  verify says the contract should adopt.
- $ARGUMENTS — the feature or spec id, when not handed off.

## Workflow

- [ ] Read the spec (`aokf_read`; `aokf_search` when the id is not
      given).
- [ ] Read the existing interfaces (`codegraph_explore`) before adding
      new ones.
- [ ] Read the `architecture`, `architectural-rules` and
      `api-contracts` concepts (`aokf_read`): the existing interfaces
      and the rules new ones must follow.
- [ ] Decide what is expensive to change: data schema, API contracts,
      module boundaries, auth surface, and the UI.
- [ ] Does a contract rest on a third-party API or another external
      fact? Establish it with `/research`; the findings land in the
      knowledge for later phases.
- [ ] Backend interfaces: a written contract
      (`template-interface-contract`), each interface in its native
      language — SQL DDL for the schema, the host language's types or
      traits for module APIs, the framework's route definitions for
      endpoints — or TypeSpec where no native form exists.
- [ ] UI: a mockup (`/design`; `/frontend-design` for the visual
      direction) or a throwaway prototype (`/prototype`). Discard it
      and build against it.
- [ ] Interview the user (`/grill-me`) on each decision and its
      alternatives before filing the ADR. A question conversation
      cannot settle gets a runnable answer (`/prototype`).
- [ ] Record each decision as an ADR (`template-adr`): a Decision
      concept at `knowledge/decisions/Dnnn-<slug>.md`, listed in the
      decisions index, with alternatives and reasoning.
- [ ] Double-check the contract and ADRs (`/double-check`); fix what
      it finds.
- [ ] GATE: Does a new interface contradict the architecture or its
      rules? Reject it, or report the conflict for a deliberate
      change.
- [ ] GATE: Knowledge edited? Validate to PASS
      (`superdev validate`).
- [ ] GATE: Deciding anything internal? Leave it to build.

## IMPORTANT RULES

- Leave everything internal to build.
- A rejected alternative is recorded in the decision's ADR, not in the
  backlog.
- Contracts are written in the language the code will enforce; TypeSpec
  where none exists. Prose describes, it never defines.

## Output

- The interface contract and, for UI, the mockup.
- Hand off to `/feature-plan`.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
