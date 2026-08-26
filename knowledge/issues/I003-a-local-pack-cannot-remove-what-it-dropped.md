---
type: Issue
id: issue-003-a-local-pack-cannot-remove-what-it-dropped
title: Deleting an item from a local pack leaves its live copy in place, and the drift check stays green
description: A path pack layers rather than replacing, so an item deleted or renamed under pack/ is still written from the embedded snapshot; sync reports nothing and status --drift exits 0 until the binary is rebuilt.
status: draft
tags: [needs-triage]
links:
  - rel: references
    to: spec-content-packs
---

# Bug: a local pack cannot remove what it dropped

## Summary

Against [S014](../specs/S014-content-packs-design.md).

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
([ADR-004](../decisions/D004-base-pack-identity.md),
[ADR-011](../decisions/D011-path-pack-identity-is-root-relative.md)). A path
pack therefore layers, and a layer adds and supersedes but never removes — by
design, since a stranger's pack must not delete superdev's content. The
dogfood is the first case where the layering source and the base are meant to
be the same content, which is where the rule stops fitting.

## Proposed fix / workaround

- Fix: needs an interface decision, not a local change. Either a way to say
  "this path pack is the base" — which the identity rules deliberately do not
  allow — or a check that notices the embedded snapshot carrying an item the
  local pack no longer has, and reports it rather than silently writing it.
- Workaround: rebuild the binary after deleting or renaming under `pack/`;
  `cargo run -- sync` does both in one step.

## Regression risk

`pack/resolve.rs`'s base decision and the orphan pass. A test would drop an
item from a local pack and assert the live copy goes, or that the run says why
it cannot.
