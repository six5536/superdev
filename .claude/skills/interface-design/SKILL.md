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
- $ARGUMENTS — the feature or spec id, when not handed off.

## Workflow

- [ ] Read the spec (`aokf_read`; `aokf_search` when the id is not
      given).
- [ ] See the existing seams (`codegraph_explore`) before adding new
      ones.
- [ ] Read the `architecture`, `architectural-rules` and
      `api-contracts` concepts (`aokf_read`) — the seams already
      decided and the rules new ones must obey.
- [ ] Decide the expensive-to-change: data schema, API contracts,
      module boundaries, auth surface, and the UI.
- [ ] Backend seams: a written contract (`template-interface-contract`),
      each interface in its native language — SQL DDL for the schema,
      the host language's types or traits for module APIs, the
      framework's route definitions for endpoints — or TypeSpec where
      no native form exists.
- [ ] UI: a mockup (`design`; `frontend-design` for a deliberate
      visual direction) or throwaway prototype — discard it and build
      properly against it.
- [ ] Record each expensive decision as an ADR (`template-adr`): a
      Decision concept at `knowledge/decisions/Dnnn-<slug>.md`, listed
      in the decisions index — alternatives and reasoning included.
- [ ] GATE: Does a new seam contradict the architecture or its rules?
      Reject it, or surface the conflict for a deliberate change.
- [ ] GATE: Bundle edited? Validate to PASS
      (`superdev aokf validate knowledge`).
- [ ] GATE: Deciding anything internal? Leave it to build.

## IMPORTANT RULES

- Leave everything internal to Build.
- An alternative that lost to a decision lives in that decision's ADR,
  not in the backlog.
- Contracts are written in the language the code will enforce; TypeSpec
  where none exists. Prose describes, it never defines.

## Output

- The interface contract and, for UI, the mockup.
- Hand off to `/slice`.
