---
type: Decision
id: adr-036-a-contract-is-bound-to-its-implementation-by-a-test
title: A contract is bound to its implementation by a test the project owns
description: Each contract kind's schema obliges the project to carry a test proving the implemented interface equals the contract's declared surface element for element, and leaves the mechanism to the project — superdev demands a form, never a framework.
lifecycle: active
---

# ADR-036: A contract is bound to its implementation by a test the project owns

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

Each contract kind's schema obliges the project to carry a test that
proves the implemented interface equals the contract's declared surface
element for element, in both directions: an element the implementation
has and the contract omits fails, and an element the contract declares
and the implementation lacks fails. The schema names the obligation and
never the mechanism, so a project binds its contract through whatever
introspection its framework offers. A facet no introspection reports —
an exit code, a stream — is bound by exercising the interface and
asserting the result. This repository carries such a test for every
contract whose interface it implements, and its exit codes are bound by
running the binary.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Exact equality, mechanism the project's | Catches drift in both directions; holds whatever framework implements the interface | Each project writes its own binding, and superdev cannot check that it did |
| Name the framework's introspection in the schema | superdev could ship the test | Binds every managed repository to one framework per kind, which superdev has no business doing |
| Contract as a subset of the implementation | Cheaper, and no contract ever fails for being behind | An undocumented flag passes in silence, which is the drift that produced this decision |
| Review instead of a test | No machinery | Judgement applied to completeness has failed twice on this corpus |

## Consequences

- Positive: changing an interface means editing its contract first,
  because the test fails until it is edited.
- Negative: the obligation is stated, not enforced — superdev cannot
  tell whether a managed repository wrote its binding test.
- Follow-ups: this repository's tests bind the CLI, MCP, config,
  format and interface contracts, exit codes included.
