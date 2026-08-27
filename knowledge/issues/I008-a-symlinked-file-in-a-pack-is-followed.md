---
type: Issue
id: issue-008-a-symlinked-file-in-a-pack-is-followed
title: A symlinked file in a pack is followed, copying the target's contents into the repo
description: read_dir skipped a symlinked directory but not a symlinked file, so a pack could name a link to any readable file on the machine and superdev wrote its contents into the working tree as pack content; fixed in slice 16 and hardened by P005 slices 2 and 3 into a refusal.
status: stable
tags: [done]
links:
  - rel: references
    to: spec-content-packs
  - rel: relates-to
    to: security-requirements
---

# Bug: a symlinked file in a pack is followed

## Resolved

P003 slice 16. Every symlink in a pack tree is skipped, not only a linked
directory, so no file outside the pack is read through one. The two paths the
walk never visits — the pack root and `pack.toml` — are refused outright,
because a link standing in for either means the pack is not where it says it
is; a linked manifest would otherwise have picked the format gate with bytes
no digest covers.

What the skip leaves behind is
[I009](I009-a-skipped-symlink-says-nothing.md): it is silent, so a pack that
dedupes an item with a link loses that item without saying so.

## Summary

Against [S014](../specs/S014-content-packs-design.md).

A pack's tree is walked by `read_dir`, which deliberately skips a symlinked
**directory** — but the same guard does not cover a symlinked **file**.
`path.is_dir()` follows the link and answers false, so the walk falls through
to `read_to_string`, which follows it and reads the target. The bytes are then
written into the repository as that pack item.

The written path is always in-pack, so this is not a traversal *write*; what
escapes is the *content*. A pack can name any file the user running superdev
can read and have its contents materialised into the working tree, where it is
liable to be committed. [security-requirements](../security-requirements.md)
says a pack declares no executable action and refuses rejected paths before
reading; it does not cover a path the pack reaches by link.

## Environment

- Version/commit: 0.2.0 / P003 complete (`e1ac431`)
- Platform: unix; Windows needs the symlink privilege

## Steps to reproduce

1. `printf 'SUPER-SECRET-KEY-abc123\n' > /tmp/secret.txt`
2. Build a pack whose `pack/skills/leak/SKILL.md` is a symlink to
   `/tmp/secret.txt`.
3. Pin it in a scratch repo and `superdev sync`.
4. `cat .claude/skills/leak/SKILL.md`

## Expected behaviour

A symlink in a pack is skipped whatever it points at, as a linked directory
already is.

## Actual behaviour

Step 4 prints `SUPER-SECRET-KEY-abc123`. The repo now holds a real file
carrying the secret — confirmed, not inferred.

## Root cause (if known)

`crates/lib/superdev-core/src/pack/resolve.rs:349`:

```rust
if linked && path.is_dir() {
    continue;
}
```

`linked` is computed correctly from `symlink_metadata`, but it is only acted on
for directories. The file case needs no extra syscall — `linked` is already in
hand.

## Proposed fix / workaround

- Fix: `if linked { continue; }` — skip every symlink regardless of what it
  points at. The directory comment stays true and the check gets simpler.
- Workaround: only pin packs you trust, which the trust model already asks —
  but a local-path pack is read with no digest check at all, and a git pack's
  first fetch has no recorded digest to check against, so neither catches this.

## Regression risk

`pack/resolve.rs`'s walker, and any pack that legitimately ships a symlink —
superdev's own `/pack/` does not. A test would put a symlinked file in a
fixture pack and assert it is not among the resolved files.
