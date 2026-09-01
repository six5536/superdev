---
type: FeatureRequest
id: issue-035-feature-request-a-contract-does-not-define-its-interface
title: A contract describes its interface in prose instead of defining it, so nothing can be built from it
description: The contracts name their surfaces in narrative rather than defining them, so a caller cannot reproduce an interface from its contract — the MCP contract carries no tool schema at all, and the CLI contract's usage block has already drifted from the binary.
lifecycle: open
links:
  - rel: references
    to: issue-034-feature-request-normative-shapes-are-described-but-not-enforced
    note: Acceptance withheld — the shapes it enforces do not make a contract buildable.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Gains the block-language, block-keys and block-entry-keys rows (ADR-035).
---

# Feature: a contract describes its interface instead of defining it

## Summary

A contract is meant to bind an interface, and none of the nine on file
defines one completely enough to build against. A caller reading a
contract cannot reproduce the interface it names without reading the
code, which is the work the contract exists to have already done.

## Motivation

The evidence is on file. The MCP contract describes four tools, each
backed by a JSON Schema in the server, and carries no schema block at
all — a generator reading it gets nothing. The CLI contract's usage
block is the one place its surface is defined, and it has already
drifted from the binary: `status --drift`, `-h`/`--help`, the `help`
command and a shipped project template are all absent, and nothing in
the repository compares the block to the parser. Exit codes appear in
five separate prose paragraphs while the CLI schema offers an Exit
codes table the contract does not use. Only the config contract's
settings table approaches a definition.

The standard that permits this is
[ADR-029][sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification],
which set a contract against being a specification and left "what
callers rely on" to the writer's judgement.
[I034][sokf:issue-034-feature-request-normative-shapes-are-described-but-not-enforced]
then enforced the shape of normative sentences without touching
whether the surface is defined at all, so its acceptance was withheld.

## Proposed behaviour

A contract defines its interface: a competent implementer, or a code
generator, reproduces the interface exactly from the contract alone,
without reading the implementation and without being told anything
about how it works inside. Once this is done:

- Every enumerable surface a contract binds is written in a structured
  block that carries the whole of it: for a command line every command,
  flag, argument, exit code and stream; for an MCP server every tool's
  input schema and result shape; for a configuration every setting's
  key, type, default and source; for a file format the file's shape;
  for an internal interface every exported signature and type.
- A contract states what it binds and stops: how an interface is
  implemented stays out. No pattern decides this, so it stays a rule the
  contract-style fragment carries and review applies, beside the other
  rules a validator cannot settle.
- Every demand is language-agnostic. superdev governs other people's
  repositories, whose command line may be built on any framework and
  whose modules may be in any language, so a schema demands a form and
  never a toolchain.
- The contract leads. When the implemented interface and its contract
  disagree, a test fails naming the difference, so changing an interface
  means editing its contract first. Each schema states that obligation;
  this repository carries such a test for every contract whose interface
  it implements, including one that runs the binary and asserts each
  declared exit code.
- The nine contracts on file define their interfaces, the contract-kind
  schemas demand it, and the drift between the CLI contract and the
  binary is gone.
- A file-format contract names which kind of file it binds: the kind
  splits into a text format, whose shape is a schema or a worked
  example, and a binary format, whose shape is a byte layout.

## Acceptance criteria

1. [ubiquitous] THE SYSTEM SHALL define, in each contract-kind schema, the
   structured block its kind must carry and what that block must
   enumerate.
2. [event] WHEN a contract's structured block omits an element of the
   surface it binds THE SYSTEM SHALL fail validate naming the contract
   and the missing element.
3. [ubiquitous] THE SYSTEM SHALL demand a form and never a toolchain, so
   every declaration holds for a command line, a module or a served
   interface built on any framework.
4. [event] WHEN the implemented interface and its contract disagree THE
   SYSTEM SHALL fail a test naming the difference, for every contract
   whose interface this repository implements.
5. [event] WHEN a command's exit code differs from the code its contract
   declares THE SYSTEM SHALL fail a test that runs the binary.
6. [ubiquitous] THE SYSTEM SHALL define every command, flag, positional
   argument, exit code and stream of the superdev command line in the
   CLI contract.
7. [ubiquitous] THE SYSTEM SHALL define every MCP tool's input schema and
   result shape in the MCP contract.
8. [ubiquitous] THE SYSTEM SHALL define every exported signature and type
   an internal interface contract binds.
9. [ubiquitous] THE SYSTEM SHALL govern a text format and a binary format
   by a schema of its own, each demanding the shape its kind of file
   has, and refile every contract the split renames.
10. [ubiquitous] THE SYSTEM SHALL validate the shipped knowledge and the
    pack mirror clean with every new demand enforced.
11. [ubiquitous] THE SYSTEM SHALL record the superseding of ADR-029 and
    document each new demand in the changelog.

## Alternatives considered

- Generate each block from the code — cannot drift and costs nothing to
  keep, but the contract becomes a mirror of the implementation and can
  never refuse a change, which is the opposite of binding.
- Keep the current standard and sweep harder — the sweep is what
  produced the drifted block and the schemaless MCP contract; judgement
  applied twice has now failed twice.
- Raise the bar for public contracts only — an internal boundary is
  built against by the next module, so the same gap costs the same.
- Adopt one interface description language for every kind — TypeSpec
  reaches the served and data-shaped kinds and has no CLI concept, so a
  command line would need custom decorators shipped as a JavaScript
  library, and checking a contract would need that toolchain on the
  validator's path.
- Bind the contract to one framework's introspection — the fastest drift
  test to write here, and superdev governs repositories whose command
  line is built on anything at all.

## Scope

- In: every contract-kind schema's demands, the
  [document-schemas contract][sokf:contract-010-interface-document-schemas],
  the validator checks behind them, the split of the file-format kind into text and binary, the
  drift tests binding each on-file contract to its implementation, the
  nine contracts rewritten to define their interfaces, the ADR-029
  supersession, and the pack mirror.
- Out: a drift test for a kind with no contract on file to bind;
  generating any block from the code; a contract for an interface
  superdev does not have; teaching superdev to read any interface
  description language, which stays the project's own business.

<!-- sokf:links -->
[sokf:adr-029-a-contract-is-a-binding-surface-not-a-specification]: /knowledge/adrs/deprecated/adr-029-a-contract-is-a-binding-surface-not-a-specification.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-034-feature-request-normative-shapes-are-described-but-not-enforced]: /knowledge/issues/open/issue-034-feature-request-normative-shapes-are-described-but-not-enforced.md
