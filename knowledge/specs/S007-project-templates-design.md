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
([spec](S006-workflows-skill-overrides-design.md)) scoped itself to
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

Five tokens, exact-match only: `{{superdev:project-name}}`,
`{{superdev:project-slug}}`, and three spellings of the slug that a
target language forbids the slug itself in:

- `{{superdev:project-ident}}` — the slug with `_` for `-`, because Rust
  source refers to a hyphenated crate by its underscore identifier.
- `{{superdev:project-compact}}` — the slug with the hyphens dropped, for
  reverse-domain app ids: Android forbids `-` in an `applicationId` and
  iOS forbids `_` in a bundle id, so only alphanumeric segments satisfy
  both.
- `{{superdev:project-pascal}}` — the slug's segments capitalised and
  joined, for Swift and Kotlin type names, Xcode project and scheme
  names, and Gradle root projects, none of which admit a separator.

Each was added when a template needed a spelling no existing token could
express; the derivations live on `Tokens` so `substitute` and
`template render`'s printout share one source of truth. Tokens substitute
in file contents and in target paths (`crates/app/{{superdev:project-slug}}/`
lands renamed). Anything else passes through untouched — including GitHub
Actions' `${{ … }}`, which template CI files legitimately contain. No
user-defined variables. A name that yields an empty slug falls back to
`project`.

# The rust-npm template

The first shipped template, derived from this repo's shape:

- Workspace and launcher: the Cargo workspace with app and lib crate
  stubs, `rust-toolchain.toml`, `rustfmt.toml`, the `packages/` npm
  launcher and platform-package skeleton, `package.json` scripts,
  `.gitignore`, `.gitattributes`.
- CI workflows: a thin `ci.yml` calling a reusable `checks.yml` (fmt,
  clippy, test matrix), audit, and the tag-driven release pipeline
  building, smoke-testing and publishing per-target binaries — plus the
  release scripts the workflows and the runbook call:
  `verify-version.mjs` (the one-version-everywhere gate),
  `set-version.mjs`, `release.mjs` (changelog gate, bump, commit, tag),
  `release-smoke.mjs` and `launcher-smoke.mjs`. Crates are
  `publish = false` and the pipeline publishes npm only, consistent
  with the proprietary default. The stub binary honours the exit-code
  contract the smokes assert: usage errors exit 2.
- Repo docs: README, CONTRIBUTING, CHANGELOG seed, SECURITY,
  CODE_OF_CONDUCT — skeletons with the project name substituted. The
  LICENSE ships proprietary — "Copyright (c) the owners of
  {{superdev:project-name}}. All rights reserved." — with no year, so
  nothing goes stale; the user replaces it at will.
- Policy configs: `deny.toml` and `.prettierignore`, so the CI slice
  passes as shipped.
- A dev container: `.devcontainer/` with the definition, its feature
  lock, a Dockerfile and `post-create.sh`. It targets a
  superdev-managed repo, not just a Rust one — the aokf hook calls a
  bare `superdev`, so the container has to supply it or validation goes
  silently unenforced. mise owns the tool versions: Rust, Node and the
  cargo tooling are pinned in a seeded `mise.toml` and installed by
  `post-create.sh`, so the same versions are one `mise install` away
  outside the container; only git stays a devcontainer feature, and the
  image bootstraps mise itself. The project's file is `mise.toml`, not
  `.mise.toml`, for two reasons: `.mise.toml` is superdev's to write,
  and the pin phase creates it before any scaffold applies, which would
  make a seeded `.mise.toml` skip as already-existing. mise merges
  every config in a directory, so the pair coexists. Rust is pinned
  twice by necessity — `rust-toolchain.toml` is what CI and a plain
  rustup read, mise exports `RUSTUP_TOOLCHAIN` — and both files say so.
  Tools that belong to the container rather than the project —
  `superdev`, Claude Code — stay in the container's global mise config.
  `Action::WriteFile` sets no mode, so nothing seeded is executable and
  every script is invoked through an interpreter.
  Named volumes carry the slug token: they are global to the Docker
  host, and hardcoded names would make two seeded projects share one
  `target/`. The shipped `.gitattributes` forces `*.sh` and
  `.devcontainer/**` to LF, because bash and Docker both fail on the
  CRLF a Windows checkout would otherwise introduce.

# The web-react-android-ios-native template

