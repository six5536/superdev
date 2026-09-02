---
type: FeatureRequest
id: issue-049-feature-request-a-contract-cannot-point-at-its-definition
title: A contract must paste its definition in, and nothing says what keeps the paste true
description: Every contract kind requires its definition inline in a fenced block, so a declaration the code is built from must be copied to be contracted, and no contract states whether construction, a currency check or a test keeps its definition honest.
lifecycle: open
links:
  - rel: references
    to: adr-033-a-contract-defines-its-interface
    note: Requires a machine-readable definition block; amended so the definition may be a reference.
  - rel: references
    to: adr-034-each-kind-defines-in-the-form-its-ecosystem-reads
    note: Strengthened — the form the ecosystem reads is usually a file, so the contract points at it.
  - rel: references
    to: adr-035-a-schema-declares-its-definition-blocks-contract
    note: Retired — the validator looks inside no definition block, for any kind.
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: Stands; construction, a currency check and a test are the three ways to meet it, and the contract now says which.
  - rel: references
    to: adr-038-a-contract-may-promise-what-is-not-built-yet
    note: A pending element is bound by the reverse assertion the marker requires, so the judgement step does not report it.
  - rel: references
    to: issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test
    note: The first unbound contract found by a person reading; no step existed to find it.
  - rel: references
    to: issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test
    note: The second, found the same way; stays open and is fixed as the defect it is.
  - rel: references
    to: adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate
    note: Draws the line between what the validator checks and what the agent judges.
  - rel: references
    to: issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose
    note: Dissolved — declaring a block's form was only needed while the validator read blocks.
  - rel: references
    to: issue-048-feature-request-no-step-asks-whether-a-contract-is-bound
    note: Folded in — its seven criteria are the judgement layer's binding half, carried as criteria 11 to 17.
---

# Feature: a contract cannot point at its definition

## Summary

Every contract kind requires its definition inline, in a fenced block.
A project whose API is served from `openapi.yaml`, whose stubs compile
from `.proto` files, or whose database is built from its migrations
must copy that declaration into a markdown file to have a contract at
all — a second source of truth, stale on the next change, which is the
condition
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]
exists to prevent. And no contract says what keeps its definition true,
so a reader cannot tell a hand copy from a file the code cannot
disagree with.

## Motivation

A contract is the outside of the AI-written black box: the one place a
person or an agent looks to see what an interface promises without
reading the code. What makes it *outside* is not where the file sits
but what the code depends on. `contract-002`'s YAML block sits in
`knowledge/` and nothing is built from it; it is a claim about the
binary, kept honest by a test. An `openapi.yaml` the server is
generated from sits in `src/` and the server cannot disagree with it.
The second is more outside than the first.

Every inline block is a copy of something. `contract-002`'s YAML copies
what clap knows; `contract-003`'s JSON copies the tool registrations.
That is why they need drift tests. The database case makes the same
fact visible: `contract-data`'s Schema section is `content: code`, so
a real project's contract would carry a paste of its migrations.

The validator's block check made this worse rather than better. Three
of sixteen kinds — `contract-cli`, `contract-deployment`,
`contract-mcp` — had their block parsed and its keys checked, because
their form happened to be YAML or JSON. Thirteen did not, because the
binary parses nothing else. There was no principle behind which kinds
were checked; there was `serde_yaml_ng` in the tree. Everything the
check caught — a command without its `exit` map — the drift test
catches anyway.

Nothing asks whether any of this holds. Two contracts have been found
unbound — `contract-008`'s tokens and template set
([I038][sokf:issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test]),
`contract-002`'s `json` block
([I043][sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test])
— and both were found by a person reading the contract beside the
tests. No check found them, because whether a binding holds is not
decidable from the tree; no step asked, because none exists.

## Proposed behaviour

A contract's definition section holds one or more references to files
in the repository, one or more fenced blocks, or both — whichever the
consumer can read the interface from. A reference targets a
**declaration** the code is built from or checked against — an OpenAPI
document, a `.proto` file, a GraphQL schema, a migrations directory —
and never the implementation, so a reader following it stays outside
the code. This is
[ADR-034][sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]
taken at its word: the form the ecosystem reads is usually a file it
already has. Where a built-from file is unreadable as an interface, a
directory of forty migrations, the contract carries a rendering
generated from it rather than a copy written by hand. Where no
declaration exists — a command line, whose definition is code — the
contract carries the block, and that block is the sole declaration.

Each part of a definition states what keeps it true: **construction**,
where the code is built from the file; a **currency check**, where the
block is generated and a test proves it current; or a named **test**,
where the block is hand-written. A reader learns from the contract
alone whether they are looking at the boundary or at a claim about it.

