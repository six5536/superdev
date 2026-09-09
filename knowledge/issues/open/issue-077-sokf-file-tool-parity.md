---
type: Issue
id: issue-077-sokf-file-tool-parity
title: SOKF-routed file tools diverge from Pi's built-in contracts
description: SOKF-routed read, edit, and write return different source, result, error, diff, rendering, and prompt behaviour from Pi's built-in file tools, making targeted knowledge edits fragile and expensive in model context.
kind: feature
lifecycle: open
links:
  - rel: references
    to: idea-012-sokf-mutations-survive-validation-failures
    note: Originating mutation-survival idea; this issue settles the post-persistence cancellation behavior.
---

# Feature: SOKF-routed file tools match Pi's built-in contracts

## Summary

Agents need `read`, `edit`, and `write` to retain Pi's built-in contracts when a path routes through SOKF. Current routing makes targeted knowledge edits fragile, returns oversized mutation internals, and creates a second file-tool user experience.

## Context

The [SOKF mutations survive validation failures][sokf:idea-012-sokf-mutations-survive-validation-failures]
idea originated the mutation-survival requirement. This issue settles the
idea's post-persistence cancellation behavior.

`SOKF-EDIT-RELIABILITY-PLAN.md` records the observed gaps and the proposed parity boundary. Repository inspection confirms each gap. Virtual reads use `SokfService::read_path()` and return rendered concepts. The Pi edit adapter reconstructs a display diff from `MutationResult`, while `whole_file_diff()` marks every old and new line as changed. Edit and write return the MCP mutation JSON as model-visible content. Custom file-tool metadata replaces Pi's built-in prompt text.

Pi 0.85.1 exposes the executable reference through `createReadTool()`, `createEditTool()`, `createWriteTool()`, `ReadToolDetails`, `EditToolDetails`, diff helpers, inherited renderers, argument preparation, and `withFileMutationQueue()`. The SOKF MCP contract is unreleased and already permits tool and schema changes without a deprecation path, so preserving its current mixed semantic-and-file `sokf_read` behavior is not a requirement.

## Behaviour

A caller using a physical path or an equivalent SOKF identity observes the same Pi file-tool contract except for concise, actionable SOKF policy rejections before persistence and a separate validation follow-up after persistence.

