---
type: Contract
id: contract-003-api-sokf
kind: api
title: API contract for sokf over MCP
description: The SOKF knowledge served to agents — four read-only tools over stdio as the server declares them, and what each call promises beyond its signature.
lifecycle: active
resource: /crates/lib/superdev-core/src/sokf/mcp.rs
---

# API contract: sokf over MCP

The SOKF knowledge served to agents: four read-only tools over stdio.
The Definition is the server's argument structs and tool methods as the
source declares them; a doc comment on a struct field or a tool method is
the description the client sees and the promise the server keeps.
Behaviour carries what the source cannot say: the transport, the errors,
the limits, and how the tools rank what they return. The decisions behind
the shape are
[ADR-033][sokf:adr-033-a-contract-defines-its-interface] and
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source].

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/sokf/mcp.rs#tools -->
```rust
/// Arguments of `sokf_search`.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
struct SearchArgs {
    /// What to look for, in the caller's own words.
    query: String,
    /// Most hits to return; 8 by default.
    limit: Option<u32>,
    /// Keep only concepts of these frontmatter `type`s.
    types: Option<Vec<String>>,
    /// Keep only concepts carrying one of these tags.
    tags: Option<Vec<String>>,
    /// Keep only concepts whose `lifecycle` is one of these values, e.g.
    /// `["open"]` for live issues and plans.
    lifecycle: Option<Vec<String>>,
}

/// Arguments of `sokf_read`.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
struct ReadArgs {
    /// Concept `id`, or its bundle-relative path.
    id: String,
    /// One section's heading, or the `a > b` heading path; omit for the whole
    /// concept.
    heading: Option<String>,
}

/// Arguments of `sokf_graph`.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
struct GraphArgs {
    /// One concept's neighbours; omit for the whole edge map.
    id: Option<String>,
}

    /// Search the bundle. Returns the best sections, grouped by concept, each
    /// with a `path:start-end` locator to read next.
    #[tool]
    async fn sokf_search(&self, Parameters(args): Parameters<SearchArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .search(SearchRequest {
                query: args.query,
                limit: args.limit,
                types: args.types.unwrap_or_default(),
                tags: args.tags.unwrap_or_default(),
                lifecycle: args.lifecycle.unwrap_or_default(),
            })
            .map(text)
            .map_err(|e| e.to_string())
    }

    /// Read one concept whole, or one of its sections.
    #[tool]
    async fn sokf_read(&self, Parameters(args): Parameters<ReadArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .read(&args.id, args.heading.as_deref())
            .map(text)
            .map_err(|e| e.to_string())
    }

    /// Show the link graph: the whole edge map, or one concept's neighbours
    /// in both directions.
    #[tool]
    async fn sokf_graph(&self, Parameters(args): Parameters<GraphArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .graph(args.id.as_deref())
            .map(text)
            .map_err(|e| e.to_string())
    }

    /// Orient in the bundle: its name, size, directory tree, and anything
    /// validation found wrong.
    #[tool]
    async fn sokf_overview(&self) -> ToolResult {
        let _guard = self.exclusive();
        self.service.overview().map(text).map_err(|e| e.to_string())
    }
```
<!-- /sokf:include -->

## Behaviour

### Transport

The server reads the index at `.superdev/cache/sokf-index/` and syncs
it lazily on every tool call; there is no watcher and no daemon state.

- `P_speaks-mcp-over-stdio` [ubiquitous] `superdev mcp sokf` SHALL
  speak the MCP protocol over stdin and stdout, serving one client.
- `P_exits-on-closed-stdin` [event] WHEN the client closes stdin,
  `superdev mcp sokf` SHALL exit `0`.
- `P_fails-at-startup` [event] WHEN the knowledge is missing or the
  index directory is unusable, `superdev mcp sokf` SHALL fail at
  startup rather than at every tool call.

### Authentication

None. The harness that spawns the server is the caller; there is no
credential to present and no role to distinguish, since every tool is
read-only.

- `P_trusts-stdin` [ubiquitous] The server SHALL trust whatever
  reaches its stdin.

### Errors

A tool failure is an MCP error payload, never a process exit.

- `P_failure-is-error-payload` [event] WHEN a tool call fails, the
  server SHALL return an MCP error payload and keep running.
- `P_unknown-id-near-misses` [event] WHEN a caller names an unknown
  id, the server SHALL answer with near-miss candidates.
- `P_invalid-knowledge-served` [state] WHILE the knowledge fails
  validation, the server SHALL index and serve it.
- `P_parse-error-quoted` [event] WHEN a caller reads a file the parser
  choked on, `sokf_read` SHALL quote the parse error instead of
  guessing at near misses.

### Limits

Every hit carries the locator set — knowledge-relative path, concept
id, heading path, line range, snippet and score — so the next call
reads exactly what matched.

- `P_limit-clamped` [event] WHEN `limit` is outside 1..50,
  `sokf_search` SHALL clamp it into 1..50 rather than refuse.
- `P_limit-default` [event] WHEN `limit` is absent, `sokf_search`
  SHALL default it to 8.
- `P_filters-before-fusion` [ubiquitous] `sokf_search` SHALL apply
  `types`, `tags` and `lifecycle` before fusion, so a filtered concept
  cannot re-enter through the other ranking.
- `P_settled-down-ranked` [ubiquitous] `sokf_search` SHALL down-rank
  settled work — a `deprecated` concept, or one tagged `done`,
  `resolved` or `wontfix` — after fusion, so finished plans and issues
  sort below live knowledge without leaving the results.
- `P_graph-group-cap` [ubiquitous] `sokf_graph` SHALL cap each group at
  30 lines and then say how many it dropped.
- `P_overview-warning-cap` [ubiquitous] `sokf_overview` SHALL list at
  most 10 warnings and then say how many more there are.

### Versioning

Unreleased. A client learns of a change from the tool list the server
serves.

- `P_shape-changes-unannounced` [ubiquitous] A tool, an argument or a
  result shape MAY change in any release without a deprecation path.

### Resources and prompts

None. A client that lists resources or prompts gets an empty set rather
than an error.

- `P_tools-only` [ubiquitous] The server SHALL expose tools only.

## Stability

Unreleased.

- `P_unreleased` [ubiquitous] The tool names, their arguments and their
  result shapes MAY change without notice.

<!-- sokf:links -->
[sokf:adr-033-a-contract-defines-its-interface]: /knowledge/adrs/active/adr-033-a-contract-defines-its-interface.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
