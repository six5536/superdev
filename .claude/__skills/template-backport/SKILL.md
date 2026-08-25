---
name: template-backport
description: Update or create an embedded project template under crates/lib/superdev-core/assets/templates/ from an existing exemplar project. Use when template changes should be harvested from a real project, or when a new project template is being added to superdev.
disable-model-invocation: true
---

# Template backport

Turn a real project into template assets under
`crates/lib/superdev-core/assets/templates/<name>/` — refreshing a shipped
template or creating a new one.

Inputs: the path to the exemplar project and the template name. Confirm
both with the user before writing anything.

## 1. Establish the exemplar's tokens

Read the exemplar's name, slug and ident from its package name, workspace
name and crate paths. These concrete values become
`{{superdev:project-name}}`, `{{superdev:project-slug}}` and
`{{superdev:project-ident}}` in the assets.

## 2. Choose the file set

Walk the exemplar and decide per file whether it belongs in the template:

- Nothing may target a reserved path: `AGENTS.md`, `CLAUDE.md`, `.agents/`,
  `.claude/`, `.superdev/`, `.mise.toml`, `.mcp.json`, `knowledge/`. A unit
  test in `templates.rs` fails on a collision; `.gitignore` is the one
  allowed overlap.
- Leave out build artefacts, lockfiles, `.git/`, and anything specific to
  the exemplar (its changelog entries, secrets, submodules). Ask when in
  doubt.
- For an existing template, diff against its current file set: additions,
  content changes and removals are all part of the proposal.

## 3. Reverse-substitute

Replace exact occurrences: name, then ident, then slug. Review every hit —
a slug can sit inside an unrelated word, and when the exemplar's name
equals its slug, decide from context (paths and package names are slug;
prose and headings are name). Grep the result for any leak of the real
name, slug or ident.

On disk, strip a leading dot from the first path segment (`gitattributes`,
`devcontainer/`) and write a tokenised path segment as `_slug_`; the FILES
table restores both in the target paths.

## 4. Confirm, then write

Summarise per file — add, change, remove, plus any substitution judgement
calls — and wait for confirmation. Then write the assets and:

- keep the FILES table in `src/templates/<name>.rs` in asset-path sort
  order, with the array length in its type
- for a new template, add the module, its `shipped()` entry in
  `templates.rs`, and its name to `TEMPLATE_HELP` in `template_select.rs`
  (a test fails if the help line misses a template)

## 5. Document

Update the template's section in
`knowledge/specs/S007-project-templates-design.md` (or add one) and
`CHANGELOG.md`.

## 6. Verify

- `cargo nextest run --workspace` — template content, disjointness and
  help-line tests all run here.
- A scratch render:
  `cargo run --quiet -- template render <name> --name "Widget Forge" --dir <scratch>`
  — grep it for exemplar leaks, `bash -n` any shell scripts, parse any
  JSON.
- Assets stay LF: `crates/lib/superdev-core/assets/**` is `-text` in this
  repo's `.gitattributes`.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
