---
type: Spec
id: spec-content-packs
title: Externally Sourced Content Packs
description: superdev's prose content becomes a versioned pack resolved from a pinned source, replacing or layering over an embedded snapshot the binary still carries, so a skill or template ships without a five-platform binary release.
status: stable
tags: [done]
links:
  - rel: relates-to
    to: spec-cli-core-blueprint-engine
  - rel: relates-to
    to: spec-knowledge-carried-skills
  - rel: relates-to
    to: spec-project-templates
  - rel: relates-to
    to: security-requirements
---

# Summary

Every file superdev writes into a managed repo is compiled into the binary
by the [blueprint engine](S001-cli-core-blueprint-engine-design.md),
so content and code share one release cadence: correcting a skill or adding
a template costs a five-platform release. This feature makes that content a
**content pack** — a versioned set resolved from a pinned **pack source** —
resolved against an **embedded snapshot** the binary still carries at the
blueprint's default pin: a pack from that same source replaces the snapshot,
a pack from anywhere else layers over it. A repo that changes nothing behaves exactly as it
does today and never touches the network; a repo that pins a newer pack gets
the newer content without a new binary.

The primary user is superdev's own author, whose content changes far faster
than the engine. Third-party packs work by consequence of the mechanism;
they are not a promise before 1.0.

# Behaviour

## The default path is unchanged

- A repo with no pack entry resolves all content from the embedded snapshot,
  now and permanently: an absent entry means the snapshot, never "disabled".
- `init`, `sync`, `status` and `update` make no network request on that path.
- `status` names the content source and the pack version it resolved from.

## Migrating a repo that predates packs

- `init` writes the blueprint's default pack entry explicitly, as it already
  writes every other provider's default version.
- An existing repo has no such entry. `sync` leaves its manifest alone beyond
  the blueprint stamp it already writes, and resolves from the snapshot the
  upgraded binary carries.
- `update` — the verb that already moves pins and rewrites the manifest — is
  what adds the explicit entry.

## Pinning a pack

- The manifest gains pack entries. Each names a source — a git URL with a
  rev, or a local filesystem path — and they are ordered.
- Naming a source is the trust decision. superdev guarantees that the bytes
  it applies are the bytes the pin resolves to, and makes no claim about
  what those bytes say.
- The first apply that sees a new pack entry resolves it, records its digest
  and per-file hashes in the lock, and writes its files into the repo.
- Where a pinned rev equals the embedded snapshot's rev, superdev resolves
  it from the snapshot and makes no network request.
- Resolved content is written as ordinary repo files and committed. Every
  later `sync` and `status` reads those committed files and the lock; none
  of them reaches the network.

## Layering

- Layer 0 is the content the binary carries: the embedded snapshot of the
  blueprint's default pack source.
- A pack entry naming that same source does not layer over the snapshot, it
  **replaces** it. The snapshot is a convenience copy of that pack at an
  older rev, not a rival content set, so pinning a different rev makes that
  rev's content the whole of layer 0 — including what it no longer carries.
- A pack entry naming any other source sits above layer 0 in manifest order
  and merges item by item: an item supersedes an earlier item of the same
  name.
- Superseding layer 0 is the ordinary case and is not reported.
- One pack superseding another is reported — the item, the pack that won,
  and the pack that was shadowed.
- An item layer 0 no longer carries, and that no pack above provides, leaves
  the repo by the orphan rule rather than persisting from the snapshot.

## What a pack may carry

- Skills and document templates as the
  [knowledge capability carries them](S009-knowledge-carried-skills-design.md),
  [project templates](S007-project-templates-design.md), knowledge skeletons,
  and the general-rules scaffolds.
- Not the capability instruction files (`.agents/aokf.md`,
  `.agents/codegraph.md`, `.agents/rtk.md`) and not the AOKF spec: each
  describes a version the binary pins or a format the compiled validator
  enforces, so it moves with the binary.
- No command execution. A pack needing work superdev cannot do as a file
  write ships a skill that performs it, which runs under the agent
  harness's permission model rather than during `sync`.

