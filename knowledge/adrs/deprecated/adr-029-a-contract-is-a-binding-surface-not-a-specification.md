---
type: Decision
id: adr-029-a-contract-is-a-binding-surface-not-a-specification
title: A contract is a binding surface, not a specification
description: Contract documents state each requirement as one RFC 2119 sentence, define enumerable surfaces in their kind's native structured form, bind only what callers rely on, and link the reasoning to ADRs without restating it — one standard for every contract kind.
lifecycle: deprecated
---

# ADR-029: A contract is a binding surface, not a specification

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

Contract documents arrive as long prose: requirements are buried in
narrative, and where a schema does set a form rule the prose migrates
into doc comments inside the code fences — contract-007 carries
40-line doc comments retelling ADR reasoning (I029). A contract must
be precise without becoming a specification that carries every detail,
and one standard must serve CLIs, code APIs, config, file formats and
every other kind the 15 contract schemas govern.

## Decision

We will hold every contract document to four rules. Each normative
statement uses an RFC 2119 modal verb, one requirement per sentence.
An enumerable surface — commands, flags, keys, types, error cases,
limits — is defined in the kind's native structured form (code block,
table or list); prose, doc comments included, describes and never
defines. A contract binds only what callers rely on; behaviour it does
not list is the code's to decide. The reasoning behind a rule lives in
the linked ADR and is never restated. The standard is kind-agnostic —
each kind's schema names its native form — and the nine active
contracts are swept to conform.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Binding surface | Precision and brevity from one scope rule; checkable by reading any contract; kind-agnostic | Judged by reading, not by the validator |
| Structure only | Cheap to state | Narrative requirements stay legal; the imprecision survives in the prose |
| Hard length caps | Trivially checkable | Caps length, not imprecision; a short vague contract passes |
| Specification-style contracts | Exhaustive detail in one place | Duplicates the code and the ADRs; the corpus already shows the cost at 482 lines |

## Consequences

- Positive: a reviewer extracts requirements without excavating
  narrative; the sweep makes the corpus teach the standard by example.
- Negative: the sweep rewrites nine settled documents in one feature —
  form only, never what a contract binds.
- Follow-ups: the standard's text lives in a concept the contract
  schemas include (ADR-027); the contract-design skill enforces it at
  writing time (ADR-028's restructured steps present it for review).
- Superseded by ADR-033. Judging by reading is what this decision
  accepted as its cost, and the corpus it produced cannot be built
  from: the MCP contract carried no tool schema and the CLI contract's
  usage block drifted from the binary unnoticed. A contract now defines
  its interface. Three of the four style rules survive in the
  contract-style fragment; "bind only what callers rely on" is the one
  ADR-033 replaced, with the demand that a contract define the whole of
  what it names.
