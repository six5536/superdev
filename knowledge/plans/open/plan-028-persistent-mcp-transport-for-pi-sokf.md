---
type: Plan
id: plan-028-persistent-mcp-transport-for-pi-sokf
title: Persistent MCP transport for Pi SOKF
description: Pi reuses one repository-scoped SOKF MCP process so frequent semantic searches load the local embedding model once without changing tool behavior.
lifecycle: open
---

# Plan: Persistent MCP transport for Pi SOKF

## Goal

Pi's SOKF tools reuse one `superdev mcp sokf` process per repository for the
life of a Pi session. The first search or overview loads the configured
embedding model and synchronizes the index; later index-dependent calls reuse
that model. Direct reads and graph traversal before then do not load the model
or open the index. Existing Pi tool names, result shapes, agent-safe mutation
policy, ordinary-file delegation and bounded turn-end validation remain
unchanged.

The existing MCP server and shared `SokfService` remain authoritative. Pi gains
a small stdio JSON-RPC client rather than an independent SOKF implementation or
a repository daemon. Out of scope: changing lexical or semantic ranking,
automatically running a background service for independent shell commands, and
adding general MCP support to Pi.

## Contract changes

- contract-003-api-sokf: revise `P_fails-at-startup` so missing or unreadable
  knowledge still fails at startup but index and embedder failures occur on the
  operation that first needs them; add `P_lazy-embedding` with
  `AC_direct-does-not-load`, `AC_first-index-call-loads`, and
  `AC_later-index-call-reuses`; preserve
  `P_speaks-mcp-over-stdio`, `P_exits-on-closed-stdin`, and every tool request
  and response shape.

## Work blocks

### Block 1: Lazy MCP embedding lifecycle

- [ ] Done — ticked by build at its commit.
- Depends-on: none.
- Change: revise contract-003 first, then make the MCP service retain embedding
  configuration and initialize its embedder on the first search or overview;
  keep startup knowledge parsing but remove eager index synchronization from
  MCP startup. Cache the initialized embedder, including lexical fallback, for
  the process lifetime. Do not pass `None` through index
  synchronization merely to avoid loading: that would rebuild a semantic index
  as lexical-only.
- Done-check: an MCP process serves direct retrieval before touching an
  unusable index, and repeated search or overview calls initialize one
  embedder instance without downgrading the index.
- Cases:
  - unit: direct read and graph calls do not initialize the embedder or open the
    index (covers contract-003-api-sokf AC_direct-does-not-load).
  - unit: the first search or overview initializes the configured embedder
    exactly once (covers contract-003-api-sokf AC_first-index-call-loads).
  - unit: later search and overview calls reuse the initialized embedder
    (covers contract-003-api-sokf AC_later-index-call-reuses).
  - unit: a failed local-model load selects lexical fallback once without
    repeatedly attempting initialization during the same process.
  - regression: overview never replaces an existing semantic index with a
    lexical-only index as an optimization shortcut.
  - integration: missing or unreadable knowledge fails MCP startup, while an
    unusable index fails the first index-dependent call rather than process
    startup.

### Block 2: Session-scoped MCP client

- [ ] Done — ticked by build at its commit.
- Depends-on: none.
- Change: add a narrow MCP stdio client beside `.pi/extensions/sokf.ts` that
  lazily spawns one `superdev mcp sokf` child per repository, performs only the
  required initialize and `tools/call` exchange, parses fragmented newline-
  delimited JSON-RPC safely, drains bounded stderr, and closes every child on
  `session_shutdown`. Limit one protocol response to 16 MiB and retained stderr
  to 64 KiB. Serialize SOKF calls per repository because the server already
  serializes them; do not build general MCP discovery or concurrency.
- Done-check: one initialized child serves repeated calls in order; a dead or
  cancelled child rejects the active call and the next queued call starts a
  fresh process; session shutdown leaves no child running.
- Cases:
  - unit: initialization and the initialized notification complete before the
    first tool call is sent, and an incompatible protocol fails clearly.
  - unit: chunked lines, multiple lines in one chunk, notifications and matching
    response IDs are parsed without treating server output as arbitrary JSON.
  - unit: oversized or malformed output and process exit reject the active call
    with bounded stdout and stderr diagnostics.
  - unit: aborting the active call terminates the child and permits the next
    queued call to restart; calls not yet sent are not lost.
  - unit: shutdown closes stdin, waits 1 second for the promised clean exit,
    then forces termination.
  - integration: repositories receive distinct children and repeated calls in
    one repository reuse one child.

### Block 3: Pi tool parity over MCP

- [ ] Done — ticked by build at its commit.
- Depends-on: 1, 2.
- Change: route overview, read, search, graph, edit and write through MCP;
  translate MCP text and mutation `structuredContent` into the adapter's
  existing Pi results. Keep ordinary file operations and turn-end
  `superdev validate` on their current paths.
- Done-check: the existing real-Pi smoke suite passes against MCP and records
  one server process across repeated semantic searches.
- Cases:
  - integration: virtual overview, concept and section reads preserve read
    offset, limit and truncation behavior.
  - integration: search filters and graph IDs map to the corresponding MCP
    arguments; MCP protocol errors and tool results marked as errors become Pi
    tool errors rather than successful text.
  - integration: edit and write consume MCP `structuredContent` and preserve Pi
    details, applied-invalid handling, automatic repair and agent-safe
    restrictions.
  - integration: ordinary paths never start MCP and still use fresh built-in
    tools rooted at Pi's current working directory.
  - integration: final validation remains bounded to two repair-feedback turns.

### Block 4: Performance evidence and canonical documentation

- [ ] Done — ticked by build at its commit.
- Depends-on: 3.
- Change: update architecture, software components, development commands and
  the changelog; add model-free fixtures for process and embedder reuse. Keep
  elapsed-time measurements as benchmarks, not pass/fail tests.
- Done-check: documentation names MCP as Pi's persistent transport, validation
  passes, and deterministic counters prove that repeated search excludes a
  second process and model initialization.
- Cases:
  - integration: the first search starts one MCP child and two later searches
    reuse it and one embedder instance.
  - regression: direct SOKF retrieval before any search does not initialize the
    embedder or index.
  - validation: `superdev validate` passes after every affected canonical
    concept is updated.