## Ownership is unchanged by provenance

- A pack-provided owned file is hashed into the lock, rewritten by `sync`,
  and drift-reported on a user edit — exactly as an embedded one.
- A pack-provided scaffold is written once and never revisited.
- The `custom` list releases an item from management whatever layer
  provided it.
- Dropping a pack entry removes its files by the existing orphan rule:
  pruned when they still hash to the locked value, released and reported
  when they do not.

## Versions

- The pack carries its own version, released on its own tag series,
  independent of the binary's.
- The blueprint records a default pack source and rev, as it records a
  default version for every other provider; the embedded snapshot is that
  pin's provenance, in place of a checksum.
- The pack manifest declares a **pack format**. A binary refuses a format it
  does not know. The format is not stable before 1.0.
- `update` moves a pack pin only when that entry names the blueprint's
  default source. A pin the user pointed elsewhere is the user's, and
  `update` reports it rather than moving it.
- For the default source, `update` asks the source for its newest release
  and moves the pin there — which may be ahead of the blueprint's default,
  since content is released without a binary. This is the path by which a
  content fix reaches a repo that has not upgraded its binary.
- With no network, `update` moves the pin no further than the blueprint's
  default and reports that it could not check for anything newer.

## Never substituting content

superdev applies the pinned content or it applies nothing. It does not fall
back to the embedded snapshot, to a cached copy, or to a different rev when
the pinned one cannot be resolved — silently applying content nobody pinned
would break the only promise the trust model makes.

# Acceptance criteria

1. Given a repo whose manifest names no pack, when `init` runs with the
   network unavailable, then it succeeds and materialises the same files it
   materialises today.
2. Given a managed repo whose manifest names no pack, when `sync` and
   `status` run with the network unavailable, then both succeed and neither
   attempts a network request.
3. Given a repo pinning a pack at the embedded snapshot's rev, when `sync`
   runs with the network unavailable, then it succeeds and no network
   request is attempted.
4. Given a repo pinning a git pack at a rev the snapshot does not carry,
   when `sync` runs, then the pack is resolved once, its digest and per-file
   hashes are recorded in the lock, and its files are written to the repo.
5. Given the repo from criterion 4 with its files committed, when `sync` and
   `status --drift` run with the network unavailable, then both succeed and
   neither attempts a network request.
6. Given a pack from a source other than the blueprint's default providing
   an item layer 0 also provides, when `sync` runs, then the pack's version
   is written and the report does not mention the substitution.
7. Given two such packs providing the same item name, when `sync` runs, then
   the later entry's version is written and the report names the item, the
   pack that won, and the pack that was shadowed.
8. Given a pack providing an item no other layer provides, when `sync` runs,
   then that item is written and managed like any other.
9. Given a pack entry naming the blueprint's default source at a rev that
   drops an item the embedded snapshot carries, when `sync` runs, then the
   repo's content matches that rev — the dropped item is removed by the
   orphan rule and is not resurrected from the snapshot.
10. Given the item in criterion 9 has been edited by the user, when `sync`
    runs, then it is left in place, dropped from the lock, and reported
    once.
11. Given a pack whose manifest declares a format the binary does not know,
    when any verb runs, then it exits non-zero naming the pack and the
    formats the binary supports, and no file is written.
12. Given a pack source that cannot be resolved — unreachable host, missing
    rev, absent local path — when `sync` runs, then it exits non-zero naming
    the pack and the reason, no file is written, and the embedded snapshot
    is not substituted.
13. Given a pack whose rev is already in the lock but now resolves to a
    different digest, when `sync` runs, then it exits non-zero naming the
    recorded and resolved digests, telling the user to re-pin the rev to
    accept the change, and writes nothing. No flag overrides this.
14. Given a pack-provided owned file a user has edited, when `status` runs,
    then it reports drift on that file exactly as it does for an embedded
    one.
