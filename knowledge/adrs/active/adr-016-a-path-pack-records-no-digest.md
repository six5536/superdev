---
type: Decision
id: adr-016-a-path-pack-records-no-digest
title: A Path Pack Records No Digest
description: The lock's digest becomes optional and is omitted for a path source, because a directory is read afresh every run so the value is never checked, and recording it rewrites one committed line on every content commit.
lifecycle: active
links:
  - rel: references
    to: contract-007-interface-pack-resolution
  - rel: relates-to
    to: adr-011-path-pack-identity-is-root-relative
---

# ADR-016: A path pack records no digest

- Status: accepted
- Date: 2026-08-26
- Deciders: project owner

## Context

Every resolved pack gets a `[[packs]]` record in `.superdev/lock.toml`
carrying a digest over its whole tree. For a git pack that digest is the
promise [C007][sokf:contract-007-interface-pack-resolution] makes: the next run proves it got the same bytes the pin was
made against, and a mismatch fails the run
([security-requirements][sokf:security-requirements]).

A path pack has no such promise to keep. There is no pinned rev whose bytes
could have moved — the directory is read from disk every run, and being
editable without a re-pin is the entire point of a path source. `resolve_one`
records the digest anyway, and nothing ever reads it back.

What the recording does produce is churn. The digest covers the tree, so
every commit touching any file under `pack/` rewrites that one line, and any
two branches editing content conflict on it. A commit made without running
`sync` leaves a digest that does not describe the tree, and nothing reports
that either
([I004][sokf:issue-004-bug-a-path-packs-digest-churns-and-is-never-checked]).
This repository is the first to commit such a line, and met both immediately.

[ADR-011][sokf:adr-011-path-pack-identity-is-root-relative] considered omitting
`identity` for a path source and rejected it — "the field becomes optional
for one source kind, a schema that has to explain itself". That objection is
real and applies here too, so the difference has to be worth it.

## Decision

`PackLock.digest` becomes optional and is omitted for a path source.

```rust
pub struct PackLock {
    pub source: String,
    pub identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// What a fetched pack was verified against. Absent for a path source: a
    /// directory is read afresh every run, so no pinned bytes exist to
    /// verify and a recorded value would be rewritten by every content
    /// commit. ADR-016.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    pub format: u32,
}
```

A lock written before this parses unchanged and loses its path entry's digest
on the next write.

The difference from `identity` is what the field does when it is wrong.
`identity` is constant for a given repository and pack: it is written once
and then written identically forever, and there is no ordinary sequence of
events that makes it disagree with reality. A path pack's digest disagrees
with reality after any content commit that did not run `sync`, which is a
thing people do. Recording a value that is routinely false and never read is
worse than recording a constant that is rarely read — and the constant costs
no conflicts.

Nothing is lost. Whether the live files match what the pack says is checked
by the per-file hashes in the lock's `files` map, which are unaffected, and
that is the check `status --drift` has always run. The `[[packs]]` record
keeps its source, identity and format, so a dropped entry's files still
become orphans by the existing rule.

Verifying the digest instead was the other way to make it honest, and it is
the wrong way: it would fail every run after every edit until the pack was
re-synced, which is the workflow a path source exists to remove.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Omit it for a path source | The committed lock stops churning, so two content branches no longer conflict; nothing false is recorded; nothing is lost, since the value was never read | The lock has two shapes of pack record, and a reader must know which kind omits the field |
| Keep it and verify it | Consistent with a git pack; a tree edited without `sync` is caught | Fails the run after every edit until re-synced, which removes the reason a path source exists |
| Keep it, unchecked | No change | A guaranteed one-line conflict between any two content branches, and a recorded value that is routinely wrong |
| Record something that does not churn — the item names, or a per-file map | Still a record of what resolved | Still never read, and now a second digest scheme to explain |
| Drop the whole record for a path source | No optional field at all | Loses the source and format, and the set of records is what says which packs resolved |

## Consequences

- Positive: a commit touching `pack/` no longer rewrites a lock line, so
  concurrent content branches stop conflicting on a value neither of them
  meant to change.
- Positive: the lock stops asserting something that is false whenever
  somebody edits content without syncing.
- Negative: `PackLock` has an optional field whose absence carries meaning,
  which a reader must learn. The field's own doc is where it is said, and the
  absence is exactly co-extensive with the entry having no `rev`.
- Neutral: `status`'s content reporting is unaffected — it reads the manifest
  and the resolution, never the recorded digest.
- Neutral: the guarantee in
  [security-requirements][sokf:security-requirements] is unchanged in
  substance. It is about a *pinned* pack applying the bytes it was pinned to,
  and a directory is not pinned. The wording gains that distinction.
- Follow-ups: [C007][sokf:contract-007-interface-pack-resolution] carries the new
  shape; [configuration][sokf:configuration] documents the lock's fields and
  gains the optional digest at integrate.

<!-- sokf:links -->
[sokf:adr-011-path-pack-identity-is-root-relative]: /knowledge/adrs/active/adr-011-path-pack-identity-is-root-relative.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:issue-004-bug-a-path-packs-digest-churns-and-is-never-checked]: /knowledge/issues/done/issue-004-bug-a-path-packs-digest-churns-and-is-never-checked.md
[sokf:security-requirements]: /knowledge/security-requirements.md
