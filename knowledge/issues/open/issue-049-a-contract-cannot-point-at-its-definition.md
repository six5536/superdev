---
type: Issue
id: issue-049-a-contract-cannot-point-at-its-definition
title: A contract cannot include its definition from source, so every definition is a hand copy kept true by a test
description: The include block materialises a concept's body and cannot name a source region, so a contract's definition is a hand-written copy of what the code declares, bound by a drift test that exists only because the copy does, and two such copies were found unbound anyway.
kind: feature
lifecycle: open
links:
  - rel: references
    to: contract-010-interface-document-schemas
    note: Changed at CONTRACT-DESIGN — a sixth content kind, `include`, and the three `block-*` declarations withdrawn.
  - rel: references
    to: adr-041-an-include-block-materializes-a-source-region
    note: Decided at CONTRACT-DESIGN — the mechanism, criteria 1 to 7.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: Decided at CONTRACT-DESIGN — the Definition section, the binding split, doc comments as contract text; criteria 8 to 10, 16, 17.
  - rel: references
    to: adr-043-one-contract-schema-and-twelve-kinds
    note: Decided at CONTRACT-DESIGN — one schema, twelve kinds, per-kind sections as tagged rules; criteria 8, 11, 12, 15.
  - rel: references
    to: adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source
    note: Decided at CONTRACT-DESIGN — criterion 18 and the pending half of 17.
  - rel: references
    to: adr-045-a-schema-declares-variants
    note: Decided at CONTRACT-DESIGN — criteria 13 to 15.
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
    to: issue-038-the-template-format-contract-is-bound-by-no-drift-test
    note: The first hand copy found unbound by a person reading.
  - rel: references
    to: issue-043-the-cli-contracts-json-keys-are-bound-by-no-test
    note: Dissolved — no hand-written JSON block remains to leave unbound.
  - rel: references
    to: issue-044-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag
    note: Dissolved — no copy remains to compare, so no comparison to report undirected.
  - rel: references
    to: issue-046-audit-the-one-directional-drift-bindings
    note: Dissolved — the three one-directional tests compared copies, and are deleted with them.
  - rel: references
    to: issue-047-a-kinds-definition-form-is-stated-in-prose
    note: Dissolved — the materialised block's tag comes from the file's extension, and nothing parses it.
  - rel: references
    to: issue-048-no-step-asks-whether-a-contract-is-bound
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
Two copies were found unbound anyway, by a person reading. And sixteen
schemas govern what is one document shape — a definition, its
behaviour, its stability — cut sixteen ways by the language the copy
was written in.

## Context

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
([I046][sokf:issue-046-audit-the-one-directional-drift-bindings]);
two report a difference as two struct dumps
([I044][sokf:issue-044-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag]);
and two copies were unbound by any of them —
`contract-008`'s tokens and template set
([I038][sokf:issue-038-the-template-format-contract-is-bound-by-no-drift-test]),
`contract-002`'s `json` block
([I043][sokf:issue-043-the-cli-contracts-json-keys-are-bound-by-no-test]).
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

The sixteen contract-kind schemas are the same cut seen from the
schema side. Every one requires a definition section, one or more
behaviour sections and a stability section, under sixteen sets of
names: `rest` requires Authentication and `mcp` does not, though an MCP
server can have auth; `binary-format` and `text-format` differ only in
encoding; `rest`, `graphql`, `rpc` and `mcp` are one reader's question
— "the API" — split four ways by the file type the copy took. The four
kinds that define in tables — authz, telemetry, binary-format, ui —
were hand copies in table form: roles live in a policy file, metrics in
`register_counter!` calls, a byte layout in a `#[repr(C)]` struct. Once
every definition is an include from source, nothing is left to
distinguish sixteen shapes but the names of their sections.

The mechanism that fixes it is already in the binary. An include block
materialises content in place, regenerates it under `--fix`, and fails
the run when the copy is stale — for a concept. Let it name a source
region and the contract carries the declaration itself: readable in one
place, generated rather than authored, and unable to drift because the
validator will not let it.

## Behaviour

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

One schema governs every contract: a title naming its kind, a
Definition section of source includes, a Behaviour section of
normative prose with free subsections, and a Stability section. A
contract's `kind` is one of twelve, chosen by what a reader asks for
rather than by the form its definition takes — `api`, `events`, `cli`,
`library`, `interface`, `ui`, `data`, `format`, `config`, `telemetry`,
`authz`, `deployment` — carried in the frontmatter and in the id's
third segment, and the two must agree. An MCP server is an `api` over
stdio; a binary layout and a lock file are both a `format`; a module's
internal boundary is an `interface` and a published crate's surface a
`library`, because "how is this bounded" and "what can I import" are
different questions. A surface the twelve do not name is one line in
the enum and one section in the checklist.

