---
type: FeatureRequest
id: issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose
title: A contract kind states its definition form in prose, so nothing checks the block a contract carries
description: Nine of the sixteen kinds name their definition form in a section description that nothing reads, and a code section is satisfied by any fence whatever its tag, so a contract may carry a block in the wrong form and pass.
lifecycle: wontfix
links:
  - rel: references
    to: issue-049-feature-request-a-contract-is-not-written-to-be-compared
    note: Supersedes this issue; criteria 4, 5 and 12 there carry the declaration work.
  - rel: references
    to: adr-034-each-kind-defines-in-the-form-its-ecosystem-reads
    note: Stands unchanged — this declares the form each kind already chose, rather than changing the choice.
  - rel: references
    to: adr-035-a-schema-declares-its-definition-blocks-contract
    note: Supplies `block-language`, which is fatal outside yaml and json and so unavailable to the nine kinds that need it.
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: The obligation the unparsed kinds meet by drift test alone, which no schema says.
  - rel: references
    to: issue-048-feature-request-no-step-asks-whether-a-contract-is-bound
    note: Overlaps at a different level — this names the binding per kind, I048 weighs naming it per contract.
---

# Feature: a kind's definition form is stated in prose

## Won't fix

Superseded 2026-09-02 by
[I049][sokf:issue-049-feature-request-a-contract-is-not-written-to-be-compared].
A form nothing reads is the enumerability property failing — the first
of the four a comparison needs — and stating the property once is what
keeps the same fault from being specified again in a fifth vocabulary.
Criteria 4, 5 and 12 of I049 carry this issue's declaration work, and
the open question about a set of forms against a single string goes with
them.

## Summary

Nine of the sixteen contract-kind schemas name their definition block's
form in a section description — TypeSpec, SDL, protobuf, TOML, the host
language — and nothing reads a section description. The validator's one
form declaration, `block-language`, is a fatal error outside `yaml` and
`json`, so those nine cannot use it. A contract of those kinds may carry
a definition block in any form at all and pass every check.

## Motivation

Twelve of the sixteen kinds carry a fenced definition section. The other
four — `contract-authz`, `contract-binary-format`,
`contract-telemetry` and `contract-ui` — define entirely in markdown
tables, and their schemas declare `columns`, which `check_columns`
(`crates/lib/superdev-core/src/validate/schema/document.rs:1316-1350`)
requires exactly and in order. Their form is already declared where a
machine reads it, and already checked.

Of the twelve, three declare a `block-language` the validator parses:
`contract-cli` and `contract-deployment` as yaml, `contract-mcp` as
json. Their blocks are parsed, and their declared keys and entry keys
checked.

The remaining nine name their form only in prose: `contract-rest`,
`contract-graphql`, `contract-rpc`, `contract-events`, `contract-data`,
`contract-interface`, `contract-library`, `contract-text-format`, and
`contract-config`, whose `File` section takes a fenced block beside the
`Settings` table that is checked. Two consequences follow, both measured
in the tree.

Nothing checks the fence tag. `content: code` is satisfied by the first
fence in the section body whatever its tag (`document.rs:1254-1262`),
and the tag check at `document.rs:848-869` runs only for a declared
`block-language`. This repository already carries an instance:
`contract-008`'s Shape section is a `text` block, and
`contract-text-format` names JSON Schema, a TOML or YAML example, a DTD
or a grammar — `text` is none of them, and the contract passes. A
`contract-rest` whose Endpoints section carried `yaml` rather than
TypeSpec would pass the same way.

The declaration is unavailable. `document.rs:360` reports a fatal
finding on any schema declaring a `block-language` outside
`BLOCK_LANGUAGES`, which is `["yaml", "json"]`
(`document.rs:82`). A kind cannot say `typespec` even to record it.

Beneath both sits the reader's problem.
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]
obliges every contract to be bound to its implementation, and for these
nine kinds a hand-written drift test is the only thing that can do it —
the validator reaches none of their content. No kind schema says so, so
a reader cannot tell from the schema which checks apply to the form it
names.

## Proposed behaviour

Every kind whose definition section takes a fenced block declares the
fence tags that section accepts, in the schema, where a machine reads
it. A contract carrying a block tagged otherwise is an error naming the
declared tag and the tag found. A kind whose form the validator does not
parse says two things in its schema: that a drift test is what binds its
definition block, and why its ecosystem needs that form.

The validator parses what it has always parsed. A declared form it
cannot read is recorded and its contents left alone, so widening the
declaration adds a check on the tag and none on the block.

## Acceptance criteria

1. [ubiquitous] THE SYSTEM SHALL accept any fence tag as a
   contract-kind schema's declared definition form, and SHALL parse a
   block's contents only where that form is `yaml` or `json`.
2. [event] WHEN a contract's definition section carries no fenced block
   tagged with a form its kind declares THE SYSTEM SHALL report an
   error naming the declared form and the tags the section carries.
3. [conditional] IF a kind declares a definition form the validator does
   not parse THE SYSTEM SHALL report no finding about that block's
   contents.
4. [conditional] IF a kind declares block keys or block entry keys
   alongside a definition form the validator does not parse THE SYSTEM
   SHALL report an error on the schema, naming the keys that bind
   nothing.
5. [ubiquitous] THE SYSTEM SHALL declare, in every contract-kind schema
   whose definition section takes a fenced block, the fence tags that
   section accepts.
6. [ubiquitous] THE SYSTEM SHALL state, in each contract-kind schema
   whose declared form the validator does not parse, that a drift test
   is what binds that section's block.
