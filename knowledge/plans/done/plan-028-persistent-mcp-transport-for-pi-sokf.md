---
type: Plan
id: plan-028-persistent-mcp-transport-for-pi-sokf
title: Persistent MCP transport for Pi SOKF
description: SOKF MCP adopts familiar coding-tool semantics and Pi reuses one repository-scoped server so frequent semantic searches load the local embedding model once.
lifecycle: done
---

# Plan: Persistent MCP transport for Pi SOKF

## Goal

SOKF MCP presents project knowledge as a guarded virtual filesystem using the
read, search, edit and write semantics models already know. Tool names remain
SOKF-specific to avoid collisions, while request shapes use paths, line windows
and exact replacements. `sokf_read` handles `sokf:`, `sokf:<id>`,
`sokf:<id>#<heading>` and governed physical knowledge paths;
`sokf_overview` is removed while the API remains unreleased. Virtual
addresses return rendered SOKF concepts or overview; physical paths return the
exact UTF-8 file text, matching familiar read behavior. Semantic search and
graph traversal keep their specialized interfaces.

Pi's SOKF tools reuse one `superdev mcp sokf` process per repository for the
life of a Pi session. The first search or `sokf:` overview read loads the
configured embedder and synchronizes the index; later index-dependent calls
reuse that embedder. Direct concept reads and graph traversal before then do
not load the embedder or open the index. Existing Pi tool names, result shapes,
agent-safe mutation policy, ordinary-file delegation and bounded turn-end
validation remain unchanged.

The existing MCP server and shared `SokfService` remain authoritative. Pi gains
a narrow stdio JSON-RPC client rather than an independent SOKF implementation
or repository daemon. Out of scope: arbitrary filesystem access, shell or
regular-expression emulation, changes to search ranking, a background service
for independent shell commands, and general MCP support for Pi.

## Contract changes

- contract-003-api-sokf: replace the `sokf_read { id, heading }` and
  `sokf_overview` interface with `sokf_read { path, offset?, limit? }`; add
  `P_coding-read` with explicit overview, concept, contained-file, refusal and
  line-window promises; change
  `P_direct-retrieval-skips-index` to exclude the overview address and change
  `P_overview-warning-cap` to bind that address; preserve `sokf_search`,
  `sokf_graph`, `sokf_edit` and `sokf_write`; revise `P_fails-at-startup` so
  missing or unreadable knowledge still fails at startup but index and embedder
  failures occur on the operation that first needs them; add
  `P_lazy-embedding` with explicit first-load, direct-read and reuse promises;
  preserve
  `P_speaks-mcp-over-stdio`, `P_exits-on-closed-stdin`, mutation policy, and
  standard MCP result shapes.

## Work blocks

### Block 1: Coding-tool-shaped MCP reads

- [x] Done — ticked by build at its commit.
- Depends-on: none.
- Change: revise contract-003 first, then change `sokf_read` to accept `path`,
  `offset` and `limit`. Share virtual-address parsing and rendered line
  windowing with the CLI. Route `sokf:` to overview, virtual concept addresses
  to rendered concepts, and contained physical paths to their exact UTF-8
  contents; remove `sokf_overview`. Keep search semantic, exact-replacement
  edit, complete-file write and graph as separate SOKF operations; do not
  emulate grep or sed syntax.
- Done-check: MCP advertises five tools with familiar request semantics, and
  every read target remains confined to the governed knowledge bundle.
- Cases:
  - contract: the generated MCP schema exposes `path`, `offset` and `limit` for
    `sokf_read` and no longer exposes `sokf_overview`.
  - unit: `sokf:` renders overview (covers P_overview-address), while
    `sokf:<id>` renders a concept and `sokf:<id>#<heading>` renders one section
    (covers P_concept-address).
  - unit: relative and absolute physical paths inside `knowledge/` return exact
    file text, including for a concept that does not parse, while either form
    outside that root is refused (covers P_physical-contained and
    P_physical-refused).
  - unit: offset and limit apply after metadata and headings are rendered,
    matching CLI read semantics (covers P_line-window).
  - regression: unknown IDs retain near-miss recovery and malformed addresses
    fail without reading arbitrary files.
  - integration: search, graph, edit and write retain their existing MCP
    arguments and results.