What the sixteen schemas required section by section becomes a
checklist inside the one schema, one section per kind saying what its
Behaviour must cover — auth, errors, limits and versioning for an
`api`; ordering, delivery and replay for `events`; exit codes, streams
and prompting for a `cli`; precedence, defaults and secrets for
`config`. A writer reads the schema before writing and sees the
checklist with the shape; Behaviour carries one `###` per item that
applies, so an omission is a deletion rather than a lapse; the
judgement step below asks against the same list. Shape is the
validator's; completeness is the checklist's and the agent's.

An agent asks, when a slice that touched a contract is integrated, the
one question that is not decidable: is the marked region the whole
promised surface, is the prose complete for what the shape cannot
express — against the checklist for its kind — and could a consumer
learn the interface from the document. It reports as a judgement that
blocks nothing, and names what it checked.

This repository's nine contracts move to source includes under the one
schema, each with its kind, and the four drift tests that compared
copies are deleted. `contract_exit_codes.rs` stays: it tests behaviour.
The fifteen schemas the one replaces are deleted, with their pack
mirrors.

- The include block accepts, as its argument, a `/`-rooted repository
  path, optionally followed by `#` and a region name, beside the
  concept id it accepts today.
- When `validate --fix` runs, it materialises a source include as a
  fenced block tagged by the file's extension, holding the named region,
  or the whole file when no region is named.
- A region is bounded by a line containing `sokf:begin <name>` and a
  later line containing `sokf:end <name>`, matched by substring, and
  regions sharing a name concatenate in file order.
- When a source include is absent, empty or differs from its region, the
  validator reports an error naming the path and the region.
- When a source include names a path that does not exist, resolves
  outside the repository, or names a region its file does not carry, the
  validator reports an error stating which.
- A `sokf:generated-by` line in an included file's leading lines is
  carried into the materialised block unchanged.
- The validator parses nothing inside an included region, and reports no
  finding about the contents of any contract's definition.
- One schema governs every contract, requiring a Definition section, a
  Behaviour section and a Stability section, and no other contract
  schema ships.
- A contract's Definition section holds at least one source include, and
  a `###` subsection under Behaviour is accepted without being declared.
- When a contract's Definition section carries a fenced block outside an
  include, the validator reports an error naming the section.
- A contract's `kind` is one of `api`, `events`, `cli`, `library`,
  `interface`, `ui`, `data`, `format`, `config`, `telemetry`, `authz`
  and `deployment`, and the id's third segment equals it.
- When a contract's title does not open with its kind's display name and
  "contract:", the validator reports an error naming the kind.
- A schema may name a frontmatter key as its variant discriminator and
  tag any rule with the variants it applies to, an untagged rule
  applying to all; the validator checks a document against the rules
  its value selects in the schema's declared order.
- If a schema declares a variant discriminator, it carries one example
  per enum value, each checked against the base rules and its own
  variant's, its value equal to its key.
- The contract schema declares each kind's Behaviour sections as rules
  tagged with that kind — the required ones required — so a writer
  reading the schema sees them with the shape and the validator enforces
  them.
- The contract standard states that a doc comment inside an included
  region is contract text, and that the contract's prose carries what no
  single element can.
- The contract standard states that behaviour an include cannot reach is
  stated in prose and bound by a test, and that a `pending` marker
  applies to prose alone.
- CONTRACT-DESIGN writes a new definition element into its source
  region, behaviour unbuilt, under the approval it already requires.
- When a slice that touched a contract is integrated, the integrating
  agent reads the contract as its consumer would, and reports where an
  included region omits part of the promised surface, where the prose
  omits what the checklist for its kind requires, and where a reader
  could not learn the interface from the document.
- For each contract the agent reports, it names what it checked, and
  presents the report as a judgement that blocks nothing and is not a
  validator finding.
- If the slice touched no contract, the agent says so and reports
  nothing further.
- The judgement step ships in the pack, so a project adopting superdev
  inherits it with the schemas.
- Every contract this repository owns is defined by source includes
  under the one schema, each with its kind, and no test compares a
  hand-written copy of a definition to the code.
- A test exercises every exit code the CLI contract states.

## Scope

The mechanism, the one schema, the migration and the judgement step.

- In: the include block naming a source path and region; region
  markers; the `generated-by` line; the errors that keep an include
  current and inside the repository.
- In: one contract schema replacing sixteen — Definition, Behaviour,
  Stability — with `kind` from a closed set of twelve, and a
  hand-written block in a Definition becoming an error.
- In: schema variants — a discriminator key, a `variants` tag on any
  rule, one example per variant — and the contract schema's per-kind
  sections declared through them.
- In: deleting the fifteen kind schemas and their pack mirrors.
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
- Out: per-kind required sections. A `rest` contract with no
  Authentication passes validation; the checklist prompts it and the
  judgement step asks.
- Out: a link check reaching documentation outside the knowledge tree.

Alternatives considered:

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
- Keep the sixteen kind schemas and change only their definition
  sections. Rejected: the sections a kind requires were the copy's
  checklist, and an inconsistent one; under source includes nothing is
  left to distinguish sixteen shapes but section names, and one schema
  is less to edit than sixteen.