15. Given a pack-provided item named in the `custom` list, when `sync` runs,
    then superdev does not write it, drops its hashes from the lock, and
    `status` reports it as unmanaged.
16. Given a repo whose pack entry is removed from the manifest, when `sync`
    runs, then that pack's unmodified files are removed and its edited files
    are left in place, dropped from the lock, and reported once.
17. Given a manifest whose pack entry names the blueprint's default source
    at a rev older than the source's newest release, when `update` runs, then
    the pin moves to that newest release, even when it is ahead of the
    blueprint's default.
18. Given a manifest whose pack entry names a source other than the
    blueprint's default, when `update` runs, then the pin is left unchanged
    and reported.
19. Given a pack declaring a command to run, when it is resolved, then the
    declaration is rejected as an unknown key for its format rather than
    executed.
20. Given a local-path pack source, when its files change on disk and `sync`
    runs, then the repo's copies are updated without rebuilding the binary.
21. Given a default-source pin and no network, when `update` runs, then the
    pin moves no further than the blueprint's default and the run reports
    that it could not check for a newer release.
22. Given a repo managed by an earlier binary and carrying no pack entry,
    when the upgraded binary runs `sync`, then it succeeds, resolves from
    that binary's snapshot, and adds no pack entry to the manifest.
23. Given the repo from criterion 22, when `update` runs, then the
    blueprint's default pack entry is written into the manifest explicitly.

# Edge cases & errors

- Network unavailable, pin equals snapshot rev → succeeds, no request made.
- Network unavailable, pin differs from snapshot rev, content not yet
  committed → exits non-zero naming the pack and the rev; nothing written.
- Network unavailable, pin differs from snapshot rev, content already
  committed → succeeds from the committed files and the lock.
- Git tag moved under a recorded rev → digest mismatch, exits non-zero; the
  user's re-pin in the manifest is the only way forward, and is itself the
  new trust decision. A commit SHA cannot move, so this reaches only tags
  and branches.
- Default-source pack pinned at a rev *older* than the snapshot → applied as
  pinned; a pin is a pin in both directions.
- Pack carries a file outside the families it may provide → rejected naming
  the file and the reason; nothing written.
- Pack carries a capability instruction file or the AOKF spec → rejected as
  above.
- Two packs shadow each other across many items → every shadowed item
  reported, not just the first.
- Manifest orders two packs identically on re-read → resolution is
  deterministic; manifest order is the only tiebreak.
- Pack manifest missing, unparseable, or missing its format key → exits
  non-zero naming the pack; nothing written.
- `status` on a pack pinned but never applied → reports it as pending and
  names `sync` as the next step; `status` never resolves a source itself.
- Source written in a form equivalent to the blueprint's default but not
  identical → compared after normalisation, so it still replaces layer 0
  rather than silently layering over it.
- Local-path source pointing outside the repo → resolved and applied;
  the lock records the digest of what was read, not the path's contents at
  any later time.

# Out of scope

- Signing, key custody, a pack registry, or superdev vouching for pack
  content. The trust model is the pin, deliberately, as Cargo's and npm's
  are.
- Command execution declared by a pack, in any form.
- Moving the capability instruction files or the AOKF spec out of the
  binary.
- Pack discovery — search, listing, an index of available packs.
- A stable pack format before 1.0.
- Any flag that accepts a changed digest without a manifest edit.
- A CLI verb for adding a pack. Editing the manifest and running `sync` is
  the whole interface.
- Fetching during `status` or `sync` for content already committed.
- Third-party packs as a supported product surface. They work; they are not
  promised.

# Open questions

Settled at interface design; kept here as pointers.

- Where pack entries live in the manifest — a top-level `[[packs]]` array
  ([ADR-001](../decisions/D001-packs-manifest-section.md)).
- How two pack sources are compared for identity — normalised, and the base
  named by `status` ([ADR-004](../decisions/D004-base-pack-identity.md)).

Still open:

- Whether the first-party pack's tag series lives on this repo or moves to
  its own repository later — the frame chose this repo; nothing here depends
  on that choice.

