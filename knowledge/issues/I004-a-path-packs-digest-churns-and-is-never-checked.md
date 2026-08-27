---
type: Issue
id: issue-004-a-path-packs-digest-churns-and-is-never-checked
title: A path pack's lock digest is rewritten by every content commit and verified by nothing
description: The lock recorded a digest over a path pack's whole tree that resolution never checked, so every commit touching pack/ rewrote the same line and conflicted between concurrent content PRs; fixed in P005 slice 7, which makes the digest optional and records none for a path source.
status: stable
tags: [done]
links:
  - rel: references
    to: spec-content-packs
---

# Bug: a path pack's digest churns and is never checked

## Decided

[ADR-016](../decisions/D016-a-path-pack-records-no-digest.md).
`PackLock.digest` becomes optional and is omitted for a path source. The
churn goes, nothing false is recorded, and nothing is lost: the value was
written and never read, and whether the live files match the pack is checked
by the per-file hashes in the lock's `files` map, which are untouched.

Verifying it instead was the other way to make it honest, and it is the wrong
way — it would fail every run after every edit until the pack was re-synced,
which is the workflow a path source exists to remove.


## Summary

Against [S014](../specs/S014-content-packs-design.md).

A path pack is re-read from disk every run, so its digest is recorded but never
verified — the verification path a git pack goes through is skipped entirely.
What the recording does produce is churn: the digest covers the whole tree, so
every commit touching any file under `pack/` rewrites that one line. Two
content PRs in flight always conflict on it. And a commit that edits `pack/`
without running `sync` leaves a digest that does not describe the tree, which
nothing reports.

## Environment

- Version/commit: 0.2.0 / slice 14 of P003
- Platform: all

## Steps to reproduce

1. In a repo pinning `./pack`, edit any file under `pack/` and do not run
   `sync`.
2. `superdev status --drift`

## Expected behaviour

Either the digest is not recorded for a source that is read fresh every run, or
a digest that does not match the tree is reported.

## Actual behaviour

Step 2 exits 0 with the recorded digest describing content that is no longer
there. Separately, every `sync` after a content edit rewrites the line, so it
is a guaranteed conflict between any two branches that both touch `pack/`.

## Root cause (if known)

`resolve_one`'s path arm returns `Resolved::Layer` with `record(...)` directly,
bypassing `verified()` — correct, since there is no pinned rev whose bytes
could have moved. The digest is recorded anyway, because `PackLock` requires
one for every entry.

## Proposed fix / workaround

- Fix: make the digest optional for a path source and omit it, or keep it and
  check it, reporting a tree that no longer matches. The first is a lock-schema
  question and belongs to interface-design.
- Workaround: run `sync` in any commit that touches `pack/`, and resolve the
  one-line conflict by re-running it.

## Regression risk

`PackLock`'s shape and `resolve_one`'s path arm; the committed lock in this
repository is the first one affected.
