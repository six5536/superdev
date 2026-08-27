---
type: BugReport
id: issue-005-a-backport-leaves-the-lock-stale
title: Sync re-records a hash only for a file it writes, so backporting an edit leaves the lock stale
description: After a live edit was mirrored into the pack, sync had nothing to write and never refreshed that file's recorded hash, so the next legitimate write reported it as user-edited and backed it up; fixed in slice 17, which reconciles every claim against disk before saving the lock.
status: stable
tags: [done]
links:
  - rel: references
    to: spec-content-packs
---

# Bug: a backport leaves the lock stale

## Resolved

P003 slice 17. A run reconciles every claim against what is actually there
before it saves the lock, so a file that changed on disk to what superdev
would write no longer leaves the lock describing what it replaced.

Two things the fix turned on, both held by tests. It runs *after* the engine:
reconciling first records a hand-edited file's own bytes as the hash the
engine then compares against, so the overwrite is reported as an ordinary
write and the user is never told the edit went into a backup. And it refreshes
only keys the lock already holds — adoption leaves a repo's own copy of a
shipped file unclaimed on purpose, and inserting would take ownership of every
one on the next run.

It covers mise pins and JSON keys too, where a stale hash cost more than on a
file: the orphan pass compares against it to decide whether an entry is
superdev's to remove, so a stale one left superdev's own registration in a
shared file for good.

One limit: it can only reconcile a claim that is still live. Stale a hash and
drop the claim in the same run — disable the capability at the same time — and
there is nothing left to reconcile against, so that entry still releases.

## Summary

Against [S014](../specs/S014-content-packs-design.md).

`apply` records a file's hash only when it writes that file. The backport
workflow — edit the live copy to try it, mirror it into `pack/`, then `sync` —
ends with live and desired identical, so `sync` writes nothing and the lock
keeps the hash from before the edit. The file is now permanently mismatched
against its own lock entry, and the next time anything does write it the run
reports `overwrote a user-edited file (backed up)` and drops a backup of a
file nobody edited.

Self-inflicted and self-repeating: every backport plants one. It is where the
stale entries already in this repo's committed lock came from, under the old
`asset-backport` workflow.

## Environment

- Version/commit: 0.2.0 / after P003
- Platform: all

## Steps to reproduce

1. Append a line to `.claude/skills/verify/SKILL.md`.
2. Copy it over `pack/knowledge/skills/verify/SKILL.md` — the backport.
3. `superdev sync` — writes nothing, as it should.
4. Compare the lock's hash for that path against the file's.

## Expected behaviour

A file superdev owns and has just confirmed matches its desired content should
carry that content's hash in the lock.

## Actual behaviour

Step 3 writes nothing and refreshes nothing. Step 4 shows the recorded hash
still describes the pre-edit file. A later `sync` that does write the file
then claims a user edit that never happened.

## Root cause (if known)

`engine/apply.rs` pushes to `written` only inside the write path, and the
"user-edited" note compares the live file against `prior_hashes`. A file that
needs no write is neither re-recorded nor reconciled, so the lock drifts from
a file that is, in fact, exactly right.

## Proposed fix / workaround

- Fix: record the hash for every owned file the run resolved as matching, not
  only for the ones it wrote — the lock is meant to describe what is on disk.
  Cheap, since the content is in hand either way.
- Workaround: delete the live file and `sync`, which rewrites and re-records
  it. Delete the specific file, not its directory: a skill directory may hold
  a repo-owned `PROJECT.md` that superdev does not ship and cannot restore.

## Regression risk

`engine/apply.rs` and every component's `owned` set. A test would resolve a
file that already matches and assert its hash lands in the lock.
