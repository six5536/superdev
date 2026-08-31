---
type: Decision
id: adr-003-items-by-layout
title: A Pack's Items Are Named by Its Directory Layout
description: The unit that supersedes is an item — a whole skill directory, one project template, one document template — identified as (owning capability, kind, name) by where it sits in the pack tree rather than by a list in the pack manifest.
lifecycle: active
links:
  - rel: implements
    to: contract-007-interface-pack-resolution
---

# ADR-003: A pack's items are named by its directory layout

- Status: accepted
- Date: 2026-08-25
- Deciders: project owner

## Context

[The pack-resolution contract][sokf:contract-007-interface-pack-resolution] says a later layer
supersedes "an earlier item of the same name", which only means something
once *item* is defined. The existing code already treats a skill as a whole
directory — `skill_dir_items` materialises "each skill its whole directory:
SKILL.md, companions, harness configs", and the `custom` list releases the
directory, not a file. Project templates are whole trees; document templates
and knowledge skeletons are single files.

## Decision

We will identify an item as (owning capability, kind, name), all three read
from its position in the pack tree. The top-level directory is the owner, the
one below it the kind:

```
knowledge/skills/<name>/**      one skill,     owned by `knowledge`
knowledge/concepts/<name>.md    one skeleton,  owned by `knowledge`
knowledge/templates/<name>.md   one template,  owned by `knowledge`
skills/<name>/**                one skill,     owned by `skills`
agents/<name>.md                one scaffold,  repo-level
projects/<name>/**              one project template, repo-level
```

The layout is the declaration; `pack.toml` carries the format version and
metadata, not an item list. Superseding replaces a whole item, never part of
one, and only ever within the same owner.

The owner is part of the identity because two capabilities both write into
`.claude/skills/`, each with its own `custom` list, and
[configuration][sokf:configuration] guarantees the lists are name-guarded —
"a name in one capability's list never releases another capability's file".
A flat skill namespace would break that guarantee and `--no-<capability>`
with it.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Owner and kind from the tree | No manifest to keep in step with the tree; matches `skill_dir_items` and the `custom` list; name-guarding and `--no-<capability>` keep working untouched | A stray file in a known directory becomes an item silently |
| Owner declared in `pack.toml` | A stray file is an error; validatable; shallower tree | Two sources of truth that drift apart, which is a bug class the convention has no room for |
| Files keyed by target path | Simplest resolver | A pack could replace one file of a skill and inherit the rest, so a stock SKILL.md and a pack companion could describe different things with nothing to reveal it |
| Merge the two skill-writing capabilities first | A flat namespace would then be correct, and the duplication is worth questioning | A separate feature touching the skills capability, its `custom` list and its manifest table — outside this spec |
| Owner inferred from the item name | No tree change at all | A pack could never introduce a skill superdev does not already know, which defeats the feature |

## Consequences

- Positive: two renames bring `assets/` to this shape — `aokf/` to
  `knowledge/` and `templates/` to `projects/` — after which the first-party
  pack is the existing directory with a `pack.toml` added.
- Negative: a file dropped into a kind directory by accident becomes an item;
  the resolver reports what it found, so this is visible rather than silent.
- Negative: the tree is one level deeper than a flat kind layout, and the
  asset tree must be reorganised before the first pack can be cut.
- The knowledge-skeleton line above reads `knowledge/concepts/<name>.md`;
  [ADR-010][sokf:adr-010-concepts-entry-is-the-item] widens it to any entry under
  `concepts/`, file or directory, because three shipped scaffolds are not one
  `.md` each.
- Follow-ups: a pack may not carry `PROJECT.md`. The
  [glossary][sokf:glossary] reserves it as the project's own extension layer
  — "superdev never writes or tracks the file" — so shipping one would take it
  under management and break that contract. The resolver rejects it by path,
  as it rejects the capability instruction files and the AOKF spec.

<!-- sokf:links -->
[sokf:adr-010-concepts-entry-is-the-item]: /knowledge/adrs/active/adr-010-concepts-entry-is-the-item.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:glossary]: /knowledge/glossary.md
