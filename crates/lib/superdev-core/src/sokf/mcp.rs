//! The SOKF MCP server serves search, graph traversal and overview.
//!
//! Every call reads current knowledge from disk. Search and overview sync the
//! index incrementally; graph traversal does not initialize or open it.
//! The shared service also retains read and mutation operations for CLI callers.

use std::path::{Path, PathBuf};

use render::{render_concept, render_edges, render_hits, render_neighbours, render_overview};

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router};
use serde::Deserialize;

use super::bundle::{Bundle, load_bundle};
use super::concept::Concept;
use super::embed::Embedder;
use super::graph::Graph;
use super::index::{Index, IndexDir, SearchOpts, SyncStats};
use super::mutation::{self, EditRequest, MutationPolicy, MutationResult, WriteRequest};
use crate::error::Error;

/// Most lines rendered per group before the tail is summarised.
const GROUP_CAP: usize = 30;

/// Most warnings listed by the knowledge overview.
const WARNING_CAP: usize = 10;

/// Most hits one search may ask for. Retrieval widens the caller's limit
/// before tantivy pre-allocates against it, so an unbounded `limit` is an
/// allocation failure — and an abort under the release profile.
const MAX_LIMIT: usize = 50;

/// What a tool returns: text, or a message the client shows as a tool error.
type ToolResult = std::result::Result<CallToolResult, String>;

/// Harness-independent knowledge operations shared by CLI and MCP callers.
pub struct SokfService {
    bundle_dir: PathBuf,
    repo_root: PathBuf,
    index_dir: IndexDir,
    /// The embedder the index was built with. MCP initializes this slot on its
    /// first index-dependent call; one-shot CLI services carry a ready value.
    embedder: EmbedderSlot,
}

type EmbedderInitializer =
    dyn Fn() -> crate::error::Result<Option<Box<dyn Embedder>>> + Send + Sync;

enum EmbedderSlot {
    Ready(Option<Box<dyn Embedder>>),
    Lazy {
        initialize: Box<EmbedderInitializer>,
        value: std::sync::OnceLock<std::result::Result<Option<Box<dyn Embedder>>, String>>,
    },
}

impl EmbedderSlot {
    fn get(&self) -> crate::error::Result<Option<&dyn Embedder>> {
        match self {
            Self::Ready(value) => Ok(value.as_deref()),
            Self::Lazy { initialize, value } => value
                .get_or_init(|| initialize().map_err(|error| error.to_string()))
                .as_ref()
                .map(|value| value.as_deref())
                .map_err(|message| Error::Embedding {
                    message: message.clone(),
                }),
        }
    }

    fn debug_value(&self) -> Option<String> {
        match self {
            Self::Ready(value) => value.as_ref().map(|embedder| embedder.model_id()),
            Self::Lazy { value, .. } => value
                .get()
                .and_then(|result| result.as_ref().ok())
                .and_then(|value| value.as_ref().map(|embedder| embedder.model_id())),
        }
    }
}

impl std::fmt::Debug for SokfService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SokfService")
            .field("bundle_dir", &self.bundle_dir)
            .field("repo_root", &self.repo_root)
            .field("index_dir", &self.index_dir.0)
            .field("embedder", &self.embedder.debug_value())
            .finish()
    }
}

/// An MCP adapter over [`SokfService`].
pub struct SokfServer {
    service: SokfService,
    /// One tool call at a time. rmcp runs each request as its own task, and a
    /// call holds an [`Index`] open across its whole body while another call's
    /// sync may delete and rebuild the index directory underneath it — and two
    /// syncs would contend on tantivy's writer lock regardless. The bodies are
    /// blocking work by design, so a plain mutex is the whole answer.
    tool_lock: std::sync::Mutex<()>,
    tool_router: ToolRouter<SokfServer>,
}

impl std::fmt::Debug for SokfServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SokfServer")
            .field("service", &self.service)
            .finish()
    }
}

// The tools are the MCP contract's definition (contract-003): the argument
// structs and the `#[tool]` methods sit in `tools` regions, and the contract
// includes them.
// sokf:begin tools
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
// sokf:end tools

impl SokfService {
    /// Serve `bundle_dir`, resolving `/`-rooted links against `repo_root` and
    /// keeping the search index in `index_dir`.
    #[must_use]
    pub fn new(
        bundle_dir: PathBuf,
        repo_root: PathBuf,
        index_dir: IndexDir,
        embedder: Option<Box<dyn Embedder>>,
    ) -> Self {
        Self {
            bundle_dir,
            repo_root,
            index_dir,
            embedder: EmbedderSlot::Ready(embedder),
        }
    }

