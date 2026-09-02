---
type: FeatureRequest
id: issue-047-feature-request-a-contract-kind-names-its-own-definition-language
title: Each contract kind names its own definition language, so a project writes in as many languages as it has surfaces
description: ADR-034 has every kind define in the form its ecosystem reads, which across the sixteen kinds names ten forms; the validator reads two of them, so most definitions are machine-readable only through a hand-written drift test.
lifecycle: open
links:
  - rel: references
    to: adr-034-each-kind-defines-in-the-form-its-ecosystem-reads
    note: The decision this questions — whether the per-ecosystem form is worth the language count it produces.
  - rel: references
    to: adr-035-a-schema-declares-its-definition-blocks-contract
    note: Supplies the block-language declaration, which thirteen of the sixteen kinds do not use.
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: The obligation that decides the question — a definition nothing can parse is bound only by a bespoke test.
---

# Feature: each contract kind names its own definition language

## Summary

[ADR-034][sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]
has each contract kind define its interface in the form its ecosystem
already reads. Across the sixteen kinds that names ten forms, and a
project adopting superdev writes in every one its surfaces touch. No
kind offers the alternative — one language expressive enough to carry
most of them — and nothing on file weighs the pluralism against its
cost.

## Motivation

The count is measurable three ways.

The sixteen contract-kind schemas name, between them, TypeSpec, JSON
Schema, Protobuf, GraphQL SDL, a JSON-RPC schema, the host language,
TOML, a DTD, a grammar and a byte layout. A project with a REST API, a
CLI, a config file and an internal module boundary writes four of them.

This repository's own nine contracts use five forms: `rust` for the
three interface contracts, `toml` for the config and the two pack
formats, `yaml` for the CLI, `json` for MCP and `text` for the template
tree.

The decisive number is the third.
[ADR-035][sokf:adr-035-a-schema-declares-its-definition-blocks-contract]
lets a schema declare the fence language its definition block takes, so
the validator can check the block generically. `BLOCK_LANGUAGES` in
`crates/lib/superdev-core/src/validate/schema/document.rs:82` is
`["yaml", "json"]` — two. Three of the sixteen kinds declare a
`block-language` at all (`contract-cli` and `contract-deployment` as
yaml, `contract-mcp` as json); the other thirteen name their form in
prose, which nothing reads. So a definition in TypeSpec, Rust, TOML or
SDL is checked by no generic machinery, and
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]'s
binding falls entirely to a drift test written by hand for that one
contract.

The pluralism therefore has a second cost beyond the languages a reader
must know: it puts most definitions outside what the validator can
check.

## Proposed behaviour

The kinds that can share one definition language do. Where a language
has native concepts for a kind's meaning — operations, models, fields,
optionality, versioning — that kind declares it, and the schema says so
in `block-language` rather than in prose. Where a kind needs a small
private vocabulary to fit, the vocabulary is declared once and reused.
Where a form genuinely cannot express the shape — production rules over
a token stream, a byte layout with conditional presence — the kind keeps
its own form and the schema states why.

Whatever the choice, every kind's definition form is declared where a
machine can read it, and a kind whose form the validator cannot parse
says plainly that a drift test is the only thing binding it.

## Acceptance criteria

1. TBD — whether one language becomes the declared default for the kinds
   that can carry it, or ADR-034's per-ecosystem pluralism stands with
   its cost recorded.
2. TBD — how a definition written in a language the binary cannot parse
   is bound to its implementation without putting a compiler in the test
   path, given ADR-036 obliges every contract to be bound.
3. TBD — whether the three text-format contracts on file are data-shaped
   enough to move to a common form, given that a kind describing a real
   grammar is not.
4. [ubiquitous] THE SYSTEM SHALL declare in every contract-kind schema
   the form its definition block takes, so the form is machine-readable
   rather than stated in prose.
5. [ubiquitous] THE SYSTEM SHALL state, for each kind whose definition
   form the validator cannot read, that a drift test is what binds it.
6. [ubiquitous] THE SYSTEM SHALL give each kind that keeps a form of its
   own the reason its ecosystem needs one.

