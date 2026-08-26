---
type: Plan
id: feature-plan-content-packs
title: Externally Sourced Content Packs — feature plan
description: Deliver S014 in fourteen slices — move the content to /pack, reorganise it into pack layout, derive it from that layout, add the manifest and lock schemas, resolve local then git sources, wire ownership, teach init and update, make each release one command, make a committed path pin portable, and dogfood it.
status: stable
tags: [done]
links:
  - rel: implements
    to: spec-content-packs
---

# Feature plan: externally sourced content packs

Spec: [S014](../specs/S014-content-packs-design.md). Contract:
[C001](../contracts/C001-content-packs.md). Decisions:
[ADR-001](../decisions/D001-packs-manifest-section.md) …
[ADR-006](../decisions/D006-pack-at-repo-root.md),
[ADR-011](../decisions/D011-path-pack-identity-is-root-relative.md).

Ordered by dependency, then risk. Slice 1 carries the Windows symlink risk
and slice 8 the layering semantics; both sit as early as their dependencies
allow. Slices 1 to 3 change no observable behaviour, so they carry no
test-plan case and are checked by the existing suite staying green — slice 1
deliberately moves the tree without touching its shape, so the symlink is the
only variable when the Windows job runs.

## Slices

### Slice 1: Move the content to /pack, unchanged

- [x] Done — ticked by integrate at merge.
- Change: move `crates/lib/superdev-core/assets/` to `/pack/` with its
  internal layout untouched, and leave `crates/lib/superdev-core/assets` as a
  relative symlink to it. Every `asset!()` path is therefore unchanged. Move
  the `assets/** -text` patterns in `.gitattributes` to `pack/**`. Set
  `core.symlinks=true` before `actions/checkout` in `checks.yml`'s Windows
  job, and note the Windows checkout requirement in CONTRIBUTING.
- Done-check: the full suite is green on Linux, macOS **and Windows**;
  `cargo package -p superdev-core --list` still carries all 254 files under
  `assets/`; no source file changed. A failure here is the symlink and
  nothing else, and the slice reverts cleanly.
- Cases: none — regression coverage (the whole `cli.rs` suite; the default
  path must be byte-identical to today).

### Slice 2: Reorganise /pack into pack layout

- [x] Done — ticked by integrate at merge.
- Change: `aokf/skills/` → `knowledge/skills/`, `aokf/knowledge/*.md` →
  `knowledge/concepts/`, `aokf/knowledge/templates/` →
  `knowledge/templates/`, `templates/` → `projects/`; `skills/` and `agents/`
  keep their names. Add `pack.toml`. Update the 94 `asset!()` paths that
  follow. Retarget `no_template_asset_is_named_cargo_toml` to
  `pack/projects/`. The capability instruction files and the AOKF spec keep
  their present home — they are not pack content.
- Done-check: the suite is green on all three platforms and `superdev init`
  in a scratch repo writes byte-identical files to slice 1's binary; no
  `asset!()` path resolves outside `pack/`.
- Cases: none — a path-only change; covered by the regression suite.

### Slice 3: The content module — items, layout rules, ContentSet

- [x] Done — ticked by integrate at merge.
- Change: add `superdev-core::content` — `Owner`, `ItemKind`, `Item`,
  `Origin`, `Shadowed`, `ContentSet` — and the rules that turn a pack tree
  into items. Build the embedded snapshot's `ContentSet` from `/pack/`.
  Nothing consumes it yet.
- Done-check: the snapshot's `ContentSet` enumerates every item the
  components currently ship, with identical bytes, asserted item by item; the
  existing suite is unchanged and green.
- Cases: none — new code behind no caller; covered by its own unit tests.

### Slice 4: Components read content from Ctx

- [x] Done — ticked by integrate at merge.
- Change: add `content: &ContentSet` to `Ctx`, built by `pipeline` from the
  snapshot for now. Convert `components/aokf.rs`, `components/skillpack.rs`
  and the general-rules scaffolds in `pipeline.rs` to read items from
  `ctx.content`, retiring aokf.rs's hand-written `(repo path, asset!())`
  lists. `codegraph.rs`, `rtk.rs` and the AOKF spec keep their constants.
