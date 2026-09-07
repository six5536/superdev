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

- [x] BUILD discovery: Final review after correction exhaustion found that integration can leave an already-checked-out default worktree stale and cache operations allow a symlinked private parent directory; re-scope Block 6 to address both findings.
- [x] BUILD discovery: Block 6 checkpoint correctly refused the final correction because its SCOPE-approved Areas did not explicitly include the Rust workflow paths and primary issue record; re-scope only the Areas metadata needed for the approved corrections.
- [x] BUILD discovery: Final review found the cache symlink checks remain check-then-use raceable; re-scope Block 6 to permit a descriptor-relative no-follow filesystem dependency and Cargo lockfile changes for atomic cache access.
- [ ] BUILD discovery: Preserve all execution history and Block 6. Permit the final integration correction in crates/lib/superdev-core/src/workflow/git.rs and its existing tests: construct the prepared tree before publication, reserve the default branch through a worktree, synchronize the primary worktree to the prepared candidate before CAS, and make the immutable two-ref CAS the final fallible integration operation so no post-publication cleanup or checkout can fail.
- [ ] BUILD discovery: Add a stable Block 7 publication unit for the approved final integration correction because Block 6 is preserved complete and the fresh re-scope intentionally preserves the exhausted historical correction count. Block 7 is limited to crates/lib/superdev-core/src/workflow/ and the owning plan, depends on Block 6, and reruns the full approved Verification commands.
- [ ] BUILD discovery: Final review 59a5aa521b0ced87f2fce9a45ad27c3796a8dd5b79a5ca68 found that cache transactions can report failure after a successful mutation if explicit advisory unlock fails. Add a stable Block 8 limited to crates/lib/superdev-core/src/workflow/cache.rs and the owning plan; make descriptor close the release fallback and never supersede an operation result with a post-mutation unlock error; rerun full Verification.
## Comments

The bootstrap implementation specification is `SCOPE-BUILD-ACCEPT-IMPLEMENTATION-PLAN.md`; it intentionally predates the plan schema it installs.

The human explicitly approved that specification before commit `83e04ea`. A separate requirements pass found no unresolved requirement before BUILD began on `work/059-scope-build-accept-workflow`.
