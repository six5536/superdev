# Contributing to superdev

How to get set up, what to run before you push, and how a release is cut.

> Status: released, pre-1.0. The tooling layer is settled; the CLI surface
> above it is still moving, and minor versions may break it.

## Prerequisites

Toolchains are pinned and managed with [mise](https://mise.jdx.dev/):

- `rust-toolchain.toml` pins the project toolchain (`1.96.0`, with `rustfmt` and
  `clippy`).
- `.mise.toml` pins everything else: Node, a `nightly` Rust used only by the
  coverage job, `zig` (the cross C compiler behind `cargo-zigbuild`, whose
  version the release workflow reads straight out of this file), and the
  cargo tools (`cargo-nextest`, `cargo-llvm-cov`, `cargo-zigbuild`).

```sh
mise install     # install all pinned tools
npm install      # install the JS workspace (the launcher package)
```

Use `npm install`, not `npm ci`: the launcher pins the current version of
every platform package, and until a release has published those versions the
lockfile cannot carry resolved entries for them, which `npm ci` treats as
fatal.

A plain `cargo build` needs no Node; Node is only needed for the npm packages
and the repo scripts.

### Windows checkouts need symlinks

The shipped content lives at `/pack`, and `crates/lib/superdev-core/assets` is
a symlink to it — that is what keeps the files inside the published crate while
leaving them browsable at the repo root. Git for Windows only materialises a
symlink when `core.symlinks` is on, and creating one needs Developer Mode or an
elevated shell. Set it before you clone:

```sh
git config --global core.symlinks true
```

Without it the link checks out as a one-line text file and the build fails on
`include_str!`. A clone made without symlink support records `core.symlinks =
false` in its own `.git/config`, and that overrides the global setting — so
repair an existing clone from inside it:

```sh
git config core.symlinks true    # this clone; --global alone will not do it
git checkout -- crates/lib/superdev-core/assets
```

This repo manages its own skills with superdev, and the Claude Code validation
hook it installs calls a bare `superdev`. Link the dev shim so that resolves to
your working tree:

```sh
ln -sf "$PWD/scripts/superdev" ~/.local/bin/superdev
```

## Everyday commands

All wrapped as npm scripts (see `package.json`):

```sh
npm run build           # cargo build --workspace
npm run test            # cargo nextest run --workspace, then check:validate
npm run check:validate  # validate the bundle and the superdev-format files
npm run check:blueprint # the superdev-owned files match the blueprint
npm run lint            # cargo clippy --workspace
npm run fmt             # cargo fmt --all
npm run check           # cargo check --workspace --tests

npm run coverage         # cargo-llvm-cov, HTML report
npm run coverage:summary # coverage summary in the terminal
npm run coverage:check   # enforce the gate: line coverage >= 90% per crate

npm run test:launcher   # node test for the npm launcher shim
npm run test:scripts    # node tests for the release scripts

npm run smoke           # behavioural smoke of a release binary (build --release first)
npm run smoke:launcher  # npm-pack the launcher + host platform package, run the
                        # real binary through it (stage the binary into
                        # packages/superdev-<host>/bin/ first)
npm run smoke:manage    # real init + status in a scratch repo against the real
                        # mise/claude/codegraph; devcontainer only, never in CI

npm run verify-version  # every version in the tree agrees (16 locations)
npm run release <ver>   # bump + verify + commit + tag both series (no push)
npm run release:pack    # cut a content release alone (does not push)
```

Only the launcher (`packages/superdev`) is an npm workspace. The five
platform-binary packages deliberately are not: npm enforces their `os`/`cpu`
fields on workspace members unconditionally, so including them made a plain
`npm install` fail with `EBADPLATFORM` on every host. Nothing needs them to be
members. `set-version` and the release workflow address them by path.

Before opening a PR, run everything CI runs. Note that `npm run lint` is only
`cargo clippy --workspace` — CI is stricter, so use the full command here:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo test --doc --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
npm run test:launcher
npm run test:scripts
npm run verify-version
npm run check:validate
npm run check:blueprint
npm run coverage:check     # slow; needs the nightly toolchain
```

`cargo-deny check licenses bans sources` also gates CI, but only fails when you
change dependencies.

## Standards and tests

The canonical rules live in the knowledgebase, not here:

- Code, prose, and documentation standards — including the module rules and
  what the rustdoc CI gate enforces — are in
  [knowledge/coding-standards.md](knowledge/coding-standards.md).
- The test layers and the choices behind them are in
  [knowledge/testing-strategy.md](knowledge/testing-strategy.md). Tests run
  under `cargo-nextest` (per-test process isolation).

## Project layout

- `pack/` — the content superdev ships: skills, agent instructions, the
  knowledge bundle and the project templates. Symlinked into
  `superdev-core` as `assets/`, which is where the binary embeds it from.
- `crates/lib/superdev-core` — all domain logic (no arg parsing).
- `crates/app/superdev` — the binary: CLI parsing, wiring, output rendering.
- `packages/` — the npm launcher and per-platform prebuilt-binary packages.
- `knowledge/` — the AOKF knowledgebase: canonical project knowledge,
  including the design overview (see `AGENTS.md`). `knowledge/specs/` holds
  design specs (permanent decision records); `knowledge/plans/` holds
  implementation plans (ephemeral — deleted in the commit that lands them).

## Commits and pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org/) for messages
  (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`).
