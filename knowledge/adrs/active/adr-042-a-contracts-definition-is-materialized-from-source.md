---
type: Decision
id: adr-042-a-contracts-definition-is-materialized-from-source
title: A contract's definition is materialized from source
description: A contract's Definition section holds source includes and nothing authored — superdev binds the definition by materialising it and failing the run when it is stale, the project binds behaviour by test, and a doc comment inside an included region is contract text.
lifecycle: active
links:
  - rel: supersedes
    to: adr-035-a-schema-declares-its-definition-blocks-contract
    note: Nothing parses a definition now, for any kind; the three block declarations go with the check.
  - rel: supersedes
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: The obligation to bind survives with its mechanism decided — materialisation for the definition, which superdev supplies; a test for behaviour, which the project does.
  - rel: references
    to: adr-033-a-contract-defines-its-interface
    note: Stands — a contract still carries a machine-readable definition block; the block is generated rather than authored.
  - rel: references
    to: adr-034-each-kind-defines-in-the-form-its-ecosystem-reads
    note: Taken at its word — the form the ecosystem reads is the source, and the contract carries the source.
  - rel: references
    to: adr-041-an-include-block-materializes-a-source-region
    note: The mechanism the Definition section is built on.
---

# ADR-042: A contract's definition is materialized from source

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

A contract is the outside of the AI-written black box: the one place a
person or an agent reads what an interface promises without reading
the code. It must be readable in one place and it must be true. Under
[ADR-033][sokf:adr-033-a-contract-defines-its-interface] it was
readable, because the definition block sat in the document; under
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation] it
was true only as far as a drift test made it so, because the block was
a hand-written copy of what the code declares.

Every copy needed a test, and each test compared its copy by whatever
the framework allowed — clap introspection, signature text-matching,
parsing the shipped file. Three of four bound in one direction; two
printed struct dumps; two copies were unbound by any of them and were
found by a person reading. Meanwhile
[ADR-035][sokf:adr-035-a-schema-declares-its-definition-blocks-contract]'s
block check parsed the three kinds whose copy happened to be YAML or JSON and none of the
other thirteen, because the binary parses nothing else — a check
applied by accident of parser rather than by principle, and one whose
every finding the drift test also reports.

[ADR-041][sokf:adr-041-an-include-block-materializes-a-source-region]
lets an include block name a source region. A contract whose
definition is such an include is readable in place, generated rather
than authored, and cannot drift because `validate` fails the run when
it does.

## Decision

A contract's Definition section holds one or more source includes and
nothing authored. A fenced block there outside an include is an error.
The section rule declares `content: include`, a sixth content kind
beside the five: it is satisfied by at least one include block naming
a source path. The `block-language`, `block-keys` and
`block-entry-keys` declarations are withdrawn; the validator parses
nothing inside a definition, for any kind.

The obligation to bind a contract to its implementation is met in two
halves. superdev binds the definition, by materialising it and failing
the run when the copy is stale — the binding is the include, decidable
at every edit, in this repository and every managed one. The project
binds behaviour — exit codes, error semantics, ordering, whatever the
declaration cannot say — by a test that exercises it. There is no
drift test: a test that compared a hand-written copy to the code has
nothing left to compare.

A doc comment inside an included region is contract text. It arrives
in the document with the element it annotates, so a `MUST` on a
field's doc comment binds as a `MUST` in the Behaviour section does,
and it cannot drift from the contract because it is the contract. The
contract's own prose carries what no single element can say —
stability, consumers, behaviour across elements — and what no include
reaches.

The `resource` frontmatter key keeps its meaning — the implementation
the contract describes, for a reader who wants the code — and is not
an include. An include names a declaration; `resource` names what
implements it; the two may be the same file and are never the same
thing.

Where the built-from source is unreadable as a surface — a migrations
directory, a Terraform tree — the project generates a rendering, marks
it `sokf:generated-by`, and the contract includes the rendering.
Keeping the rendering current is the project's, by the same
regenerate-and-diff pattern as a golden file.
[ADR-034][sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]
holds throughout: the form the ecosystem reads is the source, and the
contract carries the source.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Materialised source include | Readable in one place; generated; cannot drift; the binding is decidable and superdev supplies it; no drift test to write | A marker pair in the source; a region's boundary is the author's to draw, so what is not marked is not promised |
| Hand-written block and a drift test per contract (ADR-036) | No mechanism beyond a test | Every copy needs a test the project writes in a language superdev cannot read; the copies on file drifted anyway |
| A reference the reader follows, nothing materialised | No copy at all | The definition moves into the code tree, so the contract stops being readable in one place |
| Validate more of the block (extend ADR-035) | Earlier findings for the kinds it reaches | Reaches two forms of sixteen, by accident of parser; every finding the drift test also makes |
| Promises only in the contract's prose; doc comments are usage notes | One place for every promise | The developer writes the rule away from the element it rules, and a `MUST` that lands in a doc comment is quietly non-binding |
| Superdev runs the project's generator | No intermediate rendering file | Arbitrary commands on the hook's path at every edit |

## Consequences

- Positive: a contract cannot be wrong about its definition; the
  binding obligation is met by machinery that runs at every edit; a
  project adopting superdev inherits the binding with the schema
  rather than an obligation to write one; nothing is written twice.
- Negative: the source carries marker pairs; a promise the author
  forgets to mark is not in the contract, which the judgement step
  asks after and no check can; a contract's readable form is whatever
  the source is, so an unreadable source needs a generated rendering.
- Follow-ups: the contract standard states the doc-comment rule and the
  `generated-by` convention; contract-010 gains the sixth content kind
  and loses the three block declarations; this repository's contracts
  move to includes and its four copy-comparing tests are deleted; the
  non-goal in `constraints-non-goals` says what superdev now supplies.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]: /knowledge/adrs/active/adr-034-each-kind-defines-in-the-form-its-ecosystem-reads.md
[sokf:adr-035-a-schema-declares-its-definition-blocks-contract]: /knowledge/adrs/deprecated/adr-035-a-schema-declares-its-definition-blocks-contract.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/deprecated/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-041-an-include-block-materializes-a-source-region]: /knowledge/adrs/active/adr-041-an-include-block-materializes-a-source-region.md