Quality is enforced in three layers, each by the enforcer that can
reach it. The line between the first and the third is the one
[ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]
drew: what the repository alone can decide is the validator's, and the
rest is a judgement.

The **validator** checks shape, at every edit, from the tree alone: the
kind's sections are present and normative, a definition exists, every
reference resolves, every part states its binding, and a named test
file exists. It looks inside no block, for any kind, beyond that the
fence carries a language tag.

The **project** supplies truth — that the definition matches the code —
by whichever of the three bindings it declared. superdev states the
obligation and names the strategies a project may use; it supplies none.

An **agent** supplies judgement, when a slice that touched a contract
is integrated, reading each such contract as its consumer would: whether
the definition can be read from it, whether a block duplicates a file
the same section points at, and whether the declared binding holds —
the named test compares what the contract declares, the referenced file
is what the code is built from, the currency check proves the rendering
current. For each contract it reports, it names what it checked and
what it did not find, so a reader confirms or dismisses the report
without repeating the search. A contract element marked pending
([ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet])
is bound by the reverse assertion that marker requires, and is not
reported. The report is a judgement that blocks nothing, because a
reviewer is right most of the time and a gate must be right every
time. The step ships in the pack, so a project adopting superdev
inherits the question with the obligation.

The contract points at the declaration, and nothing is copied along the
chain.

## Acceptance criteria

1. [ubiquitous] THE SYSTEM SHALL accept, as a contract's definition,
   one or more references to files in the repository, one or more
   fenced blocks, or both.
2. [event] WHEN a contract's definition section carries neither a
   reference nor a fenced block THE SYSTEM SHALL report an error naming
   the section.
3. [event] WHEN a definition reference names a path that does not exist
   in the repository THE SYSTEM SHALL report an error naming the path.
4. [ubiquitous] THE SYSTEM SHALL require each part of a contract's
   definition to state which of construction, a currency check or a
   named test keeps it true.
5. [event] WHEN a binding names a test file that does not exist THE
   SYSTEM SHALL report an error naming the path.
6. [ubiquitous] THE SYSTEM SHALL report no finding about the contents
   of a definition block, for any contract kind, beyond the absence of
   a language tag on its fence.
7. [ubiquitous] THE SYSTEM SHALL keep every section a contract kind
   requires today, so the checklist for each area of concern is
   unchanged by this feature.
8. [ubiquitous] THE SYSTEM SHALL state in the contract standard that a
   definition reference targets a declaration the code is built from or
   checked against, and never the implementation.
9. [ubiquitous] THE SYSTEM SHALL state in the contract standard that a
   block copying a file the same definition references is a defect, and
   that a built-from file unreadable as an interface is rendered from
   rather than copied.
10. [ubiquitous] THE SYSTEM SHALL name, in the contract standard, the
    strategies by which a project may bind a hand-written block —
    introspecting the framework, running the interface, parsing the
    artifact the code writes, matching declared elements against source
    — and SHALL supply an implementation of none.
11. [event] WHEN a slice that touched a contract is integrated THE
    SYSTEM SHALL have the integrating agent read that contract as its
    consumer would, and report where the definition cannot be read from
    the contract, where a block duplicates a referenced file, and where
    the declared binding does not hold.
12. [ubiquitous] THE SYSTEM SHALL name, for each contract the agent
    reports, what it checked and what it did not find, so a reader
    confirms or dismisses the report without repeating the search.
13. [ubiquitous] THE SYSTEM SHALL present that report as a judgement
    that blocks nothing, and SHALL NOT record it as a validator finding.
14. [state] WHILE a contract element is marked pending THE SYSTEM SHALL
    count the reverse assertion ADR-038 requires as that element's
    binding, and SHALL NOT report it as unbound.
15. [conditional] IF the slice touched no contract THE SYSTEM SHALL say
    so and report nothing further, so an empty report is distinguishable
    from a step that did not run.
16. [conditional] IF the agent cannot read the project's tests THE
    SYSTEM SHALL report what it could not read and SHALL NOT report the
    contracts it could not judge as unbound.
17. [ubiquitous] THE SYSTEM SHALL ship the judgement step in the pack,
    so a project adopting superdev inherits it with the schemas that
    state the obligation.
18. [ubiquitous] THE SYSTEM SHALL leave every contract on file passing
    validation, each part of each definition stating its binding.

## Alternatives considered

- Keep the definition inline and validate more of it. Rejected: the
  validator can parse two of the forms the sixteen kinds take, so any
  check it applies is applied by accident of parser rather than by
  principle, and the drift test catches everything it would.
- Validate the referenced declaration files. Rejected for the same
  reason one level out — superdev cannot parse an OpenAPI document, a
  `.proto` file or a migration, and the project's own toolchain already
  does.
