---
type: Idea
id: idea-001-schemas-carry-a-reading-reminder
title: Schemas carry a reading reminder
description: Every schema fixes a short instruction its documents must carry, so an agent reading one is told how to treat it without opening the schema.
status: draft
---

# Idea: schemas carry a reading reminder

A schema gains a `reminder` — one or two sentences, written by the schema
author, saying what this kind of document is and how to treat it. Every
document the schema governs must carry that text, and the validator checks it
is there. An agent that opens a public contract is then told, in the document,
that it states what is promised to callers and must not be edited without a
stability note. It learns that from the document it already has, not from a
second file it would have had to know to fetch.

## Motivation

An agent reads a document mid-task, with the document in context and its
schema nowhere. The frontmatter says `type: PublicContracts`, which is a name,
not an instruction: what that type obliges lives in
`knowledge/schemas/contract-cli.md`, one `sokf_read` away, and the agent has no
reason to suspect it needs it. So the obligations that make a document type
what it is — a contract's promise, a plan's load-bearing done-markers, an
ADR's immutability once accepted — reach the reader only when someone thought
to look them up.

The knowledge base already solves the human half of this. A person browsing
`knowledge/contracts/public/` reads the index blurb and the directory name.
An agent handed one file has neither.

## Sketch

Add `reminder` to the schema contract vocabulary, beside `description` and
`line-limit`, in `.agents/sokf/grammar.yaml` under
`kinds.schema.document.keys`. The slot to carry it in the governed document
already exists: `preamble`, which the grammar defines as what sits between the
frontmatter and the first heading.

The validator then checks, for every governed document, that its preamble
carries the schema's `reminder` verbatim. Verbatim is what keeps it from
drifting into a paraphrase that no longer says what the author meant. A
template ships the line pre-filled, so a document written from one starts
compliant, and `sokf_read` returns it with the `(root)` section — which is the
section an agent gets first.

The cheaper variant reuses the schema's existing `description` rather than
adding a key. It costs nothing to specify and gets the wrong text: a
description is a summary written for an index, and the reminder wants an
imperative written for whoever is about to edit.

## Trade-offs

- Eighty-nine governed documents each gain a line, and the sweep to add them
  is one-off but real.
- Verbatim enforcement means rewording a reminder breaks every document of
  that type at once. superdev does not rewrite the user's knowledge, so the
  sweep falls to whoever changed the wording — which is a brake on improving
  a reminder after the fact.
- Reminders compete for attention. A reader meeting six of them in one task
  starts skipping them, and a skipped reminder costs its tokens and returns
  nothing.
- It duplicates the schema by design, in every document the schema governs.
  Documents are not compared by the duplication check today, so nothing
  objects; if they ever are, reminders need the exemption `skeletonConstants`
  gives the units.
- The benefit is asserted, not measured. Nothing here says how much behaviour
  actually changes.

## Open questions

- Verbatim, or a pattern the author may vary per document?
- Preamble prose, or an HTML comment — invisible when the markdown renders,
  and just as easy for a human reader to lose?
- Does it earn its place for an agent that holds `sokf_read` and the schema
  index? The reminder wins only where the agent reads the document without
  the schema, and how often that happens is not known.
- One line, or does a longer reminder pay for itself on the high-stakes types?

## Next step

Spike it on two schemas and compare. Pick one where a careless edit is
expensive — a public contract, where an unnoted change breaks a promise — and
one where it is cheap, then watch whether the reminder changes what happens
to those documents. Two schemas need no grammar change to try: the reminder
can go in the template and the schema's section descriptions first, and only
becomes a `reminder` key once it is worth enforcing.
