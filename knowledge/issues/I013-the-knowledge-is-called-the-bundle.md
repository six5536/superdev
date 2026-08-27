---
type: Issue
id: issue-013-the-knowledge-is-called-the-bundle
title: The canonical knowledge is called "the bundle" on every surface, and the word describes nothing
description: AOKF SPEC §1 names the knowledge tree a "bundle" and the word has spread to 880 places — the --bundle flag, the JSON report key, the Rust API, the skills and the concepts — where it tells a reader nothing about a directory of markdown the repository owns.
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
`knowledge` on its own, `AOKF knowledge` where the format is the point,
`canonical knowledge` where authority is the point. A reader who has never
opened the AOKF spec understands all three.

## Actual behaviour

880 occurrences across 163 tracked files. The term is defined in exactly one
place and used everywhere:

- **AOKF SPEC** — §1 Terminology: "**Bundle**: the directory tree of
  knowledge documents", and §2 is headed "Bundle structure". 20 uses. This
  is the origin; everything below quotes it.
- **CLI** — `--bundle <DIR>` on `validate` and `aokf index`; the `validate`
  summary line; the error `no AOKF bundle here — run \`superdev init\``
  (`crates/app/superdev/src/aokf_cli.rs:87`); the covered-paths line
  `bundle: {}, roots: {}` (`aokf_cli.rs:150`).
- **JSON report** — the `"bundle"` key written at `aokf_cli.rs:134`, read
  back by `crates/app/superdev/tests/cli.rs:171`. A machine-readable name,
  not just prose.
- **Rust API** — 362 occurrences in `crates/`, including `load_bundle` (33),
  `bundle_dir` (40), `BundleManifest` and `bundle_root`.
- **MCP** — the `superdev-aokf` server introduces itself as "Read-only
  access to this repository's AOKF knowledge bundle."
- **Knowledge** — 199 occurrences under `knowledge/`, starting with the
  first line of `knowledge/issue-tracker.md` ("Issues live as markdown files
  in this bundle") and including the live manifest's own description,
  "Canonical project knowledge for superdev, as an AOKF bundle."
- **Pack** — 111 occurrences under `pack/`, so every repository superdev
  seeds inherits the word.

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

- Settle the replacement first: `knowledge` as the bare noun, `AOKF
  knowledge` where the format matters, `canonical knowledge` where authority
  matters. `bundle` is retired outright.
- Change the spec first — §1 Terminology and the §2 heading — and bump the
  AOKF version, because the manifest's `aokf` key is what tells a consumer
  which vocabulary a tree targets.
- Rename the user-facing surfaces: `--bundle` becomes `--knowledge`, with
  the old spelling accepted as a deprecated alias for one release; the JSON
  key `"bundle"` becomes `"knowledge"`; the help text and error strings
  follow.
- Rename the API: `load_bundle`, `BundleManifest`, `bundle_dir`,
  `bundle_root`. This is a breaking change to `superdev-core` and belongs in
  one commit with the release note.
- Sweep the prose in `knowledge/`, `.agents/`, `.claude/` and `pack/` in the
  same change. `pack/` ships into other repositories, so the live copies and
  the packed copies must not diverge across releases.
- Workaround: none. The word is understood by everyone who has read the spec
  and by nobody else.

## Regression risk

The `--bundle` flag and the `"bundle"` JSON key are consumed outside the
binary — the PostToolUse hook, the MCP server and any script reading the
report — so the alias and the release note are what keep an existing caller
working. Dropping the alias without a release is the way this breaks
silently.

`crates/lib/superdev-core/tests/fixtures/aokf/clean/manifest.aokf.yaml`
carries the word in a fixture description. The AOKF goldens were rewritten by
hand when the conformance ladder went (see [the conformance
decision](../decisions/D017-aokf-conformance-is-pass-or-fail.md)), so nothing
can re-derive them and any golden text carrying the word changes as a recorded
projection rather than a rerun. Everything else — test names, temp directory names — is inert and
safe to leave or sweep at will.
