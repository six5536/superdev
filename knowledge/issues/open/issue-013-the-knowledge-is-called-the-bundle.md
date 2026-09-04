---
type: Issue
id: issue-013-the-knowledge-is-called-the-bundle
title: The SOKF knowledge is called "the bundle" on every surface, and the word describes nothing
description: The specification, the documents and the CLI are clear of the word, but "bundle" remains in 365 places inside crates/ — load_bundle, bundle_dir, BundleManifest, bundle_root — where it tells a reader nothing about a directory of markdown the repository owns.
kind: chore
lifecycle: open
---

# Chore: the SOKF knowledge is called "the bundle"

## Summary

`bundle` was the specification's term for the tree under `knowledge/`, and
from there it reached every layer superdev owns. Elsewhere in software a
bundle is a packaged archive — something built, shipped and opened. Here it
is a directory of markdown the repository keeps and edits in place. The word
carries none of that, so every reader who meets it has to be told what it
means, and the term that would have told them is already in use beside it.

No defect in code: a naming decision that was never taken. The word entered
with the specification, where §1 defined it and §2 built on it, and
everything downstream adopted the spec's vocabulary without asking whether it
read well outside the spec. Nothing ever contested it, so nothing ever
stopped it spreading. The specification is this project's own document, not a
vendored standard, which is why the change could start there and nowhere
else: every other use was downstream of §1.

## Context

365 occurrences remain, all inside `crates/`
(`git grep -Io 'bundle' -- crates | wc -l`). Everything outside the code is
done:

- **The specification** — done. §1 defines **SOKF knowledge** as the term
  for the document tree and says why it is always given in full; §2 is
  headed "Knowledge structure". Neither copy uses the word.
- **The documents** — done. `knowledge/`, `.agents/`, `.claude/`, `pack/`,
  the README and CONTRIBUTING no longer use it for the tree. What remains
  there is the packaging sense — codegraph's release bundles, a pack as a
  bundle of content — and quoted code identifiers.
- **The CLI** — done in P008. `--bundle <DIR>` is `--knowledge <DIR>`, the
  JSON report's `"bundle"` key is `"knowledge"`, the summary line reads
  `knowledge: ./knowledge`, and the startup error says "no SOKF knowledge
  here". No alias: pre-1.0, and the whole `aokf` verb group went at the same
  time, so there was nothing left for one to be compatible with.
- **MCP** — done. The server introduces itself as "Read-only access to this
  repository's SOKF knowledge."
- **The Rust API** — outstanding, and the whole of what is left:
  `load_bundle` (30 uses), `bundle_dir` (28), `BundleManifest` (5),
  `bundle_root` (5), the module `sokf/bundle.rs`, and the parameter and
  local names that follow them through `validate`, the components and the
  tests.

## Behaviour

Done means:

- `git grep -Io 'bundle' -- crates` returns only the packaging sense: a
  content pack, a codegraph release. Nothing naming the SOKF knowledge.
- `superdev-core` compiles and its tests pass under the renamed API. This is
  a breaking change to a published crate, so it lands in one commit with the
  changelog entry that records it — the same treatment P008's module rename
  had.
- A reader who has never opened the specification understands the name.

## Comments

The prose half landed before P008 and the CLI half inside it. Only the Rust
API is left, and P008 renamed the module around it — `aokf/` became `sokf/`
— without touching the names inside. That is the one sweep remaining, and
it is mechanical: no behaviour changes, and the tests move with the names.
