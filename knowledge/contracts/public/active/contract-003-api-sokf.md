---
type: Contract
id: contract-003-api-sokf
kind: api
title: API contract for sokf over MCP
description: The SOKF knowledge served to agents through semantic search, graph traversal, and an orienting overview over stdio.
lifecycle: active
resource: /crates/lib/superdev-core/src/sokf/mcp.rs
links:
  - rel: references
    to: adr-033-a-contract-defines-its-interface
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
  - rel: references
    to: adr-055-sokf-does-not-intercept-file-tools
---

# API contract: sokf over MCP

The SOKF knowledge served to agents through semantic search, graph traversal,
and an orienting overview over stdio. The server leaves canonical knowledge
unchanged, maintains its search index, and serves no file tool.

The Definition is the server's argument structs and tool methods as the source
declares them. A doc comment on a struct field or a tool method is the
description the client sees and the promise the server keeps. Behaviour
carries what the source cannot say: transport, errors, limits, and retrieval
ranking. [ADR-033][sokf:adr-033-a-contract-defines-its-interface] and
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
govern the contract form. [ADR-055][sokf:adr-055-sokf-does-not-intercept-file-tools]
decided that the server serves no file tool: an agent reads and edits knowledge
with Pi's own `read`, `edit`, and `write` on physical paths, and this server
answers only what a file tool cannot.

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/sokf/mcp.rs#tools -->
```rust
/// Arguments of `sokf_search` and [`SokfService::search`].
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct SearchRequest {
    /// A short question or phrase describing what you need. Preserve known
    /// concept IDs, titles or paths; do not add instructions about searching.
    pub query: String,
    /// Most hits to return; 8 by default.
    pub limit: Option<u32>,
    /// Narrow by known frontmatter type, e.g. `["Contract"]` for a contract
    /// or `["Decision"]` for an architectural decision. Omit for broad discovery.
    pub types: Option<Vec<String>>,
    /// Narrow by existing tags. Do not invent tag names from query keywords.
    pub tags: Option<Vec<String>>,
    /// Narrow by lifecycle when the task asks for it, e.g. `["open"]` for
    /// open work. Omit otherwise: many relevant concepts have no lifecycle.
    pub lifecycle: Option<Vec<String>>,
}

/// Arguments of `sokf_graph`.
#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
struct GraphArgs {
    /// One concept's neighbours; omit for the whole edge map.
    id: Option<String>,
}

/// Arguments of `sokf_overview`: none. The knowledge is the whole subject,
/// so the request carries nothing to narrow it.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct OverviewArgs {}

    /// Find knowledge using lexical terms and semantic similarity when available.
    /// Use a short question or phrase, preserving known IDs, titles or paths.
    /// Narrow with types for a document kind, lifecycle for an explicit work
    /// state, and tags only when their values are known. Omit uncertain filters.
    /// Returns sections grouped by concept with `path:start-end` locators.
    #[tool]
    async fn sokf_search(&self, Parameters(args): Parameters<SearchRequest>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .search(SearchRequest {
                query: args.query,
                limit: args.limit,
                types: args.types,
                tags: args.tags,
                lifecycle: args.lifecycle,
            })
            .map(text)
            .map_err(tool_error)
    }

    /// Show the link graph: the whole edge map, or one concept's neighbours
    /// in both directions. Every concept carries the path to read next.
    #[tool]
    async fn sokf_graph(&self, Parameters(args): Parameters<GraphArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .graph(args.id.as_deref())
            .map(text)
            .map_err(tool_error)
    }

    /// Show the knowledge at a glance: its name, how many concepts it holds,
    /// the tree of them, the index state, and anything wrong with it.
    #[tool]
    async fn sokf_overview(&self, Parameters(_): Parameters<OverviewArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service.overview().map(text).map_err(tool_error)
    }

```
<!-- /sokf:include -->

## Behaviour

### Transport

The server reads the index at `.superdev/cache/sokf-index/`. Search and the
overview sync it lazily; graph traversal parses current knowledge without
opening it. There is no watcher or daemon state.

A symlink-reached Markdown file is eligible when its selected logical path has
no hidden directory component and its final name ends in `.md`; `index.md`
remains a reserved index and every other eligible file remains a concept
candidate or broken file.

- `P_speaks-mcp-over-stdio` [ubiquitous] `superdev mcp sokf` SHALL
  speak the MCP protocol over stdin and stdout, serving one client.
- `P_exits-on-closed-stdin` [event] WHEN the client closes stdin,
  `superdev mcp sokf` SHALL exit `0`.
- `P_fails-at-startup` [event] WHEN the knowledge root is missing or unreadable,
  `superdev mcp sokf` SHALL fail at startup rather than at every tool call.
- `P_index-failure-is-tool-error` [event] WHEN the index or configured embedder
  fails, the first index-dependent call SHALL return the failure as a tool
  error while the MCP process remains available.
- `P_lazy-embedding` [ubiquitous] The MCP server SHALL retain one lazily
  initialized embedder result for its process lifetime.
- `P_active-worktree-root` [ubiquitous] The MCP server SHALL
  treat the canonical active checkout root as its
  repository, including when `.git` is a linked-worktree pointer file.
  - `AC_worktree-local-surfaces` [event] WHEN the server runs in a linked
    worktree, search, graph traversal, overview, and index activity SHALL
    use only that worktree's files and cache.