- Keep PRs focused; update `knowledge/` when behaviour or design changes.
- Make sure the full check list above passes. CI runs tests on macOS and
  Windows, and the coverage gate on Linux.

## Dependencies

The rules are in [knowledge/dependency-policy.md](knowledge/dependency-policy.md):
ask first, take the latest version, hoist to the workspace `Cargo.toml`.

## Releasing

There are two release paths, and each is one command.

- **A binary release** cuts `vX.Y.Z` and, from the same commit, the content
  release `assets-vA.B.C` its `DEFAULT_PACK.rev` names. Tag-driven
  (`.github/workflows/release.yml`, triggered by a `v*` tag).
- **A content release** cuts `assets-vA.B.C` alone: superdev's skills,
  templates and scaffolds change without a new binary. No workflow runs for it
  and nothing reaches a registry — `superdev update` finds it by asking this
  repository for its newest `assets-v` tag.

### The binary release

**1. Write the changelog.** Add a `## [X.Y.Z]` section to `CHANGELOG.md`
(promote `[Unreleased]` if that is where the notes already are). This is not
optional. Both `npm run release` and the release workflow refuse a version they
cannot find a section for, and that section becomes the GitHub release notes.

**2. Cut the release commit and tag.**

```sh
npm run release X.Y.Z
```

That sets the version everywhere in lockstep (Cargo workspace, the internal
`superdev-core` pin, all six `package.json` files, and **both lockfiles**),
verifies it landed consistently, then commits and tags. It deliberately stops
there.

It also cuts the content release riding on that commit: `pack/pack.toml`'s
version moves, `DEFAULT_PACK.rev` is set to the tag that version releases as,
and the commit is tagged `assets-vA.B.C` as well as `vX.Y.Z`. The pack version
it picks is the one `pack.toml` already declares while that is still
unreleased, and the next patch once it is out — so a hand-edited `pack.toml`
gets the version its author chose, and the first release cuts the version the
repo already claims. Pass a second argument (`npm run release X.Y.Z A.B.C`) to
name it instead; a version behind what `pack.toml` declares is refused, so is
a candidate for a version already released, so is one that is not
`MAJOR.MINOR.PATCH`, and all of them are refused before anything is written.

A prerelease binary cuts a prerelease content tag: `npm run release
0.3.0-rc.1` produces `assets-vA.B.C-rc.1`. `superdev update` moves a pin only
between three-number releases, so candidate content never reaches a stable
user's next update — while the candidate binary's own pin still names exactly
what it embedded.