7. [ubiquitous] THE SYSTEM SHALL give each kind whose form the validator
   does not parse the reason its ecosystem needs that form.
8. [ubiquitous] THE SYSTEM SHALL leave every contract on file passing
   validation.
9. [conditional] IF a contract on file carries a fence tag its kind's
   declared forms do not admit THE SYSTEM SHALL either admit that tag or
   change the contract, and SHALL record which, so no such contract is
   made to pass by widening the declaration unexamined.

## Alternatives considered

- Convert the eleven kinds that have one onto a YAML or JSON ecosystem
  standard — OpenAPI, AsyncAPI, JSON Schema, Kaitai Struct, the
  OpenTelemetry semantic conventions, the W3C design-token format — so
  the validator parses more of every definition. Rejected: the generic
  check is completeness and not meaning, and cannot tell OpenAPI from
  any other mapping; a project already owns the tooling for its own
  ecosystem's form, so the per-ecosystem choice costs it nothing at test
  time. The live cost is the number of languages a human must learn,
  which this change would not reduce.
- A record-only field beside `block-language`, so a parsed form and a
  recorded one never share a key. Rejected: the second field would carry
  the fence-tag check anyway, which splits one declaration in two for a
  distinction criteria 3 and 6 already state.
- Leave the form in prose and add only the drift-test sentence. Rejected:
  it leaves nothing checking that a REST contract carries TypeSpec
  rather than YAML, which is the defect measured above.
- Teach the binary to parse a third fence language. Rejected: it moves
  the cost from every project to this one, and needs a parser per
  language adopted.
- Adopt one expressive IDL and emit JSON Schema beside it for the tests.
  Rejected: one authored source and two artifacts is a pipeline, and it
  puts a generator in the path the proposal set out to avoid.

## Scope

- In: widening the declared definition form to any fence tag, and the
  fence-tag check that widening makes possible.
- In: declaring the accepted form in the nine kinds that state it in
  prose.
- In: the sentence each unparsed kind carries about what binds its
  block, and the reason it keeps its form.
- Out: changing any kind's definition form. ADR-034 stands.
- Out: the four kinds that define entirely in tables; `columns` declares
  their form and the validator already checks it.
- Out: parsing any form the binary does not parse today.
- Out: whether an individual contract names the test that binds it,
  which is
  [I048][sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]'s
  to weigh.

## Comments

Raised 2026-09-01 against
[ADR-034][sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]
while framing
[I045][sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares],
on the ground that the value of a definition language is whether it can
express the interface, not whether an emitter exists for it.

Framed 2026-09-02 after a survey that answered the question against
changing anything. Two syntaxes the validator already reads can carry
eleven of the sixteen kinds through a standard the kind's own ecosystem
publishes; `contract-binary-format` is the unexpected one, because
Kaitai Struct — bit widths, endianness, conditional presence — is itself
YAML. Four kinds resist because their complete JSON description exists
only as a generated artifact: GraphQL introspection, the protobuf
descriptor set, and rustdoc JSON for the two interface kinds, which is
nightly-only and unstable. `contract-authz` is the sixteenth, and has no
published standard at all — its vocabulary is a plain mapping, which is
why it defines in tables today. A text format that is a real grammar is
the genuine exception — its production rules are encodable as JSON, and
unreadable written by hand.

What settled it is that a drift test is written in the adopting
project's own language, so the test for a form is authorable and
parseable wherever that project already owns its ecosystem's tooling.
[ADR-035][sokf:adr-035-a-schema-declares-its-definition-blocks-contract]'s
generic check is thinner than it first appeared —
`document.rs:889-922` verifies a block parses, is a mapping, and carries
its declared keys — so converting a form to gain that check buys
completeness, not meaning. ADR-034's per-ecosystem logic survives, and
the language count stays a readability cost rather than a binding one.

Left to CONTRACT-DESIGN: most of the nine name a set of forms rather
than one — `contract-rpc` takes protobuf, Thrift or a JSON-RPC schema,
`contract-events` takes JSON Schema, Protobuf, Avro or TypeSpec, and
`contract-library` takes whatever the host language is. The three
text-format contracts on file settle it as fact rather than
possibility: `contract-005` and `contract-006` carry `toml` blocks and
`contract-008` carries `text`, so one declared tag for
`contract-text-format` cannot admit all three. A single-string
declaration cannot express a set, which is why criteria 1, 2 and 5 speak
of the tags a section accepts rather than of one tag. Whether the
declaration becomes a list, stays a string with a per-contract override,
or moves to the contract itself is that phase's call.

`contract-008` is the case criterion 9 exists for. Its Shape section
holds a directory tree, which is not a file shape at all, so admitting
`text` into `contract-text-format`'s forms would settle by widening what
may instead be a contract filed under the wrong kind. Deciding that is
CONTRACT-DESIGN's, not framing's.

<!-- sokf:links -->
[sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]: /knowledge/adrs/active/adr-034-each-kind-defines-in-the-form-its-ecosystem-reads.md
[sokf:adr-035-a-schema-declares-its-definition-blocks-contract]: /knowledge/adrs/active/adr-035-a-schema-declares-its-definition-blocks-contract.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares]: /knowledge/issues/wontfix/issue-045-feature-request-drift-tests-bind-what-the-contract-declares.md
[sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]: /knowledge/issues/open/issue-048-feature-request-no-step-asks-whether-a-contract-is-bound.md
[sokf:issue-049-feature-request-a-contract-is-not-written-to-be-compared]: /knowledge/issues/open/issue-049-feature-request-a-contract-is-not-written-to-be-compared.md
