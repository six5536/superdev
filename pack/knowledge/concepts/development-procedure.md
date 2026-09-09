---
type: DevelopmentProcedure
id: development-procedure
title: Development Procedure
description: Setup, the SCOPE to BUILD to ACCEPT workflow, and required local verification.
status: draft
---

Install the pinned project toolchain and run the repository's documented setup command before starting work.

# Workflow

1. Capture unrelated issues and ideas with `/skill:file` from any checkout; the LLM chooses the number, authors the record and index entry, validates, and commits on the default branch with ordinary tools, using a worktree when elsewhere without pausing active work. Start or resume one canonical issue and plan in SCOPE on its matching `work/<issue-number>-<slug>` branch; obtain a clean isolated requirements review and explicit human approval.
2. In BUILD, complete dependency-ready stable blocks separately. Each block records executable evidence, updates applicable documentation, and receives one path-scoped checkpoint commit.
3. Run full local verification and a fresh isolated read-only review against an immutable candidate. Correct findings within project-configured limits and rerun both gates.
4. In ACCEPT, apply project acceptance policy, close accepted records, and release ownership on the checked-out work branch. Leave merging to the human. Do not push, release, delete branches, stash, reset, discard work, or resolve conflicts automatically.
