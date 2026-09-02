---
type: McpContract
id: contract-003-mcp-sokf
title: MCP Contract
description: The SOKF knowledge served to agents — four read-only tools over stdio, each argument and result defined, and what each one promises.
lifecycle: active
resource: /crates/lib/superdev-core/src/sokf/mcp.rs
---

# MCP contract: sokf

The SOKF knowledge served to agents: four read-only tools over stdio,
defined. A caller reproduces every call from the block below alone. The
decisions behind the shape are
[ADR-033][sokf:adr-033-a-contract-defines-its-interface] and
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation],
and the test that binds this contract to the served tool list sits beside
the server.

## Server

`superdev mcp sokf` serves one stdio client and MUST exit `0` when that
client closes stdin. A missing knowledge or an unusable index directory
MUST fail at startup rather than at every tool call. The server MUST
expose tools only. It reads the index at `.superdev/cache/sokf-index/`
and syncs it lazily on every tool call; there is no watcher and no daemon
state.

## Tools

```json
{
  "sokf_search": {
    "about": "Search the knowledge. Returns the best sections, grouped by concept, each with a locator to read next.",
    "arguments": {
      "query": { "type": "string", "required": true,
                 "about": "What to look for, in the caller's own words." },
      "limit": { "type": "integer", "required": false,
                 "about": "Most hits to return; 8 by default, clamped to 1..50." },
      "types": { "type": "array", "required": false,
                 "about": "Keep only concepts of these frontmatter types." },
      "tags": { "type": "array", "required": false,
                "about": "Keep only concepts carrying one of these tags." },
      "lifecycle": { "type": "array", "required": false,
                     "about": "Keep only concepts whose lifecycle is one of these values." }
    },
    "result": "The matching sections grouped by concept, strongest concept first, each hit carrying the locator set: knowledge-relative path, concept id, heading path, line range, snippet and score."
  },
  "sokf_read": {
    "about": "Read one concept whole, or one of its sections.",
    "arguments": {
      "id": { "type": "string", "required": true,
              "about": "Concept id, or its knowledge-relative path." },
      "heading": { "type": "string", "required": false,
                   "about": "One section's heading, or an `a > b` heading path; omit for the whole concept." }
    },
    "result": "The concept, or the named section. `(root)` names the frontmatter-and-preamble section."
  },
  "sokf_graph": {
    "about": "The knowledge's declared edges, whole or around one concept.",
    "arguments": {
      "id": { "type": "string", "required": false,
              "about": "One concept's neighbours; omit for the whole edge map." }
    },
    "result": "Without `id`, every declared edge grouped by source. With `id`, that concept's single-hop neighbours in both directions. Each group caps at 30 lines and then says how many it dropped."
  },
  "sokf_overview": {
    "about": "Orient in the knowledge: its name, size, tree, and anything validation found wrong.",
    "arguments": {},
    "result": "The canonical knowledge name, its concept count, the directory tree with each concept's id and description, and every validation finding, warnings included, whenever there is one."
  }
}
```

Every hit carries the locator set, so the next call reads exactly what
matched. `sokf_search` MUST apply `types`, `tags` and `lifecycle` before
fusion, so a filtered concept cannot re-enter through the other ranking.
Settled work — a `deprecated` concept, or one tagged `done`, `resolved` or
`wontfix` — MUST be down-ranked after fusion, so finished plans and issues
sort below live knowledge without leaving the results. `limit` MUST be
clamped to 1..50 rather than refused.

## Resources and prompts

None. The server MUST expose tools only, so a client that lists resources
or prompts gets an empty set rather than an error.

## Errors

A tool failure MUST be an MCP error payload, never a process exit: an
unknown id MUST come back with near-miss candidates, and knowledge that
fails validation MUST still index and serve. Reading a file the parser
choked on MUST quote the parse error instead of guessing at near misses.

## Stability

Unreleased. The tool names, their arguments and their result shapes MAY
change without notice.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/deprecated/adr-036-a-contract-is-bound-to-its-implementation.md
