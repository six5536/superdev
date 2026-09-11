//! mcp.rs — the SOKF MCP server: source resolution, semantic retrieval, and
//! safe mutation over one bundle.
//!
//! Every call reads current knowledge from disk. Search and the semantic
//! overview sync the index incrementally; source resolution, direct semantic
//! retrieval, graph traversal, and mutations do not initialize or open it.

use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router};
use serde::{Deserialize, Serialize};

use super::bundle::{Bundle, load_bundle};
use super::concept::{Concept, Status};
use super::embed::Embedder;
use super::graph::{Edge, Graph, inverse_rel};
use super::index::{Hit, Index, IndexDir, SearchOpts, SyncStats};
use super::mutation::{
    self, EditRequest, GeneratedRegion, MutationPolicy, MutationResult, WriteRequest,
    generated_regions,
};
use crate::error::Error;
use crate::validate::sokf::validate;

/// Most lines rendered per group before the tail is summarised.
const GROUP_CAP: usize = 30;

/// Most warnings listed by the `sokf:` overview read.
const WARNING_CAP: usize = 10;

/// Most hits one search may ask for. Retrieval widens the caller's limit
/// before tantivy pre-allocates against it, so an unbounded `limit` is an
/// allocation failure — and an abort under the release profile.
const MAX_LIMIT: usize = 50;

/// What a tool returns: text, or a message the client shows as a tool error.
type ToolResult = std::result::Result<CallToolResult, String>;

/// Harness-independent read operations over one SOKF knowledge tree.
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

/// Arguments of `sokf_resolve_source`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ResolveSourceRequest {
    /// An unqualified `sokf:<id>`, or a physical path entering `knowledge/`.
    pub path: String,
}

/// Where one source is, and which of its lines are generated. Carries no
/// semantic content: `sokf_retrieve` answers for rendered knowledge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(crate = "rmcp::schemars")]
pub struct SourceResolution {
    /// Repository-relative `knowledge/` path the request entered through.
    pub ingress_path: String,
    /// Repository-relative canonical target, contained by the active checkout.
    pub canonical_path: String,
    /// Whether the canonical target is an existing regular file.
    pub exists: bool,
    /// Generated spans of the target, in line order.
    pub generated_regions: Vec<GeneratedRegion>,
}

