---
type: Procedure
id: development-procedure
title: Development Procedure
description: Setup, the spec-and-plan change workflow, what to run before a PR, how this repo manages its own skills, and how it serves and searches its own knowledgebase.
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

1. Significant changes travel the knowledge-carried workflow skills in
   `.claude/skills/` (`/frame` → `/spec` → `/interface-design` →
   `/feature-plan` → `/build` → `/verify` → `/integrate`; see
   `.agents/process.md`): the spec lands in `knowledge/specs/`
   (permanent decision record), the plan in `knowledge/plans/` (tagged
   `done` in the commit that completes the work). One-off work takes
   `/adhoc-plan`.
2. Implement with focused commits, using
   [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
   `fix:`, `docs:`, `test:`, `refactor:`, `chore:`).
3. Update this knowledgebase when behaviour or design changes.
4. Before a PR, run the full CI-equivalent check list (see
   [development-commands](development-commands.md)) and meet
   [definition-of-done](definition-of-done.md). CI runs tests on macOS and
   Windows, the blueprint-drift check on every platform, and the coverage gate
   on Linux.

# This repo manages its own skills and knowledge machinery

superdev fills the `skills`, `knowledge`, `code-index` and
`bash-output-filter` capabilities here: committed `.superdev/config.toml`
and `.superdev/lock.toml`, with `cargo run -- sync` writing the three
pack skills, the knowledge-carried skill set with its PostToolUse hook
entry, the `.agents` files, the codegraph pin, index and agent wiring,
and the rtk pin files with their PreToolUse rewrite hook. The knowledge
scaffolds (the bundle) were this repo's before the capability was
enabled, so they are untouched; `frontend` stays off.
`npm run check:blueprint` is what catches drift in the shipped skill assets —
in the pre-PR list and in CI, through the product's own drift detection rather
than a parity test.

The manifest also pins `/pack/` as a local-path pack, so this repo's content
comes from the tree rather than from the copy compiled into the binary: edit a
skill, template or scaffold under `pack/` and `cargo run -- sync` writes it to
`.claude/skills/` with no rebuild in between. That retired the `asset-backport`
skill; `pack-backport` replaced it. The pin removed the pack-to-live round
trip — no rebuild stands between them — not the live-to-pack one, so an edit
made to a live copy to try it still has to be mirrored into `pack/` before the
next `sync` overwrites it.

Two things the pin does not do. It **layers** rather than replacing, because
only the blueprint's default git source is the base
([ADR-004](decisions/D004-base-pack-identity.md)), so **deleting or renaming**
an item under `pack/` does not remove its live copy — that still needs a
rebuild, and `status --drift` stays green until then
([I003](issues/I003-a-local-pack-cannot-remove-what-it-dropped.md)). And the
lock records a digest over the whole tree, so any commit touching `pack/`
should be made with `sync` run
([I004](issues/I004-a-path-packs-digest-churns-and-is-never-checked.md)).

The managed hook entry names a bare `superdev`, and this repo has no installed
copy. `scripts/superdev` execs `cargo run` against this tree; symlink it onto
your PATH once, as [CONTRIBUTING](/CONTRIBUTING.md) says.

# Working with this repo's knowledgebase

The bundle is served to agents over MCP. `.mcp.json` and the hook name a
bare `superdev`, which the dev shim (`scripts/superdev`, symlinked onto PATH
per [CONTRIBUTING](/CONTRIBUTING.md)) execs as `cargo run` against this
tree; `npm run check:aokf` runs `cargo run --quiet -- aokf validate
knowledge` directly. Compilation is cached, so the cost after the first
build is negligible — and every check tests the code you are editing rather
than a binary from last month.

One search trap: specs and plans quote the question you are asking, at length,
in prose. A search for behaviour will happily return the spec that proposed it
over the concept that documents it. `aokf_search`'s `types` filter keeps only
the types you name, so scope the hunt — `["Reference", "Convention"]` for how
things work now, `["Spec"]` when you want the reasoning behind them.

[^contributing]: Contributing guide
