---
type: Schema
id: schema-glossary
title: Glossary Schema
description: The domain terms the project's code and docs assume, one definition each, in knowledge/glossary.md.
---

# Glossary Schema

Structural rules for `knowledge/glossary.md`, the bundle's Glossary concept.
The document is a list of terms with no headings, so it declares a preamble
and no sections.

````yaml
target-files: "knowledge/glossary.md"
description: >
  The domain terms this project's code, issues and specs rely on, one
  definition each, in plain language or in terms already defined above.
line-limit: 800

frontmatter:
  type:
    const: Glossary
  id:
    const: glossary
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: bullet-list
  description: >
    One bullet per term: the term, then its definition in one or two
    sentences, using plain language or terms already defined above, linking the
    concept that details it. Group related terms under a short lead-in line
    when the list outgrows one screen. Every term the code, issues or specs
    rely on belongs here; synonyms the project avoids do not.

example: |
  ---
  type: Glossary
  id: glossary
  title: Domain Glossary
  description: The terms this project uses — pack, pin, bundle, manifest.
  status: stable
  ---

  - Pack — a versioned bundle of agent content fetched from a git source.
    Detailed in `knowledge/architecture.md`.
  - Pin — the exact revision a pack source resolved to, recorded in
    `superdev.lock`. A pin is a fact about a fetch that happened, never a
    claim derived from the manifest.
  - Bundle — the AOKF knowledge tree under `knowledge/`. Distinct from a
    pack: a pack may carry one, but the bundle is what the agent reads.
````
