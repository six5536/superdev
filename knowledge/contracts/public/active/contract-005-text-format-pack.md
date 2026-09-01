---
type: TextFormatContract
id: contract-005-text-format-pack
title: Pack Format Contract
description: What a content pack must look like for superdev to read it — pack.toml, the tree that names each item, and what is refused.
lifecycle: active
resource: /crates/lib/superdev-core/src/pack
---

# Text format contract: pack

What a content pack must look like for superdev to read it: pack.toml,
the tree that names each item, and what is refused.

## Files

`pack.toml` at the root of the pack, and the tree beneath it. A pack is
authored outside this repository — in a git repository superdev fetches by
`source` and `rev`, or in a directory on the machine — so this format is the
one superdev promises to people who write no Rust.

superdev only reads it. It resolves the pack on every `sync`, and a path
source is read afresh each run, so editing a local pack and syncing again
lands the new bytes with no rebuild. Nothing superdev does writes into a pack.

## Shape

```toml
# pack.toml at the pack root. Unknown keys within a known format are ignored.
format      = 1                    # refused when the binary does not know it
name        = "superdev-assets"
version     = "1.4.0"
description = "superdev's stock skills, templates and scaffolds"
```

The tree names all three parts of every item's identity — the owning
capability, the kind, and the name. `<name>` is the entry directly under the
kind directory; where the table shows `/**` that entry is a directory and the
item is its whole subtree:

```
pack.toml
knowledge/skills/<name>/**             → .claude/skills/<name>/**              owned
knowledge/concepts/<name>              → knowledge/<name>                      scaffold
knowledge/schemas/<name>.md            → knowledge/schemas/<name>.md           owned
knowledge/schemas/fragments/<name>.md  → knowledge/schemas/fragments/<name>.md owned
skills/<name>/**                       → .claude/skills/<name>/**              owned
agents/<name>.md                       → .agents/<name>.md                     scaffold
projects/<name>/**                     → repo root, tokenised                  scaffold
```

Owned files are rewritten every sync; scaffolds are write-once and the repo's
copy wins. Two paths are refused wherever a pack carries them —
`agents/sokf.md` and `agents/codegraph.md`, which move with the binary that
pins them — as is any file named `PROJECT.md`, which is the project's own
extension layer and never superdev's to write.

## Compatibility

An unknown key inside a known `format` is ignored, so a pack written for a
later superdev still loads as long as the format number has not moved. An
unknown `format` is refused before a single file is read, naming what this
binary supports: `pack `<name>` declares format <n>; this superdev supports
<set>`.

A pack resolves whole or not at all, and MUST NOT carry a symlink
anywhere in its tree. A refused path, an unparseable `pack.toml`, an
unknown format or a symlink fails the resolve and leaves the repository
untouched. Symlinks are decided by git's index for a fetched pack and by
the filesystem for a path pack, so the same rev resolves alike on
Windows without `core.symlinks`.

Packs layer in the order the manifest writes them, and a later item of the
same `(owner, kind, name)` wins. An author therefore replaces a stock item by
carrying the same path, and adds one by carrying a new name.

## Stability

Unreleased. `format = 1` is the only format this binary reads, and the layout
table above is what that number means. A layout change that a format-1 pack
could not survive MUST take `format = 2`, and both MUST stay readable for at
least one release; anything a format-1 pack can ignore MUST NOT require a
bump.