## Accepted

Two passes. The first checked all twenty-three criteria against the real
binary and found three gaps, none of them reachable by a criterion: the
feature was undocumented for users, a manifest could get `git` to run a
command, and a pack could read a file it did not contain. All three were cut
as slices and closed. The second pass re-checked the criteria, confirmed each
gap closed as a user meets it, and a security review found both fixes airtight
with nothing new.

Five issues stay open, each needing an interface decision before it can be
cut: [I001](../issues/I001-update-can-pin-an-unreadable-pack-format.md),
[I002](../issues/I002-no-time-bound-on-the-update-query.md),
[I003](../issues/I003-a-local-pack-cannot-remove-what-it-dropped.md),
[I004](../issues/I004-a-path-packs-digest-churns-and-is-never-checked.md) and
the remaining half of
[I007](../issues/I007-a-pack-source-reaches-git-with-no-scheme-check.md).
[I009](../issues/I009-a-skipped-symlink-says-nothing.md) is open too, a
consequence of the symlink fix rather than of the feature.

Settled during delivery:

- Whether `superdev` should dogfood by pinning its own content directory as a
  local-path pack, retiring the `asset-backport` step — **yes**, and it landed
  in this feature's last slice. The manifest pins `./pack` and an edit there
  reaches `.claude/skills/` with no rebuild.

  What the pin removed is the *pack-to-live* round trip: no rebuild stands
  between the two any more. It does not remove the *live-to-pack* one, which
  is what backporting is, and which anyone iterating on a live copy still
  does. `asset-backport` was retired and `pack-backport` took its place —
  smaller, because the pack's layout is its own declaration and a build script
  enumerates it, so a new file needs no table.

  Three limits came with the pin, filed rather than fixed:
  [I003](../issues/I003-a-local-pack-cannot-remove-what-it-dropped.md), a
  layer cannot remove what it dropped, and
  [I004](../issues/I004-a-path-packs-digest-churns-and-is-never-checked.md),
  the recorded digest churns and is checked by nothing, and
  [I005](../issues/I005-a-backport-leaves-the-lock-stale.md), a backport
  leaves the lock describing the file it replaced.

# Test plan: externally sourced content packs

## Scope

- Under test: pack resolution from git and local sources, layer precedence
  including the default source replacing layer 0 rather than stacking on it,
  shadow reporting, lock and digest recording, the no-network guarantees
  on the default and committed paths, format refusal, resolution failure,
  removal by the orphan rule, and `update`'s treatment of pack pins.
- Not under test: the AOKF validator, the MCP read side, and the existing
  component behaviour for capabilities that carry no pack content — all
  covered by their own specs and unchanged here.

## Risks driving this plan

1. **Silent substitution.** A pack fails to resolve and superdev quietly
   applies the embedded snapshot, so the repo receives content nobody
   pinned. This is the single failure that would invalidate the trust model.
2. **Network creeping into the steady state.** A request during `sync` or
   `status` breaks the CI drift gate and the "local by default" guarantee in
   [security-requirements](../security-requirements.md) that the non-PIE musl
   acceptance rests on.
3. **Wrong layer resolution.** An item that should supersede does not, one
   that should not does, a pack from the default source layers instead of
   replacing — so a removal never propagates — or the outcome depends on
   something other than manifest order.
4. **Removal damaging user work.** Dropping a pack deletes an edited file,
   or leaves an unmodified one orphaned in the repo.
5. **Breaking repos that predate packs.** A manifest with no pack entry must
   keep working untouched; the blueprint-migrations design exists to make
   that true, and this is the first feature to test it against content.

## Test cases

### Automated

