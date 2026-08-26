---
type: Decision
id: adr-014-a-symlink-in-a-pack-is-refused
title: A Symlink in a Pack Is Refused, and Git Decides What One Is
description: A symlink anywhere in a pack fails the run naming the path, and for a fetched pack the index mode decides rather than the filesystem, because Windows checks a link out as a plain file and the same rev would otherwise digest differently there.
status: stable
links:
  - rel: references
    to: spec-content-packs
  - rel: relates-to
    to: adr-003-items-by-layout
---

# ADR-014: A symlink in a pack is refused, and git decides what one is

- Status: accepted
- Date: 2026-08-26
- Deciders: project owner

## Context

A pack's tree is its declaration: what sits under `knowledge/skills/` is what
the pack ships ([ADR-003](D003-items-by-layout.md)), and [S014](../specs/S014-content-packs-design.md) has a pack
resolve whole or not at all. A symlink inside that
tree is a file whose content lives somewhere the pack does not cover.
Following one let a pack name a link to any readable file on the machine and
have superdev write that file's contents into the working tree
([I008](../issues/I008-a-symlinked-file-in-a-pack-is-followed.md)), so every
link is now skipped.

Skipped, and nothing said. A pack author who dedupes an item with a link gets
a pack that resolves clean and is missing that item:

```
$ superdev sync           # writes every item except `foo`, says nothing
$ superdev status --drift # exit 0
```

That contradicts what `read_pack` says two functions above it: a pack
carrying a refused file contributes nothing rather than contributing most of
itself. A skipped link is precisely a pack contributing most of itself
([I009](../issues/I009-a-skipped-symlink-says-nothing.md)).

There is a second half, and it is why this is worth deciding rather than
patching. A symlink does not survive a checkout the same way everywhere. On
Windows without `core.symlinks` — which this repository's own `build.rs`
already depends on understanding — git writes the entry as a small plain file
holding the target's path. `symlink_metadata` sees an ordinary file, so the
link is neither skipped nor refused: it is read as content and enters the
digest. A lock written on Linux for a pack containing a link therefore fails
on Windows with

```
resolved to different bytes than the lock recorded
```

which is true, unactionable, and names nothing a reader could act on.
Refusing by filesystem type on unix does not fix it, because Windows never
sees a link to refuse.

## Decision

A symlink anywhere in a pack fails the run, naming the path, the way the pack
root and `pack.toml` already do. One rule for the whole tree, and `read_pack`
means what it says: a pack resolves whole or not at all.

What counts as a symlink is decided by whoever actually knows.

- **A fetched pack: git's index.** After the checkout and before anything is
  read or digested, superdev asks git for the pack subtree's entries and
  their modes. Mode `120000` is a symlink and is refused, whatever the
  filesystem was able to materialise. Mode `160000` — a gitlink, a submodule
  — is refused with it: a shallow sparse clone leaves that directory empty,
  so the pack would ship an empty item and say nothing, which is the same
  failure in a different costume.
- **A path pack: the filesystem.** A directory on this machine was never
  checked out, so there is no index to ask and no second platform to
  disagree with. `symlink_metadata` decides, as it does now.

The filesystem check stays for a fetched pack too, at read time. Git's answer
is taken at fetch, against the checkout; the cache is read later and has no
index of its own. Two checks, each authoritative where it runs.

The digest becomes platform-independent for the same reason the refusal does:
a link is refused on both platforms before any byte of it is hashed, so there
is no tree that digests one way on Linux and another on Windows.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Refuse, git deciding for a fetched pack | One rule for the whole tree; matches what `read_pack` promises; the author learns at the pack, naming the path; fixes the cross-platform digest split, which no filesystem check can | One extra git call per fetch; narrows what a pack may contain |
| Refuse, filesystem check only | Uses the check already written; no extra call | A pack with a link still digests differently on Windows, so a lock written on Linux fails there with an error naming neither cause nor cure |
| Report each skipped link and carry on | A pack with a link still resolves; the author is told | The pack ships incomplete, which is the thing `read_pack` says it never does, and the report arrives at the user of the pack rather than its author |
| Follow a link that stays inside the pack | Dedupe works as the author intended | Containment is exactly the check that is easy to get subtly wrong, and getting it wrong is [I008](../issues/I008-a-symlinked-file-in-a-pack-is-followed.md) again |
| Materialise an internal link as a copy of its target | Dedupe works, no escape possible | Changes what the digest covers depending on how the tree was written, so two spellings of one pack digest differently |
| Leave it skipped and silent | No change | An item a pack meant to ship is simply absent, with `sync` silent and `status --drift` green |

## Consequences

- Positive: a pack either resolves whole or names the file that stopped it.
  There is no third outcome where it resolves and is incomplete.
- Positive: one pack digests the same on Linux and Windows, so a committed
  lock is portable — which is what the lock being committed already assumed.
- Positive: a submodule in a pack stops being an empty directory nobody was
  told about.
- Negative: it narrows what a pack may contain, and a pack in the wild that
  uses a link now fails where it previously resolved short. superdev's own
  pack ships none. The failure names the path, which the silent version did
  not.
- Negative: a pack author who wanted dedupe has no answer but copying the
  file. That is a real cost, and the alternative is a containment check that
  has already been got wrong once.
- Negative: one extra git call per fetch, through the same seam as the rest.
- Follow-ups: [C001](../contracts/C001-content-packs.md) states the rule
  beside `REJECTED`; [security-requirements](../security-requirements.md)
  restates the symlink guarantee as a refusal rather than a skip.
