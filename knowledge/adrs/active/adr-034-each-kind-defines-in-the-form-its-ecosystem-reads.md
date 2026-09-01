---
type: Decision
id: adr-034-each-kind-defines-in-the-form-its-ecosystem-reads
title: Each kind defines in the form its ecosystem reads
description: A contract kind's definition block takes the form generators already consume for that kind — an interface description language where one exists, JSON Schema for data-shaped surfaces, a declared YAML block where neither does, and the host language for a code boundary.
lifecycle: active
---

# ADR-034: Each kind defines in the form its ecosystem reads

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

ADR-033 requires a machine-readable definition block, and the kinds do
not share one language. Some have an interface description language the
ecosystem already generates from — SDL for GraphQL, protobuf for RPC,
TypeSpec for REST, which `schema-contract-rest` already names. Some
describe data and are served by JSON Schema. A command line has
neither: TypeSpec has no CLI concept, and expressing commands, short
flags, exit codes and streams in it needs custom decorators shipped as a
JavaScript library. superdev governs repositories built on any
framework, so no form may assume one.

## Decision

Each contract kind names the form its ecosystem reads. Where an
interface description language exists for the kind, the contract is
written in it: SDL for GraphQL, an IDL for RPC, TypeSpec or OpenAPI for
REST, a schema language for events. Where the surface is data, the
block is JSON Schema or the file's own language carrying every key.
Where neither exists — a command line, telemetry, a UI's routes, an
authorization surface, a deployment — the block is YAML in a shape the
kind's schema declares. Where the surface is code, the block is the
host language's own signatures. Every form names a shape and never a
toolchain, so a contract holds whatever framework implements it, and
superdev reads only the forms its own binary already parses.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| The form its ecosystem reads | Generators already consume it; the CLI gets a form that fits it | Several forms to know, one per kind |
| TypeSpec for every kind it reaches | One language across the served and data kinds | No CLI concept, so a command line needs a JavaScript decorator library, and checking a contract needs that toolchain on the validator's path |
| One custom YAML form for every kind | One parser, no external toolchain, total uniformity | Throws away SDL, protobuf and TypeSpec where those are exactly what the ecosystem reads and generates from |
| The framework's own help or introspection output | Trivially compared to the implementation | It is output, not a definition: no exit codes, no streams, no value types, and it binds the contract to one framework |

## Consequences

- Positive: a contract's block drops into the generator its ecosystem
  already has, and the CLI form fits a command line rather than being
  bent to fit a language built for APIs.
- Negative: a reviewer meets more than one form across the kinds.
- Follow-ups: ADR-035 makes each declared form checkable, and the
  fifteen schemas name their form.