    /// Construct a service whose embedder is initialized on first index use.
    #[must_use]
    pub fn new_lazy(
        bundle_dir: PathBuf,
        repo_root: PathBuf,
        index_dir: IndexDir,
        initialize: impl Fn() -> crate::error::Result<Option<Box<dyn Embedder>>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            bundle_dir,
            repo_root,
            index_dir,
            embedder: EmbedderSlot::Lazy {
                initialize: Box::new(initialize),
                value: std::sync::OnceLock::new(),
            },
        }
    }

    /// Search the knowledge and render the best matching sections.
    pub fn search(&self, request: SearchRequest) -> crate::error::Result<String> {
        let embedder = self.embedder.get()?;
        let (bundle, index, stats) = self.sync(embedder)?;
        let opts = SearchOpts {
            limit: hit_limit(request.limit),
            kinds: request.types.unwrap_or_default(),
            tags: request.tags.unwrap_or_default(),
            lifecycle: request.lifecycle.unwrap_or_default(),
        };
        let hits = index.search(&request.query, embedder, &opts)?;
        Ok(render_hits(
            &bundle,
            &request.query,
            &hits,
            stats.lexical_only,
        ))
    }

    /// Read and render one concept, optionally restricted to one heading.
    pub fn read(&self, id: &str, heading: Option<&str>) -> crate::error::Result<String> {
        let bundle = load_bundle(&self.bundle_dir)?;
        let graph = Graph::build(&bundle);
        let identity = resolve(&graph, id)
            .map_err(|message| broken_file_error(&bundle, id).unwrap_or(message))
            .map_err(|message| Error::Sokf { message })?;
        let concept = concept_of(&bundle, &identity).ok_or_else(|| Error::Sokf {
            message: format!("no concept for `{identity}`"),
        })?;
        render_concept(concept, &identity, heading).map_err(|message| Error::Sokf { message })
    }

    /// Apply exact replacements to an existing concept, then repair and validate.
    pub fn edit(
        &self,
        request: EditRequest,
        policy: MutationPolicy,
    ) -> crate::error::Result<MutationResult> {
        mutation::edit(&self.bundle_dir, &self.repo_root, request, policy)
    }

    /// Replace an existing concept, or create one at a physical path, then
    /// repair and validate.
    pub fn write(
        &self,
        request: WriteRequest,
        policy: MutationPolicy,
    ) -> crate::error::Result<MutationResult> {
        mutation::write(&self.bundle_dir, &self.repo_root, request, policy)
    }

    /// Render the whole edge map or one concept's neighbours.
    pub fn graph(&self, id: Option<&str>) -> crate::error::Result<String> {
        let bundle = load_bundle(&self.bundle_dir)?;
        let graph = Graph::build(&bundle);
        let Some(id) = id else {
            return Ok(render_edges(&graph.edge_map(), &bundle, &self.repo_root));
        };
        let identity = resolve(&graph, id).map_err(|message| Error::Sokf { message })?;
        let hops = graph.neighbours(&identity).map_err(|unknown| Error::Sokf {
            message: format!("unknown id `{}`", unknown.asked),
        })?;
        Ok(render_neighbours(
            &bundle,
            &identity,
            &hops,
            &self.repo_root,
        ))
    }

    /// Render the knowledge name, size, tree, index state, and findings.
    pub fn overview(&self) -> crate::error::Result<String> {
        let embedder = self.embedder.get()?;
        let (bundle, _, stats) = self.sync(embedder)?;
        Ok(render_overview(&bundle, &stats, &self.repo_root))
    }

    /// Reload the bundle and bring the index up to date.
    fn sync(
        &self,
        embedder: Option<&dyn Embedder>,
    ) -> crate::error::Result<(Bundle, Index, SyncStats)> {
        let bundle = load_bundle(&self.bundle_dir)?;
        let (index, stats) = Index::open_and_sync(&self.index_dir, &bundle, embedder)?;
        Ok((bundle, index, stats))
    }
}

#[tool_router(router = tool_router)]
impl SokfServer {
    /// Serve `bundle_dir`, resolving `/`-rooted links against `repo_root` and
    /// keeping the search index in `index_dir`.
    ///
    /// `embedder` must be the one the index was built with; `None` leaves
    /// search lexical.
    #[must_use]
    pub fn new(
        bundle_dir: PathBuf,
        repo_root: PathBuf,
        index_dir: IndexDir,
        embedder: Option<Box<dyn Embedder>>,
    ) -> SokfServer {
        Self::with_service(SokfService::new(bundle_dir, repo_root, index_dir, embedder))
    }

    /// Construct a server that initializes its embedder on first index use.
    #[must_use]
    pub fn new_lazy(
        bundle_dir: PathBuf,
        repo_root: PathBuf,
        index_dir: IndexDir,
        initialize: impl Fn() -> crate::error::Result<Option<Box<dyn Embedder>>> + Send + Sync + 'static,
    ) -> SokfServer {
        Self::with_service(SokfService::new_lazy(
            bundle_dir, repo_root, index_dir, initialize,
        ))
    }