- Done-check: `init`, `sync` and `status` produce byte-identical output and
  identical lock contents to slice 3's binary; adding a file under
  `pack/knowledge/concepts/` makes it appear in the plan with no Rust edit.
- Cases: 1.

### Slice 5: Manifest and lock schemas

- [x] Done — ticked by integrate at merge.
- Change: add `Manifest.packs: Vec<PackEntry>` and the lock's `[[packs]]`
  `PackLock` table, both defaulting to empty and skipped when empty. Parsing
  and saving only — nothing resolves yet, and an absent array means the
  snapshot, never disabled.
- Done-check: a manifest written by the previous binary round-trips
  byte-identically; a manifest carrying `[[packs]]` round-trips; `sync` on a
  pre-pack manifest adds no pack entry.
- Cases: 21.

### Slice 6: Pack source, identity and the pack manifest

- [x] Done — ticked by integrate at merge.
- Change: add `PackSource::parse` and `PackSource::identity` with the
  normalisation rules, `PackManifest` with `SUPPORTED_FORMATS`, the
  `REJECTED` path list and the `PROJECT.md` basename rule, and
  `Error::Pack`. Pure logic over a directory — no fetching, no cache.
- Done-check: the equivalence classes in ADR-004 all normalise alike and
  distinct repos do not; an unknown `format` errors naming the pack and the
  supported set; a rejected path errors naming the file and the reason;
  nothing is written in either case. A test asserts `DEFAULT_PACK.rev` names
  the version in `/pack/pack.toml`, so the embedded snapshot and the pin that
  claims to describe it cannot drift apart — belt to slice 12's braces, since
  the release script sets both.
- Cases: 10, 18.

### Slice 7: Resolve a local pack and materialise its items

- [x] Done — ticked by integrate at merge.
- Change: add `pack::resolve` for `PackSource::Path` only, with
  `ResolveMode` and `Resolution`, returning a `ContentSet` built from one
  pack over the snapshot. Call it from `pipeline` ahead of `plan_repo`,
  replacing slice 4's snapshot-only construction. Precedence is not yet
  modelled: a single pack's items simply win.
- Done-check: a local pack's skill reaches `.claude/skills/`; editing the
  pack's file and re-running `sync` updates the repo copy with no rebuild;
  a pin equal to the snapshot's rev resolves from the snapshot and makes no
  request.
- Cases: 2, 19.

### Slice 8: Layering, base replacement and the shadow report

- [x] Done — ticked by integrate at merge.
- Change: add base-versus-layer selection on normalised source identity,
  item superseding in manifest order, shadow collection, and the removal of
  items the base no longer carries. Add the `status` content line naming the
  base and each layer, and the pack-over-pack shadow report.
- Done-check: the base replaces the snapshot so a dropped item leaves the
  repo and an edited one is released; a non-base pack supersedes without a
  report; two packs report the shadow; reversing manifest order flips the
  winner and changes nothing else; `status` names the base.
- Cases: 5, 6, 7, 8, 9.

### Slice 9: Git sources, digests and the cache

- [x] Done — ticked by integrate at merge.
- Change: add `PackSource::Git` resolution by spawning the user's `git`
  through the injected `CommandRunner` — `clone --depth 1
  --filter=blob:none --sparse --branch <rev>` then `sparse-checkout set pack`
  ([ADR-007](../decisions/D007-git-fetch-by-spawn.md)), with the commit-sha
  path implemented explicitly since `--branch` does not take one. Add the
  digest over a pack tree, verification against the lock's recorded digest,
  and the cache under `.superdev/cache/packs/<digest>/`.
  `ResolveMode::Offline` never fetches and never writes the cache; `Fetching`
  fetches only what is neither cached nor committed.
- Done-check: a first resolve records digest and per-file hashes and writes
  the files; a second run with the network blocked does nothing and makes no
  request; an unresolvable source and a changed digest each exit non-zero
  writing nothing, with the snapshot not substituted; a tag pin and a
  commit-sha pin both resolve; a missing `git` fails saying so.
- Cases: 3, 4, 11, 12. Manual: M3.

### Slice 10: Ownership — drift, custom and removal

