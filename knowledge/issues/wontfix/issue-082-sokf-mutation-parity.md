---
type: Issue
id: issue-082-sokf-mutation-parity
title: SOKF-routed edits and writes diverge from Pi's built-in mutation contracts
description: A routed edit or write returns mutation JSON instead of Pi's success content, reconstructs a whole-file diff, and can report a tool error after the bytes already persisted, so targeted knowledge mutation is fragile and expensive in model context.
kind: feature
lifecycle: wontfix
links:
  - rel: references
    to: issue-077-sokf-file-tool-parity
    note: Read routing and source resolution land first; this issue reuses the resolver and the pinned paired harness.
  - rel: references
    to: idea-012-sokf-mutations-survive-validation-failures
    note: Originating mutation-survival idea; this issue settles the persistence outcome boundary.
---

# Feature: SOKF-routed edits and writes match Pi's built-in contracts

## Summary

An agent editing canonical knowledge needs `edit` and `write` to behave as they
do on any other file. Routed mutations instead return mutation internals as
model-visible content, rebuild a display diff that marks every line changed, and
can fail after persistence succeeded. Targeted edits become fragile and consume
context that the built-in tools do not.

## Context

The Pi edit adapter in `.pi/extensions/sokf.ts` reconstructs a display diff from
`MutationResult`, and `whole_file_diff()` in `crates/lib/superdev-core/src/sokf/`
marks every old and new line as changed. Edit and write return the MCP mutation
JSON as content. A cancellation that arrives after MCP persisted the bytes still
converts the operation into `Operation aborted`, so the caller reads a failure
against a changed file.

Pi 0.85.1 exposes the executable reference through `createEditTool()`,
`createWriteTool()`, `EditToolDetails`, `generateDiffString()`,
`generateUnifiedPatch()`, inherited renderers, argument preparation, and
`withFileMutationQueue()`. Source resolution and read parity arrive with
[issue-077][sokf:issue-077-sokf-file-tool-parity]; this issue consumes the
resolver rather than redefining it.

The [SOKF mutations survive validation failures][sokf:idea-012-sokf-mutations-survive-validation-failures]
idea originated the requirement that an acknowledged mutation keep its bytes.
This issue settles that boundary for routed edits and writes.

The SOKF MCP contract is unreleased and already permits tool and schema changes
without a deprecation path, so the routed mutation transport may change shape.

## Behaviour

A caller mutating a physical path or an equivalent SOKF identity observes the
same Pi contract, except for concise SOKF policy rejections before persistence.

- Routed edits preserve Pi's `edits[]` schema, argument preparation, non-empty
  validation, exact-then-normalized matching, one-snapshot uniqueness and overlap
  checks, byte-order mark and line-ending preservation, cancellation before
  dispatch, errors, success content, compact details, metadata, and rendering.
- Routed writes preserve Pi's `{ path, content }` parameters, recursive parent
  creation, cancellation before dispatch, errors, success content, undefined
  details, metadata, and rendering. Content byte-identical to the existing target
  still persists, acknowledges, and schedules validation.
- The routed edit submits the exact original bytes and the computed final content
  through a compare-and-swap transport carrying the logical ingress and expected
  canonical target. MCP re-resolves that ingress immediately before policy checks
  and rejects canonical-target drift or stale source without changing any file.
- Resolution, argument preparation, matching, compare-and-swap, policy,
  generated-ownership, cancellation before dispatch, and an authoritative
  response establishing no apply are pre-persistence failures. Each returns a tool
  error, leaves every target unchanged, and schedules no validation.
- Once MCP acknowledges `applied: true`, SOKF keeps the requested bytes and every
  repair or refiling change that persisted. A later repair, validation, lifecycle,
  or cancellation finding never rolls back a file and never converts the
  acknowledged mutation into a tool error.
- An acknowledged mutation returns Pi's normal success content and details
  unchanged. Mutation envelopes, repair reports, validation findings, and
  whole-file patches never replace or augment that result.
- An indeterminate local MCP transport or process failure returns a tool error
  after bytes may have changed. The error directs the caller to inspect the target
  and run `superdev validate`; SOKF adds no durable outcome reconciliation.
- Virtual and physical aliases serialize on the same canonical physical
  mutation-queue key until persistence, repair, refiling, and mutation-state
  capture settle.
- An edit or write intersecting a generated projection fails before mutation and
  names its authoritative source. No routed failure performs or recommends an
  automatic whole-file rewrite.
- The routed mutation result carries literal `applied: true`, validation state,
  resolved and final paths, unique changed paths in lexical order, bounded
  actionable findings, and an explicit truncation flag, and carries no patches.
- Mutation, repair, refiling, and their queue targets stay inside the active
  checkout, including a linked Git worktree.
- The paired harness compares built-in and routed results, errors, details,
  filesystem bytes, and renderer inputs across success and edge cases.

## Scope

Routed mutation behaviour and the persistence outcome boundary.

- In: the Pi adapter's edit and write routing, the MCP mutation transport and its
  exact closed schemas, the SOKF mutation service's pre-persistence and
  post-persistence classification, bounded MCP mutation diagnostics, and paired
  mutation evidence.
- In: the mutation promises on `contract-003-api-sokf` and
  `contract-012-api-sokf-pi-file-tools`, and the outcome-boundary decision in
  `adr-053-sokf-file-tools-delegate-to-pi`.
- Out: the persisted validation follow-up state machine and its session
  recovery, which produce the separate visible turn-end message.
- Out: source resolution, read parity, and the MCP retrieval split, owned by
  [issue-077][sokf:issue-077-sokf-file-tool-parity].
- Out: rollback after acknowledged persistence, durable mutation identity or
  outcome reconciliation, whole-file rewrites as automatic recovery, validation
  content inside file-tool results, and any weakening of SOKF safety.
- Out: the command-line `superdev sokf edit` and `superdev sokf write` request
  and output shapes, which stay byte-compatible.

## Resolution

Declined on 2026-09-11 in favour of
[issue-095][sokf:issue-095-sokf-stops-intercepting-file-tools], which removes
routed mutation rather than bringing it to parity. This issue assumed SOKF keeps
intercepting `edit` and `write`; the question that settled it was whether that
interception earns its cost at all.

It does not. An agent writes knowledge through `bash`, a heredoc, `sed`, a
patch, or `git checkout` as readily as through a file tool, so mutation-time
policy guards one door of several. The policy itself protects `id`, `verified`,
and `generated`, and no concept in this repository carries a `verified` or
`generated` stamp. A check that observes the tree when the turn ends covers
every writer, which is where issue-095 puts it.

## Comments

Separated from [issue-077][sokf:issue-077-sokf-file-tool-parity] because its
requirements review exceeded the isolated-role timeout. Mutation parity depends
on issue-077's resolver and must follow it.

<!-- sokf:links -->
[sokf:idea-012-sokf-mutations-survive-validation-failures]: /knowledge/ideas/idea-012-sokf-mutations-survive-validation-failures.md
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/done/issue-077-sokf-file-tool-parity.md
[sokf:issue-095-sokf-stops-intercepting-file-tools]: /knowledge/issues/done/issue-095-sokf-stops-intercepting-file-tools.md
