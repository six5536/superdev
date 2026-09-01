---
type: FileFormatContract
id: contract-006-file-format-lock
title: Lock Format Contract
description: What superdev records of the last apply — the per-capability components, the file hashes, the resolved packs — and what a reader may conclude from it.
lifecycle: active
resource: /crates/lib/superdev-core/src/lock.rs
---

# File format contract: lock

What superdev records of the last apply: the per-capability components,
the file hashes and the resolved packs, and what a reader may conclude
from it.

## Files

`.superdev/lock.toml`, committed. superdev writes it, and other tools
SHOULD NOT edit it: it records what the last apply actually did, so a
hand edit makes it a record of something that never happened.

It is read, though, and that is why it is a contract. A reviewer reads it to
see what a sync changed, and CI reads it to tell a clean tree from a drifted
one. Its counterpart is the manifest, which says what the repo *wants*; the
lock says what it *got*.

## Shape

```toml
[[packs]]                             # one entry per pack the apply resolved
source   = "github:six5536/superdev"  # as the manifest wrote it
identity = "github.com/six5536/superdev"  # normalised: every spelling of one source
rev      = "assets-v1.4.0"            # absent for a path source
digest   = "sha256:…"                 # absent for a path source, with rev
format   = 1                          # the format the pack declared

[components.code-index]               # one per capability the apply filled
provider = "codegraph"
version  = "1.5.0"

[files]                               # sha256 of every file superdev owns
".agents/superdev.md" = "a99e4f86…"
".claude/skills/frame/SKILL.md" = "85bdea82…"
".mise.toml:rtk" = "6d128905…"        # a pin merged into a shared file
".claude/settings.json:hooks.PostToolUse[superdev hook validate]" = "fc402a3b…"
```

An entry superdev merges into a file it does not own is hashed under
`<file>:<pointer>` rather than as a whole file, because the rest of that file
is the user's. `[[packs]]` is absent when no pack was named. `rev` and
`digest` are absent together for a path source: a directory is read afresh
every run, so there are no pinned bytes to name.

## Compatibility

A reader MUST NOT conclude more from a hash than two things: superdev wrote
that file, and it wrote those bytes. Drift is not decided here — it is found
by comparing a file against the content the blueprint wants — so a hash that
no longer matches means the file was edited after superdev wrote it, which is
what lets an apply say so before overwriting, and back the file up first.

An entry no component claims any more is an orphan. Content still hashing to
the locked value is superdev's own residue and MUST be removed; content the
user changed MUST be left where it is and dropped from the lock with a line
saying so. A legacy `owners` table from an older binary MUST be cleared whole
on the first sync and never written again.

A lock from an older binary MUST load. A section it lacks MUST be treated as
absent rather than as an error, which is what lets an upgrade sync rather than
demanding a re-init.

## Stability

Unreleased. The table names and the hash algorithm MAY change without notice.
What holds even so: the file is superdev's to write, a hand edit MUST NOT be
respected, and no command asks the user to repair one — a lock superdev cannot
read MUST be rebuilt by the next apply.
