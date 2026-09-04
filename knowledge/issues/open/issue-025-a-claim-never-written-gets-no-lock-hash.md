---
type: Issue
id: issue-025-a-claim-never-written-gets-no-lock-hash
title: A claimed file superdev never wrote has no lock hash, so its first rewrite misreports as a user edit
description: The lock reconcile refreshes existing entries and never adds one for a claim already satisfied on disk, so all 53 shipped schemas were unrecorded and each first rewrite reports "overwrote a user-edited file" and spawns a backup.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-005-a-backport-leaves-the-lock-stale
    note: The reconcile that fix added updates stale entries; this is the entry that never existed.
---

# Bug: a claimed file superdev never wrote has no lock hash

## Summary

An owned file whose content already matched the desired bytes when its
claim first appeared gets no hash in `.superdev/lock.toml`. The next time
a pack edit changes its desired content, `sync` reports
`overwrote a user-edited file (backed up)` for a file no user touched.
Every shipped document schema is in this state, so plan-014's schema
sweep will raise one false report per schema it edits.

## Context

Observed on superdev at commit `b69176d`, branch
`feature/workflow-autonomy`, in a Linux devcontainer through the dev shim
(`scripts/superdev` → `cargo run`). To reproduce:

1. `git grep -c "knowledge/schemas/" .superdev/lock.toml` — one entry
   (the schema written below), not 53.
2. Edit any line of a schema under `pack/knowledge/schemas/`.
3. `cargo run --quiet -- sync`

## Behaviour

The live schema is an unedited owned file, so the write is a plain
`write knowledge/schemas/<name>.md (document schema)`, and every claimed
file carries a hash in the lock after any successful sync.

Instead, sync reports a user edit:

```
applied  write knowledge/schemas/feature-plan.md (document schema): overwrote a user-edited file (backed up)
```

A backup of the unedited file lands under `.superdev/cache/backup/`, and
only the written file gains a lock entry.

The cause is in the lock reconcile that
[I005][sokf:issue-005-a-backport-leaves-the-lock-stale] added: it
refreshes the hash of entries the lock already carries; a claim with no
entry is left absent rather than hashed from disk. The schemas shipped in
the P008/P010 backport with their live copies already in place, so no
sync ever wrote them and none gained an entry. With no recorded hash, the
engine cannot tell "never recorded" from "user-edited" and takes the safe
branch.

## Scope

The reconcile pass alone.

- Fix: the reconcile pass adds a hash for every live claim whose file
  exists on disk, not only for entries the lock already carries.
- Workaround: none needed — the backup is spurious but harmless, and each
  file self-heals on its first write.
- Regression risk: the I005 regression tests cover the refresh side; a
  new case wants a claim satisfied on disk before its first sync. The
  orphan pass reads the same entries, so a wrongly added hash would
  surface there.

<!-- sokf:links -->
[sokf:issue-005-a-backport-leaves-the-lock-stale]: /knowledge/issues/done/issue-005-a-backport-leaves-the-lock-stale.md