- Cut the kinds by audience — caller, operator, developer, user.
  Rejected: four is too coarse; a caller asks for the API, the CLI and
  the events by name.
- Keep `rest`, `graphql`, `rpc` and `mcp` as kinds. Rejected: they are
  one reader's question split by file type, and the file type is in the
  definition and the title already.
- Merge `interface` into `library`. Rejected: same shape, different
  question — "how is this module bounded" is not "what can I import" —
  and separating them costs nothing under one schema.
- An open `kind`, any slug. Rejected: it costs the id pattern and the
  checklist lookup for a case nothing on file has; adding a kind is one
  enum line and one checklist section.
- Ship the mechanism and migrate this repository's contracts later.
  Rejected: a mechanism the repository does not use on itself is not
  finished, and the migration is what makes three open defects dissolve
  rather than linger against tests about to be deleted.

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
[I047][sokf:issue-047-a-kinds-definition-form-is-stated-in-prose],
which asked how a kind names a form the validator cannot parse when the
validator now parses none.
[I048][sokf:issue-048-no-step-asks-whether-a-contract-is-bound]
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

Folded 2026-09-02, at the user's prompting: the sixteen kind schemas
collapse to one. The twelve kinds were chosen by asking what a reader
looks for and, as a check, whether the Behaviour checklist genuinely
differs — not by copying the sixteen. Three merges: `rest`, `graphql`,
`rpc` and `mcp` into `api`; `binary-format` and `text-format` into
`format`; nothing else. `events` stays out of `api` for its ordering
and delivery semantics; `interface` stays beside `library` because the
questions differ. Environment variables sit in `config`: a setting the
software reads, the environment one of its sources, precedence in
Behaviour — and a `cli` contract whose command reads one references the
`config` contract rather than restating it.

Settled at CONTRACT-DESIGN, 2026-09-02: `content: include` is a sixth
content kind in
[contract-010][sokf:contract-010-interface-document-schemas]'s
vocabulary, and the three `block-*` declarations are withdrawn; the
fence tag is the extension, mapped for `rs`, `yml`, `ts` and `py`, bare
when there is none. The decisions are
[ADR-041][sokf:adr-041-an-include-block-materializes-a-source-region],
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source],
[ADR-043][sokf:adr-043-one-contract-schema-and-twelve-kinds],
[ADR-044][sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]
and [ADR-045][sokf:adr-045-a-schema-declares-variants], superseding
ADR-032, 035, 036, 037 and 038. The last came from the user asking how
the per-kind checklist is enforced: as prose it was a nudge, and a
`variants` tag on any schema rule makes it a check without a second
schema.

Was left to CONTRACT-DESIGN: the fence tag for a file with no extension or
an unknown one; whether `resource`, which points at the implementation,
stays distinct from an include; the wording of the non-goal, which must
now say that superdev binds a definition by materialising it and the
project binds behaviour by test; and how CONTRACT-DESIGN's
declaration-only source edit sits under the approval ADR-028 requires.

<!-- sokf:links -->
[sokf:adr-027-an-include-block-materializes-shared-content-in-place]: /knowledge/adrs/active/adr-027-an-include-block-materializes-shared-content-in-place.md
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]: /knowledge/adrs/active/adr-034-each-kind-defines-in-the-form-its-ecosystem-reads.md
[sokf:adr-035-a-schema-declares-its-definition-blocks-contract]: /knowledge/adrs/deprecated/adr-035-a-schema-declares-its-definition-blocks-contract.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/deprecated/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/deprecated/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:adr-041-an-include-block-materializes-a-source-region]: /knowledge/adrs/active/adr-041-an-include-block-materializes-a-source-region.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-043-one-contract-schema-and-twelve-kinds]: /knowledge/adrs/active/adr-043-one-contract-schema-and-twelve-kinds.md
[sokf:adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source]: /knowledge/adrs/active/adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source.md
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-038-the-template-format-contract-is-bound-by-no-drift-test]: /knowledge/issues/done/issue-038-the-template-format-contract-is-bound-by-no-drift-test.md
[sokf:issue-043-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/wontfix/issue-043-the-cli-contracts-json-keys-are-bound-by-no-test.md
[sokf:issue-044-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag]: /knowledge/issues/wontfix/issue-044-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag.md
[sokf:issue-046-audit-the-one-directional-drift-bindings]: /knowledge/issues/wontfix/issue-046-audit-the-one-directional-drift-bindings.md
[sokf:issue-047-a-kinds-definition-form-is-stated-in-prose]: /knowledge/issues/wontfix/issue-047-a-kinds-definition-form-is-stated-in-prose.md
[sokf:issue-048-no-step-asks-whether-a-contract-is-bound]: /knowledge/issues/wontfix/issue-048-no-step-asks-whether-a-contract-is-bound.md
