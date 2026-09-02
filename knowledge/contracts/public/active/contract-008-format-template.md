---
type: Contract
id: contract-008-format-template
kind: format
title: Format contract for a project template
description: What a project template is — the substitution tokens and the shipped set as the engine declares them, the tree each template is embedded from, the write-once promise to a seeded repo, and what seeding does with what it does not know.
lifecycle: active
resource: /crates/lib/superdev-core/src/templates.rs
links:
  - rel: references
    to: adr-033-a-contract-defines-its-interface
    note: A contract carries a machine-readable definition; here it is the token vocabulary and the shipped set.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The definition is materialised from the `template` regions and bound by the include; the write-once promise and the pass-through rule are bound by `templates.rs`'s and `init`'s tests. Replaces the drift test ADR-036 asked for.
  - rel: references
    to: adr-043-one-contract-schema-and-twelve-kinds
    note: A text format and a binary format are one reader's question, `format`; this contract carries the kind and its id names it. Replaces the `TextFormatContract` type ADR-037 split off.
  - rel: references
    to: contract-005-format-pack
    note: A template is a pack item; the pack format names it by its tree.
sources:
  - id: rust-npm-src
    resource: /pack/projects/rust-npm
    title: rust-npm template tree
  - id: web-src
    resource: /pack/projects/web-react-android-ios-native
    title: web-react-android-ios-native template tree
---

# Format contract: project template

What a project template is: the substitution tokens, the shipped set,
where each tree lives, and the write-once promise to a seeded repo. The
Definition is the engine's own declaration — the five tokens with the
doc comment that says what each substitutes to, the `Template` record
and the registry that enumerates the shipped set, and, per template,
the tree it is embedded from and its name and one-line description.
Behaviour carries what the declaration cannot say: where a template is
rendered to, what seeding does with a file or token it does not know,
and what a seeded repo may rely on. The decisions behind the shape are
[ADR-033][sokf:adr-033-a-contract-defines-its-interface],
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
and [ADR-043][sokf:adr-043-one-contract-schema-and-twelve-kinds].

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/templates.rs#template -->
```rust
/// Replaced by the project name as given (e.g. "My Tool").
pub const TOKEN_NAME: &str = "{{superdev:project-name}}";
/// Replaced by the kebab-case slug (e.g. "my-tool") — crate and package names.
pub const TOKEN_SLUG: &str = "{{superdev:project-slug}}";
/// The slug as a Rust identifier (e.g. "my_tool") — a hyphenated crate name
/// is referenced with underscores in source, which the slug cannot express.
pub const TOKEN_IDENT: &str = "{{superdev:project-ident}}";
/// The slug with the hyphens dropped (e.g. "mytool") — reverse-domain app
/// ids, where Android forbids `-` and iOS forbids `_`, so only alphanumeric
/// segments work on both.
pub const TOKEN_COMPACT: &str = "{{superdev:project-compact}}";
/// The slug in PascalCase (e.g. "MyTool") — Swift and Kotlin type names,
/// Xcode project/scheme names and Gradle root projects, none of which admit
/// a separator.
pub const TOKEN_PASCAL: &str = "{{superdev:project-pascal}}";
/// One shipped project template.
#[derive(Debug)]
pub struct Template {
    /// The name `--template` and the prompt select by.
    pub name: &'static str,
    /// One line for the selection prompt.
    pub description: &'static str,
    /// (tokenised target path, embedded content) pairs.
    pub(crate) files: &'static [(&'static str, &'static str)],
}

/// Every template this binary ships, in prompt order.
static SHIPPED: [Template; 2] = [rust_npm::TEMPLATE, web_react_android_ios_native::TEMPLATE];

/// Every template this binary ships.
pub fn shipped() -> &'static [Template] {
    &SHIPPED
}

/// The shipped template with this name, if any.
pub fn find(name: &str) -> Option<&'static Template> {
    shipped().iter().find(|t| t.name == name)
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/templates/rust_npm.rs#template -->
```rust
/// One file of the `rust-npm` tree, embedded from
/// `assets/projects/rust-npm/` — the crate's `assets` is the repository's
/// `pack/`, so the tree lives at `pack/projects/rust-npm/`.
macro_rules! tpl {
    ($rel:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/projects/rust-npm/",
            $rel
        ))
    };
}
/// The shipped template `rust-npm`: its name, its prompt line and
/// its tree.
pub(super) const TEMPLATE: Template = Template {
    name: "rust-npm",
    description: "Rust CLI workspace deployed as prebuilt binaries through npm",
    files: &FILES,
};
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/templates/web_react_android_ios_native.rs#template -->
```rust
/// One file of the `web-react-android-ios-native` tree, embedded from
/// `assets/projects/web-react-android-ios-native/` — the crate's `assets`
/// is the repository's `pack/`, so the tree lives at
/// `pack/projects/web-react-android-ios-native/`.
macro_rules! tpl {
    ($rel:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/projects/web-react-android-ios-native/",
            $rel
        ))
    };
}
/// The shipped template `web-react-android-ios-native`: its name, its
/// prompt line and its tree.
pub(super) const TEMPLATE: Template = Template {
    name: "web-react-android-ios-native",
    description: "Three native codebases — React web, Compose Android, SwiftUI iOS — with agent debug tooling",
    files: &FILES,
};
```
<!-- /sokf:include -->

