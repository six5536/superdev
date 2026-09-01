---
type: FeatureRequest
id: issue-037-feature-request-a-contracts-behaviour-is-not-written-as-ears
title: A contract's behaviour is free prose with a modal verb, while a criterion about it must be EARS
description: Acceptance criteria are held to EARS and the contract clauses they are about are not, so one kind of statement has two forms — and the contract's form buries the trigger and admits two requirements in one sentence.
lifecycle: open
---

# Feature: a contract's behaviour is not written as EARS

## Summary

A feature-request's acceptance criteria must be EARS sentences, checked
by the validator. The contract clauses those criteria are about are free
prose carrying an RFC 2119 keyword, checked only for the keyword. One
kind of statement — a requirement — has two forms in this repository.

## Motivation

The nine active contracts carry 22 keyword-bearing list items and 39
keyword-bearing prose sentences. None is in EARS form. The current form
buries the trigger and admits more than one requirement per sentence:

> **`init`** MUST refuse a directory that is not a git repo, and MUST
> refuse a re-run once `.superdev/config.toml` exists.

which is two requirements, each with its trigger in a subordinate
clause. As EARS they separate, and each names what sets it off:

> [event] WHEN `init` runs outside a git repository THE SYSTEM SHALL
> refuse it.
> [state] WHILE `.superdev/config.toml` exists THE SYSTEM SHALL refuse
> `init`.

[ADR-032][sokf:adr-032-contract-promise-sections-declare-their-shape]'s
keyword rule counts keywords and cannot see this, which is how a bullet
carrying two MUSTs passes today. The machinery to bind the form already
ships: the `item-pattern` that binds a criterion's EARS tag
([ADR-031][sokf:adr-031-ears-criteria-are-checked-by-item-pattern])
would bind a behaviour clause's the same way.

## Proposed behaviour

Not yet designed. The shape to settle is whether a contract's
behaviour clauses become EARS statements, and whether the promise
sections that are prose today — Stability, Compatibility, Errors,
Boundaries — become lists so each statement can be bound, or stay prose
and go on unbound.

## Acceptance criteria

1. TBD — whether every behaviour clause takes EARS form, or only those
   in sections that already bind per requirement.
2. TBD — whether a promise section that is prose today becomes a list,
   so each statement can be bound, or stays prose and unbound.
3. TBD — what EARS says in a contract whose subject is not "the
   system": a command, a reader, a caller, a served tool.
4. TBD — whether the surviving RFC 2119 rules of
   [ADR-029][sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]
   are replaced by the EARS form or kept beside it.

## Alternatives considered

- Not yet thought about — framing settles this, and what is recorded
  here is the gap and the evidence for it.

## Scope

- In, provisionally: the contract-kind schemas' declarations for
  behaviour and promise sections, the validator checks behind them, and
  the nine active contracts' 61 statements.
- Out, provisionally: acceptance criteria, which are EARS already;
  the definition blocks, which bind by form rather than by sentence.

## Comments

Filed without framing, at the owner's request. The framing belongs at
the point the work is taken up, which is what
[I030][sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]
records: this issue is an instance of the lightweight filing that issue
asks for, filed by hand in the meantime.

<!-- sokf:links -->
[sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]: /knowledge/adrs/deprecated/adr-029-a-contract-is-a-binding-surface-not-a-specification.md
[sokf:adr-031-ears-criteria-are-checked-by-item-pattern]: /knowledge/adrs/active/adr-031-ears-criteria-are-checked-by-item-pattern.md
[sokf:adr-032-contract-promise-sections-declare-their-shape]: /knowledge/adrs/active/adr-032-contract-promise-sections-declare-their-shape.md
[sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]: /knowledge/issues/open/issue-030-feature-request-filing-an-issue-requires-framing-it.md
