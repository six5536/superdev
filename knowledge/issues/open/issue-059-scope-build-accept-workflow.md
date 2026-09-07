---
type: Issue
id: issue-059-scope-build-accept-workflow
title: The development workflow is contradictory and does not reliably deliver reviewed documentation
description: The workflow duplicates ownership across skills, leaves manual cases unexecuted, and treats user-facing documentation as an optional final reminder.
kind: feature
lifecycle: open
---

# Feature: the development workflow is contradictory and does not reliably deliver reviewed documentation

## Summary

Superdev needs one concise, resumable SCOPE → BUILD → ACCEPT workflow whose state and safety gates are enforced by the Rust core and orchestrated in Pi.

## Context

The current skills disagree about who owns the build loop, leave manual cases without an executor, bind unattended continuation to Claude Code hooks, and mention documentation only as a final generic update. The approved root implementation plan records the replacement design.

## Behaviour

SCOPE creates or adopts this issue and produces one approved plan; BUILD executes every block with evidence and an isolated review; ACCEPT applies project policy and integrates with a no-fast-forward merge. A project documentation map makes handwritten, generated, and website documentation a planned and verified deliverable.

## Scope

The replacement covers the complete local development workflow and its managed assets.

- In: workflow schemas and state, Pi orchestration, isolated agents, Git safety, documentation mapping, record migration, pack shipping, and Claude-skill archival.
- Out: remote CI, pushing, releases, and future Claude Code support.

## Discoveries

- [ ] BUILD discovery: Final review after correction exhaustion found that integration can leave an already-checked-out default worktree stale and cache operations allow a symlinked private parent directory; re-scope Block 6 to address both findings.

## Comments

The bootstrap implementation specification is `SCOPE-BUILD-ACCEPT-IMPLEMENTATION-PLAN.md`; it intentionally predates the plan schema it installs.

The human explicitly approved that specification before commit `83e04ea`. A separate requirements pass found no unresolved requirement before BUILD began on `work/059-scope-build-accept-workflow`.
