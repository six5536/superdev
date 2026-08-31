---
type: BugReport
id: issue-003-bug-a-local-pack-cannot-remove-what-it-dropped
title: Deleting an item from a local pack leaves its live copy in place, and the drift check stays green
description: A path pack layers rather than replacing, so an item deleted or renamed under pack/ is still written from the embedded snapshot and status --drift exits 0 until the binary is rebuilt; wontfix — the layering rule stands, and the rebuild a pack developer needs anyway is the answer.
lifecycle: wontfix
links:
  - rel: references
    to: contract-007-interface-pack-resolution
---

# Bug: a local pack cannot remove what it dropped

## Won't fix

Decided 2026-08-26. The layering rule stands and nothing is built for this.

A path pack layers because [ADR-004][sokf:adr-004-base-pack-identity]
and [ADR-011][sokf:adr-011-path-pack-identity-is-root-relative]
deliberately keep a directory from being the base, and the only people who
meet this are the ones developing a pack — for whom the binary has to be
rebuilt anyway before the removal is real. Letting an entry declare itself the
base was considered and turned down as machinery for a case the rebuild
already covers.

**The answer is `cargo run -- sync`**, which rebuilds and syncs in one step.

Deleting the live copy by hand is not a second answer, and the workaround
below is corrected accordingly: `.claude/skills/<name>/` is an owned file, so
the next `sync` finds it missing and writes it straight back from the copy
compiled into the binary. It stays gone only once that binary is rebuilt —
which is the same one step that makes the deletion take effect in the first
place.

What stays true, and is the cost of not fixing it: between deleting from
`pack/` and rebuilding, `sync` writes nothing and `status --drift` exits 0.
A contributor who retires a skill and pushes without rebuilding gets a green
CI on a repo that still ships it.
## Summary

Against [C007][sokf:contract-007-interface-pack-resolution].

This repository resolves its own content from `/pack/` so an edit lands with no
rebuild. Additions and modifications work. **Removals do not**: delete a skill
from `pack/`, run `sync`, and the live copy is still there, with nothing said
and `status --drift` still green. A contributor retiring or renaming a skill
gets a passing CI and a repo that keeps shipping the old one until someone
rebuilds the binary.

## Environment

- Version/commit: 0.2.0 / slice 14 of P003
- Platform: all

## Steps to reproduce

1. In a repo pinning `./pack`, `rm -rf pack/knowledge/skills/how-do-i`.
2. `superdev sync`
3. `superdev status --drift`

## Expected behaviour

Either the live copy goes with the item, or the run says plainly that a local
pack cannot take one away and names the rebuild as the way to do it.

## Actual behaviour

Step 2 prints `knowledge (aokf): ok` and writes nothing.
`.claude/skills/how-do-i/` is untouched. Step 3 exits 0.

## Root cause (if known)

An entry replaces the embedded snapshot only when it *is* the snapshot's
source, which `is_base` tests by identity and only a git source can satisfy
([ADR-004][sokf:adr-004-base-pack-identity],
[ADR-011][sokf:adr-011-path-pack-identity-is-root-relative]). A path
pack therefore layers, and a layer adds and supersedes but never removes — by
design, since a stranger's pack must not delete superdev's content. The
dogfood is the first case where the layering source and the base are meant to
be the same content, which is where the rule stops fitting.

## Proposed fix / workaround

- Fix: none. See [Won't fix](#wont-fix) above. The two candidates were an
  entry that declares itself the base — which the identity rules deliberately
  do not allow — and a report when the embedded snapshot carries an item the
  local pack no longer has, which is true of every third-party pack too and
  so is noise unless the entry has already said it means to be the whole
  content.
- Workaround: rebuild the binary after deleting or renaming under `pack/`;
  `cargo run -- sync` does both in one step. Deleting the live copy by hand
  instead does not work — the next `sync` writes it back from the embedded
  snapshot.

## Regression risk

`pack/resolve.rs`'s base decision and the orphan pass. A test would drop an
item from a local pack and assert the live copy goes, or that the run says why
it cannot.

<!-- sokf:links -->
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:adr-011-path-pack-identity-is-root-relative]: /knowledge/adrs/active/adr-011-path-pack-identity-is-root-relative.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