/// Arguments of `sokf_retrieve`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct RetrieveArgs {
    /// `sokf:` for the overview, `sokf:<id>`, or `sokf:<id>#<heading>`.
    path: String,
    /// First rendered line to return, starting at 1.
    #[schemars(range(min = 1))]
    offset: Option<u32>,
    /// Most rendered lines to return.
    #[schemars(range(min = 1))]
    limit: Option<u32>,
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

    /// Render the overview, one concept, or one of its sections.
    ///
    /// Semantic addresses only: a physical path is `sokf_resolve_source`'s
    /// question, and answering it here would put a second result dialect
    /// behind one operation.
    pub fn retrieve(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> crate::error::Result<String> {
        let Some(address) = path.strip_prefix("sokf:") else {
            return sokf_error(format!(
                "`{path}` is a file path; sokf_retrieve answers `sokf:`, `sokf:<id>`, \
                 and `sokf:<id>#<heading>` — use sokf_resolve_source for a file"
            ));
        };
        let text = if address.is_empty() {
            self.overview()?
        } else {
            let (id, heading) = match address.split_once('#') {
                Some((id, heading)) if !id.is_empty() && !heading.is_empty() => (id, Some(heading)),
                Some(_) => return sokf_error(format!("invalid SOKF address `{path}`")),
                None => (address, None),
            };
            self.read(id, heading)?
        };
        line_window(&text, offset, limit)
    }

    /// Locate the source behind one identity or contained physical path.
    ///
    /// The answer is a canonical target inside the active checkout, whether
    /// or not it exists yet, plus the generated spans a caller must not
    /// author by hand. Nothing is read for content.
    pub fn resolve_source(&self, path: &str) -> crate::error::Result<SourceResolution> {
        let bundle_dir = std::fs::canonicalize(&self.bundle_dir).map_err(|source| Error::Io {
            path: self.bundle_dir.clone(),
            source,
        })?;
        let repo_root = std::fs::canonicalize(&self.repo_root).map_err(|source| Error::Io {
            path: self.repo_root.clone(),
            source,
        })?;

        let mut bundle = None;
        let ingress = if let Some(address) = path.strip_prefix("sokf:") {
            if address.is_empty() || address.contains('#') {
                return sokf_error(format!(
                    "`{path}` addresses rendered knowledge, not a source file — \
                     use sokf_retrieve for the overview and for sections"
                ));
            }
            let loaded = bundle.insert(load_bundle(&self.bundle_dir)?);
            let graph = Graph::build(loaded);
            let identity = resolve(&graph, address)
                .map_err(|message| broken_file_error(loaded, address).unwrap_or(message))
                .map_err(|message| Error::Sokf { message })?;
            let concept = concept_of(loaded, &identity).ok_or_else(|| Error::Sokf {
                message: format!("no concept for `{identity}`"),
            })?;
            bundle_dir.join(&concept.path)
        } else {
            let asked = Path::new(path);
            let candidate = if asked.is_absolute() {
                asked.to_path_buf()
            } else {
                repo_root.join(asked)
            };
            let normalized = lexical(&candidate).ok_or_else(|| Error::Sokf {
                message: format!("source path `{path}` escapes the repository"),
            })?;
            // The ingress is judged by name, before any symlink is followed:
            // a direct physical argument has to enter through `knowledge/`.
            // An absolute argument may spell the root differently from its
            // canonical form, so both spellings name the same ingress.
            let under = [bundle_dir.as_path(), self.bundle_dir.as_path()]
                .into_iter()
                .find_map(|root| normalized.strip_prefix(root).ok());
            let Some(relative) = under else {
                return sokf_error(format!(
                    "source path `{path}` is outside `{}`",
                    display_path(&repo_root, &bundle_dir)
                ));
            };
            bundle_dir.join(relative)
        };

        // Follow symlinks as far as the filesystem goes, then re-attach
        // whatever does not exist yet. Containment is judged on the result,
        // so a link out of the checkout fails before anything is opened.
        let canonical = nearest_canonical(&ingress).ok_or_else(|| Error::Sokf {
            message: format!("cannot safely resolve `{path}`"),
        })?;
        if !canonical.starts_with(&repo_root) {
            return sokf_error(format!(
                "source path `{path}` resolves outside the active checkout at {}",
                repo_root.display()
            ));
        }
        if canonical.is_dir() {
            return sokf_error(format!("`{path}` is a directory, not a source file"));
        }

        let exists = canonical.is_file();
        let regions = if exists {
            let text = std::fs::read_to_string(&canonical).map_err(|source| Error::Io {
                path: canonical.clone(),
                source,
            })?;
            // A block naming a concept reports the file that authors it, so
            // the bundle is needed here — but only for a file that has one.
            let known = match bundle {
                Some(loaded) => Some(loaded),
                None if text.contains("sokf:include") => Some(load_bundle(&self.bundle_dir)?),
                None => None,
            };
            generated_regions(&text, |id| {
                let concept = concept_of(known.as_ref()?, id)?;
                Some(display_path(&repo_root, &bundle_dir.join(&concept.path)))
            })
        } else {
            Vec::new()
        };

        Ok(SourceResolution {
            ingress_path: display_path(&repo_root, &ingress),
            canonical_path: display_path(&repo_root, &canonical),
            exists,
            generated_regions: regions,
        })
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

    /// Locate the source file behind one `sokf:<id>` identity or contained
    /// physical knowledge path, and report what of it is generated.
    #[tool]
    async fn sokf_resolve_source(
        &self,
        Parameters(args): Parameters<ResolveSourceRequest>,
    ) -> ToolResult {
        let _guard = self.exclusive();
        match self.service.resolve_source(&args.path) {
            Ok(resolution) => structured_result(&resolution),
            Err(error) => Err(tool_error(error)),
        }
    }

    /// Render the `sokf:` overview, one concept, or one of its sections.
    #[tool]
    async fn sokf_retrieve(&self, Parameters(args): Parameters<RetrieveArgs>) -> ToolResult {
        let _guard = self.exclusive();
        self.service
            .retrieve(
                &args.path,
                args.offset.map(|offset| offset as usize),
                args.limit.map(|limit| limit as usize),
            )
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
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Access to this repository's canonical SOKF knowledge. Use sokf_search to find \
             project knowledge, sokf_overview to see the knowledge at a glance, \
             sokf_retrieve for the `sokf:` overview, a rendered concept or one section, \
             sokf_resolve_source to locate a concept's source file, sokf_graph to follow \
             links, and sokf_edit or sokf_write for agent-safe mutations. Mutations repair \
             and validate automatically.",
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

/// A machine result is MCP structured content, mirrored as one text item
/// holding the same JSON — so a text-only client reads the same answer.
fn structured_result(result: &impl Serialize) -> ToolResult {
    let value = serde_json::to_value(result).map_err(|error| error.to_string())?;
    let rendered = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    let mut output = CallToolResult::success(vec![ContentBlock::text(rendered)]);
    output.structured_content = Some(value);
    Ok(output)
}

/// A successful mutation is both readable text and MCP structured content.
fn mutation_result(result: MutationResult) -> ToolResult {
    structured_result(&result)
}

/// `path` with `.` and `..` resolved by name alone, or `None` when it walks
/// above the root. Nothing is read: this fixes the logical ingress before
/// the filesystem gets a say in where it points.
fn lexical(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    Some(out)
}

/// `path` canonicalized, or its nearest existing ancestor canonicalized with
/// the missing tail re-attached. Symlinks resolve on the existing part, so a
/// missing file below an escaping link still lands outside the repository.
fn nearest_canonical(path: &Path) -> Option<PathBuf> {
    if let Ok(exact) = std::fs::canonicalize(path) {
        return Some(exact);
    }
    let mut tail = Vec::new();
    let mut here = path.to_path_buf();
    let base = loop {
        if let Ok(base) = std::fs::canonicalize(&here) {
            break base;
        }
        tail.push(here.file_name()?.to_owned());
        if !here.pop() {
            return None;
        }
    };
    let mut resolved = base;
    for part in tail.iter().rev() {
        resolved.push(part);
    }
    Some(resolved)
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

/// Identity to one-line description, for the lines that name a concept
/// without printing it.
fn descriptions(bundle: &Bundle) -> HashMap<String, String> {
    bundle
        .concepts
        .iter()
        .map(|c| {
            (
                c.id.clone().unwrap_or_else(|| c.path.clone()),
                c.description.clone().unwrap_or_default(),
            )
        })
        .collect()
}

/// Every concept identity paired with the repository-relative path of its
/// file, so a traversal reaches a file without a second lookup.
fn paths(bundle: &Bundle, repo_root: &Path) -> HashMap<String, String> {
    bundle
        .concepts
        .iter()
        .map(|c| {
            (
                c.id.clone().unwrap_or_else(|| c.path.clone()),
                display_path(repo_root, &bundle.root.join(&c.path)),
            )
        })
        .collect()
}

/// `identity — description`, or the identity alone when there is none.
fn named(identity: &str, description: &str) -> String {
    if description.is_empty() {
        identity.to_string()
    } else {
        format!("{identity} — {description}")
    }
}

/// `identity  path — description`: the same line, carrying the file to open
/// next. An identity the bundle does not hold has no path and reads as
/// [`named`] does, so an unresolved target is not given one it cannot have.
fn named_at(identity: &str, paths: &HashMap<String, String>, description: &str) -> String {
    let Some(path) = paths.get(identity) else {
        return named(identity, description);
    };
    if description.is_empty() {
        format!("{identity}  {path}")
    } else {
        format!("{identity}  {path} — {description}")
    }
}

/// The frontmatter `status`, as the spec spells it.
fn status_word(status: Status) -> &'static str {
    match status {
        Status::Draft => "draft",
        Status::Stable => "stable",
        Status::Deprecated => "deprecated",
    }
}

/// A section's heading path, or `(root)` for the section above the first
/// heading.
fn heading_label(heading_path: &[String]) -> String {
    if heading_path.is_empty() {
        "(root)".to_string()
    } else {
        heading_path.join(" > ")
    }
}

/// Cap a group at [`GROUP_CAP`] lines, summarising the tail by relationship
/// type. Each entry is its line and the `rel` it came from.
fn capped(entries: Vec<(String, String)>) -> Vec<String> {
    if entries.len() <= GROUP_CAP {
        return entries.into_iter().map(|(line, _)| line).collect();
    }
    let dropped = &entries[GROUP_CAP..];
    let mut rels: Vec<&str> = dropped
        .iter()
        .map(|(_, rel)| rel.as_str())
        .filter(|rel| !rel.is_empty())
        .collect();
    rels.sort_unstable();
    rels.dedup();
    let mut lines: Vec<String> = entries
        .iter()
        .take(GROUP_CAP)
        .map(|(line, _)| line.clone())
        .collect();
    lines.push(format!(
        "  +{} more (rels: {})",
        dropped.len(),
        rels.join(", ")
    ));
    lines
}

/// Search hits, grouped by concept in score order.
fn render_hits(bundle: &Bundle, query: &str, hits: &[Hit], lexical_only: bool) -> String {
    let descriptions = descriptions(bundle);
    let mut lines = Vec::new();
    if hits.is_empty() {
        lines.push(format!("no matches for `{query}`"));
    } else {
        lines.push(format!("{} sections for `{query}`", hits.len()));
    }

    // Groups keep the order of their best hit, so the strongest concept
    // leads.
    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<&Hit>> = HashMap::new();
    for hit in hits {
        let identity = hit.concept_id.clone().unwrap_or_else(|| hit.path.clone());
        if !groups.contains_key(&identity) {
            order.push(identity.clone());
        }
        groups.entry(identity).or_default().push(hit);
    }

    for identity in &order {
        lines.push(String::new());
        let description = descriptions.get(identity).map_or("", String::as_str);
        lines.push(named(identity, description));
        for hit in &groups[identity] {
            lines.push(format!(
                "  {}:{}-{}  [{}]  {}",
                hit.path,
                hit.start_line,
                hit.end_line,
                heading_label(&hit.heading_path),
                hit.snippet
            ));
        }
    }

    if lexical_only {
        lines.push(String::new());
        lines.push("note: semantic search unavailable (lexical only)".to_string());
    }
    lines.join("\n")
}

/// One concept: a frontmatter summary, then the body section by section.
fn render_concept(
    concept: &Concept,
    identity: &str,
    heading: Option<&str>,
) -> std::result::Result<String, String> {
    let mut lines = vec![
        named(identity, concept.description.as_deref().unwrap_or_default()),
        concept.path.clone(),
        format!("type: {}", concept.kind),
        format!("status: {}", status_word(concept.status)),
    ];
    if let Some(lifecycle) = &concept.lifecycle {
        lines.push(format!("lifecycle: {lifecycle}"));
    }
    if let Some(title) = &concept.title {
        lines.push(format!("title: {title}"));
    }
    if !concept.tags.is_empty() {
        lines.push(format!("tags: {}", concept.tags.join(", ")));
    }
    if let Some(resource) = &concept.resource {
        lines.push(format!("resource: {resource}"));
    }
    if !concept.links.is_empty() {
        lines.push("links:".to_string());
        for link in &concept.links {
            let rel = link.rel.clone().unwrap_or_else(|| "?".to_string());
            let to = link.to.clone().unwrap_or_else(|| "?".to_string());
            let note = link
                .note
                .as_ref()
                .map_or(String::new(), |n| format!("  ({n})"));
            lines.push(format!("  {rel} -> {to}{note}"));
        }
    }

    let sections: Vec<_> = match heading {
        None => concept.sections.iter().collect(),
        Some(wanted) => {
            let matched: Vec<_> = concept
                .sections
                .iter()
                .filter(|s| matches_heading(&s.heading_path, wanted))
                .collect();
            if matched.is_empty() {
                let available: Vec<String> = concept
                    .sections
                    .iter()
                    .map(|s| heading_label(&s.heading_path))
                    .collect();
                return Err(format!(
                    "no heading `{wanted}` in {} — headings: {}",
                    concept.path,
                    available.join(", ")
                ));
            }
            matched
        }
    };

    for section in sections {
        lines.push(String::new());
        lines.push(format!(
            "{}:{}-{}  [{}]",
            concept.path,
            section.start_line,
            section.end_line,
            heading_label(&section.heading_path)
        ));
        lines.push(section.text.trim_end().to_string());
    }
    Ok(lines.join("\n"))
}

/// Whether `wanted` names this section: the whole heading path, or its last
/// segment, case-insensitively.
fn matches_heading(heading_path: &[String], wanted: &str) -> bool {
    let wanted = wanted.trim().to_lowercase();
    if heading_path.is_empty() {
        // The label the error message and every locator line advertise, so
        // asking for it back has to work.
        return wanted.is_empty() || wanted == "(root)";
    }
    heading_path.join(" > ").to_lowercase() == wanted
        || heading_path
            .last()
            .is_some_and(|last| last.to_lowercase() == wanted)
}

/// The declared edge map, grouped by source concept. Each group opens with
/// its source and that source's path, so the identity is stated once and
/// every concept named carries the file to read next.
fn render_edges(edges: &[Edge], bundle: &Bundle, repo_root: &Path) -> String {
    if edges.is_empty() {
        return "no links declared".to_string();
    }
    let paths = paths(bundle, repo_root);
    let descriptions = descriptions(bundle);
    // BTreeMap so sources come out in a stable, readable order.
    let mut groups: BTreeMap<&str, Vec<(String, String)>> = BTreeMap::new();
    for edge in edges {
        let rel = if edge.rel.is_empty() { "?" } else { &edge.rel };
        let unresolved = if edge.resolved { "" } else { "  [unresolved]" };
        let note = edge
            .note
            .as_ref()
            .map_or(String::new(), |n| format!("  ({n})"));
        let target = descriptions.get(&edge.to).map_or("", String::as_str);
        groups.entry(&edge.from).or_default().push((
            format!(
                "  --{rel}--> {}{unresolved}{note}",
                named_at(&edge.to, &paths, target)
            ),
            edge.rel.clone(),
        ));
    }

    let mut lines = Vec::new();
    for (from, entries) in groups {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        let description = descriptions.get(from).map_or("", String::as_str);
        lines.push(named_at(from, &paths, description));
        lines.extend(capped(entries));
    }
    lines.join("\n")
}

/// One concept's hops, outgoing then incoming.
fn render_neighbours(bundle: &Bundle, identity: &str, hops: &[Edge], repo_root: &Path) -> String {
    let descriptions = descriptions(bundle);
    let paths = paths(bundle, repo_root);
    let description = descriptions.get(identity).map_or("", String::as_str);
    let mut lines = vec![named_at(identity, &paths, description)];
    if hops.is_empty() {
        lines.push("no links".to_string());
        return lines.join("\n");
    }

    let line = |edge: &Edge| {
        let target = descriptions.get(&edge.to).map_or("", String::as_str);
        let rel = if edge.rel.is_empty() { "?" } else { &edge.rel };
        let unresolved = if edge.resolved { "" } else { "  [unresolved]" };
        if edge.synthesised {
            // The hop carries the inverse rel; inverting it back states the
            // edge as the other concept declared it. The tail summary quotes
            // the rel the lines show, not the stored one.
            let declared = inverse_rel(rel);
            (
                format!(
                    "<--{declared}-- {}{unresolved}",
                    named_at(&edge.to, &paths, target)
                ),
                declared.to_string(),
            )
        } else {
            (
                format!(
                    "--{rel}--> {}{unresolved}",
                    named_at(&edge.to, &paths, target)
                ),
                edge.rel.clone(),
            )
        }
    };

    for synthesised in [false, true] {
        let group: Vec<(String, String)> = hops
            .iter()
            .filter(|hop| hop.synthesised == synthesised)
            .map(&line)
            .collect();
        if group.is_empty() {
            continue;
        }
        lines.push(String::new());
        lines.extend(capped(group));
    }
    lines.join("\n")
}

/// The bundle at a glance: name, size, what the sync did, the tree, and
/// anything wrong with it.
fn render_overview(bundle: &Bundle, stats: &SyncStats, repo_root: &std::path::Path) -> String {
    let name = bundle
        .manifest
        .as_ref()
        .and_then(|m| m.name.clone())
        .unwrap_or_else(|| bundle.root.display().to_string());
    let mut lines = vec![format!("{name} — {} concepts", bundle.concepts.len())];

    if stats.reindexed > 0 || stats.removed > 0 || stats.full_rebuild {
        let rebuild = if stats.full_rebuild {
            " (full rebuild)"
        } else {
            ""
        };
        lines.push(format!(
            "synced: {} reindexed, {} removed{rebuild}",
            stats.reindexed, stats.removed
        ));
    }
    if stats.lexical_only {
        lines.push("search: lexical only (no embedder)".to_string());
    }

    let mut tree: BTreeMap<&str, Vec<&Concept>> = BTreeMap::new();
    for concept in &bundle.concepts {
        let directory = concept.path.rsplit_once('/').map_or("", |(dir, _)| dir);
        tree.entry(directory).or_default().push(concept);
    }
    for (directory, concepts) in tree {
        lines.push(String::new());
        lines.push(if directory.is_empty() {
            "./".to_string()
        } else {
            format!("{directory}/")
        });
        for concept in concepts {
            let identity = concept.id.clone().unwrap_or_else(|| concept.path.clone());
            let lifecycle = concept
                .lifecycle
                .as_ref()
                .map_or(String::new(), |value| format!(" [{value}]"));
            lines.push(format!(
                "  {}{lifecycle}",
                named(
                    &identity,
                    concept.description.as_deref().unwrap_or_default()
                )
            ));
        }
    }

    let report = validate(bundle, repo_root);
    let mut warnings: Vec<String> = bundle
        .broken
        .iter()
        .map(|e| format!("  {}: does not parse: {}", e.path, e.message))
        .collect();
    warnings.extend(
        report
            .findings
            .iter()
            .map(|f| format!("  {}: [{}] {}", f.path, f.severity(), f.message)),
    );
    if !warnings.is_empty() {
        lines.push(String::new());
        lines.push("warnings:".to_string());
        let total = warnings.len();
        warnings.truncate(WARNING_CAP);
        lines.append(&mut warnings);
        if total > WARNING_CAP {
            lines.push(format!("  +{} more", total - WARNING_CAP));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge(index: usize, rel: &str) -> Edge {
        Edge {
            from: "alpha".to_string(),
            rel: rel.to_string(),
            to: format!("target-{index}"),
            note: None,
            resolved: true,
            synthesised: false,
        }
    }

    #[test]
    fn a_long_group_is_capped_and_summarised() {
        let edges: Vec<Edge> = (0..35)
            .map(|i| {
                edge(
                    i,
                    if i % 2 == 0 {
                        "depends-on"
                    } else {
                        "references"
                    },
                )
            })
            .collect();
        let (bundle, dir) = bundle_with(&[]);
        let rendered = render_edges(&edges, &bundle, dir.path());
        let lines: Vec<&str> = rendered.lines().collect();
        // One source header, then the capped group and its summary.
        assert_eq!(lines.len(), GROUP_CAP + 2);
        assert_eq!(lines[0], "alpha");
        assert_eq!(lines[1], "  --depends-on--> target-0");
        assert_eq!(
            lines[GROUP_CAP + 1],
            "  +5 more (rels: depends-on, references)"
        );
    }

    #[test]
    fn a_short_group_is_untouched() {
        let edges: Vec<Edge> = (0..3).map(|i| edge(i, "part-of")).collect();
        let (bundle, dir) = bundle_with(&[]);
        let rendered = render_edges(&edges, &bundle, dir.path());
        assert_eq!(rendered.lines().count(), 4);
        assert!(!rendered.contains("more (rels"));
    }

    #[test]
    fn an_empty_map_says_so() {
        let (bundle, dir) = bundle_with(&[]);
        assert_eq!(render_edges(&[], &bundle, dir.path()), "no links declared");
    }

    #[test]
    fn an_unresolved_edge_without_a_rel_is_marked() {
        let mut e = edge(0, "");
        e.resolved = false;
        e.note = Some("why".to_string());
        let (bundle, dir) = bundle_with(&[]);
        assert_eq!(
            render_edges(&[e], &bundle, dir.path()),
            "alpha\n  --?--> target-0  [unresolved]  (why)"
        );
    }

    #[test]
    fn the_edge_map_carries_the_path_of_every_concept_it_names() {
        let (bundle, dir) = bundle_with(&[("alpha.md", ALPHA), ("beta.md", BETA)]);
        let graph = Graph::build(&bundle);
        let rendered = render_edges(&graph.edge_map(), &bundle, dir.path());
        // The source is named once with its path, and the resolved target
        // carries the path a reader opens next.
        assert!(rendered.starts_with("alpha  alpha.md — The one."));
        assert!(rendered.contains("  --depends-on--> beta  beta.md"));
        // Every path names a file that exists, so a traversal can open it.
        for line in rendered.lines() {
            for token in line.split_whitespace().filter(|t| t.ends_with(".md")) {
                assert!(dir.path().join(token).is_file(), "{token}");
            }
        }
    }

    #[test]
    fn an_unresolved_target_is_named_without_a_path_it_does_not_have() {
        let mut e = edge(0, "references");
        e.resolved = false;
        let (bundle, dir) = bundle_with(&[("alpha.md", ALPHA)]);
        let rendered = render_edges(&[e], &bundle, dir.path());
        assert!(rendered.contains("--references--> target-0  [unresolved]"));
        assert!(!rendered.contains("target-0  \u{2014}"));
        assert!(!rendered.contains("target-0.md"));
    }

    fn bundle_with(files: &[(&str, &str)]) -> (Bundle, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        for (path, text) in files {
            let path = dir.path().join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        (load_bundle(dir.path()).unwrap(), dir)
    }

    const ALPHA: &str = "---\ntype: Module\nid: alpha\ndescription: The one.\nstatus: draft\nresource: /src/alpha.rs\ntags: [core]\nlinks:\n  - rel: depends-on\n    to: beta\n  - {}\n---\n\n# Role\n\nAlpha does the work.\n\n# Notes\n\nNothing yet.\n";
    const BETA: &str =
        "---\ntype: Module\nid: beta\nstatus: deprecated\n---\n\n# Role\n\nBeta is retired.\n";

    #[test]
    fn lazy_embedder_initializes_once_on_first_index_call() {
        let (bundle, dir) = bundle_with(&[("alpha.md", ALPHA), ("beta.md", BETA)]);
        drop(bundle);
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = calls.clone();
        let service = SokfService::new_lazy(
            dir.path().to_path_buf(),
            dir.path().to_path_buf(),
            IndexDir(dir.path().join("index")),
            move || {
                observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(None)
            },
        );

        // Source resolution and direct retrieval are the operations the
        // contract names, so they are what this asserts on.
        service.resolve_source("sokf:alpha").unwrap();
        service.retrieve("sokf:alpha", None, None).unwrap();
        service.graph(Some("alpha")).unwrap();
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);

        service
            .search(SearchRequest {
                query: "work".into(),
                ..SearchRequest::default()
            })
            .unwrap();
        // The overview is retrieval's one index-dependent address.
        service.retrieve("sokf:", None, None).unwrap();
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn source_resolution_and_direct_retrieval_do_not_open_the_search_index() {
        let (bundle, dir) = bundle_with(&[("alpha.md", ALPHA), ("beta.md", BETA)]);
        drop(bundle);
        // A file where the index directory belongs: anything that opens the
        // index here fails, so answering proves it was never opened.
        let unusable_index = dir.path().join("index-is-a-file");
        std::fs::write(&unusable_index, "not a directory").unwrap();
        let service = SokfService::new(
            dir.path().to_path_buf(),
            dir.path().to_path_buf(),
            IndexDir(unusable_index),
            None,
        );
        assert_eq!(
            service.resolve_source("sokf:alpha").unwrap().canonical_path,
            "alpha.md"
        );
        assert!(
            service
                .retrieve("sokf:alpha", None, None)
                .unwrap()
                .contains("Alpha does the work")
        );
        assert!(service.graph(Some("alpha")).unwrap().contains("beta"));
    }

    #[test]
    fn a_concept_renders_its_frontmatter_then_every_section() {
        let (bundle, _dir) = bundle_with(&[("alpha.md", ALPHA)]);
        let concept = concept_of(&bundle, "alpha").unwrap();
        let rendered = render_concept(concept, "alpha", None).unwrap();
        assert!(rendered.starts_with("alpha — The one.\nalpha.md\ntype: Module\nstatus: draft\n"));
        assert!(rendered.contains("resource: /src/alpha.rs"));
        assert!(rendered.contains("tags: core"));
        assert!(rendered.contains("  depends-on -> beta"));
        // A link with neither `rel` nor `to` still prints, marked.
        assert!(rendered.contains("  ? -> ?"));
        assert!(rendered.contains("[Role]"));
        assert!(rendered.contains("[Notes]"));
    }

    #[test]
    fn a_heading_selects_one_section_and_a_wrong_one_lists_the_rest() {
        let (bundle, _dir) = bundle_with(&[("alpha.md", ALPHA)]);
        let concept = concept_of(&bundle, "alpha").unwrap();
        let one = render_concept(concept, "alpha", Some(" role ")).unwrap();
        assert!(one.contains("[Role]"));
        assert!(!one.contains("[Notes]"));

        let error = render_concept(concept, "alpha", Some("Nope")).unwrap_err();
        assert!(error.contains("no heading `Nope`"));
        assert!(error.contains("(root), Role, Notes"));
    }

    #[test]
    fn the_root_section_answers_to_the_label_it_advertises() {
        let (bundle, _dir) = bundle_with(&[("alpha.md", ALPHA)]);
        let concept = concept_of(&bundle, "alpha").unwrap();
        for wanted in ["(root)", " (ROOT) ", ""] {
            let rendered = render_concept(concept, "alpha", Some(wanted)).unwrap();
            assert!(rendered.contains("[(root)]"), "{wanted}");
            assert!(!rendered.contains("[Role]"), "{wanted}");
        }
    }

    #[test]
    fn a_capped_incoming_group_summarises_the_rels_it_showed() {
        let hops: Vec<Edge> = (0..35)
            .map(|i| Edge {
                from: "beta".to_string(),
                rel: "depended-on-by".to_string(),
                to: format!("alpha-{i}"),
                note: None,
                resolved: true,
                synthesised: true,
            })
            .collect();
        let (bundle, dir) = bundle_with(&[("beta.md", BETA)]);
        let rendered = render_neighbours(&bundle, "beta", &hops, dir.path());
        assert!(rendered.contains("<--depends-on-- alpha-0"));
        assert!(rendered.contains("+5 more (rels: depends-on)"));
    }

    #[test]
    fn a_search_limit_is_bounded_at_both_ends() {
        assert_eq!(hit_limit(None), SearchOpts::default().limit);
        assert_eq!(hit_limit(Some(3)), 3);
        // Zero would answer nothing; the ceiling keeps retrieval allocatable.
        assert_eq!(hit_limit(Some(0)), 1);
        assert_eq!(hit_limit(Some(u32::MAX)), MAX_LIMIT);
    }

    #[test]
    fn a_concept_without_a_description_is_named_by_identity_alone() {
        let (bundle, _dir) = bundle_with(&[("beta.md", BETA)]);
        let concept = concept_of(&bundle, "beta").unwrap();
        let rendered = render_concept(concept, "beta", None).unwrap();
        assert!(rendered.starts_with("beta\nbeta.md\ntype: Module\nstatus: deprecated"));
    }

    #[test]
    fn neighbours_render_both_directions_and_say_when_there_are_none() {
        let (bundle, dir) = bundle_with(&[("alpha.md", ALPHA), ("beta.md", BETA)]);
        let graph = Graph::build(&bundle);
        let outgoing = render_neighbours(
            &bundle,
            "alpha",
            &graph.neighbours("alpha").unwrap(),
            dir.path(),
        );
        assert!(outgoing.starts_with("alpha  alpha.md — The one."));
        assert!(outgoing.contains("--depends-on--> beta  beta.md"));
        let incoming = render_neighbours(
            &bundle,
            "beta",
            &graph.neighbours("beta").unwrap(),
            dir.path(),
        );
        assert!(incoming.contains("<--depends-on-- alpha  alpha.md — The one."));

        let (lone, dir) = bundle_with(&[("beta.md", BETA)]);
        let graph = Graph::build(&lone);
        let none = render_neighbours(
            &lone,
            "beta",
            &graph.neighbours("beta").unwrap(),
            dir.path(),
        );
        assert!(none.ends_with("no links"));
    }

    #[test]
    fn an_unknown_id_with_no_near_miss_still_names_itself() {
        let (bundle, _dir) = bundle_with(&[("alpha.md", ALPHA)]);
        let graph = Graph::build(&bundle);
        assert_eq!(
            resolve(&graph, "zzz").unwrap_err(),
            "unknown id `zzz`".to_string()
        );
        assert!(
            resolve(&graph, "alph")
                .unwrap_err()
                .contains("did you mean")
        );
    }

    #[test]
    fn the_overview_reports_parse_failures_and_caps_the_warning_list() {
        let mut files: Vec<(String, String)> = (0..12)
            .map(|i| {
                (
                    format!("c{i}.md"),
                    format!("---\ntype: T\nid: c{i}\n---\n\n[gone](missing-{i}.md)\n"),
                )
            })
            .collect();
        files.push(("bad.md".to_string(), "no frontmatter here\n".to_string()));
        let refs: Vec<(&str, &str)> = files
            .iter()
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect();
        let (bundle, dir) = bundle_with(&refs);

        let stats = SyncStats {
            reindexed: 0,
            removed: 0,
            full_rebuild: false,
            lexical_only: false,
        };
        let rendered = render_overview(&bundle, &stats, dir.path());
        // Nothing was reindexed, so no sync line and no lexical note.
        assert!(!rendered.contains("synced:"));
        assert!(!rendered.contains("lexical only"));
        let block: Vec<&str> = rendered
            .lines()
            .skip_while(|line| *line != "warnings:")
            .skip(1)
            .collect();
        // The parse failure leads, then the validator's findings, capped.
        assert!(block[0].contains("bad.md: does not parse"));
        assert!(block.iter().any(|line| line.contains("broken body link")));
        assert_eq!(block.len(), WARNING_CAP + 1);
        assert!(block[WARNING_CAP].starts_with("  +"));
        assert!(block[WARNING_CAP].ends_with(" more"));
    }

    #[test]
    fn the_server_debugs_without_leaking_the_embedder() {
        let server = SokfServer::new(
            PathBuf::from("/repo/knowledge"),
            PathBuf::from("/repo"),
            IndexDir(PathBuf::from("/repo/.superdev/cache")),
            None,
        );
        let shown = format!("{server:?}");
        assert!(shown.contains("/repo/knowledge"));
        assert!(shown.contains("embedder: None"));
    }
}
