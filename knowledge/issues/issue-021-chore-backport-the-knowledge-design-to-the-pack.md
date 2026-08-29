---
type: Chore
id: issue-021-chore-backport-the-knowledge-design-to-the-pack
title: The pack has not been backported since the knowledge design started moving, and holds neither the schemas nor the types they name
description: Four plans have reshaped the live knowledge design and none has been backported, so /pack/ still ships 56 templates the schemas replaced, 20 concepts typed against a vocabulary that no longer exists, and a manifest declaring SOKF 0.2.
status: draft
tags: [needs-triage]
links:
  - rel: relates-to
    to: issue-016-bug-sync-would-revert-the-schema-migration
  - rel: relates-to
    to: issue-020-bug-the-schemas-do-not-ship
---

# Chore: backport the knowledge design to the pack

## Summary

The knowledge design has been reshaped four times in the live tree — the
schema migration, ADR-017, P008 and the filename convention — and the pack
has been backported for none of them. Backporting after each would mean
rewriting the same files repeatedly while the design is still moving, so
the divergence is deliberate and this chore is the record of it. It is
done in one pass once the design settles, which is what
[I016][sokf:issue-016-bug-sync-would-revert-the-schema-migration] asked for:
something in the tree that says the drift is known and owned.

## Surfaces

- 77 drift entries (`cargo run -- status --drift | grep -c '^  - '`), of
  which 56 are `knowledge/templates/*` that the schemas replaced, 20 are
  rewritten skills under `.claude/skills/`, and one is `.agents/`.
- `pack/knowledge/concepts/` — 20 concepts, 18 of them typed `Reference`,
  `Convention` or `Policy`, a vocabulary P008 replaced with one type per
  schema. `cargo run -- validate --knowledge pack/knowledge/concepts`
  reports 19 errors, of which 18 are "names no schema".
- `pack/knowledge/concepts/manifest.sokf.yaml` declares `sokf: "0.2"`
  against a live tree [P010][sokf:plan-010-adhoc-links-address-ids] took
  to 0.4.
- `pack/sokf/agents/sokf/SPEC.md` is SOKF 0.3 — it carries neither §8's
  id-addressed body link nor §9's generated definition block, so the
  specification a managed repository reads is a version behind the binary
  that checks it. `pack/sokf/agents/sokf.md` likewise tells the agent
  nothing about the link form or about `superdev validate --fix`. Both are
  binary-owned, so `sync` rewrites the live copies from them: until they
  are backported, running `sync` reverts this repository's own spec.
- `pack/knowledge/concepts/index.md` and `pack/knowledge/templates/index.md`
  were converted with P010 rather than left, since a starter repository
  whose index fails its own hook is not a starter repository. They are the
  only two files under `pack/` that P010 touched.
- `knowledge/schemas/` ships nowhere: the pack carries the 56 templates
  that produce documents and none of the 54 schemas that check them, which
  is [I020][sokf:issue-020-bug-the-schemas-do-not-ship].
- `pack/knowledge/skills/` (17) and `pack/knowledge/templates/` (48),
  against the live copies P008 and later work rewrote.
- `.agents/sokf/changelog.md` is new and has no pack counterpart, so §12's
  link to it dangles in a managed repository. The backport ships it as a
  third binary-owned file: `pack/sokf/agents/sokf/changelog.md`, an entry
  in `BINARY_OWNED`, and a kind in `content/layout.rs`.
- `pack/agents/process.md` and `pack/sokf/agents/` — binary-owned, so
  `sync` rewrites the live copies from them.

## Definition of done

- `cargo run -- status --drift` exits 0.
- `superdev init` into an empty repository, then
  `cargo run -- validate --knowledge <that repo>/knowledge`, exits 0 — the
  check that proves a managed repo starts life valid, which nothing
  performs today.
- No file under `pack/` names AOKF, a conformance level, or a type outside
  the schemas the pack ships.
- `pack/knowledge/concepts/manifest.sokf.yaml` declares the version the
  binary enforces.
- I016 and I020 are settled, or say what of them survives this work.

## Comments

Deliberately deferred while the knowledge design moves. P010 and P011 both
name the pack as a non-goal for this reason, so their drift lands here
rather than being backported piecemeal. The `pack-backport` skill and
[I005][sokf:issue-005-bug-a-backport-leaves-the-lock-stale]'s lock
reconciliation are the mechanics; the size is the reason it waits for one
pass rather than five.

<!-- sokf:links -->
[sokf:issue-005-bug-a-backport-leaves-the-lock-stale]: /knowledge/issues/issue-005-bug-a-backport-leaves-the-lock-stale.md
[sokf:issue-016-bug-sync-would-revert-the-schema-migration]: /knowledge/issues/issue-016-bug-sync-would-revert-the-schema-migration.md
[sokf:issue-020-bug-the-schemas-do-not-ship]: /knowledge/issues/issue-020-bug-the-schemas-do-not-ship.md
[sokf:plan-010-adhoc-links-address-ids]: /knowledge/plans/plan-010-adhoc-links-address-ids.md