## Alternatives considered

- Keep ADR-034 unchanged — a definition in the form its ecosystem reads
  needs no translation for the tools that already consume it, which is
  the whole reason the decision was taken; the cost is the count.
- Take each kind's own YAML or JSON vocabulary rather than one IDL — not
  JSON Schema everywhere, but the standard each ecosystem already
  publishes in those two syntaxes: OpenAPI for REST, AsyncAPI for
  events, JSON Schema for config and data, Kaitai Struct for a binary
  layout, the OpenTelemetry semantic conventions for telemetry, the W3C
  design-token format for UI tokens. Eleven of the sixteen kinds have
  one that a human can author. The syntax stays inside what the
  validator reads and what `serde_yaml_ng` parses, so a drift test needs
  no new machinery. This is the leading candidate; see the survey below.
- Adopt an expressive IDL and emit JSON Schema beside it for the tests —
  one authored source, two artifacts. Not a reduction in languages; a
  pipeline, and one that puts a generator in the path the proposal set
  out to avoid.
- Teach the binary to parse a third fence language — moves the cost from
  every project to this one, and needs a parser per language adopted.

## Scope

- In: the language a contract kind's definition block takes, kind by
  kind, and the reason for it.
- In: closing the gap where thirteen of sixteen kinds declare no
  `block-language`.
- Out: the definition blocks of the nine contracts already on file,
  which follow whatever their kinds settle on.
- Out: any change to what a drift test must prove — ADR-036 stands
  whatever language the definition takes.

## Comments

Raised 2026-09-01 against ADR-034 while framing
[I045][sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares],
on the ground that the value of a definition language is whether it can
express the interface, not whether an emitter exists for it — which, if
accepted, makes one expressive language reach much further across the
kinds than ADR-034 assumed.

The counter-fact found while filing this is that the validator reads two
fence languages. A kind that adopts a more expressive form gains
precision for its readers and loses the generic check, so "fewer
languages" and "more of the definition checked automatically" are not
the same goal and may not have the same answer. Criterion 2 is where
that has to be settled.

A survey taken 2026-09-02 asked whether the two syntaxes the validator
already reads could carry every kind. They can carry eleven of the
sixteen through a standard the kind's own ecosystem publishes, listed in
the alternative above; `contract-binary-format` is the unexpected one,
because Kaitai Struct — bit widths, endianness, conditional presence —
is itself YAML.

Four kinds resist for one reason: a complete JSON description exists but
only as a generated artifact. GraphQL has introspection JSON, RPC has
the protobuf descriptor set, and `contract-interface` and
`contract-library` have rustdoc JSON, which is nightly-only and
unstable. None is authorable by hand, and for the two interface kinds a
YAML restatement would be a second source of truth beside the host
language that already is the definition — `contract_interfaces.rs`
would still have to bind it.

The genuine exception is a text format that is a real grammar. Its
production rules are encodable as JSON — tree-sitter's `grammar.json`
is exactly that — and unreadable written by hand. A text format that is
a document shape, which is what the three on file are, takes JSON Schema
without difficulty.

What decides this against a more expressive IDL is not the generic
check, which is thinner than it sounds: `document.rs:889-922` verifies
the block parses, is a mapping, and carries its declared keys and entry
keys. That is completeness, not meaning, and it cannot tell OpenAPI from
any other mapping. What decides it is that a YAML or JSON block is
readable by a drift test using a parser already in the tree, with no
compiler in the test path — which is what criterion 2 asks for and what
an IDL cannot supply unaided.

Filed, not framed: three criteria are open questions rather than checks.

<!-- sokf:links -->
[sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]: /knowledge/adrs/active/adr-034-each-kind-defines-in-the-form-its-ecosystem-reads.md
[sokf:adr-035-a-schema-declares-its-definition-blocks-contract]: /knowledge/adrs/active/adr-035-a-schema-declares-its-definition-blocks-contract.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares]: /knowledge/issues/open/issue-045-feature-request-drift-tests-bind-what-the-contract-declares.md
