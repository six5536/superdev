---
type: FeaturePlan
id: plan-021-feature-contracts-define-their-interface
title: Contracts define their interfaces — feature plan
description: Slices delivering the definition-block vocabulary, each kind's declared form, the drift tests that bind a contract to its implementation, and the split of the file-format kind.
lifecycle: open
links:
  - rel: implements
    to: issue-035-feature-request-a-contract-does-not-define-its-interface
    note: The plan delivers the framed issue's eleven criteria.
---

# Feature plan: contracts define their interfaces

Request:
[issue-035][sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]

## Slices

### Slice 1: The definition-block vocabulary in the engine

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `block-language`, `block-keys` and `block-entry-keys` land in
  `validate::schema` per ADR-035 — `SectionRule` fields, the block
  parse and key checks, the mis-declaration findings — and in both
  grammar copies. No schema declares them yet.
- Done-check: a probe schema declaring each produces the ADR-035
  findings on a failing document and a failing schema; the live tree's
  findings are unchanged.
- Cases:
  - unit: a block entry missing a declared key is an error naming the
    file, the section, the entry and the key — covers 2.
  - unit: a block missing a declared top-level key is an error naming
    the file, the section and the key — covers 2.
  - unit: a block that does not parse in its declared language is an
    error naming the parse failure — covers 2.
  - unit: a JSON block and a YAML block are both read — covers 2, 3.
  - unit: a `block-language` the validator cannot parse, and a block
    declaration on a section whose content is not `code`, are findings
    on the schema file and bind nothing — covers 2.
  - unit: a section declaring no block rule gains no finding — covers 2.
  - unit: a schema's example is checked against its own block rules
    (ADR-024 path) — covers 2.

### Slice 2: The CLI contract defines the command line

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `schema-contract-cli` declares the YAML definition form of
  ADR-034 — every command with its arguments, flags, exit codes and
  streams — in both trees; `contract-002` is rewritten to define the
  whole surface; a drift test binds the contract to the implemented
  command line, and a probe test runs the binary and asserts each
  declared exit code (ADR-036).
- Done-check: the drift test fails when a flag is added to the binary
  and not to the contract, and when the contract declares a flag the
  binary lacks; `status --drift`, `--help` and the shipped template set
  are in the contract.
- Cases:
  - unit: the CLI schema's block rules reject a command entry with no
    exit codes — covers 1, 2.
  - integration: the declared surface and the implemented command line
    agree element for element, both directions — covers 4, 6.
  - integration: an element added to the implementation and absent from
    the contract fails the drift test naming it — covers 4.
  - e2e: each exit code the contract declares is produced by running the
    binary — covers 5.

### Slice 3: The MCP contract defines its tools

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `schema-contract-mcp` gains the definition section it has
  never had, declaring each tool's input schema and result shape in
  JSON Schema, in both trees; `contract-003` is rewritten to define the
  four tools; a drift test binds the contract to the served tool list.
- Done-check: the drift test fails when a tool's argument is added,
  removed or retyped in the server and not in the contract.
- Cases:
  - integration: every served tool's name and input schema equals what
    the contract declares, both directions — covers 4, 7.
  - integration: a tool argument added to the server and absent from the
    contract fails the drift test naming it — covers 4.

### Slice 4: The file-format kind splits into text and binary

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `schema-contract-text-format` and
  `schema-contract-binary-format` replace `schema-contract-text-format`
  per ADR-037, each with its own worked example, in both trees;
  contracts 005, 006 and 008 take the new kind token in their ids and
  are refiled, and every link to them is rewritten by id.
- Done-check: `validate` passes with no reference to the old kind
  anywhere in the knowledge or the pack mirror; the three contracts
  resolve under their new ids.
- Cases:
  - integration: both new schemas govern their type and their examples
    conform — covers 9.
  - integration: no document, index or link names the retired kind —
    covers 9, 10.

### Slice 5: The config and format contracts define their files

- [x] Done — ticked by integrate at merge.
- Depends-on: 1, 4.
- Change: `schema-contract-config` and `schema-contract-text-format`
  declare their definition blocks; contracts 004, 005, 006 and 008 are
  rewritten to define every key, its type, its default and what a
  reader does with the unexpected; drift tests bind each to the types
  that read the file.
- Done-check: a key added to the manifest or the lock without its
  contract entry fails a test naming the key.
- Cases:
  - integration: every key the implementation reads is declared, both
    directions, for the manifest, the lock, the pack file and the
    template — covers 1, 4.

### Slice 6: The interface contracts define their boundaries

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `schema-contract-interface` declares its definition block in
  the host language; contracts 007, 009 and 010 are rewritten to carry
  every exported signature and type they bind; a drift test binds each
  block to the exported items.
- Done-check: a signature changed in the code and not in its contract
  fails a test naming it.
- Cases:
  - integration: every signature a contract declares exists as declared,
    and every exported item the contract binds is declared — covers 4, 8.

### Slice 7: The remaining kinds declare their forms

- [ ] Done — ticked by integrate at merge.
- Depends-on: 1, 4.
- Change: the kinds with no contract on file — authz, data, deployment,
  events, graphql, library, rest, rpc, telemetry, ui and the new binary
  format — declare their definition blocks per ADR-034, and each
  schema's worked example carries a block that satisfies them, in both
  trees.
- Done-check: a live-repo test enumerates every contract-kind schema's
  declared form; every example passes its own declarations.
- Cases:
  - integration: every contract-kind schema declares a definition form
    and its example satisfies it, in both trees — covers 1, 10.
  - integration: no schema names a framework or a toolchain in what it
    demands — covers 3.

### Slice 8: The standard, the obligation and the records

- [ ] Done — ticked by integrate at merge.
- Depends-on: 2, 3, 5, 6, 7.
- Change: the contract-style fragment carries ADR-033's rule and the
  drift obligation of ADR-036, materialized into every contract-kind
  schema; the changelog records each new demand and the retired kind;
  the ADR-029 supersession is stated where a reader meets it.
- Done-check: `superdev validate` passes on the knowledge and the pack
  mirror with every demand live, and the fragment materializes into
  every contract-kind schema.
- Cases:
  - integration: the fragment states the definition rule and the drift
    obligation, and every contract-kind schema carries it — covers 11.
  - e2e: a full validate run over both trees reports zero errors with
    every declared demand enforced — covers 10.

<!-- sokf:links -->
[sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]: /knowledge/issues/open/issue-035-feature-request-a-contract-does-not-define-its-interface.md
