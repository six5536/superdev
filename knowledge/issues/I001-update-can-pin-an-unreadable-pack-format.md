---
type: Issue
id: issue-001-update-can-pin-an-unreadable-pack-format
title: update can move a pin to a pack format this binary cannot read, and cannot move it back
description: update persists the moved pin before sync validates it, and a pin never moves backwards, so a content release in a newer format leaves every later sync and update failing until the manifest is hand-edited.
status: draft
tags: [ready-for-agent]
links:
  - rel: references
    to: spec-content-packs
---

# Bug: `update` can pin a pack format this binary cannot read

## Decided

[ADR-013](../decisions/D013-update-proves-a-pin-before-it-writes-it.md).
`update` will resolve a moved pin before saving the manifest, and keep the old
pin when resolution refuses, reporting the reason in the line that would have
announced the move. It proves the one entry rather than the whole manifest, so
a second broken pack cannot hold back a pin that is fine. It costs one extra
clone on a run that actually moves a pin: the cache is found by the digest the
lock records, and apply does not write that until after the sync. An `update`
that finds nothing new probes nothing.

That closes the class rather than the case: an unknown format, a tag that does
not exist, a `REJECTED` path, an unparseable `pack.toml` and a refused symlink
all stop being things a saved pin can name. The never-backwards rule needs no
relaxing once a pin can no longer arrive somewhere unreadable.

`update_pins` gains the lock, which resolution reads. Recorded in
[C001](../contracts/C001-content-packs.md).


## Summary

Against [S014](../specs/S014-content-packs-design.md).

`superdev update` moves the default pin to the newest release the source
carries without knowing whether this binary can read it. If a content release
ever raises `pack.toml`'s `format`, every user on an older binary who types
`update` lands on a pin their binary refuses — and because the pin never moves
backwards, no superdev command can recover it. Nobody can hit this today: no
release above `format = 1` exists. It becomes reachable the first time one is
cut, which is exactly when it would hit everyone at once.

## Environment

- Version/commit: 0.2.0 / slice 11 of P003 (`4ed647f`)
- Platform: all

## Steps to reproduce

Not reproducible against a real release yet; the mechanism is:

1. Cut `assets-v2.0.0` whose `pack/pack.toml` declares `format = 2`.
2. On a binary whose `SUPPORTED_FORMATS` is `&[1]`, run `superdev update`.
3. Run `superdev sync`, then `superdev update` again.

## Expected behaviour

`update` moves the pin no further than the newest release this binary can
actually read, and says so — the same degradation it already performs when
the source cannot be reached.

## Actual behaviour

Step 2 writes `rev = "assets-v2.0.0"` to `.superdev/config.toml` and saves it,
then the `sync` that follows fails:

```
pack github:six5536/superdev declares format 2; this superdev supports 1
```

Step 3 fails identically, and so does every later run. `update` cannot undo it
because `target = current.max(floor).max(remote)` never decreases. The only
way out is hand-editing `.superdev/config.toml`.

## Root cause (if known)

`crates/lib/superdev-core/src/pack/pin.rs:95` chooses the target on the tag
alone — nothing in an `assets-vX.Y.Z` tag says what format it holds.
`crates/app/superdev/src/manage.rs:364` then calls `manifest.save` before
`sync` runs, so the unreadable pin is persisted before anything validates it,
and the never-backwards rule (deliberate, and right for its own purpose) makes
the state absorbing.

## Proposed fix / workaround

- Fix: one of — resolve the candidate pin before persisting it and keep the
  old pin when it refuses; or make the readable format range part of what
  `update` selects on, so a format-2 release is simply not a candidate for a
  format-1 binary. The second needs the format to be discoverable without a
  full fetch, so it is an interface decision rather than a local fix.
- Workaround: edit `rev` in `.superdev/config.toml` back to a readable tag.

## Regression risk

`pack/pin.rs`, `pack/manifest.rs`'s `SUPPORTED_FORMATS` gate, and
`manage.rs`'s update path. A test would cut a fixture repo tagged with a
format the binary refuses and assert the pin does not move and the run still
succeeds.
