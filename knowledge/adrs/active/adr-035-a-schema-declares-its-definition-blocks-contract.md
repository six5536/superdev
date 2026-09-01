---
type: Decision
id: adr-035-a-schema-declares-its-definition-blocks-contract
title: A schema declares its definition block's contract
description: A section rule declares the fence language its definition block takes and the keys the block and each of its entries must carry, so the validator checks a block's completeness generically instead of carrying one policy per contract kind.
lifecycle: active
---

# ADR-035: A schema declares its definition block's contract

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

ADR-033 requires a definition block to carry the whole of its
interface, and a validator reading only documents cannot know what the
whole is. What it can decide is whether the block is complete against
itself: whether it parses, and whether every entry carries the facets
its kind demands — a command with no exit codes, a tool with no input
schema, a setting with no type. The vocabulary already reaches section
bodies through
[ADR-030][sokf:adr-030-a-section-rule-declares-body-patterns]'s
patterns, which match text and cannot tell a key inside one entry from
a key inside another.

## Decision

A section rule gains three declarations. `block-language` names the
fence tag the section's block must carry. `block-keys` lists the keys
the block must carry at its top level. `block-entry-keys` lists the
keys every top-level entry of the block must carry. The validator
parses the block in the declared language, and a missing key is an
error naming the file, the section, the entry and the key; a block that
does not parse is an error naming the parse failure. A schema declaring
none of the three keeps its section unchecked beyond what it declared
before. The engine reads YAML and JSON, which the binary already
parses; a block in a language it cannot parse — SDL, protobuf,
TypeSpec, a host language's signatures — declares no `block-language`,
and its completeness is the drift test's (ADR-036).

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Declared block contract | The schema config defines the check, as it does for every other rule; one mechanism serves every kind | Reaches only the languages the binary parses |
| A checker per kind inside the validator | Precise for every kind, including the ones it cannot parse | Fifteen policies inside the binary that a schema author can neither see nor change |
| Reuse `content-pattern` for key names | Nothing new to build | Matches text, so it cannot tell which entry a key belongs to — the completeness question itself |
| Compile every block through its own toolchain | Exact for SDL, protobuf and TypeSpec | Puts a Node toolchain on the PostToolUse hook's path for every edit |

## Consequences

- Positive: a contract missing a facet fails at edit time, in this
  repository and every managed one, with no toolchain beyond the
  binary.
- Negative: the completeness of a block in a language the binary does
  not parse rests on its drift test alone.
- Follow-ups: contract-010 gains the three rows; each kind's schema
  declares its block's contract.

<!-- sokf:links -->
[sokf:adr-030-a-section-rule-declares-body-patterns]: /knowledge/adrs/active/adr-030-a-section-rule-declares-body-patterns.md
