---
type: Chore
id: issue-013-the-knowledge-is-called-the-bundle
title: The canonical knowledge is called "the bundle" on every surface, and the word describes nothing
description: The AOKF spec and every document are clear of the word, but "bundle" remains in 625 places — the --bundle flag, the JSON report key and the Rust API — where it tells a reader nothing about a directory of markdown the repository owns.
status: draft
tags: [needs-triage]
---

# Bug: the canonical knowledge is called "the bundle" on every surface, and the word describes nothing

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

## Environment

- Version/commit: superdev 0.2.0, AOKF 0.3
- Platform: any; this is a naming defect on every surface, not a runtime one

## Steps to reproduce

1. Run `superdev validate --help` and read `--bundle <DIR>` and the summary
   line, "Check the knowledge bundle and the superdev-format files".
2. Run `superdev validate --json` and read the top-level `"bundle"` key.
3. Run `git ls-files | grep -v '^__old/\|^awa_experiment/\|^submodules/' | xargs grep -io bundle | wc -l`.

## Expected behaviour

One name for the canonical knowledge, and one that says what it is:
**canonical project knowledge**, shortened to **the canonical knowledge** in running
prose. `AOKF` names the format and its specification, never the tree —
"the AOKF bundle" and "AOKF knowledge" are both retired. A reader who has
never opened the specification understands the name.

## Actual behaviour

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

## Root cause (if known)

No defect in code: a naming decision that was never taken. The word entered
with the AOKF spec, where §1 defines it and §2 builds on it, and everything
downstream adopted the spec's vocabulary without asking whether it read well
outside the spec. Nothing has ever contested it, so nothing has ever stopped
it spreading.

The spec is this project's own document (`.agents/aokf/SPEC.md`, AOKF 0.3,
Draft), not a vendored standard, so the term can be changed at its source.
That is also why the change cannot start anywhere else: every other use is
downstream of §1.

## Proposed fix / workaround

- ~~Settle the replacement first~~ — settled: **canonical project
  knowledge**, **the canonical knowledge** in running prose. `bundle` is retired
  outright, and `AOKF` no longer modifies the noun.
- ~~Change the spec first~~ — done, in both copies. The AOKF version is
  **not** bumped yet; the manifest's `aokf` key is what tells a consumer
  which vocabulary a tree targets, so decide the bump before the sweep
  lands.
- Rename the user-facing surfaces: `--bundle` becomes `--knowledge`, with
  the old spelling accepted as a deprecated alias for one release; the JSON
  key `"bundle"` becomes `"knowledge"`; the help text and error strings
  follow.
- Rename the API: `load_bundle`, `BundleManifest`, `bundle_dir`,
  `bundle_root`. This is a breaking change to `superdev-core` and belongs in
  one commit with the release note.
- ~~Sweep the prose~~ — done in `knowledge/`, `.agents/`, `.claude/`,
  `pack/`, the README and CONTRIBUTING. The remaining work is code only.
- Workaround: none. The word is understood by everyone who has read the spec
  and by nobody else.

## Regression risk

The `--bundle` flag and the `"bundle"` JSON key are consumed outside the
binary — the PostToolUse hook, the MCP server and any script reading the
report — so the alias and the release note are what keep an existing caller
working. Dropping the alias without a release is the way this breaks
silently.

`crates/lib/superdev-core/tests/fixtures/aokf/clean/manifest.sokf.yaml`
carries the word in a fixture description. The AOKF goldens were rewritten by
hand when the conformance ladder went (see [the conformance
decision](../decisions/D017-aokf-conformance-is-pass-or-fail.md)), so nothing
can re-derive them and any golden text carrying the word changes as a recorded
projection rather than a rerun. Everything else — test names, temp directory names — is inert and
safe to leave or sweep at will.
