---
type: FeatureRequest
id: issue-049-feature-request-a-contract-cannot-point-at-its-definition
title: A contract cannot include its definition from source, so every definition is a hand copy kept true by a test
description: The include block materialises a concept's body and cannot name a source region, so a contract's definition is a hand-written copy of what the code declares, bound by a drift test that exists only because the copy does, and two such copies were found unbound anyway.
lifecycle: open
links:
  - rel: references
    to: adr-027-an-include-block-materializes-shared-content-in-place
    note: The mechanism this extends — an include names a source region as well as a concept.
  - rel: references
    to: adr-033-a-contract-defines-its-interface
    note: Amended — the definition block is materialised from source rather than authored.
  - rel: references
    to: adr-034-each-kind-defines-in-the-form-its-ecosystem-reads
    note: Taken at its word — the form the ecosystem reads is the source, and the contract carries the source.
  - rel: references
    to: adr-035-a-schema-declares-its-definition-blocks-contract
    note: Retired — the validator parses nothing inside a definition, for any kind.
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: Rewritten — a definition is bound by materialisation, behaviour by test; superdev supplies the first.
  - rel: references
    to: adr-038-a-contract-may-promise-what-is-not-built-yet
    note: Narrowed — a definition element cannot be ahead of its source, so pending applies to prose alone.
  - rel: references
    to: adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate
    note: A stale include is decidable, so it is an error and the turn cannot end on it.
  - rel: references
    to: issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test
    note: The first hand copy found unbound by a person reading.
  - rel: references
    to: issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test
    note: Dissolved — no hand-written JSON block remains to leave unbound.
  - rel: references
    to: issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag
    note: Dissolved — no copy remains to compare, so no comparison to report undirected.
  - rel: references
    to: issue-046-chore-audit-the-one-directional-drift-bindings
    note: Dissolved — the three one-directional tests compared copies, and are deleted with them.
  - rel: references
    to: issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose
    note: Dissolved — the materialised block's tag comes from the file's extension, and nothing parses it.
  - rel: references
    to: issue-048-feature-request-no-step-asks-whether-a-contract-is-bound
    note: Folded in — whether a definition is bound is decidable now; the agent's question narrows to whether the region is the whole surface.
---

# Feature: a contract cannot include its definition from source

## Summary

[ADR-027][sokf:adr-027-an-include-block-materializes-shared-content-in-place]'s
include block splices a concept's body into a document and errors when
the copy goes stale. It cannot name a source file. So every contract's
definition is a copy written by hand — `contract-002`'s YAML copies
what clap knows, `contract-007`'s Rust copies signatures out of
`planner.rs` — and four drift tests exist to keep four copies honest.
Two copies were found unbound anyway, by a person reading.

## Motivation

A contract is the outside of the AI-written black box: the one place a
person or an agent looks to see what an interface promises without
reading the code. It has to be readable in one place, and it has to be
true. Today it is one or the other.

Every hand-written definition block is a copy of something the code
already declares. The CLI contract copies the clap tree; the MCP
contract copies the tool registrations; the three interface contracts
copy signatures; the two format contracts copy the shape a `serde`
struct parses. Each copy needs a drift test, and each test compares a
copy to its original in whatever way the framework permits: `contract.rs`
introspects clap, `contract_interfaces.rs` matches signatures against
source text, `contract_files.rs` parses the shipped file. Three of the
four bind in one direction only
([I046][sokf:issue-046-chore-audit-the-one-directional-drift-bindings]);
two report a difference as two struct dumps
([I044][sokf:issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag]);
and two copies were unbound by any of them —
`contract-008`'s tokens and template set
([I038][sokf:issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test]),
`contract-002`'s `json` block
([I043][sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]).
Every defect on this tracker about a drift test exists because a copy
exists.

The validator's block check made this worse. Three of sixteen kinds had
their block parsed and its keys checked, because their form happened to
be YAML or JSON; thirteen did not, because the binary parses nothing
else. There was no principle behind which kinds were checked, and
everything the check caught the drift test catches.

