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
