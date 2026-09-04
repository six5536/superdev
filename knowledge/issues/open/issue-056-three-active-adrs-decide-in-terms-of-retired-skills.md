---
type: Issue
id: issue-056-three-active-adrs-decide-in-terms-of-retired-skills
title: Three active ADRs decide in terms of retired skills, and one of them is reversed with no superseding decision
description: Three active ADRs state their decisions in the names of /frame, /adhoc-plan, /execute-feature-plan and /integrate, and ADR-028's decision that contract-design commits on approval is reversed by the shipped skill with nothing superseding it.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: Found at acceptance of I052.
---

# Bug: three active ADRs decide in terms of retired skills

## Summary

A reader consults the active ADRs to learn what the project decided.
Three of them state their decisions in the names of skills the workflow
no longer ships, and one states a decision the shipped skill now
contradicts. The sweep test that keeps retired names out of the
writer-facing documents does not reach the ADRs.

## Context

[I052][sokf:issue-052-the-workflow-carries-more-process-than-it-needs]
made the workflow FILE, SCOPE, BUILD, ACCEPT and retired `/frame`,
`/feature-plan`, `/adhoc-plan`, `/integrate` and the driver's old name
`/execute-feature-plan`. Three ADRs kept the old names.

- `adr-021-nothing-unattended-reaches-the-default-branch` decides that
  "`/frame` creates `feature/<slug>` off the default branch,
  `/adhoc-plan` creates `adhoc/<slug>`". Its `description` repeats the
  claim, and `knowledge/adrs/index.md` carries that description verbatim.
  `knowledge/development-procedure.md` states the live convention:
  `/scope` cuts `feature/<nnn>-<slug>` for work an issue asks for and
  `adhoc/<nnn>-<slug>` for one-off work.
- `adr-020-a-blocked-run-ends` decides that "Resuming is a fresh
  `/execute-feature-plan`". Its Context sends a blocking gate to
  `/frame` or `/contract-design`; the live driver is `/execute-plan`, and
  `/scope` owns the gate.
- `adr-028-the-contract-design-go-ahead-is-an-explicit-interaction`
  decides that the commit step is conditioned on the user's explicit
  approval, and bounds its scope by saying "integrate commits inside
  unattended runs by design".

ADR-028 is reversed, not merely stale.
`pack/knowledge/skills/contract-design/SKILL.md` and its `.claude/skills`
copy carry `<rule level="MUST NOT">commit; /scope commits the approved
edits with the plan</rule>`, and end with an unconditional call to
`/scope` handing over "the approved contract, source declaration and
decision-record edits, for the plan and the commit". No ADR supersedes
ADR-028; ADR-050 does not name it.

`nothing_a_writer_builds_against_names_a_retired_phase` in
`crates/lib/superdev-core/tests/normative_shapes.rs` sweeps the roots
`writer_facing_roots` returns: the contracts, the ideas, the schemas, the
pack's concepts, schemas and skills, `.claude/skills`, the README and the
knowledge root's own top-level markdown. `knowledge/adrs/` is absent from
that list.

## Behaviour

An active ADR states a decision that holds. A decision that no longer
holds is superseded and filed under `adrs/deprecated/`, and the ADR that
supersedes it says what replaced it.

ADR-021 and ADR-020 need their Decision sections and ADR-021's
`description` rewritten in the live skill names, which changes what they
say about who cuts a branch and what a resume is, not what they decided.
ADR-028's decision needs a superseding ADR: the go-ahead gate survives,
`/contract-design` presents the change set and commits nothing, and
`/scope` commits the approved edits with the plan.

The sweep test excludes `knowledge/adrs/` wholesale. That exclusion is
right for `adrs/deprecated/`, whose documents are history, and wrong for
`adrs/active/`, whose Decision sections are what a writer builds against.
Extending the sweep to `adrs/active/` is what keeps this from recurring.

## Scope

The three ADRs, the index entry that mirrors one of them, and the sweep
that missed all three.

- In: ADR-021's Decision and `description`, and the ADRs index entry
  generated from that description.
- In: ADR-020's Decision and Context.
- In: a superseding ADR for ADR-028, and ADR-028's refiling under
  `adrs/deprecated/`.
- In: extending `nothing_a_writer_builds_against_names_a_retired_phase`
  to `knowledge/adrs/active/`.
- Out: `knowledge/adrs/deprecated/`, which records what the project
  decided and later reversed.
- Out: the plans, the issues and the changelog, which are history in the
  same way.
- Out: the branch-naming convention itself, which
  `knowledge/development-procedure.md` already states.

<!-- sokf:links -->
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/done/issue-052-the-workflow-carries-more-process-than-it-needs.md
