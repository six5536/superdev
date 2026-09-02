---
type: Contract
id: contract-006-format-lock
kind: format
title: Format contract for lock.toml
description: What superdev records of the last apply — lock.toml as the writer declares it, the per-capability components, the file hashes, the resolved packs — and what a reader may conclude from it.
lifecycle: active
resource: /crates/lib/superdev-core/src/lock.rs
links:
  - rel: references
    to: adr-033-a-contract-defines-its-interface
    note: A contract carries a machine-readable definition; here it is the lock's on-disk shape.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The definition is materialised from the `lock` regions and bound by the include; the orphan rule and the older-lock rule are bound by `lock.rs`'s and the apply's tests.
  - rel: references
    to: adr-043-one-contract-schema-and-twelve-kinds
    note: A text format and a binary format are one reader's question, `format`; this contract carries the kind and its id names it.
---

# Format contract: lock.toml

What superdev records of the last apply: the per-capability components,
the file hashes and the resolved packs, and what a reader may conclude
from it. The Definition is the lock as the writer declares it — the
path and every table `load` reads and `save` writes, with the doc
comment that says what each key means. Behaviour carries what the shape
cannot say: who writes the file, what a hash does and does not prove,
and what a lock from an older binary does. The decisions behind the
shape are [ADR-033][sokf:adr-033-a-contract-defines-its-interface],
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
and [ADR-043][sokf:adr-043-one-contract-schema-and-twelve-kinds].

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/lock.rs#lock -->
```rust
/// Repo-relative path of the lock file.
pub const LOCK_PATH: &str = ".superdev/lock.toml";

/// What one capability had applied last.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedComponent {
    /// Provider that was applied.
    pub provider: String,
    /// Version that was applied, when pinned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// A capability's lock shape as written: one record table for a single
/// entry, an array of tables from two up — mirroring the manifest so the
/// single case keeps its `[components.<name>]` shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum WrittenRecords {
    /// A single `[components.<name>]` table.
    One(LockedComponent),
    /// `[[components.<name>]]` entries, one per provider.
    Many(Vec<LockedComponent>),
}

/// One resolved pack, recorded so a later run can prove it got the same bytes,
/// and so a dropped entry's files become orphans by the existing rule.
/// Per-file hashes stay in the lock's existing `files` map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackLock {
    /// The source as the manifest wrote it.
    pub source: String,
    /// The normalised comparison key every spelling of one source shares.
    pub identity: String,
    /// The revision resolved, for a git source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// What a fetched pack was verified against, checked on every later run.
    ///
    /// Absent for a path source: a directory is read afresh every run, so
    /// there are no pinned bytes to verify against, and a value recorded here
    /// would be rewritten by every commit touching the pack and read by
    /// nothing. Absent exactly when `rev` is. ADR-016.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    /// The `format` the pack's own manifest declared.
    pub format: u32,
}

/// Last-applied state: how `status` tells deliberate user change from drift.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// The packs the last apply resolved, in manifest order. Empty when no
    /// pack was named, which is the default path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<PackLock>,
    /// Applied provider/version records, keyed by capability — one per
    /// enabled (capability, provider) entry.
    #[serde(default, with = "records_serde")]
    pub components: BTreeMap<String, Vec<LockedComponent>>,
    /// sha256 of superdev-owned content, keyed by repo-relative path
    /// (`.mise.toml:<tool>` for managed mise keys).
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    /// Which capability materialised each `files` entry, for entries copied
    /// from a provider checkout rather than embedded content. Everything
    /// else never appears here.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub owners: BTreeMap<String, String>,
}
```
<!-- /sokf:include -->

## Behaviour

### Files

superdev MUST write the lock to `.superdev/lock.toml`, committed, and
other tools SHOULD NOT edit it: it records what the last apply actually
did, so a hand edit makes it a record of something that never happened.
A missing file MUST be read as an empty lock.

It is read, though, and that is why it is a contract. A reviewer reads it
to see what a sync changed, and CI reads it to tell a clean tree from a
drifted one. Its counterpart is the manifest, which says what the repo
*wants*; the lock says what it *got*.

An entry superdev merges into a file it does not own MUST be hashed
under `<file>:<pointer>` rather than as a whole file, because the rest
of that file is the user's. `[[packs]]` is absent when no pack was
named. `rev` and `digest` MUST be absent together for a path source: a
directory is read afresh every run, so there are no pinned bytes to
name.

### Unknown content

A section or key superdev does not know MUST be ignored on read and
MUST NOT survive the next save: the file is superdev's to write, and the
next apply rewrites it whole from what it did. A legacy `owners` table
from an older binary MUST be cleared whole on the first sync and never
written again.

### Compatibility

A reader MUST NOT conclude more from a hash than two things: superdev
wrote that file, and it wrote those bytes. Drift is not decided here —
it is found by comparing a file against the content the blueprint wants
— so a hash that no longer matches means the file was edited after
superdev wrote it, which is what lets an apply say so before
overwriting, and back the file up first.

An entry no component claims any more is an orphan. Content still
hashing to the locked value is superdev's own residue and MUST be
removed; content the user changed MUST be left where it is and dropped
from the lock with a line saying so.

A lock from an older binary MUST load. A section it lacks MUST be
treated as absent rather than as an error, which is what lets an upgrade
sync rather than demanding a re-init.

## Stability

Unreleased. The table names and the hash algorithm MAY change without
notice. What holds even so: the file is superdev's to write, a hand edit
MUST NOT be respected, and no command asks the user to repair one — a
lock superdev cannot read MUST be rebuilt by the next apply.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-043-one-contract-schema-and-twelve-kinds]: /knowledge/adrs/active/adr-043-one-contract-schema-and-twelve-kinds.md
