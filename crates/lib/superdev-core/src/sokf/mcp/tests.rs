use super::*;
use crate::sokf::graph::Edge;

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

    // Reading one concept and walking the graph parse current knowledge
    // and answer from it, so neither needs an embedder.
    service.read("alpha", None).unwrap();
    service.graph(Some("alpha")).unwrap();
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);

    service
        .search(SearchRequest {
            query: "work".into(),
            ..SearchRequest::default()
        })
        .unwrap();
    // The overview syncs the index, so it initializes the same embedder
    // rather than a second one.
    service.overview().unwrap();
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn reading_a_concept_and_walking_the_graph_do_not_open_the_search_index() {
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
    assert!(
        service
            .read("alpha", None)
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
