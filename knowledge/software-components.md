---
type: Reference
id: software-components
title: Software Components
description: The Rust crates, the npm launcher and platform packages, the platform matrix, and the CI/CD workflows.
status: stable
links:
  - rel: relates-to
    to: architecture
    note: The design these components implement.
---

The system is one Rust library plus one binary (workspace globs
`crates/lib/*`, `crates/app/*`), and six npm packages. The design they
implement is in [architecture](architecture.md).

# `crates/lib/superdev-core` (library)

All domain logic; no argument parsing. One module per concern:

- `capability` — the five slots; `registry` — their default providers and
  versions, baked into the binary.
- `manifest` / `lock` — `.superdev/config.toml` and `.superdev/lock.toml`.
- `component` — the provider trait (`plan` observes and returns actions);
  `action` — the action enum and file ownership.
- `components::{aokf, plugin, mattskills, codegraph, mise, pin, skillpack}` —
  the providers, plus the shared helpers: targeted `.mise.toml` editing,
  registry-locked pin planning, `item` — the declarative managed-item
  list the static components derive both `plan` and `owned` from — and
  `enabled`, the manifest-to-component resolution.
- `pack` — where content comes from: the source a pack is resolved from, the
  normalised identity that decides replace-versus-layer, `pack.toml` with the
  paths and keys a pack may not carry, and `resolve` — the phase that turns
  the manifest's entries into a content set before anything plans. Depends on
  `content`; `content` never depends on it, and neither knows about
  components.
- `content` — what a pack provides: `Item` and the `(owner, kind, name)`
  identity a later layer supersedes on, the layout rules that read that
  identity out of a pack tree, and the `ContentSet` a run resolves to. Depends
  on nothing but `std` and `capability`, and never on how a pack is fetched.
  The `aokf` and `skillpack` components and the general-rules scaffolds read
  their items from the set through `Ctx`, so adding a file to `/pack` ships it
  with no Rust edit. What the binary owns rather than the pack stays a
  constant: the canonical knowledge instructions and the AOKF spec describe a version it
  pins and a format its compiled validator enforces, as codegraph's and rtk's
  instruction files do.
- `pipeline` — the verb pipeline between manifest and engine: `plan_repo`
  and `apply_repo`, owning the prune-before-plan and orphans-last ordering.
- `engine` — applies a plan and unwinds on failure, one file per concern:
  `tx` (the journal every side effect goes through), `pins` (the grouped
  mise pin phase), `materialise` (the skill copier), with the appliers in
  `apply`. `orphan` — plans the sweep of lock entries no live claim covers.
- `runner` — the process seam. `run_with` is its one required method, taking
  a `RunOptions` that carries a deadline and extra environment; `run` defaults
  onto it with neither, so a caller wanting only a command writes what it
  always did ([ADR-015](decisions/D015-the-spawn-seam-carries-a-deadline.md)).
  An expired deadline is an `Error::Command` like any other failed spawn.
  `report` — plan and apply rendering; `error` —
  the crate's error type; `fsutil` and `json_edit` — the pure file and
  JSON-pointer helpers the engine and planners share.
- `templates` — the project templates: token substitution, the init-only
  scaffold plan, and `rust_npm`, the embedded table mapping
  `assets/projects/rust-npm/` onto tokenised target paths.
- `aokf` — the read side of the canonical knowledge, one module per stage:
  `concept` (frontmatter and section parsing), `bundle` (loading, reserved-file
  rules), `validate` (document check and conformance ladder), `graph` (link
  resolution and inverse synthesis), `embed` (the embedding providers),
  `index` (tantivy plus the vector store), `mcp` (the server).
- The AOKF spec, agent files, starter concept skeleton and the 25 carried
  skill directories the `knowledge` capability writes, the three SKILL.md
  files the `skills` capability writes, and the project templates all live in
  `/pack` at the repository root, reached from the crate as `assets/` through a
  symlink and embedded at compile time. `superdev-core/build.rs` enumerates
  that tree into the file list `content` reads, so a file added to the pack
  reaches the binary without a Rust edit; the contents are still `include_str!`
  literals, and only the list of them is generated.