- SOKF-routed reads return exact UTF-8 source and preserve built-in pagination, 2,000-line and 50 KB truncation, continuation text, details, errors, cancellation, metadata, and rendering.
- The Pi `read` slot treats `sokf:<id>` as a file identity. MCP source resolution and semantic retrieval use separate operations rather than a second file-read result dialect.
- The MCP contract replaces the mixed-purpose `sokf_read` operation with a source resolver for file routing and a semantic `sokf_retrieve` operation for overviews, rendered concepts, and sections. The change requires no compatibility alias or deprecation path.
- SOKF-routed edits preserve built-in argument preparation, matching, uniqueness, overlap, byte-order mark and line-ending preservation, queueing, errors, concise content, compact details, metadata, and rendering.
- SOKF-routed writes preserve built-in parameters, parent creation, queueing, errors, cancellation, concise content, undefined details, metadata, and rendering.
- Resolution, matching, compare-and-swap, policy, generated-ownership, cancellation, and persistence failures that occur before the requested mutation persists return tool errors and leave every target unchanged.
- Once MCP acknowledges `applied: true`, SOKF keeps the requested bytes and every successfully persisted automatic repair or refiling change. Later repair, validation, lifecycle, or cancellation findings never roll back a file or convert the acknowledged mutation into a tool error.
- An edit or write acknowledged with `applied: true` returns Pi's normal success content and details unchanged. Mutation envelopes, repair reports, validation findings, and whole-file patches never replace or augment that file-tool result.
- An indeterminate local MCP transport or process failure may return a tool error after bytes changed when the adapter receives no authoritative outcome. The error requires manual filesystem inspection and `superdev validate`; SOKF does not add durable mutation-outcome reconciliation.
- Source resolution accepts existing and missing contained physical paths, repository-root and nested-working-directory spellings, and one Pi-compatible leading `@`. A direct physical argument must enter through `knowledge/`; a symlink may canonicalize to any target inside the repository, including a target outside `knowledge/`. Resolution refuses repository escapes, reports generated-region authority, and binds mutation dispatch to the resolved ingress and canonical target so symlink retargeting fails before persistence.
- An eligible Markdown file reached through a repository-contained file or directory symlink below `knowledge/` is SOKF knowledge under its logical ingress. SOKF parses, indexes, graphs, repairs, refiles, and validates the selected logical member once by canonical file identity. A direct ingress wins over a symlink ingress; otherwise the lexically first ingress wins. Repair writes the canonical file while preserving symlinks. Refiling moves an independently movable file-symlink alias; a wrong-location member reached only below a directory symlink stays in place with an actionable finding. A repository-external target, and a repository file with no logical `knowledge/` ingress, is not SOKF knowledge.
- The replacement MCP operations expose exact closed request and structured-result schemas. Source resolution returns `ingressPath`, `canonicalPath`, `exists`, and line-bounded `generatedRegions`; routed mutation results return literal `applied: true`, validation state, resolved and final paths, unique changed paths, bounded actionable findings, and an explicit truncation flag.
- The extension persists each repository's validation sequence as an exact closed, versioned, discriminated snapshot. A clean snapshot contains only its version, canonical repository root, and `clean` state. Each unresolved snapshot contains those fields plus its state, unique lexical changed paths, bounded findings, and an explicit truncation flag; only `pending-auto` also contains a follow-up count from 0 through 2.
- At `turn_end`, the extension validates one coalesced unresolved sequence. The first two failures each produce a separate bounded visible Pi follow-up that triggers one targeted correction turn. A failure after the second correction turn produces non-triggering manual guidance.
- A valid result clears the sequence, follow-up count, changed paths, and findings. Additional mutations join an unresolved sequence without resetting its count. A mutation after the cap schedules one manual validation without restoring automatic turns. Reload, resume, fork, compaction, and tree navigation restore the latest active-branch state instead of resetting the cap. If any required snapshot append fails, the extension enters an explicit repository-scoped in-memory degraded mode, preserves the intended state and whether validation is due, suppresses triggered follow-ups until a later exact snapshot append succeeds, and emits separate bounded non-triggering guidance. An append failure after `applied: true` never changes the file-tool success. An append failure before a triggered follow-up suppresses that follow-up without incrementing its count. Failures while entering `manual` or `clean` preserve those intended in-memory states without claiming durable transition.
- Virtual and physical aliases serialize on the same canonical physical mutation-queue key until persistence, repair, and mutation-state capture settle.
- SOKF continues to enforce stable identity, byte-preserved verification data, generated ownership, repair, refiling, and validation without replacing standard tool results.
- A generated-projection edit fails before mutation and names its authoritative source.
- `sokf_search` and `sokf_graph` own knowledge discovery wording. The file tools retain Pi's wording plus only the minimum SOKF routing rules.
- A paired characterization harness compares built-in and SOKF-routed results, errors, details, filesystem bytes, rendering inputs, turn lifecycle, and one-line-edit result size across success and edge cases.

## Scope

The work covers the Pi SOKF adapter, the SOKF MCP tool list and exact closed routed schemas required to separate exact-source routing from semantic retrieval, the exact closed durable validation-state schema, SOKF membership through repository-contained symlinks, the SOKF service and pre-persistence/post-persistence mutation boundary, complete source-resolver coverage, parity tests, automatic repair survival, the persisted validation-follow-up state machine, generated-content rejection, prompt metadata, existing shipped and materialized copies, packaging lock updates, and affected documentation and canonical knowledge. MCP compatibility changes are limited to removing the mixed-purpose `sokf_read`, adding source resolution, naming semantic retrieval `sokf_retrieve`, and narrowing mutation transport or diagnostics when parity requires it.

The work excludes a semantic retrieval dialect in the Pi `read` slot, changes to semantic ranking or rendered retrieval content, unrelated MCP transport lifecycle changes, durable mutation identity or outcome reconciliation, approximate forks of Pi's algorithms, validation inside file-tool results, rollback after acknowledged persistence, whole-file rewrites as automatic recovery from failed targeted edits, a new extension packaging model, the general validator symlink walk tracked by issue-031, and any weakening of SOKF safety or final validation.

<!-- sokf:links -->
[sokf:idea-012-sokf-mutations-survive-validation-failures]: /knowledge/ideas/idea-012-sokf-mutations-survive-validation-failures.md
