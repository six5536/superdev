---
type: Decision
id: adr-013-update-proves-a-pin-before-it-writes-it
title: Update Proves a Pin Before It Writes It
description: update resolves a moved pack pin before saving the manifest and keeps the old pin when it refuses, because the manifest is what every later run reads and a pin never moves backwards.
status: stable
links:
  - rel: references
    to: spec-content-packs
  - rel: relates-to
    to: adr-009-update-queries-default-source
---

# ADR-013: `update` proves a pin before it writes it

- Status: accepted
- Date: 2026-08-26
- Deciders: project owner

## Context

[S014](../specs/S014-content-packs-design.md) gives a pack a `format` and has a binary refuse one it does not know.
[ADR-009](D009-update-queries-default-source.md) makes `update` ask the
default source for its newest release and move the pin there — the one path
by which a content fix reaches a binary that has not changed. The pin moves
on the tag alone. Nothing about `assets-vX.Y.Z` says what the pack it names
contains.

`pack.toml` carries a `format`, and a binary refuses one it does not know.
The first content release that raises it strands every older binary whose
owner types `update`:

```
$ superdev update      # writes rev = "assets-v2.0.0", saves, then syncs
pack github:six5536/superdev declares format 2; this superdev supports 1
$ superdev sync
pack github:six5536/superdev declares format 2; this superdev supports 1
$ superdev update
pack github:six5536/superdev declares format 2; this superdev supports 1
```

The state is absorbing. `update` saves the manifest before `sync` validates
anything, and the never-backwards rule — deliberate, and right for its own
purpose — means `update` cannot undo what it just did. The only way out is
hand-editing `.superdev/config.toml`.

Nobody can reach it today: no release above `format = 1` exists. It becomes
reachable the first time one is cut, which is exactly when it would reach
everyone at once. The format is the sharpest case but not the only one: a tag
that names a pack with a `REJECTED` path, an unparseable `pack.toml`, or —
once [ADR-014](D014-a-symlink-in-a-pack-is-refused.md) lands — a symlink,
all fail the same way once the pin is saved.

## Decision

`update` will move a pin only to content it has read. Having chosen a target,
it resolves that entry — fetching, as the `sync` a moment later would — and
writes the moved pin only if resolution succeeded. When resolution refuses,
the old pin stays and the refusal is reported in the line that would have
announced the move:

```
packs: github:six5536/superdev stays at assets-v1.4.0 — assets-v2.0.0
       declares format 2; this superdev supports 1
```

The run then syncs on the old pin and succeeds. This is the degradation
`update` already performs when the source cannot be reached, extended to a
source that answered with something this binary cannot use.

It is one entry that is proven, not the whole manifest, so a second pack
being broken cannot hold back a pin that is fine, and the reported reason
belongs to the entry it is reported against. `update_pins` therefore needs
the lock, which resolution reads.

The probe costs a second fetch, and it is worth being exact about why rather
than assuming the cache absorbs it. A cached pack is found by the digest the
*lock* recorded for that rev, and the lock gains that record when apply writes
it — after the sync. Bytes the probe fetched are therefore in the cache under
a digest nothing yet points at, and the sync clones the same shallow, blobless,
sparse tree again.

The price is one extra clone of an order-1MB tree, paid only on a run that
actually advances a pin. An `update` that finds nothing new probes nothing and
fetches nothing, which is almost every one of them. A refused pack leaves
nothing behind either way: only what resolved is kept.

The never-backwards rule is untouched. It does not need relaxing once a pin
can no longer arrive somewhere unreadable.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Resolve the moved entry, then save | Closes the whole class — unknown format, missing tag, refused path, unparseable manifest — not just the one that prompted it; costs no extra fetch; the rule stays "a pin only ever names content this binary read" | `update` now fetches before it decides, so the first output line comes later on a slow link |
| Save only if the whole `sync` succeeded | Atomic, and no second fetch | Not actually atomic: `sync` has already written the new pin's content into the working tree before a later step fails, so reverting the manifest leaves a repo holding content from a pin it no longer names. It also ties the pack pin to failures that have nothing to do with it — a mise install, a provisioning command — and discards the capability version moves alongside it |
| Allow a recovery move backwards | Small; no fetch before deciding | Every user meets the broken state once, and the cure is the command that caused it, which nobody would guess |
| Put the format in the tag name | Decidable from `ls-remote` alone, no fetch | A naming convention doing a manifest's job, and it only ever encodes the one property somebody remembered to encode |
| Walk back from the newest until one resolves | Always lands on the best readable release | Fetches repeatedly on the unhappy path, and quietly pins something other than what the source calls newest |
| Leave it | No change | The first format bump breaks every older binary at once, unrecoverably by any superdev command |

## Consequences

- Positive: a saved pin always names content this binary has read, so
  `.superdev/config.toml` cannot be left in a state no superdev command can
  repair.
- Positive: raising `format` becomes a safe thing to do. Older binaries stay
  where they are and say why, which is what a format field is for.
- Negative: a run that moves a pin fetches the pack twice — once to prove it,
  once in the `sync` that follows — because the cache is keyed by a digest the
  lock does not carry until apply writes it. Only moving runs pay it.
- Negative: `update` reaches the network before it prints its first pack
  line. It already did so for the query.
- Negative: `update_pins` grows a lock parameter and a dependency on
  resolution, so `pin` now sits above `resolve` within `pack`. Both are
  inside one module and neither is on a component's path.
- Neutral: a pin the user hand-edited to something unreadable is unaffected.
  `update` does not move it, and `sync` reports it — which it already does.
- Follow-ups: [api-contracts](../api-contracts.md) states what `update`
  guarantees about the pin it saves; [C001](../contracts/C001-content-packs.md)
  records the lock parameter.
