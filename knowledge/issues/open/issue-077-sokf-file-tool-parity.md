---
type: Issue
id: issue-077-sokf-file-tool-parity
title: SOKF-routed file tools diverge from Pi's built-in contracts
description: SOKF-routed read, edit, and write return different source, result, error, diff, rendering, and prompt behaviour from Pi's built-in file tools, making targeted knowledge edits fragile and expensive in model context.
kind: feature
lifecycle: open
---

# Feature: SOKF-routed file tools match Pi's built-in contracts

## Summary

Agents need `read`, `edit`, and `write` to retain Pi's built-in contracts when a path routes through SOKF. Current routing makes targeted knowledge edits fragile, returns oversized mutation internals, and creates a second file-tool user experience.

## Context

`SOKF-EDIT-RELIABILITY-PLAN.md` records the observed gaps and the proposed parity boundary. The current virtual read renders retrieval-oriented knowledge instead of exact source bytes. The edit path requires independently reconstructed exact text, emits a whole-file mutation report, and duplicates patch data in model-visible results. The write path also exposes mutation internals. Custom prompt metadata repeats and blurs file operations with knowledge discovery.

Pi's installed `createReadTool()`, `createEditTool()`, `createWriteTool()`, diff helpers, result detail types, and mutation queue provide the executable reference contract.

## Behaviour

A caller using a physical path or an equivalent SOKF identity observes the same Pi file-tool contract except for concise, actionable SOKF policy rejections.

- SOKF-routed reads return exact UTF-8 source and preserve built-in pagination, truncation, details, errors, cancellation, metadata, and rendering.
- SOKF-routed edits preserve built-in argument preparation, matching, uniqueness, overlap, byte preservation, queueing, errors, concise content, compact details, metadata, and rendering.
- SOKF-routed writes preserve built-in parameters, parent creation, queueing, errors, cancellation, concise content, undefined details, metadata, and rendering.
- Virtual and physical aliases serialize on the same physical mutation-queue key.
- SOKF continues to enforce stable identity, preserved verification data, generated ownership, repair, refiling, and validation without replacing standard tool results.
- Mutation envelopes, whole-file patches, repair reports, and validation dumps stay out of model-visible file-tool content.
- `sokf_search` and `sokf_graph` own knowledge discovery wording. The file tools inherit Pi's wording plus only the minimum SOKF routing rules.
- A paired characterization harness compares built-in and SOKF-routed results, errors, details, filesystem bytes, rendering inputs, and one-line-edit context size across success and edge cases.

## Scope

The work covers the Pi SOKF adapter, the SOKF service and mutation boundary required for exact-source routing, parity tests, bounded repair reporting, generated-content rejection, prompt metadata, shipped copies, and affected canonical knowledge.

The work excludes a new semantic retrieval dialect for `read`, approximate forks of Pi's algorithms, whole-file rewrites as recovery from failed targeted edits, and weakening SOKF safety or final validation.