A project adopting superdev inherits all of this. Its API is served from
`openapi.yaml`; its stubs compile from `.proto` files; its database is
built from migrations. To have a contract at all it must paste a copy
of one of those into a markdown file, then write a test to keep the
paste honest, in a language superdev does not read.

The mechanism that fixes it is already in the binary. An include block
materialises content in place, regenerates it under `--fix`, and fails
the run when the copy is stale — for a concept. Let it name a source
region and the contract carries the declaration itself: readable in one
place, generated rather than authored, and unable to drift because the
validator will not let it.

## Proposed behaviour

An include block names a concept, as today, or a `/`-rooted source
path: `<!-- sokf:include /crates/app/superdev/src/main.rs#cli -->`.
`validate --fix` materialises the named region of the file as a fenced
block tagged by the file's extension; `validate` reports an error when
the block is absent, empty or differs from the region, when the path
does not exist or resolves outside the repository, or when the file
carries no region of that name. The turn cannot end on any of these
([ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]).

A region is bounded by `sokf:begin <name>` and `sokf:end <name>` on any
line, in whatever comment syntax the file uses, found by substring — so
the mechanism is the same in Rust, SQL, YAML and TypeSpec, and superdev
parses none of them. Regions sharing a name concatenate in file order,
so a surface scattered through one file is one include. A path with no
`#` includes the whole file. The source declares its own contract
boundary, which is the point: whoever writes the code marks what is
promised.

Every contract kind's definition section is one or more source
includes. A fenced block outside an include in a definition section is
an error: there is no hand-written definition any more, so there is
nothing for a drift test to compare. What the validator checked inside
a block it no longer checks; what it can decide — that the copy is
current — it decides at every edit.

Where the built-from source is unreadable as a surface — forty
migrations, twenty Terraform files — the project generates a rendering
and the contract includes that. The generator writes
`sokf:generated-by <what>` in the file's leading lines, the include
carries it through, and a reader sees what they are looking at. Keeping
the rendering current is the project's, by the same regenerate-and-diff
pattern this repository's goldens use; superdev checks only that the
include matches the file.

A doc comment inside an included region is contract text: it arrives in
the document with the element it annotates, so a `MUST` on a field's
doc comment binds as a `MUST` in the Behaviour section does, and cannot
drift from the contract because it is the contract. The contract's own
prose carries what no single element can say — stability, consumers,
behaviour across elements — and what no include can reach: exit codes
spread through the code, error semantics, ordering. Prose promises are
bound by tests of behaviour, which is what a drift test becomes.

A definition element cannot be ahead of its source. Contract-first for
a definition is declaration-first in source: CONTRACT-DESIGN adds the
field or the path to the marked region with its behaviour unbuilt, the
include shows it, and BUILD implements behind it. The `pending` marker
([ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet])
narrows to prose, where a promise can still run ahead of its code, and
accept still refuses a contract carrying one.

An agent asks, when a slice that touched a contract is integrated, the
one question that is not decidable: is the marked region the whole
promised surface, is the prose complete for what the shape cannot
express, and could a consumer learn the interface from the document. It
reports as a judgement that blocks nothing, and names what it checked.

This repository's nine contracts move to source includes, and the four
drift tests that compared copies are deleted. `contract_exit_codes.rs`
stays: it tests behaviour.

## Acceptance criteria

1. [ubiquitous] THE SYSTEM SHALL accept, as an include block's argument,
   a `/`-rooted repository path, optionally followed by `#` and a
   region name, beside the concept id it accepts today.
2. [event] WHEN `validate --fix` runs THE SYSTEM SHALL materialise a
   source include as a fenced block tagged by the file's extension,
   holding the named region, or the whole file when no region is named.
3. [ubiquitous] THE SYSTEM SHALL bound a region by a line containing
   `sokf:begin <name>` and a later line containing `sokf:end <name>`,
   matched by substring, and SHALL concatenate regions sharing a name
   in file order.
4. [event] WHEN a source include is absent, empty or differs from its
   region THE SYSTEM SHALL report an error naming the path and the
   region.
5. [event] WHEN a source include names a path that does not exist,
   resolves outside the repository, or names a region its file does not
   carry THE SYSTEM SHALL report an error stating which.
