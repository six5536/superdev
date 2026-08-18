# Spec Format

Specs are AOKF concepts under `knowledge/specs/`, one file per spec,
named `Snnn-<topic>-design.md` (e.g. `S004-checkout-retries-design.md`).
Scan the directory for the highest number and increment by one. The `id`
is the stable link target and never changes, even when the file is
renamed. Add each new spec to `knowledge/specs/index.md`.

A spec is written for a reader with **no conversation context**: the
implementing agent may be a fresh session that never saw the discussion.
Everything it needs is in the spec or reachable from it — link related
concepts and Decision concepts (AOKF `links` plus the mirroring body
link) rather than restating them.

## Template

```md
---
type: Spec
id: spec-{topic}
title: {Title}
description: {One line: what this spec decides.}
status: draft
---

# Problem

# Solution

# Behaviour

# Design decisions

# Testing

# Out of scope

# Open questions
```

Scale each section to the change — a small spec can fit on a page. Keep
every section heading, even when its content is one line: an empty
section is a statement, a missing one is an oversight.

## Section rules

- **Problem** — from the perspective of whoever feels it, and why it
  matters now. No solution language; if the problem can't be stated
  without the solution, the problem isn't understood yet.
- **Solution** — the decided approach in a few sentences: the shape,
  not the steps.
- **Behaviour** — a numbered, exhaustive list of observable behaviours,
  each one checkable by the implementing agent. Use user-story form
  ("As a <actor>, I want <feature>, so that <benefit>") when the change
  is user-facing; plain behaviour statements otherwise. This list is
  the implementer's checklist and the reviewer's yardstick: behaviour
  not listed here is not in scope.
- **Design decisions** — every decision with its reason: module
  boundaries, interfaces, schema changes, API contracts, technical
  clarifications. The *why* is what stops a later agent "fixing" a
  deliberate choice. No file paths or code snippets — they rot.
  Exception: a prototype-derived snippet that encodes a decision more
  precisely than prose (state machine, schema, type shape), trimmed to
  the decision-rich part and marked as coming from a prototype.
- **Testing** — the seams under test (as confirmed with the user), what
  makes a good test here (external behaviour, not implementation
  details), and prior art: similar tests already in the codebase.
- **Out of scope** — the explicit no-s, with reasons where non-obvious.
  As valuable as the yes-s: this is what stops scope creep in a fresh
  session.
- **Open questions** — what is deliberately left to implementation
  judgement, or `None`. Never omit the section: an absent answer must
  be a decision, not a gap.

## Lifecycle

`status: draft` while in flight; `stable` once implemented. A spec that
replaces an earlier one declares a `supersedes` link (with the
mirroring body link) and the old spec gets `status: deprecated`. Specs
are permanent: when one lands, move its durable knowledge into the core
concepts and keep the spec as the record of why.
