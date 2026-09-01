---
type: Decision
id: adr-033-a-contract-defines-its-interface
title: A contract defines its interface
description: A contract carries the whole of the interface it binds in a machine-readable definition block, so a caller reproduces the interface from the contract alone — superseding the standard that set a contract against being a specification.
lifecycle: active
links:
  - rel: supersedes
    to: adr-029-a-contract-is-a-binding-surface-not-a-specification
    note: Replaces "bind only what callers rely on" with "define the whole surface, and no internals".
---

# ADR-033: A contract defines its interface

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

[ADR-029][sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]
set a contract against being a specification and left "what callers
rely on" to the writer's judgement. Two features applied that
judgement, and the corpus it produced cannot be built from: the MCP
contract carries no tool schema, the CLI contract's usage block has
drifted from the binary, and only the config contract's settings table
approaches a definition. A contract that a caller cannot implement
against is describing an interface, not binding one.

## Decision

A contract defines the interface it names. Every element a caller
depends on — each command, flag, argument, exit code, stream, tool,
setting, field, signature and error — appears in a machine-readable
definition block, so a competent implementer or a code generator
reproduces the interface from the contract alone, without reading the
implementation. What a contract must not carry is unchanged and now
carries the whole of the old rule's weight: how the interface is built
inside stays out. Prose keeps its place around the block, describing
and orienting; the block defines. ADR-029 is superseded, and its four
style rules survive in the contract-style fragment as the rules a
validator cannot settle.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Define the interface | A contract can be built from, which is what the word means; drift becomes decidable | Longer contracts, and every kind needs a form named for it |
| Keep ADR-029 and sweep harder | No new machinery | The same judgement has now produced an unbuildable corpus twice |
| Raise the bar for public contracts only | Half the work | An internal boundary is built against by the next module, at the same cost |
| Generate every block from the code | Cannot drift, costs nothing to keep | The contract becomes a mirror that can never refuse a change, which is the opposite of binding |

## Consequences

- Positive: a contract is usable as a generator's input, and a
  reviewer can tell completeness from prose quality.
- Negative: the nine contracts on file are rewritten, and a contract
  now costs more to write than to describe.
- Follow-ups: ADR-034 names each kind's form, ADR-035 makes the block
  checkable, ADR-036 binds a contract to its implementation, and
  ADR-037 splits the file-format kind.

<!-- sokf:links -->
[sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]: /knowledge/adrs/deprecated/adr-029-a-contract-is-a-binding-surface-not-a-specification.md
