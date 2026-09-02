---
type: Chore
id: issue-051-chore-pin-node-in-the-managed-repo
title: Pin node in the managed repo so codegraph's npm-backed pin can install and run
description: codegraph was pinned through mise's npm backend, which needs an npm to install and a node to run its shim; pinning node in the managed repo's .mise.toml would have supplied both.
lifecycle: wontfix
links:
  - rel: references
    to: constraints-non-goals
    note: Managing toolchains is a non-goal; rejecting this pin is that rule applied.
  - rel: references
    to: architecture
    note: The code-index provider's pin — the http backend against codegraph's self-contained bundles — which made the node pin unnecessary.
---

# Chore: pin node in the managed repo

## Won't fix

Decided against, and recorded here from the backlog's decided-against
list on its retirement (ADR-048). Pinning node writes a toolchain choice
into the user's `.mise.toml` to work around a packaging detail, which
[constraints-non-goals][sokf:constraints-non-goals] rules out: superdev
pins the tools it needs and never manages a toolchain. codegraph is now
pinned through the `http` backend against its self-contained release
bundles, which vendor their own Node, so the dependency is gone rather
than papered over — see [architecture][sokf:architecture].

## Summary

Add a `node` pin to the `[tools]` table superdev writes into a managed
repository's `.mise.toml`, so codegraph's `npm:` pin can install and its
`#!/usr/bin/env node` shim can run.

## Surfaces

- The `[tools]` table the `code-index` component writes into `.mise.toml`
  (`crates/lib/superdev-core/src/components/mise.rs`, one managed key).
- The mise env every managed repository resolves `codegraph` in (one
  `mise install` per repository).

## Definition of done

- `DD_node-pinned` `superdev sync` writes a `node` entry beside `codegraph`
  in `.mise.toml` and the lock claims it.
- `DD_codegraph-installs` `mise install` in a managed repository with no
  node on the host installs `codegraph` through the `npm:` backend.
- `DD_shim-runs` `codegraph init` runs from the installed shim in that
  repository.

<!-- sokf:links -->
[sokf:architecture]: /knowledge/architecture.md
[sokf:constraints-non-goals]: /knowledge/constraints-non-goals.md
