---
type: Contract
id: contract-003-api-sokf
kind: api
title: API contract for sokf over MCP
description: The SOKF knowledge served to agents through semantic search, source resolution, semantic retrieval, graph traversal, and agent-safe mutations over stdio.
lifecycle: active
resource: /crates/lib/superdev-core/src/sokf/mcp.rs
links:
  - rel: references
    to: adr-033-a-contract-defines-its-interface
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
  - rel: references
    to: adr-053-sokf-file-tools-delegate-to-pi
---

# API contract: sokf over MCP

The SOKF knowledge served to agents through semantic search, source
resolution, semantic retrieval, graph traversal, and two agent-safe mutation
tools over stdio. The Definition remains the current source declaration until
BUILD implements and materializes the pending interface.

The Definition is the server's argument structs and tool methods as the source
declares them. A doc comment on a struct field or a tool method is the
description the client sees and the promise the server keeps. Behaviour
carries what the source cannot say: transport, errors, limits, and retrieval
ranking. [ADR-033][sokf:adr-033-a-contract-defines-its-interface] and
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
govern the contract form. [ADR-053][sokf:adr-053-sokf-file-tools-delegate-to-pi]
governs the pending source-routing and mutation boundary.

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/sokf/mutation.rs#tools -->
```rust
/// One exact replacement, evaluated against the original file.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ExactEdit {
    /// Text that must occur exactly once in the original file.
    pub old_text: String,
    /// Text that replaces the matched bytes.
    pub new_text: String,
}

/// The machine request accepted by the edit adapters.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct EditRequest {
    /// Existing concept ID, virtual address, or physical knowledge path.
    pub path: String,
    /// Non-overlapping replacements evaluated against one original file.
    pub edits: Vec<ExactEdit>,
}

/// The machine request accepted by the write adapters.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct WriteRequest {
    /// Existing concept identity, or a physical path for creation.
    pub path: String,
    /// Complete replacement document.
    pub content: String,
}
```
<!-- /sokf:include -->

<!-- sokf:include /crates/lib/superdev-core/src/sokf/mcp.rs#tools -->
```rust
/// Arguments of `sokf_search` and [`SokfService::search`].
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct SearchRequest {
    /// What to look for, in the caller's own words.
    pub query: String,
    /// Most hits to return; 8 by default.
    pub limit: Option<u32>,
    /// Keep only concepts of these frontmatter `type`s.
    pub types: Option<Vec<String>>,
    /// Keep only concepts carrying one of these tags.
    pub tags: Option<Vec<String>>,
    /// Keep only concepts whose `lifecycle` is one of these values, e.g.
    /// `["open"]` for live issues and plans.
    pub lifecycle: Option<Vec<String>>,
}

/// Arguments of `sokf_read`, shaped like a familiar coding read tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct ReadArgs {
    /// `sokf:`, a virtual concept address, or a physical path inside knowledge.
    path: String,
    /// First rendered or physical line to return, starting at 1.
    offset: Option<usize>,
    /// Most lines to return.
    limit: Option<usize>,
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

    /// Read an overview, concept, section, or contained physical knowledge file.
    #[tool]
    async fn sokf_read(&self, Parameters(args): Parameters<ReadArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .read_path(&args.path, args.offset, args.limit)
            .map(text)
            .map_err(tool_error)
    }

    /// Apply atomic exact replacements to one existing concept. Identity and
    /// verification are protected; automatic repair and validation follow.
    #[tool]
    async fn sokf_edit(&self, Parameters(args): Parameters<EditRequest>) -> ToolResult {
        let _guard = self.exclusive();
        match self.service.edit(args, MutationPolicy::AgentSafe) {
            Ok(result) => mutation_result(result),
            Err(error) => Err(tool_error(error)),
        }
    }

    /// Replace one complete concept, or create one at a physical `.md` path.
    /// Identity and verification are protected; repair and validation follow.
    #[tool]
    async fn sokf_write(&self, Parameters(args): Parameters<WriteRequest>) -> ToolResult {
        let _guard = self.exclusive();
        match self.service.write(args, MutationPolicy::AgentSafe) {
            Ok(result) => mutation_result(result),
            Err(error) => Err(tool_error(error)),
        }
    }

    /// Show the link graph: the whole edge map, or one concept's neighbours
    /// in both directions.
    #[tool]
    async fn sokf_graph(&self, Parameters(args): Parameters<GraphArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .graph(args.id.as_deref())
            .map(text)
            .map_err(tool_error)
    }

```
<!-- /sokf:include -->

## Behaviour

### Transport

The server reads the index at `.superdev/cache/sokf-index/`. Search and
semantic overview retrieval sync it lazily. Source resolution, direct semantic
retrieval, and graph traversal parse current knowledge without opening the
index. There is no watcher or daemon state.