### Block 2: Lazy MCP embedding lifecycle

- [x] Done — ticked by build at its commit.
- Depends-on: 1.
- Change: make the MCP service retain embedding configuration and initialize
  its embedder on the first search or `sokf:` overview read; keep startup
  knowledge parsing but remove eager index synchronization. Cache the
  initialized embedder, including lexical fallback, for the process lifetime.
  Do not pass `None` through index synchronization merely to avoid loading:
  that would rebuild a semantic index as lexical-only.
- Done-check: an MCP process serves direct retrieval before touching an
  unusable index, and repeated index-dependent calls initialize one embedder
  instance without downgrading the index.
- Cases:
  - unit: direct concept reads and graph calls do not initialize the embedder or
    open the index (covers contract-003-api-sokf P_direct-does-not-load).
  - unit: the first search or overview read initializes the configured embedder
    exactly once (covers contract-003-api-sokf P_first-index-call-loads).
  - unit: later search and overview reads reuse the initialized embedder
    (covers contract-003-api-sokf P_later-index-call-reuses).
  - unit: a failed local-model load selects lexical fallback once without
    repeatedly attempting initialization during the same process.
  - regression: overview never replaces an existing semantic index with a
    lexical-only index as an optimization shortcut.
  - integration: missing or unreadable knowledge fails MCP startup, while an
    unusable index fails the first index-dependent call rather than startup.

### Block 3: Session-scoped MCP client

- [x] Done — ticked by build at its commit.
- Depends-on: 1.
- Change: add a narrow MCP stdio client beside `.pi/extensions/sokf.ts` that
  lazily spawns one `superdev mcp sokf` child per repository, performs only the
  required initialize and `tools/call` exchange, parses fragmented newline-
  delimited JSON-RPC safely, drains stderr while retaining at most 64 KiB, and
  closes every child on `session_shutdown`. Limit one protocol response to
  16 MiB. Serialize SOKF calls per repository because the server already
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

### Block 4: Pi tool parity over MCP

- [x] Done — ticked by build at its commit.
- Depends-on: 2, 3.
- Change: route SOKF-aware read, search, graph, edit and write through MCP;
  translate MCP text and mutation `structuredContent` into the adapter's
  existing Pi results. For Pi reads, forward the path but let Pi's existing
  virtual-read wrapper apply offset, limit, byte limits and continuation
  notices exactly once; the MCP line-window arguments remain available to
  other clients. Keep ordinary file operations and turn-end
  `superdev validate` on their current paths.
- Done-check: the existing real-Pi smoke suite passes against MCP and records
  one server process across repeated semantic searches.
- Cases:
  - integration: virtual overview, concept and section reads preserve Pi path,
    offset, limit and truncation behavior without applying a line window
    twice.
  - integration: search filters and graph IDs map to the corresponding MCP
    arguments; MCP protocol errors and tool results marked as errors become Pi
    tool errors rather than successful text.
  - integration: edit and write consume MCP `structuredContent` and preserve Pi
    details, applied-invalid handling, automatic repair and agent-safe
    restrictions.
  - integration: ordinary paths never start MCP and still use fresh built-in
    tools rooted at Pi's current working directory.
  - integration: final validation remains bounded to two repair-feedback turns.

### Block 5: Performance evidence and canonical documentation

- [x] Done — ticked by build at its commit.
- Depends-on: 4.
- Change: update architecture, software components, development commands and
  the changelog; add model-free fixtures for process and embedder reuse. Keep
  elapsed-time measurements as benchmarks, not pass/fail tests.
- Done-check: documentation names the familiar MCP semantics and Pi's
  persistent transport, validation passes, and deterministic counters prove
  that repeated search excludes a second process and embedder initialization.
- Cases:
  - integration: the first search starts one MCP child and two later searches
    reuse it and one embedder instance.
  - regression: direct SOKF retrieval before any search does not initialize the
    embedder or index.
  - validation: `superdev validate` passes after every affected canonical
    concept is updated.
