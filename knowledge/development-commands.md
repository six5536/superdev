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

- Dailies: `npm run build` / `test` / `lint` / `fmt` / `check` — thin wrappers
  over cargo (`test` is `cargo nextest run --workspace` followed by
  `check:aokf`).
- Knowledgebase: `npm run check:aokf` is
  `cargo run --quiet -- aokf validate knowledge` — the binary validating this
  repo's own bundle, and the same command the Claude Code hook runs. It exits
  1 on errors; warnings alone still pass. `cargo run -- aokf index` forces a
  full rebuild of the search
  index, which nothing routine needs: the MCP server syncs it lazily on every
  tool call.
- Coverage: `npm run coverage` (HTML) / `coverage:summary` /
  `coverage:check` (the ≥90%-per-crate gate; needs the nightly toolchain).
- Packaging: `npm run test:launcher`, `npm run verify-version` (16 locations
  must agree), `npm run release <version>` (bumps, verifies, commits, tags —
  never pushes).
- Smoke: `npm run smoke` runs a release binary through version, help,
  completions, and the usage-error exit code; `npm run smoke:launcher`
  npm-packs the launcher plus the host's platform package and runs the real
  binary through the shim. Release CI runs both per buildable target.
  `npm run smoke:manage` is manual only: a real `init` and `status` in a
  scratch repo against the real mise, claude and codegraph, then `aokf
  validate` and `aokf index` — the one run that downloads the embedding model.

Two traps:

- `npm run lint` is only `cargo clippy --workspace`; CI runs clippy with
  `--all-targets -- -D warnings` plus fmt-check, doctests, rustdoc `-D
  warnings`, launcher tests, version consistency, and the coverage gate.
  Before a PR, run the full list in CONTRIBUTING, not the dailies.
- Only the launcher package is an npm workspace. The five platform-binary
  packages deliberately are not (npm enforces their `os`/`cpu` fields on
  workspace members, breaking `npm install` on every host); tooling addresses
  them by path.

[^contributing]: Contributing guide (everyday commands, authoritative list)
