---
type: Decision
id: adr-026-a-named-document-is-checked-with-bare-run-parity
title: A named document is checked with bare-run parity
description: validate dispatches a path argument by its frontmatter type or a schema's target-files glob and reports for that file exactly the findings a bare run gives it — the run reads the knowledge and the schema set to do so, and the grammar's fallback kind applies only to a file nothing claims.
lifecycle: active
---

# ADR-026: A named document is checked with bare-run parity

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

The path argument predates documents having schemas. A file named on
the command line takes the grammar's fallback kind, so a knowledge
concept is reported as a malformed skill (I019), and the schema half
never runs at all: its candidate list and its schema set are built only
by runs that read the knowledge, which a run naming one file does not.
Fixing the misdispatch forces a choice about how much of the bare run a
named path buys — some findings, such as whether a `sokf:` link's
target exists, are decidable only with the whole tree in hand.

## Decision

We will dispatch a named document by its frontmatter `type` or a
schema's `target-files` glob and report it with bare-run parity: the
run reads the knowledge and the schema set, and the named file gets
exactly the findings a bare run gives it — schema, filing and link
findings alike — and no findings about any other file. The grammar's
fallback kind applies only to a file no schema and no grammar kind
claims, which keeps a skill outside the roots checkable.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Bare-run parity | The two runs cannot disagree about a file; the promise is testable as a diff of their findings | A named run reads the whole knowledge and schema set, far more than it names |
| Schema-only, self-contained | Reads only the file and the schemas; faster | The same file can pass named and fail in the bare run — the disagreement the parity promise exists to prevent |
| Keep the fallback (status quo) | No change | The check a path argument most obviously invites is the one it cannot run |

## Consequences

- Positive: `validate <path>` becomes the "check this file" verb a
  reader expects, and parity makes the criterion pass/fail without
  interpretation.
- Negative: a named document run is no faster than a bare run — the
  cost of never disagreeing with it.
- Follow-ups: contract-002's PATH sentence stops promising "only what
  it names is read" and promises what is reported instead.
