---
type: Schema
id: schema-testing-strategy
title: Testing Strategy Schema
description: The test layers, the key choices behind them, and where they run, in knowledge/testing-strategy.md.
---

# Testing Strategy Schema

Structural rules for `knowledge/testing-strategy.md`, the canonical knowledge's Reference
concept for how the project is tested. This is the standing strategy; the
per-feature cases live in a spec's appended test plan.

````yaml
target-files: "knowledge/testing-strategy.md"
description: >
  How tests are run, the layers and what each covers, the deliberate choices
  behind them, and the platforms they run on.
line-limit: 800

frontmatter:
  type:
    const: Reference
  id:
    const: testing-strategy
  status:
    enum: [draft, stable, deprecated]

preamble:
  content: prose
  description: >
    How tests are run, in one line: the runner and the command.

sections-ordered: true
sections:
  - heading: "Layers"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet per layer, e.g. unit, integration, end-to-end: what it
      covers, what is faked and why, and what is asserted.
  - heading: "Key choices"
    level: 1
    required: true
    content: bullet-list
    description: >
      One bullet per deliberate choice — coverage gate, snapshot policy, what
      is never tested — and the reasoning.
  - heading: "CI platforms"
    level: 1
    required: true
    content: prose
    description: >
      Where the tests run, and anything that runs on fewer platforms than the
      rest.

example: |
  ---
  type: Reference
  id: testing-strategy
  title: Testing Strategy
  description: The test layers, the key choices behind them, and where they run.
  status: stable
  ---

  `cargo test --all-features`, wrapped as `just test`.

  # Layers

  - Unit — one module, no IO. The network and filesystem are faked at the
    trait boundary so a transport refusal can be asserted without a server.
  - Integration — the library against a real temporary git repository,
    asserting that a pin round-trips through the lockfile unchanged.
  - End-to-end — the built binary against a fixture tree, asserting exit
    codes, since those are the contract scripts depend on.

  # Key choices

  - No coverage gate. It rewards tests that execute lines over tests that
    assert behaviour, and the layers above already say what must be covered.
  - The network is never reached in tests, including end-to-end. A test that
    needs a real host is a smoke test and runs at merge, not in the suite.

  # CI platforms

  Linux, macOS and Windows on every PR. The end-to-end layer runs on Linux
  only — it asserts exit codes and path handling that the other two platforms
  exercise through the integration layer.
````
