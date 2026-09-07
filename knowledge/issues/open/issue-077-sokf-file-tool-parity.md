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

A caller using a physical path or an equivalent SOKF identity observes the same Pi file-tool contract except for concise, actionable SOKF policy rejections.

- SOKF-routed reads return exact UTF-8 source and preserve built-in pagination, 2,000-line and 50 KB truncation, continuation text, details, errors, cancellation, metadata, and rendering.
- The Pi `read` slot treats `sokf:<id>` as a file identity. MCP source resolution and semantic retrieval use separate operations rather than a second file-read result dialect.
- The MCP contract replaces the mixed-purpose `sokf_read` operation with a source resolver for file routing and a semantic `sokf_retrieve` operation for overviews, rendered concepts, and sections. The change requires no compatibility alias or deprecation path.
- SOKF-routed edits preserve built-in argument preparation, matching, uniqueness, overlap, byte-order mark and line-ending preservation, queueing, errors, concise content, compact details, metadata, and rendering.
- SOKF-routed writes preserve built-in parameters, parent creation, queueing, errors, cancellation, concise content, undefined details, metadata, and rendering.
- Before persistence starts, cancellation leaves the target unchanged. After persistence starts, edit and write keep the canonical target queued until the mutation settles. An applied mutation still records repair and validation work when Pi reports the tool as aborted.
- Virtual and physical aliases serialize on the same canonical physical mutation-queue key.
- SOKF continues to enforce stable identity, byte-preserved verification data, generated ownership, repair, refiling, and validation without replacing standard tool results.
- A generated-projection edit fails before mutation and names its authoritative source.
- Mutation envelopes, whole-file patches, repair reports, and validation dumps stay out of model-visible file-tool content.
- `sokf_search` and `sokf_graph` own knowledge discovery wording. The file tools retain Pi's wording plus only the minimum SOKF routing rules.
- A paired characterization harness compares built-in and SOKF-routed results, errors, details, filesystem bytes, rendering inputs, and one-line-edit result size across success and edge cases.

## Scope

The work covers the Pi SOKF adapter, the SOKF MCP tool list and schemas required to separate exact-source routing from semantic retrieval, the SOKF service and mutation boundary, parity tests, bounded repair reporting, generated-content rejection, prompt metadata, existing shipped and materialized copies, packaging lock updates, and affected documentation and canonical knowledge. MCP compatibility changes are limited to removing the mixed-purpose `sokf_read`, adding source resolution, naming semantic retrieval `sokf_retrieve`, and narrowing mutation transport or diagnostics when parity requires it.

The work excludes a semantic retrieval dialect in the Pi `read` slot, changes to semantic ranking or rendered retrieval content, unrelated MCP transport or lifecycle changes, approximate forks of Pi's algorithms, whole-file rewrites as automatic recovery from failed targeted edits, a new extension packaging model, and any weakening of SOKF safety or final validation.

<!-- sokf:links -->
[sokf:idea-012-sokf-mutations-survive-validation-failures]: /knowledge/ideas/idea-012-sokf-mutations-survive-validation-failures.md
