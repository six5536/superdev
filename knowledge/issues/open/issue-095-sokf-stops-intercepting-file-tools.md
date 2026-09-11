---
type: Issue
id: issue-095-sokf-stops-intercepting-file-tools
title: SOKF intercepts the file tools to guard writes it cannot see
description: Routed read, edit, and write add an identity layer over paths that already describe themselves, and enforce policy on one of several ways an agent writes knowledge, so the cost is permanent and the guarantee is partial.
kind: feature
lifecycle: open
links:
  - rel: supersedes
    to: issue-082-sokf-mutation-parity
    note: Mutation parity assumed routed mutation continues; this issue removes it instead.
  - rel: supersedes
    to: issue-083-sokf-validation-follow-ups
    note: Keeps the freshness and bounded-reporting requirements, drops the file-tool prompt metadata half.
  - rel: references
    to: issue-077-sokf-file-tool-parity
    note: Delivered routed read parity and the source resolver; this issue removes both in favour of unrouted paths.
---

# Feature: SOKF stops intercepting the file tools

## Summary

An agent should read and edit canonical knowledge with the ordinary file tools,
and the knowledge should be consistent when the turn ends. SOKF instead
intercepts `read`, `edit`, and `write` to add identity routing and mutation
policy, which costs a permanent adapter, a resolver, a mutation transport, and
two contracts — and still guards only the writes that arrive through those
tools.

## Context

Knowledge is a directory of Markdown files whose paths already carry type,
lifecycle, and topic: `knowledge/issues/open/issue-082-sokf-mutation-parity.md`.
The `sokf:<id>` alias the adapter resolves carries strictly less — it drops the
lifecycle — and needs an MCP round trip to become a path a tool can open.

An agent writes knowledge many ways. It runs `bash` with a heredoc, `sed`, or
`git checkout`; it applies a patch; it uses the file tools. Only the last passes
through `sokf_edit`, so mutation-time policy is one door of several. The policy
itself guards `id`, `verified`, and `generated`: no concept in this repository
carries a `verified` or `generated` stamp, and validation already reports an
`id` that disagrees with its filename.

Repair during mutation also changes the bytes the caller just wrote. Closing
[issue-077][sokf:issue-077-sokf-file-tool-parity] moved its record from
`issues/open/` to `issues/done/` as a repair to an edit that changed one
frontmatter line, and returned four whole-file diffs as model-visible content.

`turn_end` already exists in `.pi/extensions/sokf.ts` and already runs
`superdev validate`. It observes the tree, so it covers every way bytes arrive.

## Behaviour

The file tools are Pi's own, SOKF keeps what a file tool cannot do, and the
knowledge is repaired and checked once per turn.

- MCP serves exactly `sokf_search`, `sokf_graph`, and `sokf_overview`. The
  routed `sokf_resolve_source`, `sokf_retrieve`, `sokf_edit`, and `sokf_write`
  operations are gone, and the server's initialization instructions name only
  what it serves.
- `read`, `edit`, and `write` in a Pi session are Pi's built-ins, unmodified.
  No SOKF description, prompt guideline, schema, wrapper, or result dialect
  reaches them, and a `knowledge/` path behaves as any other path.
- `sokf_graph` returns each concept's repository-relative path beside its
  identity, so a traversal reaches a file without a second lookup, as
  `sokf_search` already does.
- `sokf_overview` returns the knowledge name, concept count, tree, index state,
  and capped validation warnings. It carries no rendered concept and no section
  addressing; `read` and `sokf_search` cover those.
- Every turn ends with one repair-and-validate pass, run unconditionally. The
  extension does not first decide whether the knowledge changed: a write through
  `bash`, a patch, or `git checkout` reaches no tool it registers, so any change
  signal would be wrong on exactly the cases this issue exists to cover. A turn
  that touched no knowledge leaves the tree byte-identical.
- Findings are re-checked against the working tree immediately before they are
  sent, and only those the tree still carries are reported. A report that
  survives no finding sends no message.
- The first report for a pending sequence sends one visible triggering
  follow-up. A later report is visible and non-triggering. Follow-up state is
  session memory keyed by repository root and nothing persists.
- Every visible message carries concise paths, locations, messages, and
  corrective actions, stops at 200 lines or 8 KiB, omits patches, and directs a
  truncated report to `superdev validate`.
- The command-line `superdev sokf edit` and `superdev sokf write`, and the
  workflow transitions that call the mutation service directly, behave exactly
  as they do today.
- The agent-facing guidance stops describing routing. The standing instruction,
  the `sokf-authoring` skill, and the SOKF behaviour evaluations name a physical
  `knowledge/` path where they name a `sokf:` address today, and name one
  turn-end repair where they promise two automatic repair follow-ups. Guidance
  that directs an agent at an operation nothing serves is worse than none.

## Scope

The agent-facing SOKF surface and where knowledge is made consistent.

- In: the Pi extension's tool registrations and its `turn_end` handler, the MCP
  tool set, `sokf_graph` output, the new `sokf_overview` operation, and the
  removal of source resolution, retrieval, and routed mutation.
- In: retiring `contract-012-api-sokf-pi-file-tools`, removing the routed
  promises from `contract-003-api-sokf`, and superseding
  `adr-053-sokf-file-tools-delegate-to-pi`.
- In: the standing agent instruction, the `sokf-authoring` skill and its pack
  mirror, and the SOKF behaviour evaluation fixtures, each of which directs an
  agent at a routed address today.
- Out: `superdev sokf edit`, `superdev sokf write`, and `SokfService::edit` and
  `write` as library calls. Workflow transitions depend on them, their callers
  are deterministic rather than agents, and `contract-002-cli-superdev` settles
  their behaviour with no pending promise. The mutation service keeps its
  inline repair for them.
- Out: semantic ranking, index lifecycle, MCP transport, search behaviour, and
  any change to what validation itself checks or repairs.
- Out: restoring mutation-time policy through another mechanism. A check that
  observes the tree at turn end covers every writer; a check on one tool does
  not.

## Comments

Implementation is complete under
[plan-078][sokf:plan-078-sokf-stops-intercepting-file-tools], outside the workflow
at the user's direction. The plan records executable verification, the skipped
approval and review gates, and the unchanged application coverage failure.
The issue remains open; implementation evidence is not workflow acceptance.

Supersedes [issue-082][sokf:issue-082-sokf-mutation-parity], whose premise was
that routed mutation continues and should match Pi's contracts, and
[issue-083][sokf:issue-083-sokf-validation-follow-ups], whose freshness and
bounded-reporting requirements survive here while its file-tool prompt metadata
does not.

This reverts most of [issue-077][sokf:issue-077-sokf-file-tool-parity], which
delivered routed reads that return exact source. That is the right outcome by a
cheaper route: an unrouted read returns exact source because it opens the file.

The question that settled it is what the work is for. Consistent interfaces,
consistent behaviour, and knowledge that is correct when the turn ends. Routing
serves none of them: it adds a second interface over the file tools, makes a
`knowledge/` path behave unlike every other path, and guards a fraction of the
writes.

<!-- sokf:links -->
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/done/issue-077-sokf-file-tool-parity.md
[sokf:issue-082-sokf-mutation-parity]: /knowledge/issues/wontfix/issue-082-sokf-mutation-parity.md
[sokf:issue-083-sokf-validation-follow-ups]: /knowledge/issues/wontfix/issue-083-sokf-validation-follow-ups.md
[sokf:plan-078-sokf-stops-intercepting-file-tools]: /knowledge/plans/open/plan-078-sokf-stops-intercepting-file-tools.md