The MCP server exposes four read-only tools over stdio — `aokf_search`,
`aokf_read`, `aokf_graph`, `aokf_overview` (see
[api-contracts](api-contracts.md)). It holds one index directory and
serialises its own tool calls with a mutex: a call keeps the index open across
its whole body while another call's sync could delete and rebuild that
directory underneath it. Search is hybrid — tantivy BM25 and cosine over
section embeddings, fused by reciprocal rank fusion — and drops to lexical-only
when no model loads.

# `crates/app/superdev` (binary)

Depends on `superdev-core`. Binary name `superdev`. `main.rs` is clap parsing
and exit codes; `manage.rs` holds the `init`, `status`, `sync` and `update`
verbs — each loads, calls the core pipeline, renders its lines and turns
its facts into an exit code. `template_select.rs` decides init's project
template: flags and TTY-ness feed logic behind a `Prompter` trait, with the
dialoguer adapter as untested glue. `aokf_cli.rs` holds `aokf validate`, `aokf index` and
`mcp aokf`: path defaults, printed output, and the current-thread tokio
runtime the server blocks on. Also present is the plumbing the release
pipeline needs:
`--version`, `completions` (clap_complete), and a hidden `man` subcommand
(clap_mangen). The CLI contract is in [api-contracts](api-contracts.md).

# Publishing

`superdev-core` and `superdev` publish to crates.io. The compiled binary is
also redistributed via npm.

# npm (prebuilt-binary model, à la esbuild / `@swc`)

```
packages/
  superdev/              # published as superdev — launcher (bin: superdev)
  superdev-linux-x64/    # published as @six5536/superdev-linux-x64 — prebuilt binary, declares os/cpu
  superdev-linux-arm64/
  superdev-darwin-x64/
  superdev-darwin-arm64/
  superdev-win32-x64/    # superdev.exe
```

- The launcher (`superdev`) declares each platform package in
  `optionalDependencies` pinned to an **exact** version; npm installs only the
  host's match.
- A small JS shim `require.resolve`s the installed platform package's binary
  and `spawnSync`s it with `stdio: "inherit"`, forwarding `argv` and the exit
  code.
- **Version lockstep**: launcher + all platform packages share one version and
  publish atomically.
- **Unsupported platform** (no matching optional dep): fail with a message
  that lists the supported platforms and points at `cargo install superdev`.
  No auto-download or build-from-source fallback. The Linux packages are
  static musl builds, so they cover glibc and musl hosts alike — libc is not a
  dimension of this matrix.

# Platform matrix

Linux `x86_64`/`aarch64` (**static musl**), macOS `x86_64`/`aarch64`, and
Windows `x86_64` (msvc, built natively on the `windows-latest` runner).
`cargo-zigbuild` provides the cross C compiler for the musl targets; its musl
output is non-PIE, accepted for a local CLI with no network input.

# CI/CD (`.github/workflows`)

All checks live in a reusable `workflow_call` workflow (`checks.yml`), called
by both `ci.yml` and `release.yml`, so the release gate cannot drift from CI.

- **`checks.yml`**: `cargo fmt --check`, `clippy -D warnings`, `nextest`,
  doctests, `cargo doc -D warnings`, the npm launcher tests, version
  consistency, the canonical knowledge and format validation (`check:validate`), the
  per-crate coverage gate, and `cargo-deny` for licences/bans/sources. Tests
  and doctests also run on Windows; the OS-independent checks run once, on
  macos.
- **`ci.yml`**: calls `checks.yml` on push and PR.
- **`audit.yml`**: scheduled `cargo-deny check advisories`, opening an issue
  rather than failing builds — advisories are exogenous and must not block an
  unrelated PR.
- **`release.yml`** (tag `v*`): verify the tag against every version in the
  tree and against a `CHANGELOG.md` section → run `checks.yml` in full →
  build the five binaries (cross for musl, native for macOS and Windows),
  assert the Linux ones are static, smoke-test each binary the runner can
  execute (`release-smoke.mjs`; linux-arm64 cannot run on the x64 runner) and
  run the packed launcher end-to-end where the package matches the host →
  dry-run every publish → publish platform packages, then the launcher, then
  `cargo publish --workspace --locked` → create a GitHub Release with
  archives (`.tar.gz`; `.zip` for Windows), a man page, completions and
  `SHA256SUMS`. Prerelease tags publish under the npm `next` dist-tag and are
  flagged as prereleases.

Cross-registry atomicity is impossible, so the guarantee is *ordered,
dry-run-gated and recoverable* rather than truly atomic.