- `P_direct-does-not-load` [event] WHEN only graph calls have run, the MCP
  server SHALL leave the embedder uninitialized.
- `P_first-index-call-loads` [event] WHEN the first search or overview call
  runs, the MCP server SHALL initialize the
  configured embedder once.
- `P_later-index-call-reuses` [state] WHILE the embedder result is initialized,
  later index-dependent calls SHALL reuse that result.
- `P_serves-three-tools` [ubiquitous] The MCP tool list SHALL expose exactly
  `sokf_search`, `sokf_graph`, and `sokf_overview`.
  - `AC_no-file-tool-served` [ubiquitous] The tool list SHALL NOT expose
    `sokf_read`, `sokf_resolve_source`, `sokf_retrieve`, `sokf_edit`, or
    `sokf_write`.
  - `AC_instructions-name-served-tools` [ubiquitous] The server's
    initialization instructions SHALL name only served
    tools, naming no removed operation and no file read of a `sokf:` address.
  - `AC_instructions-name-locator-roots` [ubiquitous] The initialization
    instructions SHALL distinguish knowledge-relative search locators from
    repository-relative graph paths.
- `P_graph-skips-index` [ubiquitous] `sokf_graph` SHALL parse current knowledge
  without opening or rewriting the search index.
- `P_graph-carries-paths` [ubiquitous] `sokf_graph` SHALL name each concept it
  reports with the repository-relative path of that concept's file, so a
  traversal reaches a source file without a second lookup.
  - `AC_graph-carries-paths` [event] WHEN `sokf_graph` names a concept the
    knowledge holds, in the edge map or in one concept's neighbours, it SHALL
    render that concept's repository-relative path beside its identity.
  - `AC_graph-unresolved-has-no-path` [event] WHEN a declared link names a
    target the knowledge does not hold, `sokf_graph` SHALL name the target
    without a path.
- `P_overview-tool` [ubiquitous] `sokf_overview` SHALL return the knowledge
  name, its concept count, the tree of its concepts, the index state, and its
  capped validation warnings.
  - `AC_overview-content` [ubiquitous] The overview SHALL carry the knowledge
    name, the concept count, one entry per directory holding concepts, and the
    lexical-or-embedded index state.
  - `AC_overview-orients-only` [ubiquitous] The overview SHALL carry no
    rendered concept body.
  - `AC_overview-empty-request` [ubiquitous] `sokf_overview` SHALL accept an
    empty closed request object carrying no property.
  - `AC_overview-rejects-a-property` [event] WHEN a `sokf_overview` request
    carries any property, the server SHALL refuse it.

### Authentication

None. The harness that spawns the server is the caller; there is no
credential to present and no role to distinguish. The server answers questions
about the knowledge without editing it, so there is no knowledge-write
authority to bound.

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

### Limits

Every library hit carries the knowledge-relative path, concept ID, heading
path, line range, snippet, fused score and match kind. The served text includes
locators and match labels, not numeric scores. A direct address selects identity;
other results retain hybrid relevance order. Labels describe retrieval evidence,
not confidence in the document or its claims. A path containing spaces is
written as the whole query or quoted when embedded in prose.

- `P_limit-clamped` [event] WHEN `limit` is outside 1..50,
  `sokf_search` SHALL clamp it into 1..50 rather than refuse.
- `P_limit-default` [event] WHEN `limit` is absent, `sokf_search`
  SHALL default it to 8.
- `P_filters-before-fusion` [ubiquitous] `sokf_search` SHALL apply
  `types`, `tags` and `lifecycle` to every retrieval route, including direct
  addresses, so an excluded concept cannot re-enter through another route.
- `P_exact-address-priority` [event] WHEN a query names an exact concept ID,
  unambiguous numbered shorthand or complete knowledge-relative or physical
  path, `sokf_search` SHALL rank its permitted matches before hybrid relevance.
  - `AC_complete-path` [event] WHEN a complete path is the whole query or
    quoted in prose, direct-path matching SHALL preserve spaces and filename
    punctuation rather than match its fragments.
  - `AC_single-word-id` [conditional] IF an ID is a single word,
    direct-ID priority SHALL apply only when it is the whole query.
  - `AC_ambiguous-shorthand` [conditional] IF a numbered shorthand names
    multiple concepts, `sokf_search` SHALL leave it to hybrid retrieval.
- `P_match-provenance` [ubiquitous] `sokf_search` SHALL label each section
  with its direct identifier/path match or lexical, semantic, or combined
  retrieval provenance.
- `P_query-recovery-visible` [event] WHEN the lenient lexical parser reports
  recovered syntax, `sokf_search` SHALL include its diagnostics alongside the
  recovered results.
  - `AC_query-recovery-bounded` [ubiquitous] Query-recovery notes SHALL contain
    at most three distinct diagnostics.
- `P_settled-down-ranked` [ubiquitous] `sokf_search` SHALL down-rank
  settled work within hybrid relevance — a `deprecated` status or a declared
  lifecycle other than `open` or `active` — without dropping it from results.
- `P_graph-group-cap` [ubiquitous] `sokf_graph` SHALL cap each group at
  30 lines and then say how many it dropped.
- `P_overview-warning-cap` [event] WHEN `sokf_overview` renders validation
  warnings, it SHALL list at most 10 and then say how many more there are.

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
[sokf:adr-055-sokf-does-not-intercept-file-tools]: /knowledge/adrs/active/adr-055-sokf-does-not-intercept-file-tools.md