6. [ubiquitous] THE SYSTEM SHALL carry a `sokf:generated-by` line from
   an included file's leading lines into the materialised block
   unchanged.
7. [ubiquitous] THE SYSTEM SHALL parse nothing inside an included
   region, and SHALL report no finding about the contents of any
   contract's definition.
8. [ubiquitous] THE SYSTEM SHALL require, in every contract kind's
   definition section, at least one source include.
9. [event] WHEN a contract's definition section carries a fenced block
   outside an include THE SYSTEM SHALL report an error naming the
   section.
10. [ubiquitous] THE SYSTEM SHALL keep every section a contract kind
    requires today, so the checklist for each area of concern is
    unchanged.
11. [ubiquitous] THE SYSTEM SHALL state in the contract standard that a
    doc comment inside an included region is contract text, and that
    the contract's prose carries what no single element can.
12. [ubiquitous] THE SYSTEM SHALL state in the contract standard that
    behaviour an include cannot reach is stated in prose and bound by a
    test, and that a `pending` marker applies to prose alone.
13. [ubiquitous] THE SYSTEM SHALL have CONTRACT-DESIGN write a new
    definition element into its source region, behaviour unbuilt, under
    the approval it already requires.
14. [event] WHEN a slice that touched a contract is integrated THE
    SYSTEM SHALL have the integrating agent read the contract as its
    consumer would, and report where an included region omits part of
    the promised surface, where the prose omits a promise the shape
    cannot express, and where a reader could not learn the interface
    from the document.
15. [ubiquitous] THE SYSTEM SHALL name, for each contract the agent
    reports, what it checked, and SHALL present the report as a
    judgement that blocks nothing and is not a validator finding.
16. [conditional] IF the slice touched no contract THE SYSTEM SHALL say
    so and report nothing further.
17. [ubiquitous] THE SYSTEM SHALL ship the judgement step in the pack,
    so a project adopting superdev inherits it with the schemas.
18. [ubiquitous] THE SYSTEM SHALL define every contract this repository
    owns by source includes, and SHALL carry no test that compares a
    hand-written copy of a definition to the code.
19. [ubiquitous] THE SYSTEM SHALL keep a test that exercises every exit
    code the CLI contract states.

## Alternatives considered

- Keep hand-written blocks and fix the four drift tests. Rejected: the
  copies are the defect, and every test that compares a copy is a test
  that exists because the copy does.
- A reference the reader follows, with no materialisation. Rejected:
  it moves the definition into the code tree, so the contract stops
  being readable in one place — the property that makes it the outside
  of the box.
- Resolve a symbol name through the code index instead of markers.
  Rejected: it binds every contract to the optional `code-index`
  capability, and the mechanism stops being language-agnostic the
  moment codegraph does not know the language.
- Line ranges instead of markers. Rejected: every edit above the range
  shifts it, and `--fix` would regenerate the wrong lines without
  noticing — the one scoping form whose check cannot catch its own
  failure.
- Have `validate --fix` run the project's generator instead of
  including its output. Rejected: it puts arbitrary project commands on
  the PostToolUse hook's path, at every edit, which the non-goal rules
  out.
- Regenerate a source include only under a separate flag, so the agent
  consciously accepts an interface change. Rejected: the include diff
  lands in the same commit as the code change and the human
  fast-forwarding `main` sees both; a flag nobody remembers buys less
  than that review does.
- Keep `pending` for definition elements as a source marker. Rejected:
  a failing behaviour test and a `todo!()` already say "not built yet"
  in the language's own terms.
- Reserve promises for the contract's prose and treat doc comments as
  usage notes. Rejected: the developer would write the rule away from
  the element it rules, and a `MUST` that landed in a doc comment would
  be quietly non-binding.
- Ship the mechanism and migrate this repository's contracts later.
  Rejected: a mechanism the repository does not use on itself is not
  finished, and the migration is what makes three open defects dissolve
  rather than linger against tests about to be deleted.

## Scope

- In: the include block naming a source path and region; region
  markers; the `generated-by` line; the errors that keep an include
  current and inside the repository.
- In: every contract kind's definition section becoming source
  includes, and a hand-written block there becoming an error.
