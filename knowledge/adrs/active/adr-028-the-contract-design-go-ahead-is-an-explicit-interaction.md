---
type: Decision
id: adr-028-the-contract-design-go-ahead-is-an-explicit-interaction
title: The contract-design go-ahead is an explicit interaction
description: The contract-design skill interviews the user on every ADR decision, presents the complete change set, and commits only on explicit approval — restructured process steps rather than hook enforcement or a new gate vocabulary.
lifecycle: active
---

# ADR-028: The contract-design go-ahead is an explicit interaction

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

A `/contract-design` session commits its contract and ADR edits and
only then asks its first question (I028). The skill's prose orders the
go-ahead gate before the commit step, but a gate reads as a self-check
a session satisfies by seeing no objection, and the interview step
binds to "each decision and its alternatives", which a session that
sees no contested decision skips. Nothing marks either as a mandatory
interaction, so the review the gate promises never happens.

## Decision

We will restructure the skill's process steps. The interview step
binds to every decision recorded as an ADR, contested or not; a
present-the-change-set step precedes the commit and shows the complete
contract and ADR changes; the commit step is conditioned on the user's
explicit approval, rework is applied and re-presented, and withheld
approval leaves the edits uncommitted on the feature branch. Scope is
the contract-design skill alone: frame's records are co-written in
conversation and integrate commits inside unattended runs by design.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Restructured process steps | Prose-only; no new machinery; fixes the observed skips directly | Relies on the session following prose, the same medium that failed — mitigated by making the interaction a step, the form sessions do follow |
| PreToolUse hook blocking commit | Mechanically enforced | The harness has no notion of which phase runs, so the hook would gate every commit in every session |
| Gate-owner vocabulary (`owner="user"`) | Fixes the self-satisfied-gate class across all skills | Touches every skill; exceeds the framed scope |

## Consequences

- Positive: nothing lands unseen; the acceptance criteria of I028 are
  checkable on any transcript.
- Negative: an attended contract-design session gains two blocking
  touchpoints; unattended runs are unaffected because contract-design
  runs before a run begins.
- Follow-ups: if other skills grow self-satisfied user gates, the
  rejected gate-owner vocabulary is the class-level fix to revisit.
