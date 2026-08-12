//! graph.rs — the bundle's link graph: target resolution and inverse
//! synthesis.

use std::collections::{HashMap, HashSet};

use super::bundle::Bundle;

/// One hop of the link graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Identity of the concept the edge leaves: its `id`, or its
    /// bundle-relative path when it has none.
    pub from: String,
    /// Relationship type; empty when the link declared none.
    pub rel: String,
    /// Identity of the target concept, or the raw `to` value when it resolves
    /// to nothing.
    pub to: String,
    /// The link's one-line explanation.
    pub note: Option<String>,
    /// Whether `to` names a concept in the bundle.
    pub resolved: bool,
    /// Whether the edge was synthesised as the inverse of a declared one.
    pub synthesised: bool,
}

/// An identity that names no concept in the bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownId {
    /// The identity that was asked for.
    pub asked: String,
    /// Up to three near misses, in bundle path order.
    pub candidates: Vec<String>,
}

/// The bundle's declared edges, plus the index that resolves link targets.
///
/// Built once per bundle load; inverse edges are synthesised on demand rather
/// than stored, so the declared set stays reportable as declared.
#[derive(Debug, Clone)]
pub struct Graph {
    /// One identity per concept, in bundle path order.
    identities: Vec<String>,
    ids: HashSet<String>,
    /// Bundle-relative path to identity.
    by_path: HashMap<String, String>,
    /// Declared edges, in path order then link order.
    edges: Vec<Edge>,
}

impl Graph {
    /// Index a bundle's concepts and resolve every declared link.
    ///
    /// A link with no `to` is not an edge; everything else survives, resolved
    /// or not.
    #[must_use]
    pub fn build(bundle: &Bundle) -> Graph {
        let mut graph = Graph {
            identities: Vec::new(),
            ids: HashSet::new(),
            by_path: HashMap::new(),
            edges: Vec::new(),
        };
        for concept in &bundle.concepts {
            let identity = concept.id.clone().unwrap_or_else(|| concept.path.clone());
            if let Some(id) = &concept.id {
                graph.ids.insert(id.clone());
            }
            graph.by_path.insert(concept.path.clone(), identity.clone());
            graph.identities.push(identity);
        }

        // Second pass: resolution needs the whole index, so no edge is built
        // until every concept is known.
        let mut edges = Vec::new();
        for concept in &bundle.concepts {
            let from = concept.id.clone().unwrap_or_else(|| concept.path.clone());
            for link in &concept.links {
                let Some(target) = link.to.clone() else {
                    continue;
                };
                let resolved = graph.resolve(&target);
                edges.push(Edge {
                    from: from.clone(),
                    rel: link.rel.clone().unwrap_or_default(),
                    to: resolved.clone().unwrap_or(target),
                    note: link.note.clone(),
                    resolved: resolved.is_some(),
                    synthesised: false,
                });
            }
        }
        graph.edges = edges;
        graph
    }

    /// Every declared edge, grouped by source concept in bundle path order.
    #[must_use]
    pub fn edge_map(&self) -> Vec<Edge> {
        self.edges.clone()
    }

    /// One hop from `id` in either direction: the concept's declared edges,
    /// plus the inverse of every declared edge pointing at it.
    ///
    /// A declared edge outranks the inverse synthesised from its mirror, so a
    /// pair declared from both sides yields one hop, not two.
    ///
    /// # Errors
    ///
    /// Returns [`UnknownId`] when `id` names no concept, carrying up to three
    /// near misses.
    pub fn neighbours(&self, id: &str) -> Result<Vec<Edge>, UnknownId> {
        let identity = self.resolve(id).ok_or_else(|| UnknownId {
            asked: id.to_string(),
            candidates: self.candidates(id),
        })?;

        let mut hops: Vec<Edge> = Vec::new();
        for edge in &self.edges {
            let hop = if edge.from == identity {
                edge.clone()
            } else if edge.resolved && edge.to == identity {
                Edge {
                    from: identity.clone(),
                    rel: inverse_rel(&edge.rel).to_string(),
                    to: edge.from.clone(),
                    note: edge.note.clone(),
                    resolved: true,
                    synthesised: true,
                }
            } else {
                continue;
            };
            match hops.iter_mut().find(|e| e.rel == hop.rel && e.to == hop.to) {
                Some(seen) if seen.synthesised && !hop.synthesised => *seen = hop,
                Some(_) => {}
                None => hops.push(hop),
            }
        }
        Ok(hops)
    }

    /// Resolve a link target to a concept identity: an `id` first, then a
    /// `/`-rooted or bundle-relative path (SPEC §8).
    #[must_use]
    pub fn resolve(&self, target: &str) -> Option<String> {
        if self.ids.contains(target) {
            return Some(target.to_string());
        }
        let path = target.trim_start_matches('/').replace('\\', "/");
        if let Some(identity) = self.by_path.get(&path) {
            return Some(identity.clone());
        }
        // A `/`-rooted target carries the directories above the bundle root,
        // which the bundle itself does not know; the longest concept path
        // ending the target is the match.
        self.by_path
            .iter()
            .filter(|(concept, _)| ends_with_segment(&path, concept))
            .max_by_key(|(concept, _)| concept.len())
            .map(|(_, identity)| identity.clone())
    }

    /// Identities overlapping `asked` in either direction, first three in
    /// bundle path order.
    fn candidates(&self, asked: &str) -> Vec<String> {
        let asked = asked.to_lowercase();
        self.identities
            .iter()
            .filter(|identity| {
                let identity = identity.to_lowercase();
                identity.contains(&asked) || asked.contains(&identity)
            })
            .take(3)
            .cloned()
            .collect()
    }
}

