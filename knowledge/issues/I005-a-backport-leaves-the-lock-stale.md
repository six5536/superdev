---
type: Issue
id: issue-005-a-backport-leaves-the-lock-stale
title: Sync re-records a hash only for a file it writes, so backporting an edit leaves the lock stale
description: After a live edit is mirrored into the pack, sync has nothing to write and never refreshes that file's recorded hash, so the next legitimate write reports it as a user-edited file and backs it up.
status: draft
tags: [needs-triage]
links:
  - rel: references
    to: spec-content-packs
---

# Bug: a backport leaves the lock stale

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
