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
npm run build          # cargo build --workspace
npm run test           # cargo nextest run --workspace (install cargo-nextest first)
npm run lint           # cargo clippy --workspace
npm run fmt            # cargo fmt --all
npm run test:launcher  # the npm launcher's own tests
npm run verify-version # one version everywhere: Cargo and every package.json
```

Commit `Cargo.lock` after the first build, then add `--locked` to the cargo
steps in `.github/workflows/checks.yml` and `release.yml` so CI builds what
you committed.

## Releasing

1. Set the version everywhere (`Cargo.toml`, every `packages/*/package.json`
   and the launcher's `optionalDependencies`), and give `CHANGELOG.md` a
   `## [X.Y.Z]` section — `npm run verify-version X.Y.Z` must pass.
2. Tag `vX.Y.Z` and push the tag. Release CI verifies the tag against the
   tree, runs the full check gate, builds each platform binary, publishes the
   npm packages, and creates the GitHub release with archives and checksums.

npm publishing uses trusted publishing (OIDC) — configure it on npmjs.com for
each package, or switch the publish steps to a token.
