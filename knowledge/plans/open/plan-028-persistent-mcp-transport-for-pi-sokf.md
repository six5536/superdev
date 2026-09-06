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
life of a Pi session. The first semantic search may load the local embedding
model, while later searches reuse it. Direct reads and graph traversal before
the first search do not load the model or open the search index. Existing Pi
tool names, result shapes, agent-safe mutation policy, ordinary-file
delegation and bounded turn-end validation remain unchanged.

The existing MCP server and shared `SokfService` remain authoritative. Pi gains
a small stdio JSON-RPC client rather than an independent SOKF implementation or
a repository daemon. Out of scope: changing lexical or semantic ranking,
automatically running a background service for independent shell commands, and
adding general MCP support to Pi.

## Contract changes

- contract-003-api-sokf: revise `P_fails-at-startup` so missing knowledge still
  fails at startup but index and embedder failures occur on the operation that
  first needs them; add `P_lazy-embedding` with `AC_direct-does-not-load`,
  `AC_overview-does-not-load`, `AC_first-search-loads`, and
  `AC_later-search-reuses`; preserve `P_speaks-mcp-over-stdio`,
  `P_exits-on-closed-stdin`, and every tool request and response shape.

## Work blocks

### Block 1: Lazy MCP embedding lifecycle

- [ ] Done — ticked by build at its commit.
- Depends-on: none.
- Change: make the MCP service retain embedding configuration and initialize
  its embedder on the first semantic search; remove eager index synchronization
  from MCP startup and remove embedding initialization from overview.
- Done-check: an MCP process can serve direct retrieval before touching an
  unusable index, and two searches initialize one embedder instance.
- Cases:
  - unit: direct read and graph calls do not initialize the embedder or open the
    index (covers contract-003-api-sokf AC_direct-does-not-load).
  - unit: overview synchronizes the index without initializing the embedder
    (covers contract-003-api-sokf AC_overview-does-not-load).
  - unit: the first search initializes the configured embedder exactly once
    (covers contract-003-api-sokf AC_first-search-loads).
  - unit: a later search reuses the initialized embedder
    (covers contract-003-api-sokf AC_later-search-reuses).
  - integration: missing knowledge fails MCP startup, while an unusable index
    fails the first index-dependent call rather than process startup.

### Block 2: Session-scoped MCP client

- [ ] Done — ticked by build at its commit.
- Depends-on: 1.
- Change: add a small MCP stdio client beside `.pi/extensions/sokf.ts` that
  lazily spawns one `superdev mcp sokf` child per repository, performs the MCP
  initialization handshake, correlates JSON-RPC responses, captures bounded
  stderr, and closes every child on `session_shutdown`.
- Done-check: concurrent requests receive their own responses; a dead child
  rejects pending calls and the next call starts a fresh process; session
  shutdown leaves no child running.
- Cases:
  - unit: initialization completes before the first tool call is sent.
  - unit: out-of-order response IDs resolve the matching pending requests.
  - unit: malformed output and process exit reject every pending request with
    bounded diagnostic text.
  - unit: aborting a request terminates the child, rejects that request, and
    permits a clean restart.
  - integration: repositories receive distinct children and repeated calls in
    one repository reuse one child.

### Block 3: Pi tool parity over MCP

- [ ] Done — ticked by build at its commit.
- Depends-on: 2.
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
    arguments.
  - integration: edit and write preserve Pi details, applied-invalid handling,
    automatic repair and agent-safe restrictions.
  - integration: ordinary paths never start MCP and still use fresh built-in
    tools rooted at Pi's current working directory.
  - integration: final validation remains bounded to two repair-feedback turns.

### Block 4: Performance evidence and canonical documentation

- [ ] Done — ticked by build at its commit.
- Depends-on: 3.
- Change: update architecture, software components, contract-003, development
  commands and the changelog; add a model-free timing fixture for process reuse.
- Done-check: documentation names MCP as Pi's persistent transport, validation
  passes, and measured repeated search excludes a second model startup.
- Cases:
  - behavioral: the first search starts one MCP child and two later searches
    reuse it without another embedder initialization.
  - regression: direct SOKF retrieval before any search does not pay embedding
    startup.
  - validation: `superdev validate` passes after every affected canonical
    concept is updated.
