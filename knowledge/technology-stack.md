---
type: Reference
id: technology-stack
title: Technology Stack
description: Languages, runtime and dev dependencies, and the pinned toolchain set.
status: stable
sources:
  - id: cargo-toml
    resource: /Cargo.toml
    title: Workspace manifest (dependency set)
  - id: mise-toml
    resource: /.mise.toml
    title: Pinned tool versions
---

Rust core (library + binary) with a Node launcher layer for npm
distribution. Toolchains are pinned: `rust-toolchain.toml` pins the project
Rust (with `rustfmt` and `clippy`); `.mise.toml` pins Node, a nightly Rust
used only by the coverage job, and the cargo tooling below.[^mise-toml] Setup
detail lives in [CONTRIBUTING](/CONTRIBUTING.md).

Adding a dependency requires explicit approval and the latest version at the
time, per the [dependency policy](dependency-policy.md). The current
set:[^cargo-toml]

- **Rust**: `clap` (derive), `clap_complete` + `clap_mangen` (completions and
  man page generated from the same clap definition); `serde` (derive) for the
  manifest and lock types; `toml_edit` to read them and to edit `.mise.toml`
  in place, preserving the user's layout and comments; `sha2` for the sha256
  hashes that detect drift in superdev-owned files.
- **Rust (dev)**: `assert_cmd` for CLI tests, `tempfile` for the throwaway
  repos the component and engine tests work in.
- **Tooling** (pinned in `.mise.toml`): `cargo-zigbuild` and `zig` (the cross
  C compiler for the musl targets), `cargo-nextest`, `cargo-llvm-cov`.
- **Agent tooling**: the [Superpowers](https://github.com/obra/superpowers)
  Claude Code plugin, pinned in `.mise.toml` via the `http` backend (tag
  tarball + sha256 checksum — Superpowers publishes no release assets, so the
  `github` backend cannot install it). The devcontainer post-create script
  wires the checkout into Claude Code as a local marketplace.[^mise-toml]

superdev pins codegraph into *managed* repos the same way: the `http` backend
against the release bundles, one checksummed URL per platform. Those bundles
vendor their own Node, unlike the npm package, whose shim needs one on the
host.

[^cargo-toml]: Workspace manifest (dependency set)
[^mise-toml]: Pinned tool versions
