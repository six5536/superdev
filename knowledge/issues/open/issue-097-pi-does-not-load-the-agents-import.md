---
type: Issue
id: issue-097-pi-does-not-load-the-agents-import
title: Pi does not load Superdev instructions through the AGENTS.md import
description: AGENTS.md imports .agents/superdev.md with Claude Code syntax that Pi does not expand, so Superdev instructions must be embedded before existing user content.
kind: bug
lifecycle: open
---

# Bug: Pi does not load Superdev instructions through the AGENTS.md import

## Summary

Pi does not expand the `@.agents/superdev.md` line that Superdev writes into
`AGENTS.md`. Put the complete Superdev instructions directly at the start of
`AGENTS.md` so Pi receives them without an extra file read.

## Context

The user reported that Pi does not process this Claude Code import syntax.
`repo_entry` in `/crates/lib/superdev-core/src/pipeline.rs` currently ensures
that import line and writes the instructions to `/.agents/superdev.md` from
`/crates/lib/superdev-core/src/agent-instructions.md`. The repository's own
`/AGENTS.md` contains only the import.

## Behaviour

On `init` and `sync`, Superdev places the complete instructions in a managed
block at the start of `AGENTS.md` and keeps all other existing content after it.

- Remove the obsolete standalone `@.agents/superdev.md` import line.
- Preserve all other existing user content unchanged.
- Create `AGENTS.md` when absent.
- Update the managed block when instructions change; repeated syncs do not
  duplicate the block or change an already current file.
- Keep `.agents/superdev.md` and its existing canonical source.

## Scope

Change the generator, not only this repository's materialized file. Include
tests for creation, migration, content preservation, updates, and repeated
syncs, plus the affected CLI contract and documentation.

Removing `.agents/superdev.md`, changing instruction prose, and changing Pi's
context loader are outside scope.

## Comments

The user confirmed this scope, including the managed block and preservation of
existing content, before filing.