## Behaviour

### Files

A template is a directory tree under `pack/projects/<name>/` — a pack
item per the [pack format][sokf:contract-005-format-pack], written by
maintainers through the `template-backport` skill and embedded in the
binary at build time. The Definition names each template's tree; a
reader who wants the file list opens that directory. On disk the
assets mirror the seeded repo except for three manglings the target
paths restore: a leading dot is stripped, a tokenised path segment is
written `_slug_`, `_pascal_` or `_compact_`, and a `Cargo.toml` is
stored as `Cargo.toml.tpl`. A binary file cannot be seeded.

- `P_utf8-throughout` [ubiquitous] A template SHALL be UTF-8 text
  throughout.
- `P_docs-say-bootstrap` [conditional] IF a seeded repo needs a binary
  file, the template's docs SHALL say how the seeded repo bootstraps
  it.

A template is identified by its `name`, which `--template` and the
selection prompt take; a name is `<platforms>-<stack>`, so a variant
gets its own name rather than a flag. Seeding is `init --template`, or
the prompt the [CLI contract][sokf:contract-002-cli-superdev]
describes. From the moment of seeding every seeded file is the user's,
and the user is authoritative. The canonical knowledge is a reserved
path that belongs to the knowledge component: every knowledge-enabled
repo gets the concept skeleton from that component's scaffold, template
or not, so no template ships one.

- `P_seed-writes-tree-substituted` [event] WHEN seeding runs, seeding
  SHALL write the tree into the target repository with the five tokens
  substituted in file contents and in target paths, so
  `crates/app/{{superdev:project-slug}}/` lands renamed.
- `P_seeded-file-write-once` [ubiquitous] The engine SHALL NOT hash,
  sync or revisit a seeded file.
- `P_disjoint-from-capabilities` [ubiquitous] A template SHALL stay
  disjoint from capability files.

Each token spelling exists because a target language forbids the slug
itself in that position; the derivations live on `Tokens`, so
`substitute` and `template render`'s printout share one source of
truth. The slug is the name lowercased with every run of characters
outside `[a-z0-9]` collapsed to one `-`, never leading or trailing.
There are no user-defined variables.

- `P_empty-slug-falls-back` [event] WHEN a name yields an empty slug,
  the slug SHALL fall back to `project`.

The shipped set:

1. `rust-npm` — a Rust CLI workspace deployed as prebuilt binaries
   through npm, derived from this repo's shape.[^rust-npm-src] The
   Cargo workspace with app and lib crate stubs, the `packages/` npm
   launcher, a thin `ci.yml` calling a reusable `checks.yml`, audit,
   and the tag-driven release pipeline with its scripts and smokes;
   crates are `publish = false` and the pipeline publishes npm only.
   The stub binary honours the exit-code contract the smokes assert: a
   usage error exits 2. The LICENSE ships proprietary with no year, for
   the user to replace. The dev container is built for a
   superdev-managed repo: mise owns the tool versions in a seeded
   `mise.toml`, Rust is pinned twice by necessity —
   `rust-toolchain.toml` for CI and rustup, mise's `RUSTUP_TOOLCHAIN`
   export — and named volumes carry the slug token so two seeded
   projects never share one `target/`.
