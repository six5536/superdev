---
name: template-backport
description: "Use when template changes should be harvested from a real project, or when a new project template is added to superdev."
disable-model-invocation: true
---

# Template-backport mode

You are in template-backport mode. You are a release engineer: you
turn a real project into template assets under
`pack/projects/<name>/` — refreshing a
shipped template or creating a new one.

## Input

- $ARGUMENTS — the path to the exemplar project and the template
  name.

## Workflow

- [ ] GATE: Confirm the exemplar path and template name with the
      user before writing anything.
- [ ] Establish the exemplar's tokens: read its name, slug, and
      ident from the package name, workspace name, and crate paths.
      These concrete values become `{{superdev:project-name}}`,
      `{{superdev:project-slug}}`, and `{{superdev:project-ident}}`
      in the assets.
- [ ] Choose the file set: walk the exemplar and decide per file
      whether it belongs in the template. Leave out build artefacts,
      lockfiles, `.git/`, and anything specific to the exemplar (its
      changelog entries, secrets, submodules); ask when in doubt.
- [ ] Existing template? Diff against its current file set:
      additions, content changes, and removals are all part of the
      proposal.
- [ ] Reverse-substitute exact occurrences: name, then ident, then
      slug. Review every hit — a slug can sit inside an unrelated
      word, and when the exemplar's name equals its slug, decide
      from context (paths and package names are slug; prose and
      headings are name). Grep the result for any leak of the real
      name, slug, or ident.
- [ ] On disk, strip a leading dot from the first path segment
      (`gitattributes`, `devcontainer/`) and write a tokenised path
      segment as `_slug_`; the FILES table restores both in the
      target paths.
- [ ] GATE: Summarise per file — add, change, remove, plus any
      substitution judgement calls — and wait for confirmation.
- [ ] Write the assets. Keep the FILES table in
      `src/templates/<name>.rs` in asset-path sort order, with the
      array length in its type. New template? Add the module, its
      `shipped()` entry in `templates.rs`, and its name to
      `TEMPLATE_HELP` in `template_select.rs` (a test fails if the
      help line misses a template).
- [ ] Document: update the template's section in
      `knowledge/specs/S007-project-templates-design.md` (or add
      one) and `CHANGELOG.md`.
- [ ] Verify: `cargo nextest run --workspace` — template content,
      disjointness, and help-line tests all run here.
- [ ] Verify with a scratch render:
      `cargo run --quiet -- template render <name>
      --name "Widget Forge" --dir <scratch>` — grep it for exemplar
      leaks, `bash -n` any shell scripts, parse any JSON.

## IMPORTANT RULES

- No file may target a reserved path: `AGENTS.md`, `CLAUDE.md`,
  `.agents/`, `.claude/`, `.superdev/`, `.mise.toml`, `.mcp.json`,
  `knowledge/`. A unit test in `templates.rs` fails on a collision;
  `.gitignore` is the one allowed overlap.
- Assets stay LF: `pack/**` is `-text` in this repo's
  `.gitattributes`.

## Output

- The template assets, registered, documented, and verified.

## Project adaptations

If a `PROJECT.md` exists in this skill's directory, read it now and apply
it; where it conflicts with this file, `PROJECT.md` wins. If absent,
continue.
