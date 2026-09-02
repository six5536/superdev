---
type: Decision
id: adr-038-a-contract-may-promise-what-is-not-built-yet
title: A contract may promise what is not built yet
description: A contract element marked pending is bound in reverse — the drift test fails when the implementation has it, so the marker cannot outlive its purpose — and accept refuses a contract still carrying one, so a promise cannot ship unbuilt.
lifecycle: deprecated
---

# ADR-038: A contract may promise what is not built yet

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation] binds a
contract to its implementation by exact equality, and
[ADR-028][sokf:adr-028-the-contract-design-go-ahead-is-an-explicit-interaction]
has contract-design commit the contract before build starts. A contract
that leads its implementation is therefore red from the contract-design
commit until the slice that builds it, and integrate runs the full suite
at every slice boundary — so a slice is failed by a promise another
slice owns. The red is correct: the interface is promised and unbuilt.
What is wrong is that nothing tells the two kinds of red apart, and
nothing bounds how long the first may last.

## Decision

A drift test reports its two directions differently. An element the
contract declares and the implementation lacks is a pending promise. An
element the implementation has and the contract omits is a defect, and
is reported as one. The wording is the mechanism: a reader sees which
they have without reading the diff.

A contract element MAY be marked pending, naming the plan slice that
will build it. The drift test binds a pending element in reverse: it
fails when the implementation has it, so the marker cannot outlive the
work it names. `accept` refuses a contract still carrying a pending
marker, so a promise cannot reach a settled feature unbuilt.

The feature plan orders a slice that closes a contract-implementation
gap before slices that do not, so most features never need the marker.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Name the directions, order the slices, mark what remains | The common case needs no marker at all; the rare case is bounded by a marker that expires by construction | Three mechanisms rather than one |
| Name the directions only | Cheapest, and makes an expected red legible | A multi-slice interface change still fails the slices that do not own it |
| Let the branch stay red until the feature ends | No machinery | Integrate's gate cannot tell an expected red from a regression, and sends the wrong slice back to build |
| Have contract-design leave its contract edits uncommitted until build | No red between the phases | Contradicts ADR-028's approved-and-committed change set, and loses the record of what was approved |
| Generate every surface from its contract | The window closes entirely | Generation reaches only part of most surfaces, and superdev cannot demand a generator of a managed repository |

## Consequences

- Positive: a red drift test says which kind of red it is, and a promise
  that outruns its implementation is bounded by a marker that fails once
  the work lands.
- Negative: a pending marker is a place to hide unbuilt work, which is
  why accept refuses one rather than reporting it.
- Follow-ups: the drift tests carry the wording, feature-plan carries
  the ordering rule, and the contract-kind schemas carry the marker.

<!-- sokf:links -->
[sokf:adr-028-the-contract-design-go-ahead-is-an-explicit-interaction]: /knowledge/adrs/active/adr-028-the-contract-design-go-ahead-is-an-explicit-interaction.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/deprecated/adr-036-a-contract-is-bound-to-its-implementation.md
