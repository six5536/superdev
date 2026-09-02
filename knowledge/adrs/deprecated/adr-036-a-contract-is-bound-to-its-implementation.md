---
type: Decision
id: adr-036-a-contract-is-bound-to-its-implementation
title: A contract is bound to its implementation
description: Each contract kind's schema obliges the project to bind its implemented interface to the contract's declared surface, element for element — by generating the surface from the contract, or by a test where it is hand-written — and leaves the mechanism to the project.
lifecycle: deprecated
---

# ADR-036: A contract is bound to its implementation

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

A definition block complete against itself can still be wrong: the CLI
contract's usage block omitted a flag the binary has offered for
months, and nothing noticed, because no check compares the contract to
what runs. Only the project can make that comparison — it alone knows
its framework, and superdev governs repositories whose command line may
be built on anything and whose modules may be in any language.

## Decision

Each contract kind's schema obliges the project to bind its implemented
interface to the contract's declared surface, element for element in
both directions: an element the implementation has and the contract
omits fails, and an element the contract declares and the
implementation lacks fails. The schema names the obligation and never
the mechanism.

Generating the surface from the contract satisfies the obligation for
whatever it generates, and satisfies it more strongly than any test
can: generated code cannot disagree with its own input, so there is
nothing left to detect. Generation binds only where the build performs
it — a project that commits generated output MUST prove the committed
copy is what the contract generates today, or the staleness it invites
is drift under another name. Generation is also usually partial: a
generator emits a command's flags and not the code it exits with, so
whatever it does not reach is bound by a test that exercises the
interface and asserts the result.

This repository's interfaces are hand-written, so tests bind them: one
per contract whose interface it implements, with the exit codes bound
by running the binary.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Exact equality, mechanism the project's | Catches drift in both directions; holds whatever framework implements the interface, and admits generation as the stronger form | Each project writes its own binding, and superdev cannot check that it did |
| A test in every case, generation or not | One rule to state | Demands a test of the generated surface, which tests the generator rather than the contract, and marks the more rigorous project non-compliant |
| Name the framework's introspection in the schema | superdev could ship the test | Binds every managed repository to one framework per kind, which superdev has no business doing |
| Contract as a subset of the implementation | Cheaper, and no contract ever fails for being behind | An undocumented flag passes in silence, which is the drift that produced this decision |
| Review instead of a test | No machinery | Judgement applied to completeness has failed twice on this corpus |

## Consequences

- Positive: changing an interface means editing its contract first,
  because the binding fails until it is edited. A project already
  generating from its contract is compliant without writing a test it
  does not need.
- Negative: the obligation is stated, not enforced — superdev cannot
  tell whether a managed repository bound its contract at all.
- Follow-ups: this repository's tests bind the CLI, MCP, config,
  format and interface contracts, exit codes included.