- Require a drift test for every contract, reference or not. Rejected:
  a file the code is built from cannot disagree with the code, and
  [ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]
  already says construction binds more strongly than a test.
- Gate integration on the agent's judgement. Rejected: a reviewer is
  right most of the time, a gate must be right every time, and a gate
  that accuses falsely is a gate people learn to bypass.
- Reduce the sixteen kinds to one contract type with a `kind` field.
  Deferred: the kinds are right where they follow an area of concern —
  a reader asks for the database, the REST API — and wrong only where
  they follow a definition language, which is `rest`/`graphql`/`rpc`
  and `binary-format`/`text-format`. This feature removes the reason
  for those splits; merging them is a separate, smaller decision.
- Make the binding question a validator check. Rejected: whether a
  named test compares what the contract declares lives in the project's
  test suite, in a language superdev does not read, so the question
  falls on the far side of ADR-039's line. What the validator can
  decide — that the named file exists — it does; the rest is judgement.
- A line in `definition-of-done` and nothing else. Rejected: that is
  what exists today, one level of indirection away, and it caught
  neither I038 nor I043.

## Scope

- In: the definition section taking references, blocks or both, and the
  validator checks that keep them resolvable.
- In: each definition part stating its binding, and the check that a
  named test file exists.
- In: retiring the block-content check for every kind
  ([ADR-035][sokf:adr-035-a-schema-declares-its-definition-blocks-contract]),
  and amending
  [ADR-033][sokf:adr-033-a-contract-defines-its-interface] so the
  definition may be a reference.
- In: the contract standard's statements on what a reference targets,
  what counts as duplication, and the binding strategies.
- In: the agent's judgement step at integration.
- Out: supplying a harness, a generator, a drift test or a gate for a
  managed project — the non-goal stands, sharpened: superdev checks
  that a binding is named, never that it holds.
- In: the judgement step asking whether each declared binding holds,
  folded from
  [I048][sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]
  — one step, one report, the readability and duplication questions
  beside the binding one.
- Out: merging the kinds that were split by definition language.
- Out: generating `contract-002`'s block from clap. A proof of the
  model this repository could give; not required by it.
- Out: this repository's own drift tests being wrong in their reporting
  and their direction — I043, I044 and I046 are defects in tests and
  are fixed as defects.

## Comments

Framed 2026-09-02 from a conversation that stepped back three times.
The first frame was five mechanism issues, each about a different
symptom. The second unified them as a property a contract must have to
be comparable, and put four requirements on the contract's form. The
third, this one, came from asking what superdev's machinery actually is
— a schema validator — and what a contract is actually for — the
outside of the AI-written black box, read by a person and an agent who
will not read the code.

Three things fell out. superdev should look inside no definition
block, because it can parse two forms of sixteen and a check applied by
accident of parser is not a check. A definition should be a pointer
wherever a declaration the code is built from exists, because an
inline copy is a second source of truth. And the enforcer that reaches
a project superdev will never know is not the validator but the agent,
reading the contract as its consumer would.

Two things the reframe changed about the earlier one. It dropped the
requirement that each element carry a matchable name — that is the
drift test's concern, and contracts already do it wherever they define
in a keyed form. And it moved
[I047][sokf:issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose]
from superseded to dissolved: declaring a block's form was needed only
so the validator could parse the block, and it no longer does.

I048 was folded in on 2026-09-02 once the three layers were clear: its
question — does the declared binding hold — is the judgement layer's
binding half, and it lands in the same step as the readability and
duplication questions. A criterion requiring the link check to reach
the README was dropped the same day; "correctly referenced from the
docs" meant the contract knowledge referencing its declarations, which
criterion 3 already binds.

Left to CONTRACT-DESIGN: the shape of a definition reference and a
binding statement in a section rule and a contract; whether the
`resource` frontmatter key, which points at the implementation, stays
distinct from a definition reference, which must not; and the wording
of the non-goal in `constraints-non-goals`, which says superdev
supplies "no gate that one exists" while criterion 5 checks that a
named test file does.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]: /knowledge/adrs/active/adr-034-each-kind-defines-in-the-form-its-ecosystem-reads.md
[sokf:adr-035-a-schema-declares-its-definition-blocks-contract]: /knowledge/adrs/active/adr-035-a-schema-declares-its-definition-blocks-contract.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/active/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test]: /knowledge/issues/done/issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test.md
[sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/open/issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test.md
[sokf:issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose]: /knowledge/issues/wontfix/issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose.md
[sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]: /knowledge/issues/wontfix/issue-048-feature-request-no-step-asks-whether-a-contract-is-bound.md