    fn with_service(service: SokfService) -> SokfServer {
        SokfServer {
            service,
            tool_lock: std::sync::Mutex::new(()),
            tool_router: SokfServer::tool_router(),
        }
    }
    // sokf:begin tools

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

    // sokf:end tools

    /// Serve MCP over stdio until the client disconnects.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mcp`] when the connection cannot be initialised or
    /// ends in failure.
    // Needs a real stdio peer; covered by the CLI smoke run.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub async fn serve_stdio(self) -> crate::error::Result<()> {
        use rmcp::ServiceExt as _;

        let running = self
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| Error::Mcp {
                message: e.to_string(),
            })?;
        running.waiting().await.map_err(|e| Error::Mcp {
            message: e.to_string(),
        })?;
        Ok(())
    }

    /// Take the tool lock for the rest of the call, so no two calls touch the
    /// index at once. Poisoning carries no bad state: the lock guards nothing.
    fn exclusive(&self) -> std::sync::MutexGuard<'_, ()> {
        self.tool_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SokfServer {
    fn get_info(&self) -> ServerInfo {
        let knowledge = display_path(&self.service.repo_root, &self.service.bundle_dir);
        let knowledge = if knowledge.is_empty() {
            "."
        } else {
            &knowledge
        };
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            format!(
                "Access to this repository's canonical SOKF knowledge. Use sokf_overview to see \
                 the knowledge at a glance, sokf_search to find project knowledge, and sokf_graph \
                 to follow links. Search locators are relative to `{knowledge}/`; graph paths \
                 are repository-relative. Resolve them to physical paths for your own file tools."
            ),
        )
    }
}

/// Apply a coding-tool line window after SOKF content is rendered.
pub fn line_window(
    text: &str,
    offset: Option<usize>,
    limit: Option<usize>,
) -> crate::error::Result<String> {
    let lines: Vec<&str> = text.lines().collect();
    let start = offset.unwrap_or(1).saturating_sub(1);
    if lines.is_empty() && start == 0 {
        return Ok(String::new());
    }
    if start >= lines.len() {
        return sokf_error(format!(
            "offset {} is beyond end of concept ({} rendered lines total)",
            offset.unwrap_or(1),
            lines.len()
        ));
    }
    let selected = lines.into_iter().skip(start);
    Ok(match limit {
        Some(limit) => selected.take(limit).collect::<Vec<_>>().join("\n"),
        None => selected.collect::<Vec<_>>().join("\n"),
    })
}

fn sokf_error<T>(message: impl Into<String>) -> crate::error::Result<T> {
    Err(Error::Sokf {
        message: message.into(),
    })
}

/// The caller's `limit`, bounded to something the retrieval stage can
/// allocate for. Zero would return nothing at all, which no caller means.
fn hit_limit(requested: Option<u32>) -> usize {
    requested.map_or(SearchOpts::default().limit, |n| {
        (n as usize).clamp(1, MAX_LIMIT)
    })
}

/// One text block, the only shape these tools return.
fn text(body: String) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(body)])
}

/// A repository-relative, slash-separated path for a caller to act on.
fn display_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Preserve the MCP tool's established domain-error text while retaining
/// prefixes that identify infrastructure failures.
fn tool_error(error: Error) -> String {
    match error {
        Error::Sokf { message } => message,
        other => other.to_string(),
    }
}

/// Resolve a caller's identity to a concept identity, naming near misses when
/// it names nothing.
fn resolve(graph: &Graph, id: &str) -> std::result::Result<String, String> {
    if let Some(identity) = graph.resolve(id) {
        return Ok(identity);
    }
    // `neighbours` is the only source of candidates, and it must fail here:
    // the target did not resolve.
    let candidates = match graph.neighbours(id) {
        Err(unknown) if !unknown.candidates.is_empty() => {
            format!(" — did you mean {}?", unknown.candidates.join(", "))
        }
        _ => String::new(),
    };
    Err(format!("unknown id `{id}`{candidates}"))
}

/// The parse failure for a path that names a file the bundle could not read.
///
/// Such a file has no id and no path entry, so `resolve` calls it unknown and
/// offers near misses. The caller asked for a file that is right there; say why
/// it cannot be served instead.
fn broken_file_error(bundle: &Bundle, asked: &str) -> Option<String> {
    let asked = asked.trim_start_matches('/').replace('\\', "/");
    bundle
        .broken
        .iter()
        .find(|e| asked == e.path || asked.ends_with(&format!("/{}", e.path)))
        .map(|e| format!("`{}` does not parse: {}", e.path, e.message))
}

/// The concept behind an identity: its `id`, or its path when it has none.
fn concept_of<'a>(bundle: &'a Bundle, identity: &str) -> Option<&'a Concept> {
    bundle
        .concepts
        .iter()
        .find(|c| c.id.as_deref() == Some(identity) || (c.id.is_none() && c.path == identity))
}

mod render;
#[cfg(test)]
mod tests;
