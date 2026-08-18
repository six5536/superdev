# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
superdev uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While
superdev is pre-1.0, minor versions may contain breaking changes.

Every released tag needs its own section here. The release workflow refuses to
publish a version it cannot find a heading for.

## [Unreleased]

### Added

- Project templates: `init --template rust-npm` (or a prompt, on a TTY)
  seeds a new repo with a Rust CLI workspace deployed as prebuilt binaries
  through npm — crates, launcher and platform packages, CI and release
  workflows, repo docs, and policy configs, all token-substituted with the
  project's name. Template files are write-once scaffolds: existing files
  win, and `sync` never touches them. `--template none` and `--name` script
  the answers; `.superdev/config.toml` records the choice under `[template]`
- Knowledge-owned skills: the aokf component carries its lifecycle skills —
  the new `aokf-bootstrap` (harvest a repo's existing docs into the bundle,
  then interview the owner to flesh out the seeded skeleton) and the
  relocated `aokf-maintain` — plus the validation hook, so all three
  exist exactly where a bundle exists and a `--no-knowledge` repo gets no
  hook. `[knowledge]` takes a `custom` list like the other skill-writing
  capabilities, and a knowledge-enabled `init` ends with the
  `/aokf-bootstrap` hint
- A fuller knowledge seed: `init` now scaffolds a starter concept skeleton
  (glossary, architecture, testing strategy and the rest) instead of a
  three-file stub, ready for agents to fill in
- Project scaffold: CI and release machinery, npm launcher + platform
  packages, cargo workspace, and the AOKF knowledgebase
- `superdev init`, `status`, `sync` and `update`: set a repo up for
  agent-driven development and keep it matching the blueprint compiled into
  the binary. `status` exits 1 when there is work to do; a failed apply rolls
  back and reports anything it could not undo
- `.superdev/config.toml` (what the repo wants) and `.superdev/lock.toml`
  (what was applied, with hashes of superdev-owned files), plus a gitignored
  `.superdev/cache/` for backups
- Managed capabilities: `knowledge` (a native AOKF bundle), `code-index`
  (codegraph), `workflows` (Superpowers), `frontend` (Anthropic's
  frontend-design plugin) and `skills` (superdev's own pack, below); each can
  be disabled with `init --no-<capability>`
- `workflows` and `code-index` install from checksum-verified release
  bundles pinned in the binary, so `update <capability>@<version>` refuses
  an explicit version for them — bare `update` moves them to the binary's
  pins
- `superdev mcp aokf`: an MCP server over the knowledge bundle with four
  read-only tools — `aokf_search`, `aokf_read`, `aokf_graph` and
  `aokf_overview`. Search is hybrid (BM25 plus a pinned local embedding
  model, fused by reciprocal rank fusion) and degrades to lexical-only when
  no model is available. The index sits in `.superdev/cache/aokf-index/` and
  re-syncs lazily on every call, so edits are visible to the next question
- `superdev aokf validate` and `superdev aokf index`: validate the bundle
  against the AOKF spec (exit 1 on errors, `--json`, `--level`,
  `--repo-root`) and force a full index rebuild
- Optional `[knowledge.embeddings]` in `.superdev/config.toml` to embed
  through an API instead of the local model; the key comes from the
  environment, never the file
- `init` registers the server in `.mcp.json` under
  `mcpServers.superdev-aokf`, merging into whatever servers are already
  there
- The `skills` capability: five skills (aokf-maintain, double-check,
  grill-me, humanise, self-improve) written into `.claude/skills/` as
  superdev-owned files, plus a PostToolUse hook in `.claude/settings.json`
  that runs `superdev aokf hook validate` and blocks edits that break the
  bundle. Claude Code loads both natively — nothing to install
- Per-skill customisation: a `PROJECT.md` beside a skill extends it and is
  never touched; `custom = ["<name>"]` under `[skills]` releases a skill
  from management entirely. The pack's version is the binary's, so
  `update skills@<version>` is refused like the other pinned capabilities
- `superdev aokf hook validate`: the hook as a subcommand — payload on
  stdin, validates in-process, works on every platform superdev ships for

### Added

- `init` adopts a repo's existing skills: one already sitting under a pack
  name, with content of its own, is released into `[skills] custom` and
  reported, instead of being overwritten and backed up
- Blueprint migrations: `sync` now removes what the blueprint no longer
  ships — dropped files, renamed paths' old copies, a disabled capability's
  pins and registrations. Unmodified leftovers are removed with a backup;
  user-edited ones are left in place, released from the lock, and reported
- `sync` ensures `CLAUDE.md` imports `AGENTS.md` (`@AGENTS.md`), so Claude
  Code actually loads the managed entry point
- `blueprint` in `.superdev/config.toml` now records the version last
  applied: `sync` stamps it, `status` reports a difference without failing
- Workflows provider selection: the manifest's `provider` field is now
  honoured, `init --workflows-provider <id>` and
  `update workflows --provider <id>` choose between `mattpocock-skills`
  (the new default — materialised into `.claude/skills/` as repo files, so
  collaborators need nothing installed) and `superpowers` (the plugin flow,
  unchanged). Switching sweeps the old provider's pin and files
- The knowledge scaffold's framework override now matches the workflows
  provider: `.agents/SUPERPOWERS.md` or `.agents/MATT-POCOCK-SKILLS.md`

### Fixed

- `sync` no longer installs the repo's whole toolchain. `mise install` and
  `mise exec` now name superdev's own pinned tools, so an unrelated pin that
  cannot build on this machine no longer fails the entire apply — found
  adopting superdev in a repo pinning `cargo:cargo-ndk`

### Changed

- The AOKF validator is now the binary's own `aokf validate`; the bundled
  Python `validator.py` is deleted, and the validation hook, `check:aokf`
  and CI all call the Rust one. Findings, JSON and exit codes are unchanged
- AGENTS.md no longer preloads every concept in the knowledgebase. It keeps
  `knowledge/index.md` as the map and tells agents to search the MCP server
  for the rest

### Removed

- The skill pack's `grill-me` — the default workflows provider ships its
  own; the next sync sweeps the packaged copy (a user-edited copy is left
  in place and released). A `[skills] custom` name that is no longer in
  the pack now reports instead of failing the plan
