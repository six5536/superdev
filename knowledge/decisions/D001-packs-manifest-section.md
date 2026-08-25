---
type: Decision
id: adr-001-packs-manifest-section
title: Pack Entries Are a Top-Level Manifest Array
description: Content packs are declared as a top-level [[packs]] array in config.toml rather than as capability tables, because an absent pack means the embedded snapshot while an absent capability means disabled.
status: stable
links:
  - rel: implements
    to: spec-content-packs
---

# ADR-001: Pack entries are a top-level manifest array

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

[Externally sourced content packs](../specs/S014-content-packs-design.md)
adds a second thing a repo can want: not only which provider fills a
capability, but which content source superdev resolves skills and templates
from. `config.toml` already has a plural shape — the `skills` slot's
`[[skills]]` array-of-tables — so reusing it was the obvious first thought.

Three facts argue against it. An absent capability table means *disabled*,
which is what `init --no-<capability>` produces; an absent pack entry must
mean *the embedded snapshot*, the exact opposite. Capability names are the
user-facing flag surface, and `--no-packs` names nothing a user would want.
And a pack pin is a source and a rev, not the provider-and-version pair the
registry and `update <capability>[@<version>]` are built around.

## Decision

We will declare packs as a top-level `[[packs]]` array of tables in
`config.toml`, each entry carrying a `source` and, for a git source, a `rev`.
Capability tables keep their present meaning untouched. An absent `[[packs]]`
array parses as empty and resolves from the embedded snapshot.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Top-level `[[packs]]` array | Additive serde field; every existing manifest parses unchanged; absent-means-snapshot needs no exception carved out of the capability rules | A second plural shape in the same file |
| Extend the `[[skills]]` many-slot | Reuses the cardinality machinery whole | Packs carry project templates and knowledge skeletons, so the slot would stop being about skills; absent-means-disabled would need an exception |
| A new `content` capability | Reuses `Capability`, the registry, `update`, `--no-<capability>` | `--no-content` leaves a repo with no skills at all; a git source and rev do not fit a registry keyed by provider id and version |
| A separate `.superdev/packs.toml` | Clean separation of concerns | A fourth committed file, and "what the repo wants" split across two places the lock must reconcile |

## Consequences

- Positive: existing manifests are untouched and keep parsing; the capability
  rules in [configuration](../configuration.md) need no exception.
- Negative: `config.toml` grows a second array-of-tables idiom, so the file's
  shape is marginally less uniform.
- Follow-ups: [configuration](../configuration.md) documents the new section;
  `update` learns which entry's pin it may move (see
  [ADR-004](D004-base-pack-identity.md)).
