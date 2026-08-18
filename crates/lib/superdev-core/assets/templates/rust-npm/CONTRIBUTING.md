# Contributing

## Layout

- `crates/app/{{superdev:project-slug}}` — the binary.
- `crates/lib/{{superdev:project-slug}}-core` — the library behind it; keep
  logic here, keep the binary thin.
- `packages/{{superdev:project-slug}}` — the npm launcher; the
  `{{superdev:project-slug}}-*` siblings carry one prebuilt binary each and
  are published by release CI.
- `scripts/` — repo scripts wired as npm scripts (see `package.json`).

## Rust rules

- Hoist dependency versions, profiles, and shared package metadata to the
  workspace `Cargo.toml`; member crates inherit with `workspace = true`.
- Use the latest version when adding a dependency, and never add one without
  maintainer confirmation.
- `mod.rs` contains only `mod` declarations and `pub use` re-exports; all
  code lives in named files.
- Declare submodules privately (`mod foo;`) and expose their API via
  `pub use foo::Item;`; flatten re-exports at `lib.rs` so callers write
  `crate::Item`.
- Default to private; widen visibility via `pub(crate)` -> `pub(super)` ->
  `pub` only as needed.
- Group imports `std` -> external -> `crate`/`super`/`self`, collapse with
  nested paths, and prefer `use crate::...` over `super::super::...`.
- Import types and traits directly; import the parent module for free
  functions (`module::func()`); no glob imports except preludes, enum
  variants in `match`, and tests.
- Every `unsafe` block lives in a dedicated `*_unsafe.rs` module behind safe
  public functions, is clearly documented, and needs maintainer confirmation.

## Everyday commands

```sh
npm run build           # cargo build --workspace
npm run test            # cargo nextest run --workspace (install cargo-nextest first)
npm run lint            # cargo clippy --workspace
npm run fmt             # cargo fmt --all
npm run test:launcher   # the npm launcher's own tests

npm run smoke           # behavioural smoke of a release binary (build --release first)
npm run smoke:launcher  # npm-pack the launcher + host platform package, run the
                        # real binary through it (stage the binary into
                        # packages/<slug>-<host>/bin/ first)

npm run verify-version  # one version everywhere: Cargo and every package.json
npm run set-version <v> # set that version everywhere, lockfiles included
npm run release <v>     # changelog gate + set-version + verify + commit + tag
```

Commit `Cargo.lock` after the first build, then add `--locked` to the cargo
steps in `.github/workflows/checks.yml` and `release.yml` so CI builds what
you committed.

## Releasing

1. Give `CHANGELOG.md` a `## [X.Y.Z]` section (promote `[Unreleased]`); the
   release script and the workflow both refuse a version without one, and the
   section becomes the GitHub release notes.
2. `npm run release X.Y.Z` — sets the version everywhere, verifies it, and
   commits and tags. It deliberately does not push.
3. Review (`git show vX.Y.Z`), then `git push --follow-tags`. The push
   triggers `.github/workflows/release.yml`: tag checks, the full CI gate,
   per-platform builds and smoke tests, npm publishes, and the GitHub
   release — the workflow file is the authoritative sequence.

npm publishing uses trusted publishing (OIDC) — configure it on npmjs.com for
each package, or switch the publish steps to a token.
