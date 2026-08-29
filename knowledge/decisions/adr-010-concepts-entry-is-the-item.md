---
type: Decision
id: adr-010-concepts-entry-is-the-item
title: A Knowledge Skeleton Is Any Entry Under concepts/
description: The knowledge/concepts/ kind names its item by the entry directly beneath it — a file of any extension or a directory — because the canonical knowledge ships three scaffolds that are not one .md each.
status: stable
links:
  - rel: implements
    to: spec-014-content-packs
  - rel: relates-to
    to: adr-003-items-by-layout
---

# ADR-010: A knowledge skeleton is any entry under `concepts/`

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

[Externally sourced content packs][sokf:spec-014-content-packs]
layers content by item, and [ADR-003][sokf:adr-003-items-by-layout] reads an item's
identity out of the pack tree. [The
contract][sokf:contract-001-interface-content-packs] wrote the knowledge-skeleton kind
as `knowledge/concepts/<name>.md`. Reorganising the stock content into pack
layout showed that rule cannot describe what the `knowledge` capability already
ships. Three of its twenty-five scaffolds are not one `.md` each:

- `manifest.sokf.yaml` — the canonical knowledge manifest, not Markdown.
- `plans/index.md` and `specs/index.md` — one level deeper, because the repo
  knowledge keeps plans and specs in their own directories.

Every scaffold's target is `knowledge/` plus its path under `concepts/`, so
the subtree is an exact mirror of the canonical knowledge it seeds. Only the naming rule
was too narrow, and it was written before the tree existed to check it
against. The slice that builds `ContentSet` must enumerate every item the
components ship today, with identical bytes, so this has to be settled before
that code is written rather than worked around inside it.

## Decision

We will name a knowledge skeleton by the entry directly under
`knowledge/concepts/`, whether that entry is a file of any extension or a
directory. A directory entry is one item covering its whole subtree, exactly
as a skill directory is. Each item's files land at `knowledge/` plus their
path relative to `concepts/`.

The generalisation is confined to this kind. `knowledge/templates/<name>.md`
and `agents/<name>.md` keep their `.md` rule: both are flat directories of
Markdown documents, and nothing in the canonical knowledge suggests otherwise.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Entry under `concepts/` is the item, file or directory | Describes what ships today without special cases; `plans/` and `specs/` supersede as units, which is how a pack would want to replace them; identical to how a skill directory already behaves | A stray directory under `concepts/` becomes one item rather than several, so a pack replacing it replaces all of it |
| Keep `<name>.md`, special-case the three | The common rule stays exact | Three names hard-coded in the resolver, which is the drift the layout-as-declaration rule exists to avoid; a pack could never carry them |
| Keep `<name>.md`, flatten the canonical knowledge so every scaffold is one `.md` | The rule holds unchanged | Changes what `init` writes into every repo, breaking the byte-identity the reorganisation slices are built on, for a naming convenience |
| Any depth: every file under `concepts/` is its own item | Finest possible granularity | `plans/index.md` and `specs/index.md` would supersede independently of the directory they define, and a pack could half-replace knowledge section |
| Declare the skeleton set in `pack.toml` | A stray file is an error | Two sources of truth, rejected in ADR-003 for exactly this kind |

## Consequences

- Positive: the layout rule covers the whole shipped set, so the embedded
  snapshot and a fetched pack take one code path with no exceptions in it.
- Positive: `plans/` and `specs/` supersede as units, matching how they are
  read — an index and the directory it indexes are one thing.
- Negative: the kinds no longer share one shape; `concepts/` admits
  directories where `templates/` and `agents/` do not. The contract states
  each kind's rule rather than one rule for all.
- Negative: a directory dropped under `concepts/` by accident becomes a
  single item. As in ADR-003, the resolver reports what it found, so this is
  visible rather than silent.

<!-- sokf:links -->
[sokf:adr-003-items-by-layout]: /knowledge/decisions/adr-003-items-by-layout.md
[sokf:contract-001-interface-content-packs]: /knowledge/contracts/private/contract-001-interface-content-packs.md
[sokf:spec-014-content-packs]: /knowledge/specs/spec-014-content-packs.md