| # | Case | Type | Inputs / setup | Expected result |
|---|------|------|----------------|-----------------|
| 1 | Default path, no pack | integration | Manifest with no pack entry; network blocked | Same files as today; zero network calls |
| 2 | Pin equals snapshot rev | integration | Pack pinned at the blueprint default | Resolves from snapshot; zero network calls |
| 3 | First resolve of a newer rev | integration | Local git fixture repo at a second rev | Files written; digest and per-file hashes in lock |
| 4 | Steady state after commit | integration | Repo from case 3, files committed, network blocked | `sync` and `status --drift` succeed; zero network calls |
| 5 | Pack supersedes snapshot | unit | Pack providing an existing skill name | Pack's bytes written; no shadow report emitted |
| 6 | Pack supersedes pack | unit | Two packs providing one name | Later entry wins; report names item, winner, shadowed |
| 7 | Manifest order is the only tiebreak | unit | Same two packs, order reversed | Winner flips; no other difference |
| 8 | Default source replaces layer 0 | integration | Default-source pack at a rev dropping one snapshot item | Repo matches the rev; dropped item removed, not resurrected |
| 9 | Replacement spares an edited file | integration | As 7a, dropped item edited by the user | Left in place, delocked, reported once |
| 10 | Unknown pack format | unit | Pack manifest with a future format | Non-zero exit naming pack and supported formats; nothing written |
| 11 | Unresolvable source | integration | Absent local path; unreachable git URL; missing rev | Non-zero exit per case; nothing written; snapshot not substituted |
| 12 | Moved tag / digest mismatch | integration | Fixture rev re-pointed after lock entry exists | Non-zero exit reporting mismatch; nothing written |
| 13 | Drift on a pack-provided file | integration | Hand-edit a pack-provided owned file | Reported as drift, identically to an embedded one |
| 14 | Custom list releases a pack item | unit | Pack item named in `custom` | Not written; hashes dropped; reported unmanaged |
| 15 | Removal prunes and releases | integration | Drop pack entry; one file untouched, one edited | Untouched removed; edited left, delocked, reported once |
| 16 | `update` moves a default-source pin to newest | integration | Entry naming blueprint default; source has a later release tag | Pin moves to the newest release, ahead of the blueprint default |
| 17 | `update` leaves a user-chosen pin | unit | Entry naming another source | Pin unchanged and reported |
| 18 | Rejected file families | unit | Pack carrying `.agents/aokf.md`; pack carrying a `run` key | Rejected naming file/key; nothing written |
| 19 | Local-path source updates without rebuild | integration | Local pack dir; mutate a file; `sync` | Repo copy updated from the new bytes |
| 20 | `update` offline stops at the blueprint default | integration | Default-source pin, network blocked | Pin moves no further than the blueprint default; reports it could not check |
| 21 | Pre-pack repo syncs untouched | integration | Manifest from an earlier binary, no pack entry | Succeeds from snapshot; manifest gains no pack entry |
| 22 | `update` migrates a pre-pack repo | integration | Repo from case 21 | Default pack entry written explicitly |

### Manual verification

1. Point this repo's manifest at its own `/pack/` as a local-path pack. Edit
   `pack/knowledge/skills/frame/SKILL.md`, run `superdev sync` without
   rebuilding, and confirm `.claude/skills/frame/SKILL.md` carries the edit —
   the dogfood the `asset-backport` skill existed to work around.
2. Tag an `assets-vN` release, pin a scratch repo at it, run `init`, and
   confirm the skills materialise from the tag rather than the snapshot.
3. Disconnect the network and run `superdev status --drift` in that scratch
   repo; confirm it completes and reports nothing unexpected.

## Regression coverage

- The existing `cli.rs` integration suite, in full — the default path must
  be byte-identical to today.
- The orphan and migration tests from the blueprint-migrations spec, which
  pack removal reuses rather than replaces.
- The skills-cardinality tests, whose many-provider manifest shape pack
  entries sit beside or extend.

## Environments / data

- Local git fixture repositories, created in a temp dir per test, serving as
  pack sources at known revs — no external network in CI.
- A network-blocked test mode, so the no-network criteria are asserted
  rather than assumed.

## Exit criteria

- Every automated case passes on all CI platforms.
- The three manual checks are signed off.
- No test in the suite reaches the network.
