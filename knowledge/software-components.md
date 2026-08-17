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
- `components::{aokf, plugin, mattskills, codegraph, mise, skillpack}` — the
  providers, plus the targeted `.mise.toml` editing they share.
- `engine` — applies a plan, journals every side effect, unwinds on failure;
  `orphan` — plans the sweep of lock entries no live claim covers.
- `runner` — the process seam; `report` — plan and apply rendering; `error` —
  the crate's error type.
- `aokf` — the read side of the knowledge bundle, one module per stage:
  `concept` (frontmatter and section parsing), `bundle` (loading, reserved-file
  rules), `validate` (document check and conformance ladder), `graph` (link
  resolution and inverse synthesis), `embed` (the embedding providers),
  `index` (tantivy plus the vector store), `mcp` (the server).
- The AOKF spec and agent files the `knowledge` capability writes, and the four
  SKILL.md files the `skills` capability writes, ship as `assets/`, embedded at
  compile time. The mattpocock-skills files are not embedded: they come from
  the checksummed checkout at sync time.

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
verbs — plan, print, apply, and the repo-level `.gitignore` entries no
capability owns. `aokf_cli.rs` holds `aokf validate`, `aokf index` and
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
  consistency, the AOKF knowledgebase validation (`check:aokf`), the
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
