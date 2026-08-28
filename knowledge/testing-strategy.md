---
type: TestingStrategy
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
  test-only `FakeRunner` records each command line, and the `RunOptions` that
  came with it, and scripts outcomes,
  including a missing program and a mid-apply failure to exercise the
  rollback. No test shells out to a real tool. Orchestration detail — call
  ordering, targeted install lists, per-provider flows — is asserted here,
  in core, not end-to-end. A caller that sets a deadline is checked on what it
  asked for rather than on a real process; the seam's own behaviour under one
  — the kill, the environment, both pipes draining — is what `runner`'s unit
  tests spawn `sh` for.
- **CLI end-to-end.** Invoke the real binary (`assert_cmd`) against every
  surface: `--version`, help, completions per shell, `man`, usage-error exit
  codes, `validate`'s three exit codes and its JSON, `sokf index`, and
  `mcp sokf`'s startup failures. The manage verbs get five smoke journeys —
  fresh `init`, `sync` on a fresh clone, a provider switch swept both ways,
  disabling `code-index`, and a failed `init` reporting the manifest it
  leaves behind — against fake `mise`, `claude` and `codegraph` on `PATH` as
  shell scripts; `mise where` answers with a fixture skills checkout, so
  materialisation runs against real files. The fakes make these unix-only;
  Windows runs the rest.
- **Validator snapshots.** Three trees, one per half of the check and one for
  the rules that join them, each case carrying a `.golden.json` of the report
  it produces: `tests/fixtures/sokf/` holds one knowledge tree per failure
  class of the specification checks, `tests/fixtures/schema/` one file tree
  per failure class of the grammar checks, and `tests/fixtures/documents/`
  one case per document rule — a missing section, a misordered one, a
  prohibited one, wrong table columns, an over-limit line count, a type
  naming no schema, two schemas claiming one type, and a schema that governs
  nothing.

  All three compare verbatim: the goldens are the contract over the finding
  texts, their severities, the verdict and the order findings arrive in, none
  of which the inline tests pin. The first two began as captures from the
  Python and Node references this code replaced, which are no longer the
  authority. Regenerate with `UPDATE_GOLDENS=1` and read the diff — a
  reworded message is the diff working, while a moved severity or a finding
  that appears or vanishes is a behaviour change and wants the argument one
  deserves.
- **MCP integration.** A real rmcp client drives all four tools over an
  in-process duplex pipe against fixture knowledge trees — the transport is
  the only
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
  `validate` and `sokf index` over the canonical knowledge that `init` just wrote.
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
