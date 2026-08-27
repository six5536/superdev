---
type: Chore
id: issue-013-the-knowledge-is-called-the-bundle
title: The canonical knowledge is called "the bundle" on every surface, and the word describes nothing
description: The AOKF spec and every document are clear of the word, but "bundle" remains in 625 places — the --bundle flag, the JSON report key and the Rust API — where it tells a reader nothing about a directory of markdown the repository owns.
status: draft
tags: [needs-triage]
---

# Chore: the canonical knowledge is called "the bundle" on every surface, and the word describes nothing

## Summary

`bundle` is the AOKF spec's term for the tree under `knowledge/`, and from
there it has reached every layer superdev owns: the `--bundle` flag, the
`"bundle"` key in the JSON report, `load_bundle` and `BundleManifest` in the
public API, the MCP server's own description of itself, and the prose of 163
files. Elsewhere in software a bundle is a packaged archive — something
built, shipped and opened. Here it is a directory of markdown the repository
keeps and edits in place. The word carries none of that, so every reader who
meets it has to be told what it means, and the terms that would have told
them — knowledge, AOKF knowledge, canonical knowledge — are already in use
beside it.

## Surfaces

625 occurrences across 74 tracked files, 492 of them in `crates/`. The
remainder are the packaging sense of the word (codegraph's release
bundles, a pack as a bundle of content), quoted code identifiers, and the
changelog's record of what shipped. The term was defined in exactly one
place and used everywhere:

- **AOKF SPEC** — **fixed.** §1 Terminology now reads "**Canonical project
  knowledge**, or just **the canonical knowledge**", §2 is headed "Knowledge
  structure", and neither spec copy uses the word. `.agents/` is clear.
  The code below still carries it.
- **CLI** — `--bundle <DIR>` on `validate` and `aokf index`; the `validate`
  summary line; the error `no AOKF bundle here — run \`superdev init\``
  (`crates/app/superdev/src/aokf_cli.rs:87`); the covered-paths line
  `bundle: {}, roots: {}` (`aokf_cli.rs:150`).
- **JSON report** — the `"bundle"` key written at `aokf_cli.rs:134`, read
  back by `crates/app/superdev/tests/cli.rs:171`. A machine-readable name,
  not just prose.
- **Rust API** — the bulk of the 492 occurrences in `crates/`, including
  `load_bundle`, `bundle_dir`, `BundleManifest` and `bundle_root`.
- **MCP** — the `superdev-aokf` server introduces itself as "Read-only
  access to this repository's AOKF knowledge bundle."
- **Knowledge** — **swept.** `knowledge/`, `.claude/`, `pack/`, the README
  and CONTRIBUTING no longer use the word for the tree. What remains there
  is the packaging sense (codegraph's release bundles, a pack as a bundle of
  content) and quoted code identifiers.
- **Pack** — **swept**, so no repository superdev seeds inherits the word.

## Definition of done

One name for the canonical knowledge, and one that says what it is:
**canonical project knowledge**, shortened to **the canonical knowledge** in running
prose. `AOKF` names the format and its specification, never the tree —
"the AOKF bundle" and "AOKF knowledge" are both retired. A reader who has
never opened the specification understands the name.


## Comments

The prose half landed before P008: the specification, the skills, the
concepts and the schemas no longer say "bundle". The code half did not.
`load_bundle` (30 uses), `bundle_dir` (40), `BundleManifest` and the
`--bundle` flag still carry the word, and P008 renamed the module around
them without touching them. This stays open for that.
