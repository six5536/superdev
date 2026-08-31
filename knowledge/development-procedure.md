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

1. Significant changes travel the knowledge-carried workflow skills in
   `.claude/skills/` (`/frame` → `/contract-design` → `/feature-plan` →
   `/build` → `/integrate`; see `.agents/process.md`): the feature is
   framed as a tracker issue whose acceptance criteria are EARS
   sentences, the contracts it touches are updated in place and linked
   from the issue, and the plan is filed as `plan-<nnn>-feature-<slug>`
   (`lifecycle: done` in the commit that completes the work). One-off
   work takes `/adhoc-plan`, and its plan is filed beside it as
   `plan-<nnn>-adhoc-<slug>`.
2. One branch per feature: `/frame` cuts `feature/<slug>` off `main`, and
   an adhoc plan that touches code runs on `adhoc/<slug>`. A human
   fast-forwards `main`; an unattended run commits and merges only on the
   feature's branch (ADR-021).
3. Implement with focused commits, using
   [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
   `fix:`, `docs:`, `test:`, `refactor:`, `chore:`).
4. Update this knowledge when behaviour or design changes.
5. Before a PR, run the full CI-equivalent check list (see
   [development-commands][sokf:development-commands]) and meet
   [definition-of-done][sokf:definition-of-done]. CI runs tests on macOS and
   Windows, the blueprint-drift check on every platform, and the coverage gate
   on Linux.

# This repo manages its own skills and knowledge machinery

superdev fills the `skills` and `code-index` capabilities here, and carries
the SOKF knowledge as part of itself: committed `.superdev/config.toml`
and `.superdev/lock.toml`, with `cargo run -- sync` writing the two
pack skills, the SOKF-carried skill set with its PostToolUse hook
entry, the `.agents` files, and the codegraph pin, index and agent wiring.
The knowledge scaffolds were this repo's before the component was
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
([ADR-004][sokf:adr-004-base-pack-identity]), so **deleting or renaming**
an item under `pack/` does not remove its live copy — that still needs a
rebuild, and `status --drift` stays green until then
([I003][sokf:issue-003-bug-a-local-pack-cannot-remove-what-it-dropped]). What it
no longer does is record a digest for the pin: a path pack has none, so a
commit touching `pack/` no longer rewrites a lock line
([ADR-016][sokf:adr-016-a-path-pack-records-no-digest]). Run `sync` with
such a commit anyway — the per-file hashes still move when a live copy does,
and a lock that has stopped describing what is on disk is the failure
[I005][sokf:issue-005-bug-a-backport-leaves-the-lock-stale] closed.

The managed hook entry names a bare `superdev`, and this repo has no installed
copy. `scripts/superdev` execs `cargo run` against this tree; symlink it onto
your PATH once, as [CONTRIBUTING](/CONTRIBUTING.md) says.

# Working with this repo's knowledge

The canonical knowledge is served to agents over MCP. `.mcp.json` and the hook name a
bare `superdev`, which the dev shim (`scripts/superdev`, symlinked onto PATH
per [CONTRIBUTING](/CONTRIBUTING.md)) execs as `cargo run` against this
tree; `npm run check:validate` runs `cargo run --quiet -- validate`
directly. Compilation is cached, so the cost after the first
build is negligible — and every check tests the code you are editing rather
than a binary from last month.

One search trap: plans and issues quote the question you are asking, at
length, in prose. A search for behaviour will happily return the plan that
proposed it over the concept that documents it. `sokf_search`'s `types` and
`lifecycle` filters scope the hunt — filter to the reference kinds for how
things work now, to `["FeaturePlan", "AdhocPlan"]` or the decision kinds
when you want the reasoning behind them.

[^contributing]: Contributing guide

<!-- sokf:links -->
[sokf:adr-004-base-pack-identity]: /knowledge/adrs/active/adr-004-base-pack-identity.md
[sokf:adr-016-a-path-pack-records-no-digest]: /knowledge/adrs/active/adr-016-a-path-pack-records-no-digest.md
[sokf:definition-of-done]: /knowledge/definition-of-done.md
[sokf:development-commands]: /knowledge/development-commands.md
[sokf:issue-003-bug-a-local-pack-cannot-remove-what-it-dropped]: /knowledge/issues/wontfix/issue-003-bug-a-local-pack-cannot-remove-what-it-dropped.md
[sokf:issue-005-bug-a-backport-leaves-the-lock-stale]: /knowledge/issues/done/issue-005-bug-a-backport-leaves-the-lock-stale.md
