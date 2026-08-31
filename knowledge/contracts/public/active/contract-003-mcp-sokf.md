---
type: McpContract
id: contract-003-mcp-sokf
title: MCP Contract
description: The SOKF knowledge served to agents — four read-only tools over stdio, and what each one promises.
lifecycle: active
resource: /crates/lib/superdev-core/src/sokf/mcp.rs
---

# MCP contract: sokf

The SOKF knowledge served to agents: four read-only tools over stdio,
and what each one promises.

## Server

`superdev mcp sokf` serves one stdio client and exits `0` when that client
closes stdin. A missing knowledge or an unusable index directory fails at
startup rather than at every tool call, because a client cannot act on the
latter. The server exposes tools only — no resources, no prompts. It reads the
index at `.superdev/cache/sokf-index/` and syncs it lazily on every tool call,
so there is no watcher and no daemon state to go stale.

## Tools

Four read-only tools. Every hit carries the locator set — knowledge-relative
path, concept id, heading path, line range, snippet, score — so the next call
can read exactly what matched.

- **`sokf_search`** — `query`, optional `limit` (8 by default, clamped to
  1..50), `types` and `tags`. Filters apply before fusion, so a filtered
  concept cannot re-enter through the other ranking. Settled work — a
  `deprecated` concept, or one tagged `done`, `resolved` or `wontfix` — is
  down-ranked after fusion, so finished plans and issues sort below live
  knowledge without leaving the results. Results group by concept,
  strongest concept first.
- **`sokf_read`** — `id` (or knowledge-relative path), optional `heading`: the
  whole concept, or one section named by heading or `a > b` heading path.
  `(root)` names the frontmatter-and-preamble section.
- **`sokf_graph`** — no argument: the knowledge-wide map of *declared* edges,
  grouped by source. With `id`: that concept's single-hop neighbours in both
  directions. Each group caps at 30 lines and then says how many it dropped.
- **`sokf_overview`** — the canonical knowledge name, its concept count, the directory tree
  with each concept's id and description, and every validation finding,
  warnings included, whenever there is one.

## Errors

A tool failure is an MCP error payload, never a process exit: an unknown id
comes back with near-miss candidates, and knowledge that fails validation still
indexes and serves — agents need search most while fixing one. Reading a file
the parser choked on quotes the parse error instead of guessing at near
misses.

## Stability

Unreleased. The tool names, their arguments and their result shapes may change
without notice.