/// Whether `path` ends with `suffix` on a `/` boundary.
fn ends_with_segment(path: &str, suffix: &str) -> bool {
    path.len() > suffix.len()
        && path.ends_with(suffix)
        && path.as_bytes()[path.len() - suffix.len() - 1] == b'/'
}

/// The inverse of a relationship type, per SPEC §8. An unknown `rel` reads as
/// `relates-to`, so its inverse is too.
#[must_use]
pub fn inverse_rel(rel: &str) -> &str {
    match rel {
        "part-of" => "has-part",
        "has-part" => "part-of",
        "depends-on" => "depended-on-by",
        "depended-on-by" => "depends-on",
        "references" => "referenced-by",
        "referenced-by" => "references",
        "supersedes" => "superseded-by",
        "superseded-by" => "supersedes",
        "contradicts" => "contradicts",
        _ => "relates-to",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aokf::load_bundle;

    fn bundle_with(files: &[(&str, &str)]) -> crate::aokf::Bundle {
        let dir = tempfile::tempdir().unwrap();
        for (p, t) in files {
            let path = dir.path().join(p);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, t).unwrap();
        }
        load_bundle(dir.path()).unwrap()
    }

    const A: &str = "---\ntype: T\nid: alpha\ndescription: A.\nlinks:\n  - rel: depends-on\n    to: beta\n---\nbody\n";
    const B: &str = "---\ntype: T\nid: beta\ndescription: B.\n---\nbody\n";

    #[test]
    fn edge_map_lists_declared_edges_only() {
        let g = Graph::build(&bundle_with(&[("a.md", A), ("beta.md", B)]));
        let edges = g.edge_map();
        assert_eq!(edges.len(), 1);
        assert_eq!(
            (
                edges[0].from.as_str(),
                edges[0].rel.as_str(),
                edges[0].to.as_str()
            ),
            ("alpha", "depends-on", "beta")
        );
        assert!(edges[0].resolved);
        assert!(!edges[0].synthesised);
    }

    #[test]
    fn neighbours_include_synthesised_inverse() {
        let g = Graph::build(&bundle_with(&[("a.md", A), ("beta.md", B)]));
        let n = g.neighbours("beta").unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].rel, "depended-on-by");
        assert!(n[0].synthesised);
    }

    #[test]
    fn unknown_id_names_candidates() {
        let g = Graph::build(&bundle_with(&[("a.md", A), ("beta.md", B)]));
        let err = g.neighbours("bet").unwrap_err();
        assert_eq!(err.candidates, vec!["beta"]);
    }

    #[test]
    fn unresolved_targets_are_flagged_not_dropped() {
        let g = Graph::build(&bundle_with(&[("a.md", A)]));
        let edges = g.edge_map();
        assert!(!edges[0].resolved);
    }

    #[test]
    fn targets_resolve_by_bundle_path_and_repo_path() {
        let a = "---\ntype: T\nid: pathy\nlinks:\n  - rel: references\n    to: sub/no-id.md\n  - rel: references\n    to: /repo/knowledge/beta.md\n  - rel: references\n---\nbody\n";
        let g = Graph::build(&bundle_with(&[
            ("a.md", a),
            ("beta.md", B),
            ("sub/no-id.md", "---\ntype: T\n---\nbody\n"),
        ]));
        let edges = g.edge_map();
        // The third link declares no target, so it is not an edge.
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].to, "sub/no-id.md");
        assert_eq!(edges[1].to, "beta");
        assert!(edges.iter().all(|e| e.resolved));
        assert_eq!(g.resolve("nowhere.md"), None);
    }

    #[test]
    fn a_pair_declared_from_both_sides_is_one_hop() {
        let beta_back =
            "---\ntype: T\nid: beta\nlinks:\n  - rel: depended-on-by\n    to: alpha\n---\nbody\n";
        let g = Graph::build(&bundle_with(&[("a.md", A), ("beta.md", beta_back)]));
        for id in ["alpha", "beta"] {
            let hops = g.neighbours(id).unwrap();
            assert_eq!(hops.len(), 1, "{id}");
            assert!(!hops[0].synthesised, "{id}");
        }
    }

    #[test]
    fn candidates_match_either_way_and_stop_at_three() {
        let note = |id: &str| format!("---\ntype: T\nid: {id}\n---\nbody\n");
        let files: Vec<(String, String)> = ["a", "b", "c", "d"]
            .iter()
            .map(|s| (format!("{s}.md"), note(&format!("note-{s}"))))
            .collect();
        let refs: Vec<(&str, &str)> = files
            .iter()
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect();
        let g = Graph::build(&bundle_with(&refs));
        assert_eq!(
            g.neighbours("note").unwrap_err().candidates,
            vec!["note-a", "note-b", "note-c"]
        );
        // The asked identity may be the longer string, as a typo suffix is.
        let err = g.neighbours("note-a-old").unwrap_err();
        assert_eq!(err.asked, "note-a-old");
        assert_eq!(err.candidates, vec!["note-a"]);
    }

    #[test]
    fn inverse_vocabulary_matches_the_spec() {
        assert_eq!(inverse_rel("part-of"), "has-part");
        assert_eq!(inverse_rel("contradicts"), "contradicts");
        assert_eq!(inverse_rel("custom-thing"), "relates-to");
    }
}