2. `web-react-android-ios-native` — one product as three native
   codebases, backported from a real three-platform project.[^web-src]
   Three hello-world app stubs that pass CI as shipped (`apps/web`,
   `apps/android-native`, `apps/ios-native`); a debug-build-only HTTP
   debug server per platform with an MCP server wrapping its API and
   `scripts/` for the build/install/launch/logs/screenshot loop; a
   fastlane release pipeline keyed off `release/release.yaml`, the one
   place a version or app id is written; the same CI shape as
   `rust-npm`; and an Android-capable dev container. Two artefacts are
   bootstrapped rather than seeded, as the template's `docs/BUILD.md`
   says: the Gradle wrapper jar and the Xcode project `xcodegen`
   generates from the committed `project.yml`. CI is green before
   either bootstrap runs.

### Unknown content

Substitution is exact-match on the five tokens.

- `P_non-token-passes-through` [ubiquitous] Substitution SHALL pass
  through untouched anything that is not one of the five tokens,
  including GitHub Actions' `${{ … }}`, which template CI files
  legitimately contain, and a near-miss such as
  `{{superdev:project}}`.
- `P_unknown-name-fails` [event] WHEN a template name is outside the
  shipped set, the engine SHALL fail naming the shipped set.
- `P_existing-file-kept` [event] WHEN a file the template names
  already exists, seeding SHALL NOT overwrite it: the existing file
  wins and is reported as kept, so re-running `init` in a populated
  repo is safe.
- `P_unnamed-file-untouched` [ubiquitous] Seeding SHALL leave a file
  the template does not name as it is.
- `P_render-needs-empty-dir` [event] WHEN `template render` is given a
  directory that is not empty, `template render` SHALL refuse it.

### Compatibility

A seeded repo keeps working whatever the binary later ships: the
engine never revisits a seeded file (`P_seeded-file-write-once`), so a
template change reaches an existing repo only through the
`template-update` skill. `Action::WriteFile` sets no mode, so nothing
seeded is executable and every script is invoked through an
interpreter; a file that needs its executable bit (`gradlew`) says so
in the template's docs. The manifest's `[template]` table carries an
optional `version`.

- `P_pre-version-manifest-parses` [event] WHEN a manifest predates
  the `version` field, `parse` SHALL parse it unchanged.

## Stability

The token vocabulary is promised. The write-once promise is permanent
(`P_seeded-file-write-once`), and the only update path is the
`template-update` skill, which discovers the template (`[template]` in
the manifest, or shape analysis confirmed with the user), renders the
binary's current content, three-way-compares against the file as
seeded (recovered from git history), and applies what the user
approves as ordinary user edits — restamping `version` so an update
can short-circuit when the repo already matches. The set grows one
entry per template as `template-backport` captures it.

- `P_token-meaning-fixed` [ubiquitous] A token's meaning SHALL NOT
  change.
- `P_token-may-be-added` [ubiquitous] A new token spelling MAY be
  added.
- `P_token-never-replaced` [ubiquitous] A new token spelling SHALL NOT
  replace an existing one.
- `P_template-verbs-read-only` [ubiquitous] `template list` and
  `template render` SHALL stay read-only views of the shipped set.
- `P_removal-needs-notice` [ubiquitous] A template MAY be removed only
  with a release-notes notice, since a seeded repo keeps working
  regardless.

[^rust-npm-src]: rust-npm template tree
[^web-src]: web-react-android-ios-native template tree

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-043-one-contract-schema-and-twelve-kinds]: /knowledge/adrs/active/adr-043-one-contract-schema-and-twelve-kinds.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-005-format-pack]: /knowledge/contracts/public/active/contract-005-format-pack.md
