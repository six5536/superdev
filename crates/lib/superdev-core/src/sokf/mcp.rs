//! mcp.rs — the SOKF MCP server: four read-only tools over one bundle.
//!
//! Every call reloads the bundle from disk and syncs the index before it
//! answers, so a concept edited between calls is visible to the next one; the
//! sync is incremental, so the cost is the files that changed.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router};
use serde::Deserialize;

use super::bundle::{Bundle, load_bundle};
use super::concept::{Concept, Status};
use super::embed::Embedder;
use super::graph::{Edge, Graph, inverse_rel};
use super::index::{Hit, Index, IndexDir, SearchOpts, SyncStats};
use crate::error::Error;
use crate::validate::sokf::validate;

/// Most lines rendered per group before the tail is summarised.
const GROUP_CAP: usize = 30;

/// Most warnings listed by `sokf_overview`.
const WARNING_CAP: usize = 10;

/// Most hits one search may ask for. Retrieval widens the caller's limit
/// before tantivy pre-allocates against it, so an unbounded `limit` is an
/// allocation failure — and an abort under the release profile.
const MAX_LIMIT: usize = 50;

/// What a tool returns: text, or a message the client shows as a tool error.
type ToolResult = std::result::Result<CallToolResult, String>;

/// An MCP server over one SOKF bundle.
///
/// Read-only: no tool writes to the bundle. The index directory is the
/// server's alone.
pub struct SokfServer {
    bundle_dir: PathBuf,
    repo_root: PathBuf,
    index_dir: IndexDir,
    /// The embedder the index was built with, or `None` for lexical-only
    /// search. The same instance must reach every search, or the index's
    /// vectors go unused.
    embedder: Option<Box<dyn Embedder>>,
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
            .field("bundle_dir", &self.bundle_dir)
            .field("repo_root", &self.repo_root)
            .field("index_dir", &self.index_dir.0)
            .field("embedder", &self.embedder.as_ref().map(|e| e.model_id()))
            .finish()
    }
}

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
        SokfServer {
            bundle_dir,
            repo_root,
            index_dir,
            embedder,
            tool_lock: std::sync::Mutex::new(()),
            tool_router: SokfServer::tool_router(),
        }
    }

    /// Search the bundle. Returns the best sections, grouped by concept, each
    /// with a `path:start-end` locator to read next.
    #[tool]
    async fn sokf_search(&self, Parameters(args): Parameters<SearchArgs>) -> ToolResult {
        let _guard = self.exclusive();
        let (bundle, index, stats) = self.sync().map_err(|e| e.to_string())?;
        let opts = SearchOpts {
            limit: hit_limit(args.limit),
            kinds: args.types.unwrap_or_default(),
            tags: args.tags.unwrap_or_default(),
            lifecycle: args.lifecycle.unwrap_or_default(),
        };
        // The embedder that built the vectors is the only one that can search
        // them; anything else silently degrades to lexical.
        let hits = index
            .search(&args.query, self.embedder.as_deref(), &opts)
            .map_err(|e| e.to_string())?;
        Ok(text(render_hits(
            &bundle,
            &args.query,
            &hits,
            stats.lexical_only,
        )))
    }

    /// Read one concept whole, or one of its sections.
    #[tool]
    async fn sokf_read(&self, Parameters(args): Parameters<ReadArgs>) -> ToolResult {
        let _guard = self.exclusive();
        let (bundle, _, _) = self.sync().map_err(|e| e.to_string())?;
        let graph = Graph::build(&bundle);
        let identity = resolve(&graph, &args.id)
            .map_err(|e| broken_file_error(&bundle, &args.id).unwrap_or(e))?;
        let concept =
            concept_of(&bundle, &identity).ok_or_else(|| format!("no concept for `{identity}`"))?;
        Ok(text(render_concept(
            concept,
            &identity,
            args.heading.as_deref(),
        )?))
    }

    /// Show the link graph: the whole edge map, or one concept's neighbours
    /// in both directions.
    #[tool]
    async fn sokf_graph(&self, Parameters(args): Parameters<GraphArgs>) -> ToolResult {
        let _guard = self.exclusive();
        let (bundle, _, _) = self.sync().map_err(|e| e.to_string())?;
        let graph = Graph::build(&bundle);
        let Some(id) = args.id else {
            return Ok(text(render_edges(&graph.edge_map())));
        };
        let identity = resolve(&graph, &id)?;
        let hops = graph
            .neighbours(&identity)
            .map_err(|unknown| format!("unknown id `{}`", unknown.asked))?;
        Ok(text(render_neighbours(&bundle, &identity, &hops)))
    }

    /// Orient in the bundle: its name, size, directory tree, and anything
    /// validation found wrong.
    #[tool]
    async fn sokf_overview(&self) -> ToolResult {
        let _guard = self.exclusive();
        let (bundle, _, stats) = self.sync().map_err(|e| e.to_string())?;
        Ok(text(render_overview(&bundle, &stats, &self.repo_root)))
    }

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

    /// Reload the bundle and bring the index up to date — the freshness rule,
    /// run once per tool call.
    fn sync(&self) -> crate::error::Result<(Bundle, Index, SyncStats)> {
        let bundle = load_bundle(&self.bundle_dir)?;
        let (index, stats) =
            Index::open_and_sync(&self.index_dir, &bundle, self.embedder.as_deref())?;
        Ok((bundle, index, stats))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SokfServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Read-only access to this repository's SOKF knowledge. Start with \
             sokf_overview to see what exists, sokf_search to find sections, sokf_read to \
             read one, and sokf_graph to follow links.",
        )
    }
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

