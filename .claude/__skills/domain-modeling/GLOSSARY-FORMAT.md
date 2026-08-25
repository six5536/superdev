# Glossary Format

The glossary is the AOKF concept `knowledge/glossary.md`, seeded by
`superdev init`. This file describes its body in a dev repo; a teaching
workspace's glossary follows `/teach`'s own format instead — the two
never coexist, since a teaching workspace is its own repo.

## Structure

```md
---
type: Reference
id: glossary
title: Domain Glossary
description: The terms this project's domain uses.
---

# Language

- **Order** — a customer's confirmed request for goods. _Avoid_:
  purchase, transaction.
- **Invoice** — a request for payment sent to a customer after
  delivery. _Avoid_: bill, payment request.
```

## Rules

- **Be opinionated.** When multiple words exist for the same concept, pick the best one and list the others under _Avoid_.
- **Keep definitions tight.** One or two sentences max. Define what it IS, not what it does.
- **Only include terms specific to this project's context.** General programming concepts (timeouts, error types, utility patterns) don't belong even if the project uses them extensively. Before adding a term, ask: is this a concept unique to this context, or a general programming concept? Only the former belongs.
- **Group terms under subheadings** when natural clusters emerge. If all terms belong to a single cohesive area, a flat list is fine.

## When one glossary stops being enough

Split only when the same word means different things in different areas. Each area then gets its own glossary concept (e.g. `knowledge/ordering-glossary.md`), linked `part-of` to the subsystem or component concept that owns that language, with the mirroring body link the AOKF spec requires. `aokf_graph` renders the resulting context map; there is no separate map file. With multiple glossaries, infer which one the current topic belongs to; if unclear, ask.
