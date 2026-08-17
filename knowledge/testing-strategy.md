---
type: Reference
id: testing-strategy
title: Testing Strategy
description: The current test layers, the key choices behind them, and the CI platforms.
status: stable
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Contributing guide (test layers and commands)
---

Tests run under `cargo-nextest` (`npm test`); the commands and the coverage
gate are in [CONTRIBUTING](/CONTRIBUTING.md).[^contributing]

# Layers

- **Unit.** Per-crate `#[cfg(test)]` tests, plus rustdoc examples as
  doctests. Planning is pure, so most of these feed a temp-dir repo and a
  manifest in and assert on the action list that comes out.
- **Fake runner.** Every process spawn goes through `CommandRunner`; the
  test-only `FakeRunner` records each command line and scripts outcomes,
  including a missing program and a mid-apply failure to exercise the
  rollback. No test shells out to a real tool.
- **CLI end-to-end.** Invoke the real binary (`assert_cmd`) against every
  surface: `--version`, help, completions per shell, `man`, usage-error exit
  codes, the manage verbs (`init` → clean `status` → tamper → `status`
  exits 1 → `sync` repairs), `aokf validate`'s three exit codes and its JSON,
  `aokf index`, and `mcp aokf`'s startup
  failures. The manage tests put fake `mise`, `claude` and
  `codegraph` on `PATH` as shell scripts — `mise where` answers with a
  fixture skills checkout, so materialisation runs against real files — and
  are therefore unix-only; Windows runs the rest.
- **Validator parity.** One fixture bundle per failure class under
  `tests/fixtures/aokf/`, each with a `.golden.json` captured from the Python
  validator before that script was deleted. The Rust output is compared to it
  verbatim, bar two documented normalisations. Those goldens are now the only
  record of the reference behaviour, so they are regenerated only against a
  validator worth trusting.
- **MCP integration.** A real rmcp client drives all four tools over an
  in-process duplex pipe against fixture bundles — the transport is the only
  thing stubbed. Assertions cover locators, line numbers, group truncation and
  the lexical-only degradation. A `FakeEmbedder` keeps vector results
  deterministic; no test downloads the real model.
- **npm launcher.** A JS test that resolves + spawns a stub binary, and
  errors cleanly when no platform package matches.
- **Release smoke.** `scripts/release-smoke.mjs` runs a compiled release
  binary through version/help/completions and the usage-error exit code;
  `scripts/launcher-smoke.mjs` npm-packs the launcher and the host's platform
  package into a temp `node_modules` and runs the real binary through the
  shim — catching a binary missing from a `files` manifest and broken
  exit-code forwarding. The release build job runs the first on every target
  its runner can execute and the second where the package matches the host;
  locally: `npm run smoke` / `npm run smoke:launcher`.
- **Manage smoke (manual).** `npm run smoke:manage` runs a real `init` and
  `status` in a scratch repo against the real mise, claude and codegraph, then
  `aokf validate` and `aokf index` over the bundle that `init` just wrote.
  This is the only place the real embedding model is downloaded and loaded.
  Devcontainer-only and never in CI: it needs the network and Claude auth.

Domain logic in `superdev-core` carries the bulk of the tests as pure units —
see [architecture](architecture.md).

# Key choices

- **Per-crate coverage gate.** Line coverage ≥ 90% for each crate, enforced in
  CI via `cargo-llvm-cov` on nightly (so `coverage(off)` markers on
  untestable glue take effect).
- **Explicit assertions** on output and exit codes, not snapshots, while the
  surface is this small.

# CI platforms

Tests run on **Linux, macOS, and Windows** — see
[software-components](software-components.md) for the workflow layout.

[^contributing]: Contributing guide (test layers and commands)
