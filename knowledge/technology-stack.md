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
  hashes that detect drift in superdev-owned files; `dialoguer` for init's
  template and project-name prompts.
- **Rust (knowledge side)**: `rmcp` (the official Rust MCP SDK) for the stdio
  server; `tantivy` for the BM25 index; `model2vec-rs` for local static
  embeddings; `serde_yaml_ng` to parse concept frontmatter; `pulldown-cmark`
  to split bodies into sections at their headings; `serde_json` for MCP
  payloads, the validator's `--json`, and `.mcp.json` editing; `ureq` for the
  model download and the OpenAI calls. `model2vec-rs` runs `local-only`, so it
  brings no HTTP stack of its own and `ureq` is the tree's only client. `tokio`
  runs the MCP server's async transport in the binary, pinned to the version
  `rmcp` already resolves.
- **Rust (dev)**: `assert_cmd` for CLI tests, `tempfile` for the throwaway
  repos the component and engine tests work in, and `rmcp`'s `client` feature
  (with `tokio`) to drive the server in-process.
- **Tooling** (pinned in `.mise.toml`): `cargo-zigbuild` and `zig` (the cross
  C compiler for the musl targets), `cargo-nextest`, `cargo-llvm-cov`.
- **Agent tooling**: this repo's engineering skills are the knowledge-carried
  set superdev itself writes into `.claude/skills/` — embedded in the binary.

The embedding model is pinned like a dependency:
`minishlab/potion-retrieval-32M`, at commit `6fc8051…`, fetched once per
machine into the user cache ([configuration](configuration.md)). Every crate
above is pure Rust, which is the point — the static-musl release builds rule
out anything wanting an ONNX runtime or a C toolchain. `cargo-deny`'s allow
list gained `CDLA-Permissive-2.0` for this: it arrives as `webpki-roots`,
Mozilla's root certificates behind `ureq`'s TLS on the download path, and
licenses data rather than code ([deny.toml](/deny.toml)).

superdev pins its providers into *managed* repos the same way: the `http`
backend against a checksummed URL. codegraph gets one release bundle per
platform; those bundles vendor their own Node, unlike the npm package, whose
shim needs one on the host.

[^cargo-toml]: Workspace manifest (dependency set)
[^mise-toml]: Pinned tool versions
