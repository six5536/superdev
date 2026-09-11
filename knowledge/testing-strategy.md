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
  manifest in and assert on the action list that comes out. The agent-entry
  cases also apply the plan: instructions come first, user bytes survive
  migration and updates (including CRLF and a missing final newline), repeated
  syncs converge, malformed markers refuse, and failed applies roll back.
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
  naming no schema, two schemas claiming one type, a schema that governs
  nothing, a section starving its content kind, a kind outside the five, the
  frontmatter contract broken, an uncompilable pattern, a missing required
  key, and a schema whose example breaks its own contract.

  All three compare verbatim: the goldens are the contract over the finding
  texts, their severities, the verdict and the order findings arrive in, none
  of which the inline tests pin. The first two began as captures from the
  Python and Node references this code replaced, which are no longer the
  authority. Regenerate with `UPDATE_GOLDENS=1` and read the diff — a
  reworded message is the diff working, while a moved severity or a finding
  that appears or vanishes is a behaviour change and wants the argument one
  deserves.
- **MCP integration.** A real rmcp client drives every tool over an
  in-process duplex pipe against fixture knowledge trees — the transport is
  the only
  thing stubbed. Assertions cover locators, line numbers, group truncation and
  the lexical-only degradation. A `FakeEmbedder` keeps vector results
  deterministic; no test downloads the real model. The roster is asserted
  exactly — three tools, and no file tool behind a removed name. A real Git
  fixture builds a main checkout and a linked worktree carrying different bytes
  for one identity, and proves the server answers from the active checkout
  alone and writes its index there.
- **The SOKF Pi extension.** `scripts/test/sokf-pi-adapter.test.mjs` loads the
  extension against the pinned `@earendil-works/pi-coding-agent` 0.85.1 test
  dependency rather than a `pi` on `PATH`, so the evidence fails rather than
  skips when Pi cannot load. It proves the session receives exactly three
  tools and no `read`, `edit` or `write`, that a graph result carries the
  repository-relative path of a concept, and that the turn-end check behaves:
  a file written directly — by nothing the extension registered — is still
  found, the first report triggers one turn, later reports stay visible and
  non-triggering, a valid tree sends nothing, and a clean run resets the
  sequence. The runner explicitly invokes each fixture factory; `--list-models`
  can exit without running one. A deterministic process stub proves freshness
  after an intervening write, separate state per canonical root, and message
  caps including headings and truncation notices. Real Pi file tools prove
  read-to-edit anchors, no inline repair, and a byte-identical clean turn.
  A scripted provider drives the real Pi loop without network calls: first
  reports arrive before subsequent tool turns, and later reports are visible
  without waiting for another user prompt or triggering another turn.
- **LLM search development experiment.** `scripts/sokf-search-eval.mjs` uses
  six tasks from `evals/sokf/search.json`. Luna composes search requests and
  selects evidence from actual CLI results; expected concepts are hidden.
  Frozen corpus/binary comparisons and verbatim request replay separate
  guidance, ranking and labels. The script is opt-in, has no tools or hidden
  repository instructions in model calls, and records token/cost estimates.
  `scripts/test/sokf-search-eval.test.mjs` checks its scorer and failure paths
  offline. This sample does not measure final answer quality or adaptive agent
  retries, and is not a statistical acceptance gate.
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
see [architecture][sokf:architecture].

# Key choices

- **Per-crate coverage gate.** Line coverage ≥ 90% for each crate, enforced in
  CI via `cargo-llvm-cov` on nightly (so `coverage(off)` markers on
  untestable glue take effect).
- **Explicit assertions** on output and exit codes, not snapshots, while the
  surface is this small.

# CI platforms

Tests run on **Linux, macOS, and Windows** — see
[software-components][sokf:software-components] for the workflow layout.

[^contributing]: Contributing guide (test layers and commands)

<!-- sokf:links -->
[sokf:architecture]: /knowledge/architecture.md
[sokf:software-components]: /knowledge/software-components.md
