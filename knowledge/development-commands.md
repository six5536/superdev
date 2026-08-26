---
type: Reference
id: development-commands
title: Development Commands
description: The npm-script command set and the pre-PR check list's shape.
status: stable
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Contributing guide (everyday commands, authoritative list)
---

Everything is wrapped as npm scripts (defined in
[package.json](/package.json); the authoritative annotated list is in
[CONTRIBUTING](/CONTRIBUTING.md)):[^contributing]

What the annotated list does not say:

- `npm run check:aokf` is the binary validating this repo's own bundle; it
  exits 1 on errors, and warnings alone still pass. The Claude Code hook
  checks the same bundle by a different route (`superdev aokf hook
  validate`), and `cargo run -- aokf index` forces a full index rebuild,
  which nothing routine needs: the MCP server syncs lazily on every call.
- `npm run check:blueprint` is `cargo run --quiet -- status` — the
  superdev-owned files here (the pack skills, the knowledge-carried skill
  set with its hook entry, and the `.agents` files) still match the
  blueprint. It exits 1 on drift, so CI gates on it.
- Release CI runs `smoke` and `smoke:launcher` per buildable target;
  `smoke:manage` is manual-only and the one run that downloads the real
  embedding model.

Two traps:

- `npm run lint` is only `cargo clippy --workspace`; CI runs clippy with
  `--all-targets -- -D warnings` plus fmt-check, doctests, rustdoc `-D
  warnings`, launcher tests, release-script tests (`npm run test:scripts`),
  version consistency, and the coverage gate.
  Before a PR, run the full list in CONTRIBUTING, not the dailies.
- Only the launcher package is an npm workspace. The five platform-binary
  packages deliberately are not (npm enforces their `os`/`cpu` fields on
  workspace members, breaking `npm install` on every host); tooling addresses
  them by path.

[^contributing]: Contributing guide (everyday commands, authoritative list)
