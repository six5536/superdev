---
type: Idea
id: idea-004-schemas-anchor-a-section-style
title: Schemas anchor a section's style to a known artifact
description: A section's guidance names a well-known artifact whose register the writer matches — "each `about` reads as `rg --help` prints one" — so the text is written for where it lands.
status: draft
---

# Idea: schemas anchor a section's style to a known artifact

A schema's per-section guidance gains a clause naming an artifact the writer
matches: "each `about` value reads as `rg --help` prints one". The schema
already fixes the section's structure and, through its `example`, its shape.
The anchor fixes its register, by pointing at a body of text the model has
read a great deal of rather than describing the register in prose.

## Motivation

Text a schema governs often lands somewhere with a house style of its own — a
terminal, a crate summary on docs.rs, a tool description a model selects on.
The schema says what the text must contain and how long it may be, and says
nothing about how it should read, so it comes out in the writer's default
register: even, explanatory, and wrong for the destination.

The tree already carries one of these anchors and it works.
`knowledge/schemas/changelog.md` names Keep a Changelog and Semantic
Versioning, and the changelogs written from it come out in that genre without
the schema having to describe the genre.

## Sketch

The anchor is one clause inside the `description` a section already carries —
the key `.agents/sokf/grammar.yaml` documents as "the template's own guidance
for this section". No new grammar key, no validator change, and the cost is a
sentence in each schema that wants one.

The stronger form of the clause asks for the convention before the text:
"before writing the `about` values, list the conventions for CLI help
summaries." The writer produces the style guide itself — a line length, the
imperative mood, no trailing period, no rationale — and then writes to rules
it has just stated. Following its own stated rules is more reliable than
following the schema's, and the guide comes out specific to the genre the
anchor named, which no schema author could keep current for every genre.

Candidates, where the text has a destination with a genre:

- `contract-cli`, the `about` values in the Commands block — `rg --help`.
- `contract-mcp`, the `description` values in the Tools block — text a model
  chooses a tool on, not text a person reads.
- `contract-library`, Package and Public API — a docs.rs crate summary and
  rustdoc item docs.
- `error-handling` — the way `rustc` states a diagnostic: what failed, where,
  and what to do about it.

## Trade-offs

- An anchor is the one line in a schema that no validator checks. Everything
  else there is data a tool enforces; this is an instruction the writer is
  asked to remember, and it fails silently when ignored.
- Naming an external artifact binds the schema to a recollection of it. The
  model's memory of `rg --help` is approximate and the artifact itself moves.
- Most superdev documents have no external genre — a feature plan, an ADR, an
  issue. Reaching for an anchor there imports a foreign register and fights
  the project's own grammar rules.
- The `example` block is the stronger signal wherever it can carry the
  register itself. An anchor earns its place only where the example cannot,
  which is where the register lives in wording the example shows one instance
  of.

## Open questions

- Which sections have a real external genre, and which merely look like they
  do? The list above is a guess, not a survey.
- Does an anchor beat rewriting the section's `example` to demonstrate the
  register directly?
- Which form wins: the anchor alone, or the anchor plus the instruction to
  state the conventions first? The second costs output before the text starts
  and puts a self-written guide in front of the writer.
- Does the stated guide belong anywhere after the writing — a note in the
  document, or discarded once the text is written?
- Does it belong in the section's `description`, or beside the `example` where
  the writer is already comparing against a model instance?
- README has no single house style. Is there a useful anchor for its opening
  paragraph, or does that section stay as it is?

## Next step

Try one section three ways. Write a command line from `contract-cli` as it
stands, then with the `rg --help` anchor, then with the anchor and the
instruction to state the conventions first, and compare the `about` strings.
One section answers whether the idea is worth a sweep and which form to
sweep with.