Setting both together is what makes it impossible to ship a binary whose pin
names content it did not embed; a Rust test holds the pair, and the release
fails rather than writing one without the other.

**3. Review, then push.**

```sh
git show vX.Y.Z
git push --follow-tags
```

Pushing the tag is what triggers the publish, and publishes cannot be undone
(crates.io never; npm after 72 hours).

**4. The workflow takes over**, in this order:

1. `meta` — the tag must match every version in the tree and have a changelog
   section.
2. `checks` — the full CI gate, via the shared reusable workflow.
3. `build` — build five binaries (`cargo-zigbuild` for the static-musl
   Linux targets, native `cargo` on macOS and Windows) and assert the Linux ones are static.
4. `publish` — smoke-test the binary, dry-run every publish, then publish the
   platform packages, then the launcher, then
   `cargo publish --workspace --locked`.
5. `github-release` — archives with the man page and completions, `SHA256SUMS`,
   and notes from the changelog.

A prerelease tag (`vX.Y.Z-rc.1`) publishes to npm under the `next` dist-tag and
is marked as a prerelease on GitHub, so it never becomes `latest`.

### The content release

To ship a skill, template or scaffold fix without a binary:

**1. Write the changelog.** Add a `## [assets-vA.B.C]` section to
`CHANGELOG.md` saying what the content change is. A content release has no
binary section to be described by, so it carries its own; `npm run
release:pack` refuses a tag it cannot find one for.

**2. Cut it.**

```sh
npm run release:pack           # the version pack.toml declares, or the next patch
npm run release:pack A.B.C     # or name the version
```

That moves `pack/pack.toml` and `DEFAULT_PACK.rev` together, commits, and tags
`assets-vA.B.C`. Like the binary release it stops before pushing.

**3. Review, then push.**

```sh
git show assets-vA.B.C
git push --follow-tags
```

The `v*` filter on `release.yml` does not match an `assets-v` tag, so no
workflow runs and nothing is published. What the push does is make the release
visible to `superdev update`, which asks this repository for its newest
`assets-v` tag and moves a default pin there — that is how the fix reaches
repos whose binary has not changed.

### Publishing credentials

npm uses **trusted publishing (OIDC)**, so there is no npm token. The workflow's
`id-token: write` permission is the credential, and each package needs a trusted
publisher configured on npmjs.com pointing at this repository and `release.yml`.
Provenance is generated automatically as a result.

A trusted publisher can only be attached to a package that already exists, so
**before the first release** each npm package (the launcher and all five
platform packages) needs a `0.0.0` placeholder published by hand and a trusted
publisher attached. Don't unpublish those placeholders afterwards: removing a
package's only version can take the package and its trusted-publisher
configuration with it.

crates.io still uses a token (`CARGO_REGISTRY_TOKEN`); it does not require a
one-time password, so it works unattended.

### Adding a platform package

A new `@six5536/superdev-<os>-<cpu>` package needs setup **before** the release
that first ships it:

1. Publish a `0.0.0` placeholder by hand (see above — one `npm publish` of the
   new package with an OTP), so the name exists.
2. Attach a trusted publisher to it on npmjs.com, pointing at this repository
   and `release.yml`. Without this the release's publish step fails.
3. Add the package to the launcher's `optionalDependencies` and the release
   workflow's build matrix and publish loops. (`verify-version` discovers
   `packages/*` itself — no change needed there.)

Until the next release publishes a real version, `npm ci` fails on the new
optional dependency (the `npm install` note under Prerequisites). That window
is expected; land the change and the release together or in quick succession.

### Version consistency

`npm run verify-version [version]` checks that the Cargo workspace, the
`superdev-core` pin, every `package.json`, the launcher's
`optionalDependencies`, `Cargo.lock` and `package-lock.json` all agree. That is
16 locations. It runs in CI and again against the tag at release time.
