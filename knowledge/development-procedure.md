---
type: Procedure
id: development-procedure
title: Development Procedure
description: Setup, the spec-and-plan change workflow, what to run before a PR, how this repo manages its own skills and workflows, and how it serves and searches its own knowledgebase.
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

1. Significant changes follow the mattpocock/skills flow with this
   project's overrides
   ([MATT-POCOCK-SKILLS.md](/.agents/MATT-POCOCK-SKILLS.md)): grill the
   requirements and write the spec into `knowledge/specs/` (permanent
   decision record) with `to-spec`, then break it into an implementation
   plan in `knowledge/plans/` (tagged `done` in the commit that
   completes the work) with `to-tickets` or `wayfinder`.
2. Implement with focused commits, using
   [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
   `fix:`, `docs:`, `test:`, `refactor:`, `chore:`).
3. Update this knowledgebase when behaviour or design changes.
4. Before a PR, run the full CI-equivalent check list (see
   [development-commands](development-commands.md)) and meet
   [definition-of-done](definition-of-done.md). CI runs tests on macOS and
   Windows, the blueprint-drift check on every platform, and the coverage gate
   on Linux.

# This repo manages its own skills, workflows and knowledge machinery

superdev fills the `skills`, `workflows` and `knowledge` capabilities here:
committed `.superdev/config.toml` and `.superdev/lock.toml`, with
`cargo run -- sync` writing the three pack skills, the aokf lifecycle skills
and PostToolUse hook entry, the `.agents` files, and the materialised
mattpocock-skills set. The knowledge scaffolds (AGENTS.md, the bundle) were
this repo's before the capability was enabled, so they are untouched;
`code-index` and `frontend` stay off.
`npm run check:blueprint` is what catches drift in the shipped skill assets —
in the pre-PR list and in CI, through the product's own drift detection rather
than a parity test.

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
