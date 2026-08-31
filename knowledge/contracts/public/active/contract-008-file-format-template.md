---
type: FileFormatContract
id: contract-008-file-format-template
title: Template Format Contract
description: What a project template is — where its tree lives in the pack, the five substitution tokens, the write-once promise to a seeded repo, and one section per shipped template.
lifecycle: active
links:
  - rel: references
    to: contract-005-file-format-pack
    note: A template is a pack item; the pack format names it by its tree.
sources:
  - id: rust-npm-src
    resource: /pack/projects/rust-npm
    title: rust-npm template tree
  - id: web-src
    resource: /pack/projects/web-react-android-ios-native
    title: web-react-android-ios-native template tree
---

# File format contract: template

What a project template is: where its tree lives in the pack, the five
substitution tokens, and the write-once promise to a seeded repo.

## Files

A template is a directory tree under `pack/projects/<name>/` — a pack item
per the [pack format][sokf:contract-005-file-format-pack], written by
maintainers through the `template-backport` skill and read by the binary.
Seeding (`init --template`, or the prompt the
[CLI contract][sokf:contract-002-cli-superdev] describes) writes the tree
into the target repository, token-substituted; from that moment every
seeded file is the user's, and the user is authoritative — the engine
never hashes, syncs or revisits one. A template stays disjoint from
capability files, and the canonical knowledge is a reserved path that
belongs to the knowledge component: every knowledge-enabled repo gets the
concept skeleton from that component's scaffold, template or not, so no
template ships one.

## Shape

```text
pack/projects/<name>/        # one template; <name> is <platforms>-<stack>
  <the tree to seed>         # UTF-8 files, token-substituted on write

# The token vocabulary, exact-match only, substituted in file contents
# and in target paths (crates/app/{{superdev:project-slug}}/ lands renamed):
{{superdev:project-name}}     # the name as the user gave it
{{superdev:project-slug}}     # lowercased, hyphen-separated
{{superdev:project-ident}}    # slug with _ for -   (Rust identifiers)
{{superdev:project-compact}}  # slug with - dropped (Android/iOS app ids)
{{superdev:project-pascal}}   # slug segments capitalised and joined
                              # (Swift/Kotlin types, Xcode, Gradle)
```

Each spelling exists because a target language forbids the slug itself in
that position; the derivations live on `Tokens`, so `substitute` and
`template render`'s printout share one source of truth. There are no
user-defined variables, and a name that yields an empty slug falls back
to `project`.

### Template: rust-npm

The first shipped template, derived from this repo's shape.[^rust-npm-src]

- Workspace and launcher: the Cargo workspace with app and lib crate
  stubs, `rust-toolchain.toml`, `rustfmt.toml`, the `packages/` npm
  launcher and platform-package skeleton, `package.json` scripts,
  `.gitignore`, `.gitattributes`.
- CI workflows: a thin `ci.yml` calling a reusable `checks.yml`, audit,
  and the tag-driven release pipeline with its scripts
  (`verify-version.mjs`, `set-version.mjs`, `release.mjs`, the two
  smokes). Crates are `publish = false`; the pipeline publishes npm
  only. The stub binary honours the exit-code contract the smokes
  assert: usage errors exit 2.
- Repo docs and policy configs: README, CONTRIBUTING, CHANGELOG seed,
  SECURITY, CODE_OF_CONDUCT, `deny.toml`, `.prettierignore`; the
  LICENSE ships proprietary with no year, for the user to replace.
- A dev container built for a superdev-managed repo: mise owns the tool
  versions in a seeded `mise.toml` (`.mise.toml` is superdev's to
  write, and mise merges the pair); Rust is pinned twice by necessity —
  `rust-toolchain.toml` for CI and rustup, mise's `RUSTUP_TOOLCHAIN`
  export — and both files say so. Named volumes carry the slug token so
  two seeded projects never share one `target/`; `.gitattributes`
  forces `*.sh` and `.devcontainer/**` to LF.

### Template: web-react-android-ios-native

The second shipped template: one product as three native codebases,
backported from a real three-platform project.[^web-src] Naming
anticipates siblings — `<platforms>-<stack>` — so a variant gets its own
name rather than a flag.

- Three hello-world app stubs that pass CI as shipped: `apps/web`
  (Vite, React Router, Tailwind, Vitest), `apps/android-native`
  (Compose, Robolectric so `gradlew test` needs no emulator),
  `apps/ios-native` (SwiftUI, SPM).
- Agent debug tooling, the reason this stack is worth templating: a
  debug-build-only HTTP debug server per platform, an MCP server
  wrapping its API, and `scripts/` for the
  build/install/launch/logs/screenshot loop with a host-side adb
  bridge.
- A fastlane release pipeline keyed off `release/release.yaml`, the one
  place a version or app id is written, plus a store-metadata skeleton;
  the same thin-`ci.yml`-over-`checks.yml` CI shape as rust-npm.
- An Android-capable dev container: SDK and cmdline-tools in the image,
  amd64 multiarch for Rosetta, and the adb server location resolved by
  a runtime probe rather than a static value.

Two artefacts cannot be seeded and are bootstrapped instead, documented
in the template's `docs/BUILD.md`: the Gradle wrapper jar (binary, and
templates are UTF-8 `include_str!`) and the Xcode project, which
`xcodegen` generates from the committed `project.yml`. `checks.yml` uses
`gradle` via `setup-gradle` rather than `./gradlew`, so CI is green
before either bootstrap runs.

## Compatibility

Anything that is not one of the five tokens passes through untouched —
including GitHub Actions' `${{ … }}`, which template CI files
legitimately contain. Seeding never overwrites: an existing file wins and
is reported as kept, so re-running `init` in a populated repo is safe. An
unknown template name fails naming the shipped set. `Action::WriteFile`
sets no mode, so nothing seeded is executable and every script is invoked
through an interpreter; a file that needs its executable bit (`gradlew`)
says so in the template's docs. The manifest's `[template]` table gains
an optional `version`; manifests from before the field parse unchanged.

## Stability

The token vocabulary is promised: a token's meaning never changes, and a
new spelling may be added but never replaces one. The write-once promise
is permanent — template files are not hashed, not synced and never drift,
and the only update path is the `template-update` skill, which discovers
the template (`[template]` in the manifest, or shape analysis confirmed
with the user), renders the binary's current content, three-way-compares
against the file as seeded (recovered from git history), and applies what
the user approves as ordinary user edits — restamping `version` so an
update can short-circuit when the repo already matches. `template list`
and `template render` stay read-only views. The shipped-template sections
above grow one per template as `template-backport` captures them; a
template may be removed only with a release-notes notice, since a seeded
repo keeps working regardless.

[^rust-npm-src]: rust-npm template tree
[^web-src]: web-react-android-ios-native template tree

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-005-file-format-pack]: /knowledge/contracts/public/active/contract-005-file-format-pack.md
