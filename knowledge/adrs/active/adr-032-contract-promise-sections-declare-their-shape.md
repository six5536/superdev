---
type: Decision
id: adr-032-contract-promise-sections-declare-their-shape
title: A contract's promise sections declare that they bind
description: The contract-kind schemas declare a content-pattern requiring an RFC 2119 keyword on every promise-bearing section, and an item-pattern on the sections where each entry is a promise — the shape future contracts must take, with the nine on file reconciled to it.
lifecycle: active
---

# ADR-032: A contract's promise sections declare that they bind

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

ADR-029's binding-surface standard ships as fragment prose the
validator never reads, and the sweep it drove passed acceptance while
requirements stayed narrative (I033, I034). A probe of the on-file
contracts shows their definitional lists — tools by signature,
precedence by order, boundaries by naming the owner — legitimately
carry no RFC 2119 keywords, so a blanket keyword rule misfires. The
declarations must target sections by purpose, and they bind what future
contracts must look like, not what the current nine happen to satisfy.

## Decision

The keyword pattern is
`\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b` —
uppercase forms only bind, so descriptive prose keeps its ordinary
words. Sections where every entry is a caller-facing promise declare it
per item (`item-pattern`): cli Behaviour, mcp Tools, data Constraints,
deployment Health and lifecycle, events Ordering and delivery. Sections
whose job is promises declare it for the body (`content-pattern`):
Stability in all fourteen public kinds; mcp Server and Errors; config
Sources and precedence, and Secrets; data Migration; deployment
Runtime; file-format Compatibility; graphql Errors and Limits; library
Errors; rest Authentication; rpc Authentication; authz Boundaries;
interface Module boundaries and Cross-cutting concerns. Definitional
sections — code surfaces, tables, Key flows, Screens and states and
their like — declare nothing: they bind by form. The nine contracts on
file are reconciled to the declarations.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Promise sections declare their shape | An under-specified promise section fails validate everywhere; definitional lists stay untouched | The section assignment is a judgment the ADR must carry |
| Declare nothing yet | Zero reconciliation | Judges by current conformance when the point is the future shape; the I034 drift continues unchecked |
| Keyword-per-item on every list section | Uniform | Misfires on definitional items — 27 of 53 in the probe carry no keyword legitimately |
| An engine-wide keyword-in-prose error | No per-schema work | The engine would impose a check no schema declared; scope belongs in the schema config (rejected at framing) |

## Consequences

- Positive: a promise section that states no promise is a validate
  error at edit time, here and in every managed repository.
- Negative: the nine on-file contracts take another sweep — bounded to
  the sections named above.
- Follow-ups: build declares the patterns in the fifteen contract
  schemas and the pack mirror, then sweeps the contracts until the
  declared checks pass; the contract-style fragment keeps the rules no
  pattern can decide.
