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
  too (ADR-040). `superdev hook validate` remains a thin adapter over the same
  whole-set check. `cargo run -- sokf index` forces a full index rebuild, which
  nothing routine needs: MCP or CLI `sokf search` and `overview` sync the index
  lazily. `sokf read` and `graph` parse current knowledge without opening the
  index or loading embeddings. Those four CLI commands expose the same service
  for shell users and harness adapters; their `--json` form wraps text
  in the versioned `sokf-tools/v1` tool-result envelope. `cargo run -- sokf
  edit` makes exact replacements in an existing concept; `sokf write` replaces
  one or creates it at a physical path. Both repair and validate automatically,
  default to preserving `id` and `verified`, and report applied-but-invalid
  intermediate states without turning them into retryable command failures.
  Pi auto-loads `.pi/extensions/sokf.ts`, which maps its built-in file-tool
  shapes plus `sokf_search` and `sokf_graph` onto one lazily started
  `superdev mcp sokf` process per repository. The MCP process initializes the
  embedder on its first search or `sokf:` overview read and reuses it until Pi
  session shutdown. The adapter performs final validation after mutation turns
  through a one-shot CLI call and limits automatic repair feedback to two
  follow-up turns. `node --test scripts/test/sokf-mcp-client.test.mjs` checks
  MCP framing, process reuse, restart and shutdown without a model call. Load
  Pi's native `/skill:sokf-authoring` for format-sensitive knowledge changes.
  Run `/system-prompt` in Pi to refresh
  the ignored `.pi/current-system-prompt.md` when inspecting effective
  instructions.
- `npm run eval:sokf` validates the behavioral fixture without making model
  calls. Add `-- --run --trials=3` to execute the acceptance matrix with the
  `PI_PROVIDER` and `PI_MODEL` environment values. Add
  `--without-instruction` for the comparison arm or `--scenario=<id>` for one
  case. Progress goes to stderr and a `sokf-evaluation-result/v1` report goes
  to stdout.
- `cargo run -- validate --fix` is the same check with its repairs applied
  first: a link naming a concept by path becomes the id form, every
  `<!-- sokf:links -->` block is regenerated, and every include block is
  materialised — a concept's body, or the source region a `/`-rooted
  path names, so a contract's Definition follows the code it includes.
  Run it before committing a knowledge change. It is not what CI runs, and not what the hook runs —
  a gate that repairs what it is measuring reports on a repository nobody
  wrote.
- `cargo run -- workflow status --json` reports canonical workflow identity
  and transient Pi ownership. The other typed `workflow` subcommands bind or
  resume ownership, apply compare-and-swap transitions and evidence updates,
  cancel, abandon after human confirmation, or integrate locally with checked
  `git merge --no-ff`.
- `npm run check:blueprint` is `cargo run --quiet -- status` — the owned Pi
  extension, Pi authoring skill, schemas, and `.agents` files still match the
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

Two things about the dev shim (`scripts/superdev`): it runs `cargo run`, so
a hook after a Rust edit rebuilds the binary first — the shim therefore
answers `hook validate` itself, without cargo, for any path outside the
trees the hook watches, since the rebuild was the whole cost of a Rust
edit; and there is no `rustc-wrapper` — sccache cached nothing for the
incremental dev profile and its idle-exit made concurrent builds race to
restart it, so `.cargo/config.toml` no longer names it.

Two traps:

- `npm run lint` is only `cargo clippy --workspace`; CI runs clippy with
  `--all-targets -- -D warnings` plus fmt-check, doctests, rustdoc `-D
  warnings`, launcher tests, release-script tests (`npm run test:scripts`),
  version consistency, and the coverage gate. The script tests also protect
  the `sokf-behavior/v1` fixture roster and shape. The same command loads the
  real SOKF extension against the pinned `@earendil-works/pi-coding-agent`
  0.85.1 test dependency — not a `pi` on `PATH` — and exercises retrieval,
  mutation-result parity, subdirectory routing, and bounded validation feedback
  in a temporary repository. Its paired harness compares routed reads against
  Pi's built-in read case by case and fails, rather than skipping, when that
  dependency cannot load. Behavioral scoring still requires a separate
  model-session evaluator.
  Before a PR, run the full list in CONTRIBUTING, not the dailies.
- Only the launcher package is an npm workspace. The five platform-binary
  packages deliberately are not (npm enforces their `os`/`cpu` fields on
  workspace members, breaking `npm install` on every host); tooling addresses
  them by path.

[^contributing]: Contributing guide (everyday commands, authoritative list)
