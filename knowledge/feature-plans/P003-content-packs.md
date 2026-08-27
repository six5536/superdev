---
type: Plan
id: feature-plan-content-packs
title: Externally Sourced Content Packs — feature plan
description: Deliver S014 in eighteen slices — move the content to /pack, reorganise it into pack layout, derive it from that layout, add the manifest and lock schemas, resolve local then git sources, wire ownership, teach init and update, make each release one command, make a committed path pin portable, dogfood it, then close the gaps acceptance found and the one deferred issue small enough to fix.
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
  affirmative. (Superseded after the fact: the pack-to-live direction is
  what the pin removed, and `pack-backport` replaced the skill for the
  live-to-pack one, which iterating on a live copy still needs.)
- Done-check: editing `pack/knowledge/skills/frame/SKILL.md` and running
  `superdev sync` updates `.claude/skills/frame/SKILL.md` with no rebuild;
  `asset-backport` is gone and nothing references it.
- Cases: none automated. Manual: M1.

## Slices after the first pass

Reopened after `/accept`. All twenty-three acceptance criteria passed; these
close the gaps the criteria do not reach. Every slice here is buildable without
a contract change — what needs a decision is listed at the end and stays
unplanned, so nothing scheduled can stall behind one.

### Slice 15: Nothing superdev spawns can be talked into running a command

- [x] Done — ticked by integrate at merge.
- Gap: [I007](../issues/I007-a-pack-source-reaches-git-with-no-scheme-check.md),
  the half of it that needs no decision — and the half that closes the hole.
- Change: add `-c protocol.ext.allow=never` (and the same for the other
  command-running helpers) to the `verbatim()` overrides every git invocation
  already carries, so the guarantee holds whatever the user's git config says.
  Put `--` before the operands in `fetch.rs`'s clone and fetch and in
  `pin.rs`'s `ls-remote`, and refuse a source or rev beginning with `-`, so a
  value that satisfies `is_git` cannot present to git as an option.
- First because it is the confirmed exploit. On a stock git the transport is
  already refused; the override is what stops a user who has enabled it from
  being exploited by a manifest they cloned. It changes no accepted spelling,
  so it needs no contract change and can land immediately.
- Done-check: with `protocol.ext.allow = always` in the git config, a manifest
  naming `ext::touch /tmp/x` runs no command and the file is not created — the
  reproduction in I007, inverted; a source beginning with `-` is refused
  naming it; every spelling of the default source still resolves, and a
  `file://` pack and the scp form still resolve.
- Cases: none from the test plan, which does not reach the transport. New
  tests per the issue, the reproduction among them.

### Slice 16: A pack's symlinks are not followed

- [x] Done — ticked by integrate at merge.
- Gap: [I008](../issues/I008-a-symlinked-file-in-a-pack-is-followed.md).
- Change: skip every symlink in a pack tree, not only a linked directory —
  `read_dir` already computes `linked` and acts on it for one case out of two.
- Done-check: a pack whose item file is a symlink to a file outside the pack
  resolves without that item and writes nothing from it; the existing
  linked-directory behaviour is unchanged; superdev's own `/pack/` still
  resolves whole, since it ships no symlink.
- Cases: none from the test plan. A new test putting a symlinked file in a
  fixture pack and asserting it is not among the resolved files.

### Slice 17: The lock describes what is on disk, not only what was written

- [x] Done — ticked by integrate at merge.
- Gap: [I005](../issues/I005-a-backport-leaves-the-lock-stale.md), filed
  during delivery rather than by accept, scheduled because it is contained and
  fires on every backport, which is how this repo's owner edits skills.
- Change: record an owned file's hash whenever the run resolves it as
  matching, not only when it writes it. `apply` pushes to `written` inside the
  write path alone, so a file that needed no write keeps whatever hash the
  lock last recorded.
- Done-check: edit a live owned file, mirror it into `pack/`, `sync`, and the
  lock's hash for that file describes the file on disk; a later `sync` that
  rewrites it reports a plain write with no backup and no `user-edited` note;
  a genuinely hand-edited file is still reported and still backed up.
- Cases: none from the test plan. A new test resolving a file that already
  matches and asserting its hash lands in the lock.

### Slice 18: Document packs, and what `update` actually does

- [x] Done — ticked by integrate at merge.
- Gap: [I006](../issues/I006-content-packs-are-undocumented-for-users.md).
- Change: correct `update`'s description at
  `crates/app/superdev/src/main.rs` — clap renders it into `--help`, the man
  page and the completions — and the matching rustdoc in `manage.rs`, so both
  say the pack pin may move to the source's newest release. Add a packs
  section to `README.md`: the `[[packs]]` entry and the source spellings,
  layering and base replacement, the two release series, and that `update` is
  the verb that reaches the network.
- Last, and deliberately not made to wait on the scheme allowlist below: it
  documents the spellings superdev accepts today, and if the allowlist later
  narrows them that slice updates this text as part of its own work.
- Done-check: `superdev --help`, the man page and `README.md` each describe
  packs and none of them says `update` moves pins only to this binary's
  defaults; a reader who has never seen the canonical knowledge can pin a pack from
  the README alone.
- Cases: none from the test plan. A test asserting the help text names the
  network behaviour would stop it going stale again.

## Not scheduled — each needed an interface decision first

Cutting any of these as a slice would only have bounced it to
`/interface-design`, so they went there on their own. All five are now
decided — [ADR-012](../decisions/D012-pack-source-schemes-are-allowlisted.md)
to [ADR-016](../decisions/D016-a-path-pack-records-no-digest.md) — and
scheduled as [P005](P005-content-pack-hardening.md), except
[I003](../issues/I003-a-local-pack-cannot-remove-what-it-dropped.md), closed
`wontfix`: a path pack keeps layering, and the rebuild a pack developer needs
anyway is the answer.

- **The scheme allowlist**, the other half of
  [I007](../issues/I007-a-pack-source-reaches-git-with-no-scheme-check.md):
  refusing a source whose scheme superdev does not support, rather than
  handing it to git. Defence in depth once slice 15 lands, and a clearer
  error. Which schemes are in the set changes what a manifest may say, which
  [C001](../contracts/C001-content-packs.md) documents as `parse`'s rejections.
- [I001](../issues/I001-update-can-pin-an-unreadable-pack-format.md) — a
  format range the tag does not carry.
- [I002](../issues/I002-no-time-bound-on-the-update-query.md) — a deadline on
  the process boundary every component shares.
- [I003](../issues/I003-a-local-pack-cannot-remove-what-it-dropped.md) — a
  path source that may be the base, which ADR-004 and ADR-011 forbid.
- [I004](../issues/I004-a-path-packs-digest-churns-and-is-never-checked.md) —
  a lock schema whose digest is optional.