/// `identity — description`, or the identity alone when there is none.
fn named(identity: &str, description: &str) -> String {
    if description.is_empty() {
        identity.to_string()
    } else {
        format!("{identity} — {description}")
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

/// The declared edge map, grouped by source concept.
fn render_edges(edges: &[Edge]) -> String {
    if edges.is_empty() {
        return "no links declared".to_string();
    }
    // BTreeMap so sources come out in a stable, readable order.
    let mut groups: BTreeMap<&str, Vec<(String, String)>> = BTreeMap::new();
    for edge in edges {
        let rel = if edge.rel.is_empty() { "?" } else { &edge.rel };
        let unresolved = if edge.resolved { "" } else { "  [unresolved]" };
        let note = edge
            .note
            .as_ref()
            .map_or(String::new(), |n| format!("  ({n})"));
        groups.entry(&edge.from).or_default().push((
            format!("{} --{rel}--> {}{unresolved}{note}", edge.from, edge.to),
            edge.rel.clone(),
        ));
    }

    let mut lines = Vec::new();
    for entries in groups.into_values() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(capped(entries));
    }
    lines.join("\n")
}

/// One concept's hops, outgoing then incoming.
fn render_neighbours(bundle: &Bundle, identity: &str, hops: &[Edge]) -> String {
    let descriptions = descriptions(bundle);
    let description = descriptions.get(identity).map_or("", String::as_str);
    let mut lines = vec![named(identity, description)];
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
                format!("<--{declared}-- {}{unresolved}", named(&edge.to, target)),
                declared.to_string(),
            )
        } else {
            (
                format!("--{rel}--> {}{unresolved}", named(&edge.to, target)),
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
        let rendered = render_edges(&edges);
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), GROUP_CAP + 1);
        assert_eq!(lines[0], "alpha --depends-on--> target-0");
        assert_eq!(lines[GROUP_CAP], "  +5 more (rels: depends-on, references)");
    }

    #[test]
    fn a_short_group_is_untouched() {
        let edges: Vec<Edge> = (0..3).map(|i| edge(i, "part-of")).collect();
        let rendered = render_edges(&edges);
        assert_eq!(rendered.lines().count(), 3);
        assert!(!rendered.contains("more (rels"));
    }

    #[test]
    fn an_empty_map_says_so() {
        assert_eq!(render_edges(&[]), "no links declared");
    }

    #[test]
    fn an_unresolved_edge_without_a_rel_is_marked() {
        let mut e = edge(0, "");
        e.resolved = false;
        e.note = Some("why".to_string());
        assert_eq!(
            render_edges(&[e]),
            "alpha --?--> target-0  [unresolved]  (why)"
        );
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
        let (bundle, _dir) = bundle_with(&[("beta.md", BETA)]);
        let rendered = render_neighbours(&bundle, "beta", &hops);
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
        let (bundle, _dir) = bundle_with(&[("alpha.md", ALPHA), ("beta.md", BETA)]);
        let graph = Graph::build(&bundle);
        let outgoing = render_neighbours(&bundle, "alpha", &graph.neighbours("alpha").unwrap());
        assert!(outgoing.contains("--depends-on--> beta"));
        let incoming = render_neighbours(&bundle, "beta", &graph.neighbours("beta").unwrap());
        assert!(incoming.contains("<--depends-on-- alpha — The one."));

        let (lone, _dir) = bundle_with(&[("beta.md", BETA)]);
        let graph = Graph::build(&lone);
        let none = render_neighbours(&lone, "beta", &graph.neighbours("beta").unwrap());
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

    /// The MCP contract's declared tools: name to (argument, required, type).
    fn declared_tools() -> std::collections::BTreeMap<String, BTreeMap<String, (bool, String)>> {
        let path: std::path::PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "../../..",
            "knowledge/contracts/public/active/contract-003-mcp-sokf.md",
        ]
        .iter()
        .collect();
        let text = std::fs::read_to_string(path).expect("the MCP contract is on file");
        let block = text
            .split("```json\n")
            .nth(1)
            .and_then(|rest| rest.split("\n```").next())
            .expect("the Tools section carries a json block");
        let raw: BTreeMap<String, serde_json::Value> =
            serde_json::from_str(block).expect("the definition block parses as json");
        raw.into_iter()
            .map(|(name, entry)| {
                let args = entry
                    .get("arguments")
                    .and_then(|v| v.as_object())
                    .map(|map| {
                        map.iter()
                            .map(|(arg, spec)| {
                                let required = spec
                                    .get("required")
                                    .and_then(serde_json::Value::as_bool)
                                    .unwrap_or(false);
                                let ty = spec
                                    .get("type")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                (arg.clone(), (required, ty))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (name, args)
            })
            .collect()
    }

    /// The served tools, in the same shape. An optional argument's type
    /// arrives as a union with `null`; the type it declares is the other one.
    fn served_tools() -> std::collections::BTreeMap<String, BTreeMap<String, (bool, String)>> {
        SokfServer::tool_router()
            .list_all()
            .into_iter()
            .map(|tool| {
                let required: Vec<String> = tool
                    .input_schema
                    .get("required")
                    .and_then(|v| v.as_array())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|v| v.as_str().map(ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                let args = tool
                    .input_schema
                    .get("properties")
                    .and_then(|v| v.as_object())
                    .map(|props| {
                        props
                            .iter()
                            .map(|(arg, spec)| {
                                let ty = match spec.get("type") {
                                    Some(serde_json::Value::String(s)) => s.clone(),
                                    Some(serde_json::Value::Array(items)) => items
                                        .iter()
                                        .filter_map(serde_json::Value::as_str)
                                        .find(|s| *s != "null")
                                        .unwrap_or("")
                                        .to_string(),
                                    _ => String::new(),
                                };
                                (arg.clone(), (required.contains(arg), ty))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (tool.name.to_string(), args)
            })
            .collect()
    }

    /// Covers I035 criteria 4 and 7: every tool the server offers is declared
    /// in the contract with the same arguments, types and requiredness, and
    /// the contract declares no tool the server does not offer (ADR-036).
    #[test]
    fn the_served_tools_match_the_contract() {
        let served = served_tools();
        let declared = declared_tools();
        let missing: Vec<&String> = served
            .keys()
            .filter(|k| !declared.contains_key(*k))
            .collect();
        assert!(
            missing.is_empty(),
            "DEFECT — the server offers tools its contract does not declare: {missing:?}"
        );
        let extra: Vec<&String> = declared
            .keys()
            .filter(|k| !served.contains_key(*k))
            .collect();
        assert!(
            extra.is_empty(),
            "PENDING — the contract promises tools the server does not offer yet: {extra:?}"
        );
        for (name, want) in &declared {
            assert_eq!(
                &served[name], want,
                "DRIFT — `{name}`'s arguments differ between the server and its contract"
            );
        }
    }
}
