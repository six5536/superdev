---
type: Spec
id: spec-project-templates
title: Project Templates
description: Design for project templates — embedded write-once repo scaffolds init seeds a new repo from, token-substituted and disjoint from capability files — with rust-npm as the first, and the fuller knowledge seed it surfaced.
status: stable
---

# Motivation

superdev sets a repo up for agent-driven development, but the repo shape
itself — workspace layout, launcher packaging, CI — is left to the user.
The concrete want: a "Rust with npm deployment" template of this repo's
own shape, so `superdev init` can seed a new repo that builds, tests and
releases the way this one does. The skill-overrides spec
([spec](2026-08-18-workflows-skill-overrides-design.md)) scoped itself to
skills but chose the kind-scoped assets layout so future artefact kinds
get sibling directories; this is the first such kind.

# Decision

A **project template** ([glossary](../glossary.md)) is a set of
write-once scaffolds embedded in the binary that `init` seeds a repo
from:

- **Scaffolds, plus a recorded name.** Every template file has scaffold
  semantics: written only where absent, the user's from the moment it
  exists, never hashed, never drifting. Existing files win and are
  reported. `config.toml` gains a `[template]` table recording the name
  and the substituted token values — provenance, not management; `sync`
  never revisits template files.
- **Purely files, orthogonal to capabilities.** A template ships repo
  shape only. The capability set and providers stay with the existing
  init flags; a template presets nothing in the manifest beyond its own
  table.
- **Disjoint by construction.** Template paths must not overlap any
  component's claims or scaffolds (`AGENTS.md`, `knowledge/`,
  `.claude/`, `.superdev/`, `.mise.toml`, `.mcp.json`); a build-time
  consistency test enforces it in both directions — the static
  components' writes and claims must all fall under the reserved
  prefixes, and no template target may — so a component growing a file
  outside them fails the test rather than colliding silently.
  Shared-line files are the exception: a component's ensure-line merges
  into whatever exists, so a template may ship the file the lines land
  in (`.gitignore`). The template entry applies first, ahead of every
  capability, so those lines see the template's file.
- **Embedded.** Templates live at `assets/templates/<name>/`, a sibling
  of `skills/` and `overrides/`, and mirror the target repo — the whole
  template installs as one unit under one init-time choice, so identity
  mapping is right here where the condition-scoped kinds need semantic
  paths. Two on-disk deviations, both mechanical: leading dots are
  stripped (a real `.gitignore` in the assets would hide sibling assets
  from git) and a tokenised path segment is written `_slug_`, because
  `:` cannot appear in Windows file names; the embedded table restores
  both in the target paths. The binary is the provenance; new templates
  arrive with releases. Fetchable sources are out of scope.

# Selection

- On a TTY, `init` prompts with the shipped template list plus "none",
  then for the project name, prefilled with the directory name. The
  prompts use `dialoguer` (a new dependency, approved for this).
- `--template <name>` and `--name <name>` answer in advance and skip
  the prompts; `--template none` scripts the explicit "no".
- Without a TTY there is no prompt and no template unless `--template`
  says otherwise — init's non-interactive contract is unchanged, and
  every existing scripted init keeps working.
- An unknown template name fails with the shipped list, matching the
  provider-refusal pattern. `init --help` names the shipped templates.
- Selection logic — the list, validation, the derived default — lives
  behind a small trait tested with a fake; the thin dialoguer adapter
  is `coverage(off)` glue. E2e journeys stay non-TTY and never see a
  prompt.

# Tokens

Three tokens, exact-match only: `{{superdev:project-name}}`,
`{{superdev:project-slug}}`, and `{{superdev:project-ident}}` — the slug
with `_` for `-`, added during implementation because Rust source refers
to a hyphenated crate by its underscore identifier, which the slug cannot
express. Tokens substitute in file contents and in target paths
(`crates/app/{{superdev:project-slug}}/` lands renamed). Anything else
passes through untouched — including GitHub Actions' `${{ … }}`, which
template CI files legitimately contain. No user-defined variables. A name
that yields an empty slug falls back to `project`.

# The rust-npm template

The first shipped template, derived from this repo's shape:

- Workspace and launcher: the Cargo workspace with app and lib crate
  stubs, `rust-toolchain.toml`, `rustfmt.toml`, the `packages/` npm
  launcher and platform-package skeleton, `package.json` scripts,
  `.gitignore`, `.gitattributes`.
- CI workflows: a thin `ci.yml` calling a reusable `checks.yml` (fmt,
  clippy, test matrix), audit, and the tag-driven release pipeline
  building per-target binaries and publishing the npm packages — plus
  `scripts/verify-version.mjs`, the one-version-everywhere gate the
  release workflow runs against the tag. Crates are `publish = false`
  and the pipeline publishes npm only, consistent with the proprietary
  default.
- Repo docs: README, CONTRIBUTING, CHANGELOG seed, SECURITY,
  CODE_OF_CONDUCT — skeletons with the project name substituted. The
  LICENSE ships proprietary — "Copyright (c) the owners of
  {{superdev:project-name}}. All rights reserved." — with no year, so
  nothing goes stale; the user replaces it at will.
- Policy configs: `deny.toml` and `.prettierignore`, so the CI slice
  passes as shipped.

# The knowledge seed

The template want surfaced a gap that belongs elsewhere: a fresh
repo's knowledge bundle is a three-file seed, while a useful one has
this repo's concept structure. The fuller seed goes to the **aokf
component's scaffold**, not the template — every knowledge-enabled
repo gets the concept skeleton (glossary, architecture,
testing-strategy and the rest, emptied to frontmatter, headings and
TBD prompts), template or not. The issue-tracker and domain-docs
concepts stay out of the seed: they record one workflow provider's
conventions and are created by its setup skill. This keeps
`knowledge/` single-owner and the disjointness rule intact. Placing it in the template was
rejected: it would need a carve-out in that rule and leave
non-template repos on the thin seed.

# Out of scope

- Fetchable or third-party template sources — nothing vouches for
  fetched content; revisit with a trust story.
- Templates presetting capability configuration.
- Managed (synced) template files — a third ownership regime; scaffold
  semantics are the point.
- Further templates beyond rust-npm.
