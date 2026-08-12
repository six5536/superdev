# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
superdev uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html). While
superdev is pre-1.0, minor versions may contain breaking changes.

Every released tag needs its own section here. The release workflow refuses to
publish a version it cannot find a heading for.

## [Unreleased]

### Added

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
  (codegraph), `workflows` (Superpowers) and `frontend` (Anthropic's
  frontend-design plugin); each can be disabled with `init --no-<capability>`
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
  against the AOKF spec (exit 1 on findings, `--json`, `--level`,
  `--repo-root`) and force a full index rebuild
- Optional `[knowledge.embeddings]` in `.superdev/config.toml` to embed
  through an API instead of the local model; the key comes from the
  environment, never the file
- `init` registers the server in `.mcp.json` under
  `mcpServers.superdev-aokf`, merging into whatever servers are already
  there

### Changed

- The AOKF validator is now the binary's own `aokf validate`; the bundled
  Python `validator.py` is deleted, and the validation hook, `check:aokf`
  and CI all call the Rust one. Findings, JSON and exit codes are unchanged
- AGENTS.md no longer preloads every concept in the knowledgebase. It keeps
  `knowledge/index.md` as the map and tells agents to search the MCP server
  for the rest