The routed schema notation below names JSON objects. `?` marks an optional
field. Every object is closed to additional properties. Every path is a
repository-relative slash-separated string. A symlink-reached Markdown file is
eligible when its selected logical path has no hidden directory component and
its final name ends in `.md`; `index.md` remains a reserved index and every
other eligible file remains a concept candidate or broken file. A `GeneratedRegion` is
`{ startLine: integer >= 1, endLine: integer >= startLine,
authoritativePath: string, authoritativeRegion?: string }`. A `Finding` is
`{ severity: "error" | "warning", path?: string, location?: { line: integer
>= 1, column?: integer >= 1 }, message: string, action: string }`.

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
  PENDING(issue-077) treat the canonical active checkout root as its
  repository, including when `.git` is a linked-worktree pointer file.
  - `AC_worktree-local-surfaces` [event] WHEN the server runs in a linked
    worktree, source resolution, semantic retrieval, search, graph traversal,
    index activity, mutation, repair, refiling, and validation SHALL
    PENDING(issue-083) use only that worktree's files and cache.
  - `AC_worktree-other-checkout-refused` [event] WHEN a routed path enters
    another checkout, including the main checkout, the server SHALL
    PENDING(issue-077) reject it as outside the active worktree.
- `P_direct-does-not-load` [event] WHEN only source resolution, direct semantic
  retrieval, or graph calls have run, the MCP server SHALL PENDING(issue-077)
  leave the embedder uninitialized.
- `P_first-index-call-loads` [event] WHEN the first search or semantic overview
  retrieval runs, the MCP server SHALL PENDING(issue-077) initialize the
  configured embedder once.
- `P_later-index-call-reuses` [state] WHILE the embedder result is initialized,
  later index-dependent calls SHALL reuse that result.
- `P_routed-schemas` [ubiquitous] The routed MCP operations SHALL
  PENDING(issue-082) expose only the closed request and success-result
  schemas specified below.
  - `AC_resolve-schema` [ubiquitous] `sokf_resolve_source` SHALL
    PENDING(issue-077) accept `{ path: string }` and return structured
    content `{ ingressPath: string, canonicalPath: string, exists: boolean,
    generatedRegions: GeneratedRegion[] }` together with exactly one
    `{ type: "text", text: string }` content item carrying the pretty-printed
    JSON of that structured content and nothing else.
  - `AC_retrieve-schema` [ubiquitous] `sokf_retrieve` SHALL
    PENDING(issue-077) accept `{ path: string, offset?: integer >= 1,
    limit?: integer >= 1 }` and return one text content item with no structured
    content.
  - `AC_edit-schema` [ubiquitous] `sokf_edit` SHALL
    PENDING(issue-082) accept `{ ingressPath: string,
    expectedCanonicalPath: string, expectedContent: string, content: string }`.
  - `AC_write-schema` [ubiquitous] `sokf_write` SHALL
    PENDING(issue-082) accept `{ ingressPath: string,
    expectedCanonicalPath: string, content: string }`.
  - `AC_mutation-result-schema` [ubiquitous] A successful `sokf_edit` or
    `sokf_write` SHALL PENDING(issue-082) return structured content
    `{ applied: true, validation: "valid" | "invalid" | "unknown",
    resolvedPath: string, finalPath: string, changedPaths: string[], findings:
    Finding[], diagnosticsTruncated: boolean }`.
- `P_source-resolution` [ubiquitous] `sokf_resolve_source` SHALL
  PENDING(issue-077) return one logical `knowledge/` ingress, canonical
  repository-relative target, existence flag, and generated-region metadata
  without semantic content.
  - `AC_source-identity` [event] WHEN the resolver receives an unqualified
    `sokf:<id>` or physical knowledge path, it SHALL PENDING(issue-077)
    resolve the corresponding source and refuse missing identities, overview
    addresses, section-qualified addresses, directories, and direct paths
    outside `knowledge/`.
  - `AC_source-contained` [event] WHEN resolution encounters an existing target
    or nearest existing ancestor, the resolver SHALL PENDING(issue-077)
    accept only a canonical target inside the repository, including a contained
    symlink target outside `knowledge/` and a normalized missing suffix.
  - `AC_source-section-refused` [event] WHEN source resolution receives an
    overview or section-qualified address, the resolver SHALL
    PENDING(issue-077) return concise guidance to semantic retrieval.
- `P_semantic-retrieve` [ubiquitous] `sokf_retrieve` SHALL
  PENDING(issue-077) retain semantic overview, rendered-concept,
  section, and one-indexed line-window behavior.
  - `AC_semantic-addresses` [event] WHEN `sokf_retrieve` receives `sokf:`,
    `sokf:<id>`, or `sokf:<id>#<heading>`, it SHALL PENDING(issue-077)
    return the corresponding semantic rendering.
  - `AC_semantic-physical-refused` [event] WHEN `sokf_retrieve` receives a
    physical path, it SHALL PENDING(issue-077) refuse the path.
- `P_no-read-alias` [ubiquitous] The MCP tool list SHALL NOT
  PENDING(issue-077) expose `sokf_read`.
  - `AC_instructions-name-served-tools` [ubiquitous] The server's
    initialization instructions SHALL PENDING(issue-077) name only served
    tools, excluding `sokf_read` and any directed file read of the `sokf:`
    overview address.
