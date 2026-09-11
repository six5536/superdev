//! Text rendering shared by the CLI service and MCP tools.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use super::{GROUP_CAP, WARNING_CAP, display_path};
use crate::sokf::bundle::Bundle;
use crate::sokf::concept::{Concept, Status};
use crate::sokf::graph::{Edge, inverse_rel};
use crate::sokf::index::{Hit, SyncStats};
use crate::validate::sokf::validate;

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

/// Search hits, grouped by concept in retrieval order.
pub(super) fn render_hits(
    bundle: &Bundle,
    query: &str,
    hits: &[Hit],
    lexical_only: bool,
    warnings: &[String],
) -> String {
    let descriptions = descriptions(bundle);
    let mut lines = Vec::new();
    if hits.is_empty() {
        lines.push(format!("no matches for `{query}`"));
    } else {
        lines.push(format!("{} sections for `{query}`", hits.len()));
    }

    // Groups keep the order of their first hit: direct addresses already
    // lead hybrid relevance before rendering.
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
                "  {}:{}-{} {{match: {}}}  [{}]  {}",
                hit.path,
                hit.start_line,
                hit.end_line,
                hit.match_kind.label(),
                heading_label(&hit.heading_path),
                hit.snippet
            ));
        }
    }

    if lexical_only {
        lines.push(String::new());
        lines.push("note: semantic search unavailable (lexical only)".to_string());
    }
    for warning in warnings {
        lines.push(format!("note: query syntax recovered: {warning}"));
    }
    lines.join("\n")
}

/// One concept: a frontmatter summary, then the body section by section.
pub(super) fn render_concept(
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
pub(super) fn render_edges(edges: &[Edge], bundle: &Bundle, repo_root: &Path) -> String {
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
pub(super) fn render_neighbours(
    bundle: &Bundle,
    identity: &str,
    hops: &[Edge],
    repo_root: &Path,
) -> String {
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
pub(super) fn render_overview(
    bundle: &Bundle,
    stats: &SyncStats,
    repo_root: &std::path::Path,
) -> String {
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