- In: retiring the block-content check for every kind.
- In: the contract standard's statements on doc comments, prose,
  behaviour tests and `pending`.
- In: CONTRACT-DESIGN writing declarations into source.
- In: the agent's judgement step at integration, shipped in the pack.
- In: migrating this repository's nine contracts; deleting
  `contract.rs`'s drift half, `mcp.rs`'s drift test,
  `contract_files.rs` and `contract_interfaces.rs`; keeping
  `contract_exit_codes.rs`.
- In: the ADR changes named in the links, the non-goal in
  `constraints-non-goals`, and the glossary.
- Out: parsing any included content, in any language.
- Out: running any project command from the validator.
- Out: resolving symbols through the code index.
- Out: merging the kinds that were split by definition language —
  `rest`/`graphql`/`rpc`, `binary-format`/`text-format`. This removes
  the reason for the split; the merge is a smaller, later decision.
- Out: a link check reaching documentation outside the knowledge tree.

## Comments

Framed 2026-09-02 across four reframes. The first was five mechanism
issues, one per symptom. The second unified them as properties a
contract needs to be comparable. The third asked what superdev's
machinery is — a schema validator — and what a contract is for, and
concluded the definition should be a pointer at a declaration file. The
fourth, this one, came from the user's observation that the include
block already pulls content into a document without letting it drift,
and that pointing at source *and* materialising it gives readability in
one place and truth at once.

Decisions taken in the interview, each with its rejected alternatives
above: regions are bounded by markers in the source, not by symbol
resolution or line ranges; `--fix` regenerates a stale source include
and the commit diff is the review; a generated file names its generator
in a marker the include carries through; a doc comment in a region is
contract text; `pending` narrows to prose and CONTRACT-DESIGN writes
declarations into source; this repository's contracts migrate inside
the feature.

What dissolves: I043, I044 and I046, each a defect in a test that
compared a copy, and
[I047][sokf:issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose],
which asked how a kind names a form the validator cannot parse when the
validator now parses none.
[I048][sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]
is folded: whether a definition is bound is decidable once it is an
include, and the agent's question narrows to whether the region is the
whole surface.

What changes in the decisions on file.
[ADR-033][sokf:adr-033-a-contract-defines-its-interface] still requires
a machine-readable definition block; the block is now materialised
rather than authored.
[ADR-034][sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]
is taken at its word: the form the ecosystem reads is the source, and
the contract carries the source.
[ADR-035][sokf:adr-035-a-schema-declares-its-definition-blocks-contract]
is retired, since nothing parses a definition.
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation] is
rewritten: a definition is bound by materialisation, which superdev
supplies, and behaviour by test, which the project does.

Left to CONTRACT-DESIGN: the fence tag for a file with no extension or
an unknown one; whether `resource`, which points at the implementation,
stays distinct from an include; the wording of the non-goal, which must
now say that superdev binds a definition by materialising it and the
project binds behaviour by test; and how CONTRACT-DESIGN's
declaration-only source edit sits under the approval ADR-028 requires.

<!-- sokf:links -->
[sokf:adr-027-an-include-block-materializes-shared-content-in-place]: /knowledge/adrs/active/adr-027-an-include-block-materializes-shared-content-in-place.md
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]: /knowledge/adrs/active/adr-034-each-kind-defines-in-the-form-its-ecosystem-reads.md
[sokf:adr-035-a-schema-declares-its-definition-blocks-contract]: /knowledge/adrs/active/adr-035-a-schema-declares-its-definition-blocks-contract.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/active/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test]: /knowledge/issues/done/issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test.md
[sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/wontfix/issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test.md
[sokf:issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag]: /knowledge/issues/wontfix/issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag.md
[sokf:issue-046-chore-audit-the-one-directional-drift-bindings]: /knowledge/issues/wontfix/issue-046-chore-audit-the-one-directional-drift-bindings.md
[sokf:issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose]: /knowledge/issues/wontfix/issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose.md
[sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]: /knowledge/issues/wontfix/issue-048-feature-request-no-step-asks-whether-a-contract-is-bound.md
