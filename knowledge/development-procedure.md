---
type: Procedure
id: development-procedure
title: Development Procedure
description: Setup, the spec-and-plan change workflow, and what to run before a PR.
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

1. Significant changes follow the Superpowers flow with this project's
   overrides ([SUPERPOWERS.md](/.agents/SUPERPOWERS.md)): brainstorm a spec
   into `knowledge/specs/` (permanent decision record), then write an
   implementation plan into `knowledge/plans/` (ephemeral — deleted in the
   commit that completes the work).
2. Implement with focused commits, using
   [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`,
   `fix:`, `docs:`, `test:`, `refactor:`, `chore:`).
3. Update this knowledgebase when behaviour or design changes.
4. Before a PR, run the full CI-equivalent check list (see
   [development-commands](development-commands.md)) and meet
   [definition-of-done](definition-of-done.md). CI runs tests on macOS and
   Windows, and the coverage gate on Linux.

[^contributing]: Contributing guide