- `P_direct-retrieval-skips-index` [event] WHEN source resolution or direct
  semantic retrieval runs, the MCP server SHALL PENDING(issue-077)
  answer without opening or rewriting the search index.
- `P_graph-skips-index` [ubiquitous] `sokf_graph` SHALL parse current knowledge
  without opening or rewriting the search index.

### Authentication

None. The harness that spawns the server is the caller; there is no
credential to present and no role to distinguish. Mutation authority is
bounded by the mandatory agent-safe policy rather than caller identity.

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
- `P_mutation-precondition-error` [event] WHEN a routed mutation fails source
  resolution, compare-and-swap, policy, generated ownership, or cancellation
  before dispatch, the server SHALL PENDING(issue-082) return an MCP
  error result without changing any target.
- `P_applied-invalid-result` [state] WHILE an applied mutation leaves the
  knowledge invalid or its validation unknown, the server SHALL return a
  successful structured result with `applied: true` rather than an error.
- `P_parse-error-quoted` [event] WHEN a semantic concept address resolves to a
  file the parser rejected, `sokf_retrieve` SHALL PENDING(issue-077)
  quote the parse error instead of guessing at near misses.

### Mutations

- `P_mutation-agent-safe` [ubiquitous] `sokf_edit` and `sokf_write` SHALL use
  `MutationPolicy::AgentSafe` with no caller override.
- `P_edit-compare-and-swap` [ubiquitous] `sokf_edit` SHALL
  PENDING(issue-082) accept only ingress path, expected canonical path,
  exact expected source, and complete replacement source as its private routed
  transport.
  - `AC_edit-stale-source-refused` [event] WHEN the canonical target drifts or
    current source differs byte-for-byte from expected source, `sokf_edit`
    SHALL PENDING(issue-082) reject the request before persistence.
- `P_write-whole-path-create` [ubiquitous] `sokf_write` SHALL
  PENDING(issue-082) replace a complete existing concept or create one
  from a physical `.md` ingress while preserving recursive parent creation.
  - `AC_write-routed-shape` [event] WHEN the Pi adapter invokes `sokf_write`,
    the tool SHALL PENDING(issue-082) accept only ingress path, expected
    canonical path, and complete content as its private routed transport.
- `P_mutation-contained` [ubiquitous] Routed mutation tools SHALL
  PENDING(issue-082) require a logical ingress under `knowledge/`,
  re-resolve it at dispatch, and write only to its repository-contained
  canonical target.
  - `AC_mutation-target-drift` [event] WHEN dispatch-time resolution differs
    from the expected canonical target, the mutation tool SHALL
    PENDING(issue-082) reject the request before persistence.
- `P_mutation-repair-validation` [event] WHEN a mutation is applied, the tool
  SHALL run automatic repair, refiling, and validation while retaining the
  tool-call lock.
- `P_mutation-outcome-boundary` [ubiquitous] Routed mutation tools SHALL
  PENDING(issue-082) treat authoritative `applied: true` as successful
  persistence and later repair, refiling, validation, lifecycle, or
  cancellation problems as findings.
  - `AC_pre-persistence-failure` [event] WHEN a failure occurs before requested
    persistence or an authoritative response establishes no apply, the tool
    SHALL PENDING(issue-082) return an error without applied state.
  - `AC_acknowledged-request-retained` [event] WHEN requested persistence is
    acknowledged with `applied: true`, the tool SHALL PENDING(issue-082)
    retain the requested bytes without rollback.
  - `AC_successful-repairs-retained` [event] WHEN repair or refiling persists
    before a later finding, the tool SHALL PENDING(issue-082) retain each
    successful change.
  - `AC_indeterminate-outcome-guidance` [event] WHEN local transport fails
    without an authoritative response after bytes may have changed, the tool
    SHALL PENDING(issue-082) direct target inspection and `superdev
    validate` without durable outcome reconciliation.
  - `AC_post-persistence-findings` [event] WHEN repair or validation reports a
    problem after apply, the tool SHALL PENDING(issue-082) return
    `applied: true` with an actionable finding.
- `P_mutation-result-shape` [ubiquitous] A routed mutation result SHALL
  PENDING(issue-082) carry literal applied state, validation state,
  resolved and final paths, unique repository-relative changed paths in lexical
  order, actionable findings, and an explicit diagnostic-truncation flag as
  specified by `AC_mutation-result-schema`.
- `P_mutation-diagnostics-bounded` [ubiquitous] Routed mutation results SHALL
  PENDING(issue-082) omit patches and mutation envelopes and stop
  diagnostics at 200 lines or 8 KiB.

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
- `P_overview-warning-cap` [event] WHEN `sokf_retrieve` receives the `sokf:`
  overview address, it SHALL PENDING(issue-077) list at most 10 warnings
  and then say how many more there are.

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
[sokf:adr-053-sokf-file-tools-delegate-to-pi]: /knowledge/adrs/active/adr-053-sokf-file-tools-delegate-to-pi.md