- [x] Done — ticked by integrate at merge.
- Change: wire pack-provided items through the existing ownership machinery
  so provenance changes nothing: lock hashing and drift reporting, the
  `custom` lists' name-guarded release, and orphan pruning when a pack entry
  is dropped.
- Done-check: a hand-edited pack-provided file reports as drift exactly as an
  embedded one; a pack item named in `custom` is unwritten, delocked and
  reported unmanaged; dropping a pack entry prunes its untouched files and
  releases its edited ones, reporting each once.
- Cases: 13, 14, 15.

### Slice 11: init and update

- [x] Done — ticked by integrate at merge.
- Change: `init` writes the blueprint's default pack entry explicitly.
  `update` asks the default source for its newest release tag and moves that
  pin there, even ahead of the blueprint's default
  ([ADR-009](../decisions/D009-update-queries-default-source.md)); with no
  network it moves no further than the blueprint's default and says it could
  not check. A pin naming another source is reported and left alone, and a
  manifest carrying no entry gains the default one.
- Done-check: a default-source pin moves to the source's newest release; the
  same run with the network blocked stops at the blueprint default and
  reports why; a third-party pin is untouched and reported; `init` on a fresh
  repo writes the entry; a pre-pack manifest gains it.
- Cases: 16, 17, 20, 22.

### Slice 12: One command per release

- [x] Done — ticked by integrate at merge.
- Change: teach `npm run release X.Y.Z` to set `/pack/pack.toml`'s version,
  set `DEFAULT_PACK.rev` to the pack tag it is about to cut, and create both
  `vX.Y.Z` and `assets-vA.B.C` from one commit. Add `npm run release:pack`,
  which bumps `pack.toml` and cuts `assets-vA.B.C` alone. Make the binary
  release workflow ignore `assets-v*` tags so a content release runs no
  five-platform build. Record both flows in CONTRIBUTING and
  `release-procedure` ([ADR-008](../decisions/D008-one-command-per-release.md)).
- Done-check: `release X.Y.Z` produces one commit carrying both tags with
  `DEFAULT_PACK.rev` naming the pack tag it cut — no second step, nothing to
  reconcile by hand; `release:pack` cuts one tag and triggers no binary
  workflow; a scratch repo pinned at a content tag resolves from it rather
  than from the snapshot.
- Cases: none automated. Manual: M2.

### Slice 13: A committed path pin reads the same everywhere

- [x] Done — ticked by integrate at merge.
- Change: `PackSource::identity` takes the repo root, and a path source's key
  becomes its canonicalised path relative to that root with forward slashes.
  A pack outside the root keeps its `..` prefix; where no relative form exists
  — a different Windows drive — the canonical absolute path stands. Keys are
  compared only within a source kind, so a directory can never key as the base
  pack ([ADR-011](../decisions/D011-path-pack-identity-is-root-relative.md)).
  Four call sites in `resolve.rs` and `is_default` in `source.rs` follow.
- Raised by slice 14's build: the lock is committed, so until this lands
  dogfooding writes one contributor's absolute paths into a tracked file and
  every other checkout's first `sync` rewrites them. `status --drift` passes
  either way, so CI does not catch it.
- Done-check: a repo carrying a path pack locks the same `identity` whatever
  directory it is checked out into, and on every platform; two spellings of
  one directory are still one pack and a second entry naming it is still
  refused; a directory whose relative path reads like a repository key layers
  rather than replacing the embedded pack.
- Cases: none — the test plan does not reach the lock's persisted form.
  Covered by its own tests and the existing pack suite.

### Slice 14: Dogfood — superdev pins its own pack

- [x] Done — ticked by integrate at merge.
- Change: point this repo's own manifest at `/pack/` as a local-path pack, so
  an asset edit reaches `.claude/skills/` on the next `sync` without a
  rebuild. Retire the `asset-backport` skill and the workflow note that
  documents it. Answers the spec's remaining open question in the
  affirmative.
- Done-check: editing `pack/knowledge/skills/frame/SKILL.md` and running
  `superdev sync` updates `.claude/skills/frame/SKILL.md` with no rebuild;
  `asset-backport` is gone and nothing references it.
- Cases: none automated. Manual: M1.
