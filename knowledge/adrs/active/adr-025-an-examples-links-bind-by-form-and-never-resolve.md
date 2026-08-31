---
type: Decision
id: adr-025-an-examples-links-bind-by-form-and-never-resolve
title: An example's links bind by form and never resolve
description: Inside a schema's example a concept link must take the sokf id form and a path link into the knowledge is refused, but no id or target is resolved — an example's content is fictional by design, and a link outside the knowledge keeps its ordinary markdown form.
lifecycle: active
---

# ADR-025: An example's links bind by form and never resolve

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

SOKF 0.4 made body links address concepts by id, so an example that
shows a path link teaches the form the format deprecated. But an
example's content is fictional by design — its ids name documents that
do not exist — so the resolution rules that bind a real document would
refuse any example that shows a body link at all. An example may also
legitimately link outside the knowledge — a URL, a repository path —
where markdown's ordinary forms are correct.

## Decision

We will check the form of an example's body links and never their
destination: a link to a concept must take the `[text][sokf:<id>]`
reference form, a path link into the knowledge is refused, and no id or
target is resolved — a fictional `sokf:` label passes, and a link whose
target is outside the knowledge keeps its ordinary markdown form.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Form only, no resolution | Examples teach the current form; fictional content stays legal | An example can cite a `sokf:` id that never existed, unnoticed |
| Full resolution, as for a real document | One rule everywhere | Every example must cite real concepts or carry none, which changes what an example may illustrate |
| No link checks | Nothing new to build | An example keeps teaching the deprecated path form — the fault the motivation names |

## Consequences

- Positive: examples teach the id-addressed form without being chained
  to the live tree's contents.
- Positive: external links in examples stay exactly what markdown makes
  them.
- Negative: the example check's link rule differs from the real
  document check's, and the difference must be stated where both live.
- Follow-ups: none.
