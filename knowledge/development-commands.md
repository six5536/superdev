---
type: DevelopmentCommands
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

- `npm run check:validate` is the binary validating this repo against both
  specs it owns: the SOKF knowledge, and the files the grammar governs under the
  trees the grammar names. It exits 1 on errors, and warnings alone still
  pass. It lists its errors and counts its warnings; `--warnings` lists them
  too (ADR-040). The Claude Code hook runs the same whole-set check by a
  different route (`superdev hook validate`), so the two cannot reach
  different verdicts. `cargo run -- sokf index` forces a full index rebuild, which
  nothing routine needs: the MCP server syncs lazily on every call.
- `cargo run -- validate --fix` is the same check with its repairs applied
  first: a link naming a concept by path becomes the id form, every
  `<!-- sokf:links -->` block is regenerated, and every include block is
  materialised — a concept's body, or the source region a `/`-rooted
  path names, so a contract's Definition follows the code it includes.
  Run it before committing a knowledge change. It is not what CI runs, and not what the hook runs —
  a gate that repairs what it is measuring reports on a repository nobody
  wrote.
- `npm run check:blueprint` is `cargo run --quiet -- status` — the
  superdev-owned files here (the pack skills, the knowledge-carried skill
  set with its hook entry, and the `.agents` files) still match the
  blueprint. It exits 1 on drift, so CI gates on it.
- Release CI runs `smoke` and `smoke:launcher` per buildable target;
  `smoke:manage` is manual-only and the one run that downloads the real
  embedding model.

A third, for anyone editing the CLI's descriptions: clap takes a doc comment's
whole paragraph as the description and `wrap_help` is not enabled, so anything
longer than a terminal's width renders as one line and breaks the help table's
alignment. A description that needs detail carries a hand-wrapped
`long_about`. The man page is a one-line index per subcommand and shows no
long description at all, so detail a reader needs from `man` belongs in the
top-level one.

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
