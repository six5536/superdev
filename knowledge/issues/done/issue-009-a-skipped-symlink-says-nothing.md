---
type: Issue
id: issue-009-a-skipped-symlink-says-nothing
title: A symlink inside a pack is skipped in silence, so an item a pack meant to ship quietly disappears
description: The walk dropped every symlink without reporting it, so a pack that deduped an item with a link resolved clean while that item was simply absent; fixed in P005 slices 2 and 3, which refuse a symlink anywhere in a pack and name the path.
kind: bug
lifecycle: done
links:
  - rel: references
    to: contract-007-interface-pack-resolution
---

# Bug: a skipped symlink says nothing

## Summary

Against [C007][sokf:contract-007-interface-pack-resolution].

A symlink inside a pack tree is skipped, which is what stops a pack reading a
file it does not contain ([I008][sokf:issue-008-a-symlinked-file-in-a-pack-is-followed]).
It is skipped in silence, and that is the problem: a pack author who dedupes
an item with a link — `knowledge/skills/foo/SKILL.md` pointing at a shared
file one directory up — gets a pack that resolves clean and is missing `foo`.
`sync` writes nothing about it and `status --drift` stays green.

It contradicts what `read_pack` says about itself two functions up: "a pack
carrying a refused file contributes nothing rather than contributing most of
itself". A skipped link is exactly a pack contributing most of itself.

## Context

Found at 0.2.0, P003 slice 16, on unix; on Windows without `core.symlinks`
git checks a link out as a small plain file, so it is read as content
instead — see Comments.

1. In a pack, replace `knowledge/skills/foo/SKILL.md` with a symlink to
   another file inside the same pack.
2. Pin the pack and `superdev sync`.
3. `superdev status --drift`

## Behaviour

Either the link is refused, naming it — the pack root and `pack.toml` already
are — or the run says which items were dropped and why.

Instead, step 2 writes every item except `foo` and reports nothing. Step 3
exits 0.

The root cause: `read_dir` in `crates/lib/superdev-core/src/pack/resolve.rs`
continues past a symlink with no diagnostic. Refusing was not chosen when the
skip landed because slice 16's done-check called for the pack to resolve
without the item; that was the right scope for closing the leak and the wrong
long-term answer for the author.

## Scope

The fix proposed at filing, and the way round it meanwhile:

- Fix: refuse a symlink inside a pack the way the root and the manifest are
  refused, naming the path. That makes one rule for the whole tree and
  matches `read_pack`'s stated contract. It narrows what a pack may contain,
  so it wants a moment's thought about packs in the wild — superdev's own
  ships none.
- Workaround: do not use symlinks inside a pack; copy the file.
- Regression risk: the walk is reached only from pack resolution, and the
  change adds a report where there was silence rather than altering what is
  written. A recurrence would be a symlink resolving clean again with an
  item missing, which the pack-resolution tests added in P005 slices 2 and 3
  cover directly.

## Resolution

[ADR-014][sokf:adr-014-a-symlink-in-a-pack-is-refused]. A symlink
anywhere in a pack fails the run naming the path, as the pack root and
`pack.toml` already do — one rule for the whole tree, and `read_pack` means
what it says.

The cross-platform half this issue raised is settled with it, and is why the
decision is not simply "refuse". For a fetched pack the *index* decides, not
the filesystem: mode `120000` is a symlink whatever Windows was able to
materialise, so a link is refused on both platforms before any byte of it is
hashed and the same rev digests the same either way. Mode `160000` — a
submodule, which a shallow sparse clone leaves empty — is refused with it. A
path pack has no index and no second platform, so `symlink_metadata` still
decides there, and the filesystem check stays at read time, where the cache
is read and has no index of its own.

## Comments

Related, and worth settling together: a symlink does not digest the same on
every platform. On unix it is skipped; on Windows with `core.symlinks` off —
which this repo's own CI relies on understanding, see `build.rs` — git checks
the entry out as a plain file holding the link text, so it is read as content
and enters the digest. A lock written on Linux for a pack containing a link
therefore fails on Windows with "resolved to different bytes than the lock
recorded", which is both wrong and unactionable. Refusing the link everywhere
would not fix that by itself, since Windows never sees a link to refuse.

<!-- sokf:links -->
[sokf:adr-014-a-symlink-in-a-pack-is-refused]: /knowledge/adrs/active/adr-014-a-symlink-in-a-pack-is-refused.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:issue-008-a-symlinked-file-in-a-pack-is-followed]: /knowledge/issues/done/issue-008-a-symlinked-file-in-a-pack-is-followed.md