The second shipped template: one product as three native codebases — a
React web app, a Kotlin/Jetpack Compose Android app and a SwiftUI iOS app
— backported from a real three-platform project. Naming anticipates
siblings: `<platforms>-<stack>`, so a Capacitor or shared-Rust-core
variant gets its own name rather than a flag.

- Three app stubs: `apps/web` (Vite, React Router, Tailwind, Vitest),
  `apps/android-native` (Compose, with Robolectric so `gradlew test`
  needs no emulator) and `apps/ios-native` (SwiftUI, SPM). Each is
  hello-world and passes CI as shipped; the exemplar's domain code stays
  out.
- Agent debug tooling, the reason this stack is worth templating: an HTTP
  debug server compiled into debug builds only
  (`libs/native-debug-server-{android,ios}`), an MCP server wrapping its
  API (`libs/debug-mcp-server`), and `scripts/` — a dev CLI for the
  build/install/launch/logs/screenshot loop plus a host-side adb bridge.
  Together they let an agent drive a real debug build on a device.
- A fastlane release pipeline keyed off `release/release.yaml`, the one
  place a version or app id is written, plus a store-metadata skeleton.
- The same CI shape as rust-npm: a thin `ci.yml` calling a reusable
  `checks.yml`, which `release.yml` also calls, so the release gate
  cannot drift from CI. Tool versions in the workflows track the
  `mise.toml` pins.
- An Android-capable dev container: the Android SDK and cmdline-tools in
  the image, amd64 multiarch so the SDK's x86_64 tools run under Rosetta
  on Apple Silicon, and the adb server location resolved by a runtime
  probe (`scripts/adb-env.sh`) rather than a static value — mise's
  `{{ os() }}` is `linux` inside the container regardless of host, and
  `containerEnv` cannot omit a variable, which is what a Linux host with
  USB passed through needs.

Two artefacts cannot be seeded and are bootstrapped instead, documented
in the template's `docs/BUILD.md`: the Gradle wrapper jar (binary, and
templates are UTF-8 `include_str!`) and the Xcode project, which
`xcodegen` generates from the committed `project.yml`. `checks.yml` uses
`gradle` via `setup-gradle` rather than `./gradlew`, so CI is green
before either bootstrap runs — and `Action::WriteFile` sets no mode, so
`gradlew` needs its executable bit set by hand.

The exemplar carried copy-paste from unrelated projects — Rust and
PostgreSQL tool pins, an NDK, another project's launch configs — none of
which the template inherits. The knowledge bundle stays out too: it is a
reserved path that belongs to the aokf component, so the template ships
`docs/` for `aokf-bootstrap` to harvest and points at it from the README.

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

# Updating a seeded repo

Template evolution reaches existing repos through a skill, never the
engine. The engine's stance is unchanged — template files are not
hashed, not synced, and never drift; what the out-of-scope item below
rejects is *engine-managed* template files. The `template-update` pack
skill is the one update path: it discovers the template (`[template]`
in the manifest, or — in a repo never seeded — shape analysis against
the shipped list, confirmed either way), renders the binary's current
content, three-way-compares using the file as seeded (recovered from
git history) as the base, and applies what the user approves — a
summary first, then per-area questions, per-file only at real
conflicts. Every write is an ordinary user edit.

Two engine surfaces exist for it: `superdev template list`, and
`superdev template render <name> --name <project-name> --dir <dir>` —
read-only views of the shipped templates. Render also prints the
derived token values, so nothing outside the engine re-derives slug
rules. `[template]` gains an optional `version`, stamped by `init` and
restamped by the skill, so an update can short-circuit when the repo
already matches the binary; manifests from before the field parse
unchanged. Adoption — running the skill where no `[template]` exists —
writes the table after the fact: the manifest is the user's
declaration, and the skill edits it as the user.

The reverse direction is this repo's own `template-backport` skill
(unmanaged): harvest an exemplar project into
a template's assets — reverse token substitution, the asset-layout
deviations, the FILES table — to refresh a shipped template or create a
new one.

# Out of scope

- Fetchable or third-party template sources — nothing vouches for
  fetched content; revisit with a trust story.
- Templates presetting capability configuration.
- Managed (synced) template files — a third ownership regime; scaffold
  semantics are the point. Skill-mediated updates are deliberately not
  this — see "Updating a seeded repo".
- Further templates beyond rust-npm.
