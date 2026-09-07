---
type: DevelopmentProcedure
id: development-procedure
title: Development Procedure
description: Setup, the contract-driven change workflow, what to run before a PR, how this repo manages its own skills, and how it serves and searches its own knowledge.
status: stable
sources:
  - id: contributing
    resource: /CONTRIBUTING.md
    title: Contributing guide
---

Setup is `mise install` + `npm install`; detail in
[CONTRIBUTING](/CONTRIBUTING.md).[^contributing] A plain `cargo build` needs
no Node.

# Workflow

1. Significant changes run through Pi's project extension as exactly
   `SCOPE → BUILD → ACCEPT`. `/file` is an independent capture utility.
   Every plan implements exactly one issue and uses its matching
   `work/<issue-number>-<slug>` branch.
2. SCOPE settles requirements, contracts, ADRs, documentation obligations,
   stable blocks, and executable evidence; a fresh isolated read-only review
   and explicit human approval are mandatory. BUILD delegates to one isolated
   modifying child, owns block commits and evidence, then runs complete local
   verification and a fresh isolated read-only code review. ACCEPT follows
   `.superdev/config.toml` policy and integrates locally through Rust with
   `git merge --no-ff`. It never pushes, releases, deletes the branch, stashes,
   resets, discards, or resolves conflicts implicitly.
3. Implement with focused commits, using
   [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
   `fix:`, `docs:`, `test:`, `refactor:`, `chore:`).
4. Update this knowledge when behaviour or design changes.
5. Before a PR, run the full CI-equivalent check list (see
   [development-commands][sokf:development-commands]) and meet
   [definition-of-done][sokf:definition-of-done]. CI runs tests on macOS and
   Windows, the blueprint-drift check on every platform, and the coverage gate
   on Linux.

# This repo manages its own Pi and knowledge machinery

Committed `.superdev/config.toml` and `.superdev/lock.toml` govern the SOKF
bundle, `.agents` files, code index, Pi workflow extension, and genuine Pi
SOKF-authoring skill. `npm run check:blueprint` catches drift through the
product's own lock and materialization behavior.

The manifest pins `/pack/` as a local-path pack. Edit the extension under
`.pi/extensions/superdev/`, mirror it to `pack/pi/extensions/superdev/`, then
run `cargo run -- sync`; similarly mirror the SOKF authoring skill under
`pack/pi/skills/`. Retired Claude assets remain only under
`archive/claude-code/` and are excluded from active materialization.

Two things the pin does not do. It **layers** rather than replacing, because
only the blueprint's default git source is the base
([ADR-004][sokf:adr-004-base-pack-identity]), so **deleting or renaming**
an item under `pack/` does not remove its live copy — that still needs a
rebuild, and `status --drift` stays green until then
([I003][sokf:issue-003-a-local-pack-cannot-remove-what-it-dropped]). What it
no longer does is record a digest for the pin: a path pack has none, so a
commit touching `pack/` no longer rewrites a lock line
([ADR-016][sokf:adr-016-a-path-pack-records-no-digest]). Run `sync` with
such a commit anyway — the per-file hashes still move when a live copy does,
and a lock that has stopped describing what is on disk is the failure
[I005][sokf:issue-005-a-backport-leaves-the-lock-stale] closed.

# Working with this repo's knowledge

The canonical knowledge is served to agents over MCP. `.mcp.json` names a bare
`superdev`; the dev shim (`scripts/superdev`, symlinked onto PATH per
[CONTRIBUTING](/CONTRIBUTING.md)) execs `cargo run` against this tree.
`npm run check:validate` runs `cargo run --quiet -- validate` directly. Compilation is cached, so the cost after the first
build is negligible — and every check tests the code you are editing rather
than a binary from last month.

One search trap: plans and issues quote the question you are asking, at
length, in prose. A search for behaviour will happily return the plan that
proposed it over the concept that documents it. `sokf_search`'s `types` and
`lifecycle` filters scope the hunt — filter to the reference kinds for how
things work now, to `["Plan"]` or the decision kinds when you want the
reasoning behind them.

[^contributing]: Contributing guide

<!-- sokf:links -->
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:adr-016-a-path-pack-records-no-digest]: /knowledge/adrs/active/adr-016-a-path-pack-records-no-digest.md
[sokf:definition-of-done]: /knowledge/definition-of-done.md
[sokf:development-commands]: /knowledge/development-commands.md
[sokf:issue-003-a-local-pack-cannot-remove-what-it-dropped]: /knowledge/issues/wontfix/issue-003-a-local-pack-cannot-remove-what-it-dropped.md
[sokf:issue-005-a-backport-leaves-the-lock-stale]: /knowledge/issues/done/issue-005-a-backport-leaves-the-lock-stale.md
