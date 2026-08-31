---
type: Decision
id: adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema
title: A schema's example is checked in place against its own schema
description: validate reads each schema's example block as a document and runs the document check with the declaring schema handed to it, so a failure is a finding on the schema file and the example never leaves the file agents read it from.
lifecycle: active
---

# ADR-024: A schema's example is checked in place against its own schema

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

Every schema carries an `example:` block — the one part of the file an
agent copies verbatim when writing a new document — and no check reads
it. Five example ids on file broke their own schema's id pattern after a
migration changed the pattern and not the example, and a hand review
found 26 examples breaking their own frontmatter constraints. The
grammar already declares the obligation ("It must satisfy this schema"),
so the fault is a promise stated and unenforced. The document check that
would find every one of these faults exists since I018; it lacks only a
caller that feeds it the example.

## Decision

We will have `validate` read each schema's `example:` block as a
document and run the existing document check over it with the declaring
schema handed to it — no dispatch — reporting every failure as a schema
finding naming the schema file and what the example broke.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Check in place with the declaring schema | Reuses the whole document check; the example stays where agents read it; findings land on the schema | The example extraction needs its own parse step |
| Extract examples into fixture files | Checked by existing machinery with no new code | A schema whose example lives elsewhere cannot be read on its own, which defeats the example's purpose |
| Check only the example's `id` | Cheapest; covers all five faults first found | Scoped to the fault that happened to surface; sections, kinds and links stay unread |
| Leave it to hand review | No code | The measured state: five faults across four commits, found only when someone looked |

## Consequences

- Positive: a schema becomes self-testing — the checked half of the file
  no longer lends false authority to an unchecked half.
- Positive: the check reaches every schema a managed repository ships,
  because it runs wherever schemas load.
- Negative: writing a schema gets stricter — an illustrative shortcut in
  an example now fails the run.
- Follow-ups: reconcile every live example the new check faults, in
  `knowledge/schemas/` and the pack mirror alike (I022).
