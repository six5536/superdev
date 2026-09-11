use std::fs;
use std::path::Path;

use tempfile::TempDir;

use super::search::{rrf, snippet};
use super::*;
use crate::sokf::bundle::{Bundle, load_bundle};
use crate::sokf::embed::FakeEmbedder;

/// Two headings, so this concept contributes three sections after the edit
/// in `edited_file_reindexes_only_that_file` and two before it.
const ALPHA: &str = "---\ntype: Module\nid: alpha\ntags: [core]\n---\nPlanning, before any heading.\n\n# Details\n\nThe planning stage never writes.\n";
const BETA: &str = "---\ntype: Spec\nid: beta\n---\nBeta has no headings at all.\n";

/// A tempdir holding `bundle/` (two concepts, three sections) and room for
/// `idx/` beside it.
fn fixture() -> (TempDir, Bundle) {
    let dir = tempfile::tempdir().unwrap();
    let bundle_dir = dir.path().join("bundle");
    fs::create_dir(&bundle_dir).unwrap();
    fs::write(bundle_dir.join("alpha.md"), ALPHA).unwrap();
    fs::write(bundle_dir.join("beta.md"), BETA).unwrap();
    let bundle = load_bundle(&bundle_dir).unwrap();
    (dir, bundle)
}

/// Re-read the bundle after a test edits it on disk.
fn reload(dir: &Path) -> Bundle {
    load_bundle(&dir.join("bundle")).unwrap()
}

/// Two concepts with no shared vocabulary: `release` owns "tag-driven
/// pipeline", `testing` owns "nextest".
const RELEASE: &str = "---\ntype: Reference\nid: release\ntags: [process]\n---\nReleases are cut from main.\n\n# Pipeline\n\nThe release pipeline is tag-driven: pushing a tag triggers the publish.\n";
const TESTING: &str = "---\ntype: Spec\nid: testing\ntags: [quality]\n---\nTests run under nextest.\n\n# Layers\n\nUnit tests and end-to-end tests, run on every commit.\n";

/// An index over [`RELEASE`] and [`TESTING`]. The tempdir comes back with
/// it: dropping it would delete the index under the reader.
fn search_fixture(embedder: Option<&dyn Embedder>) -> (TempDir, Index) {
    let dir = tempfile::tempdir().unwrap();
    let bundle_dir = dir.path().join("bundle");
    fs::create_dir(&bundle_dir).unwrap();
    fs::write(bundle_dir.join("release.md"), RELEASE).unwrap();
    fs::write(bundle_dir.join("testing.md"), TESTING).unwrap();
    let bundle = load_bundle(&bundle_dir).unwrap();
    let (index, _) =
        Index::open_and_sync(&IndexDir(dir.path().join("idx")), &bundle, embedder).unwrap();
    (dir, index)
}

#[test]
fn lexical_search_finds_the_right_section() {
    let (_dir, idx) = search_fixture(None);
    let hits = idx
        .search("tag-driven release pipeline", None, &SearchOpts::default())
        .unwrap();
    // Only the Pipeline section carries every term, so the AND pass
    // answers the query on its own.
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].concept_id.as_deref(), Some("release"));
    assert_eq!(hits[0].path, "release.md");
    assert_eq!(hits[0].heading_path, vec!["Pipeline".to_string()]);
    assert!(hits[0].start_line > 0 && hits[0].end_line >= hits[0].start_line);
    // The heading line through to the end of the file.
    assert_eq!((hits[0].start_line, hits[0].end_line), (8, 10));
    assert!(hits[0].snippet.contains("tag-driven"));
    assert!(!hits[0].snippet.contains('\n'));

    // A caller that asks for nothing gets nothing.
    let none = SearchOpts {
        limit: 0,
        ..SearchOpts::default()
    };
    assert!(idx.search("release", None, &none).unwrap().is_empty());
}

/// Body-only concepts, so a section's text is exactly its one body line
/// and a query can be made byte-identical to it.
const EXACT: &str = "---\ntype: Note\nid: exact\n---\nzephyr quartz lantern meadow\n";
const DENSE: &str = "---\ntype: Note\nid: dense\n---\nzephyr zephyr zephyr quartz quartz quartz lantern lantern lantern meadow meadow meadow\n";

#[test]
fn semantic_contributes_when_vectors_exist() {
    let dir = tempfile::tempdir().unwrap();
    let bundle_dir = dir.path().join("bundle");
    fs::create_dir(&bundle_dir).unwrap();
    fs::write(bundle_dir.join("exact.md"), EXACT).unwrap();
    fs::write(bundle_dir.join("dense.md"), DENSE).unwrap();
    let bundle = load_bundle(&bundle_dir).unwrap();
    let (idx, _) = Index::open_and_sync(
        &IndexDir(dir.path().join("idx")),
        &bundle,
        Some(&FakeEmbedder),
    )
    .unwrap();

    // The embedder hashes whole texts, so cosine here is an exact-text
    // test: the query is `exact`'s only section verbatim, and `dense` —
    // the same words three times over — points somewhere else entirely.
    let query = "zephyr quartz lantern meadow";
    let dense_text = &bundle.concepts[0].sections[0].text;
    assert_eq!(bundle.concepts[0].id.as_deref(), Some("dense"));
    let vectors = FakeEmbedder
        .embed(&[query.to_string(), dense_text.clone()])
        .unwrap();
    let cosine: f32 = vectors[0].iter().zip(&vectors[1]).map(|(a, b)| a * b).sum();
    assert!(
        cosine <= 0.0,
        "fixture assumes `dense` is far off: {cosine}"
    );

    // `dense` repeats every term, so it leads BM25; `exact` trails it
    // there but tops the semantic list. Two ranks beat one.
    let hits = idx
        .search(query, Some(&FakeEmbedder), &SearchOpts::default())
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].concept_id.as_deref(), Some("exact"));
    assert_eq!(hits[1].concept_id.as_deref(), Some("dense"));
    assert_eq!(hits[0].match_kind, MatchKind::LexicalAndSemantic);
    assert_eq!(hits[1].match_kind, MatchKind::Lexical);
    assert!(hits[0].score > hits[1].score);
    assert!((hits[0].score - (1.0 / 60.0 + 1.0 / 61.0)).abs() < 1e-6);
    assert!((hits[1].score - 1.0 / 60.0).abs() < 1e-6);

    // Lexical alone reverses them, which is what the semantic list had to
    // overturn.
    let lexical = idx.search(query, None, &SearchOpts::default()).unwrap();
    assert_eq!(lexical[0].concept_id.as_deref(), Some("dense"));
    // An embedder the index was not built with is ignored: its vectors
    // cannot be compared with the stored ones.
    let mismatched = idx
        .search(query, Some(&RaggedEmbedder), &SearchOpts::default())
        .unwrap();
    assert_eq!(mismatched, lexical);
}

/// Six concepts sharing one vocabulary; only their settledness differs.
/// `finished`, `closed` and `framed-issue` settle by `lifecycle`,
/// `retired` by the SOKF `status` the kinds outside the lifecycle
/// directories still use, and `open-issue` carries a live `lifecycle`
/// value.
const LIVE: &str = "---\ntype: Note\nid: live\n---\nquartz lantern meadow guide\n";
const FINISHED: &str =
    "---\ntype: Plan\nid: finished\nlifecycle: abandoned\n---\nquartz lantern meadow guide\n";
const RETIRED: &str =
    "---\ntype: Spec\nid: retired\nstatus: deprecated\n---\nquartz lantern meadow guide\n";
const CLOSED: &str =
    "---\ntype: Issue\nid: closed\nlifecycle: done\n---\nquartz lantern meadow guide\n";
const OPEN_ISSUE: &str =
    "---\ntype: Issue\nid: open-issue\nlifecycle: open\n---\nquartz lantern meadow guide\n";
/// The retired `framed` state (ADR-048, superseded by ADR-050): a value
/// outside `LIVE_LIFECYCLES`, so it ranks settled.
const FRAMED_ISSUE: &str =
    "---\ntype: Issue\nid: framed-issue\nlifecycle: framed\n---\nquartz lantern meadow guide\n";

/// Covers I052 AC_live-lifecycles: `open` and `active` rank live and
/// no other value does — the retired `framed` ranks settled with `done`.
#[test]
fn settled_work_is_downranked_not_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let bundle_dir = dir.path().join("bundle");
    fs::create_dir(&bundle_dir).unwrap();
    fs::write(bundle_dir.join("finished.md"), FINISHED).unwrap();
    fs::write(bundle_dir.join("retired.md"), RETIRED).unwrap();
    fs::write(bundle_dir.join("closed.md"), CLOSED).unwrap();
    fs::write(bundle_dir.join("open-issue.md"), OPEN_ISSUE).unwrap();
    fs::write(bundle_dir.join("framed-issue.md"), FRAMED_ISSUE).unwrap();
    fs::write(bundle_dir.join("live.md"), LIVE).unwrap();
    let bundle = load_bundle(&bundle_dir).unwrap();
    let (idx, _) = Index::open_and_sync(&IndexDir(dir.path().join("idx")), &bundle, None).unwrap();

    let hits = idx.search("quartz", None, &SearchOpts::default()).unwrap();
    // Identical text, so ranking is decided by settledness alone: the
    // live concepts first — an open issue among them (ADR-050) — the
    // settled four down-ranked behind them, but still present.
    assert_eq!(hits.len(), 6);
    let leading: HashSet<_> = hits[..2]
        .iter()
        .map(|hit| hit.concept_id.clone().unwrap())
        .collect();
    assert_eq!(
        leading,
        HashSet::from(["live".to_string(), "open-issue".to_string()])
    );
    assert!(hits[2].score < hits[1].score);
    let trailing: HashSet<_> = hits[2..]
        .iter()
        .map(|hit| hit.concept_id.clone().unwrap())
        .collect();
    assert_eq!(
        trailing,
        HashSet::from([
            "finished".to_string(),
            "retired".to_string(),
            "closed".to_string(),
            "framed-issue".to_string()
        ])
    );
}

#[test]
fn the_lifecycle_filter_keeps_only_the_named_values() {
    let dir = tempfile::tempdir().unwrap();
    let bundle_dir = dir.path().join("bundle");
    fs::create_dir(&bundle_dir).unwrap();
    fs::write(bundle_dir.join("closed.md"), CLOSED).unwrap();
    fs::write(bundle_dir.join("open-issue.md"), OPEN_ISSUE).unwrap();
    fs::write(bundle_dir.join("live.md"), LIVE).unwrap();
    let bundle = load_bundle(&bundle_dir).unwrap();
    let (idx, _) = Index::open_and_sync(&IndexDir(dir.path().join("idx")), &bundle, None).unwrap();

    let opts = SearchOpts {
        lifecycle: vec!["open".to_string()],
        ..SearchOpts::default()
    };
    let hits = idx.search("quartz", None, &opts).unwrap();
    // The open issue alone: the done one is filtered out, and so is
    // the concept carrying no lifecycle at all.
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].concept_id.as_deref(), Some("open-issue"));
}

#[test]
fn filters_restrict_kinds_and_tags() {
    let (_dir, idx) = search_fixture(Some(&FakeEmbedder));
    let opts = |kinds: &[&str], tags: &[&str]| SearchOpts {
        kinds: kinds.iter().map(|s| (*s).to_string()).collect(),
        tags: tags.iter().map(|s| (*s).to_string()).collect(),
        ..SearchOpts::default()
    };
    let search = |opts: &SearchOpts| {
        idx.search("release pipeline tests layers", Some(&FakeEmbedder), opts)
            .unwrap()
    };

    // Unfiltered, both concepts answer.
    let all = search(&SearchOpts::default());
    assert!(all.iter().any(|h| h.path == "release.md"));
    assert!(all.iter().any(|h| h.path == "testing.md"));

    // Filters apply to the semantic list as well as the lexical one, so a
    // vectored index cannot smuggle an excluded concept back in.
    let spec = search(&opts(&["Spec"], &[]));
    assert!(!spec.is_empty());
    assert!(spec.iter().all(|h| h.path == "testing.md"));
    let process = search(&opts(&[], &["process"]));
    assert!(!process.is_empty());
    assert!(process.iter().all(|h| h.path == "release.md"));

    // Kinds and tags are ANDed: no Spec is tagged `process`.
    assert!(search(&opts(&["Spec"], &["process"])).is_empty());
    // An unknown value matches nothing rather than everything.
    assert!(search(&opts(&["Nope"], &[])).is_empty());
}

#[test]
fn fusion_maths() {
    let key = |path_hash| DocKey {
        path_hash,
        ordinal: 0,
    };
    let (both, single, tail) = (key(1), key(2), key(3));
    let scored = rrf(&[vec![both, tail], vec![single, both]]);
    assert_eq!(scored.len(), 3);
    // Ranks 0 and 1 across the two lists.
    assert_eq!(scored[0].0, both);
    assert!((scored[0].1 - (1.0 / 60.0 + 1.0 / 61.0)).abs() < 1e-6);
    // Rank 0 of one list only.
    assert_eq!(scored[1].0, single);
    assert!((scored[1].1 - 1.0 / 60.0).abs() < 1e-6);
    assert_eq!(scored[2].0, tail);
    assert!((scored[2].1 - 1.0 / 61.0).abs() < 1e-6);
    assert!(rrf(&[]).is_empty());
}

#[test]
fn a_query_no_section_answers_in_full_falls_back_to_or() {
    let (_dir, idx) = search_fixture(None);
    // No section carries both terms, so the AND pass finds nothing and the
    // OR pass answers.
    let hits = idx
        .search("pipeline nextest", None, &SearchOpts::default())
        .unwrap();
    assert_eq!(hits.len(), 2);
    let mut ids: Vec<&str> = hits
        .iter()
        .filter_map(|h| h.concept_id.as_deref())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, vec!["release", "testing"]);
    // A word in neither concept finds nothing rather than everything.
    assert!(
        idx.search("kryptonite", None, &SearchOpts::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_semantic_hit_joins_back_to_its_section() {
    let (_dir, idx) = search_fixture(Some(&FakeEmbedder));
    // No section carries this word, so lexical retrieval returns nothing
    // and every hit comes from the vector store — each one looked up by
    // its path hash and ordinal.
    assert!(
        idx.search("kryptonite", None, &SearchOpts::default())
            .unwrap()
            .is_empty()
    );
    let hits = idx
        .search("kryptonite", Some(&FakeEmbedder), &SearchOpts::default())
        .unwrap();

    // Three of the four sections; the fourth's vector points away from the
    // query, and a cosine at or below zero is not a hit.
    assert_eq!(hits.len(), 3);
    assert!(
        hits.iter()
            .all(|h| h.heading_path != vec!["Pipeline".to_string()])
    );
    // Every field of a semantic-only hit comes from the joined document.
    assert!(hits.iter().all(|hit| hit.match_kind == MatchKind::Semantic));
    assert_eq!(hits[0].path, "testing.md");
    assert_eq!(hits[0].concept_id.as_deref(), Some("testing"));
    assert_eq!(hits[0].heading_path, vec!["Layers".to_string()]);
    assert_eq!((hits[0].start_line, hits[0].end_line), (8, 10));
    assert!(hits[0].snippet.contains("Unit tests"));
    // A root section has no heading path.
    assert_eq!(hits[1].heading_path, Vec::<String>::new());
    assert_eq!(hits[1].start_line, 1);
    // One list, so the scores are the bare reciprocal ranks.
    assert!((hits[0].score - 1.0 / 60.0).abs() < 1e-6);
    assert!((hits[2].score - 1.0 / 62.0).abs() < 1e-6);
}

#[test]
fn snippets_are_one_line_and_bounded() {
    let long = format!("# Heading\n\n{}", "word ".repeat(80));
    let short = snippet(&long);
    assert!(short.chars().count() <= 201);
    assert!(short.ends_with('…'));
    assert!(!short.contains('\n'));
    // Whitespace of every kind collapses to single spaces.
    assert_eq!(snippet("  a\n\n\tb  "), "a b");
}

#[test]
fn first_open_indexes_everything() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    assert!(stats.full_rebuild);
    assert_eq!(stats.reindexed, 2);
    assert_eq!(stats.removed, 0);
    assert!(!stats.lexical_only);
    assert_eq!(index.section_count(), 3);
    assert_eq!(index.vector_count(), 3);
    assert_eq!(index.model_id(), Some("fake:8"));
    assert_eq!(
        format!("{index:?}"),
        "Index { sections: 3, vectors: 3, model_id: Some(\"fake:8\") }"
    );
}

#[test]
fn stored_vectors_round_trip_intact() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();

    // Read the file back through the codec and check one known record end
    // to end: a transposed field or a flipped byte order would show here
    // and nowhere else.
    let records = read_vectors(&idx.0.join("vectors.bin")).unwrap();
    assert_eq!(records.len(), 3);
    let beta = &bundle.concepts[1];
    assert_eq!(beta.path, "beta.md");
    let record = records
        .iter()
        .find(|r| r.path_hash == path_hash("beta.md") && r.ordinal == 0)
        .expect("beta's only section");
    let expected = FakeEmbedder
        .embed(&[beta.sections[0].text.clone()])
        .unwrap();
    assert_eq!(record.vector, expected[0]);
    // Two files, three sections: alpha owns the other two ordinals.
    let mut alpha: Vec<u32> = records
        .iter()
        .filter(|r| r.path_hash == path_hash("alpha.md"))
        .map(|r| r.ordinal)
        .collect();
    alpha.sort_unstable();
    assert_eq!(alpha, vec![0, 1]);
}

#[test]
fn unchanged_bundle_syncs_nothing() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    drop(first);

    let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    assert!(!stats.full_rebuild);
    assert_eq!(stats.reindexed, 0);
    assert_eq!(stats.removed, 0);
    assert_eq!(index.section_count(), 3);
    assert_eq!(index.vector_count(), 3);
}

#[test]
fn edited_file_reindexes_only_that_file() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    drop(first);

    // A second heading, so the file goes from two sections to three: the
    // old documents must be deleted, not merely joined by the new ones.
    fs::write(
        dir.path().join("bundle/alpha.md"),
        format!("{ALPHA}\n# Extra\n\nA newly written section.\n"),
    )
    .unwrap();
    let edited = reload(dir.path());
    let (index, stats) = Index::open_and_sync(&idx, &edited, Some(&FakeEmbedder)).unwrap();
    assert!(!stats.full_rebuild);
    assert_eq!(stats.reindexed, 1);
    assert_eq!(stats.removed, 0);
    assert_eq!(index.section_count(), 4);
    assert_eq!(index.vector_count(), 4);
}

#[test]
fn deleted_file_is_removed() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    drop(first);

    fs::remove_file(dir.path().join("bundle/beta.md")).unwrap();
    let smaller = reload(dir.path());
    let (index, stats) = Index::open_and_sync(&idx, &smaller, Some(&FakeEmbedder)).unwrap();
    assert!(!stats.full_rebuild);
    assert_eq!(stats.reindexed, 0);
    assert_eq!(stats.removed, 1);
    // Only alpha's two sections survive, in both stores.
    assert_eq!(index.section_count(), 2);
    assert_eq!(index.vector_count(), 2);
    // And beta's text no longer answers a query made of it.
    assert!(
        index
            .search("Beta has no headings at all", None, &SearchOpts::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn force_rebuild_reindexes_everything() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    drop(first);

    let (index, stats) = Index::force_rebuild(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    assert!(stats.full_rebuild);
    assert_eq!(stats.reindexed, 2);
    assert_eq!(index.section_count(), 3);
}

#[test]
fn an_unusable_cache_is_rebuilt() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));

    // Each of these leaves the manifest intact, so only the damaged part
    // can send the sync down the rebuild path. The last is a well-formed
    // but empty vector file: no longer one vector per section.
    let damage: [&dyn Fn(&Path); 3] = [
        &|root| fs::remove_dir_all(root.join("tantivy")).unwrap(),
        &|root| fs::write(root.join("vectors.bin"), b"short").unwrap(),
        &|root| fs::write(root.join("vectors.bin"), [0u8; 8]).unwrap(),
    ];
    for damage in damage {
        let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        drop(first);
        damage(&idx.0);
        let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
        assert!(stats.full_rebuild);
        assert_eq!(index.section_count(), 3);
        assert_eq!(index.vector_count(), 3);
    }

    // A manifest that is not an index manifest reads as no index at all.
    fs::write(idx.0.join("manifest.json"), "{ not json").unwrap();
    let (_, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    assert!(stats.full_rebuild);
}

/// Names itself as [`FakeEmbedder`] does, so the manifest gate lets an
/// incremental sync through, but embeds twice as wide.
struct WiderEmbedder;

impl Embedder for WiderEmbedder {
    fn model_id(&self) -> String {
        "fake:8".into()
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| vec![0.25; 16]).collect())
    }
}

#[test]
fn a_change_of_vector_width_rebuilds_rather_than_failing() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    drop(first);

    // One changed file, so the sync would mix kept 8-wide records with
    // fresh 16-wide ones.
    fs::write(
        dir.path().join("bundle/alpha.md"),
        format!("{ALPHA}\nA newly written line.\n"),
    )
    .unwrap();
    let edited = reload(dir.path());
    let (index, stats) = Index::open_and_sync(&idx, &edited, Some(&WiderEmbedder)).unwrap();
    assert!(stats.full_rebuild);
    assert_eq!(stats.reindexed, 2);
    assert_eq!(index.vector_count(), 3);
    // Every record is the new width, so nothing of the old index survived.
    let records = read_vectors(&idx.0.join("vectors.bin")).unwrap();
    assert!(records.iter().all(|r| r.vector.len() == 16));
}

#[test]
fn gaining_an_embedder_forces_a_full_rebuild() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, stats) = Index::open_and_sync(&idx, &bundle, None).unwrap();
    assert!(stats.lexical_only);
    assert_eq!(first.vector_count(), 0);
    drop(first);

    let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    assert!(stats.full_rebuild);
    assert!(!stats.lexical_only);
    assert_eq!(stats.reindexed, 2);
    assert_eq!(index.vector_count(), 3);
    assert_eq!(index.model_id(), Some("fake:8"));
}

#[test]
fn an_index_from_another_schema_version_is_rebuilt() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    drop(first);

    let manifest = idx.0.join("manifest.json");
    let text = fs::read_to_string(&manifest).unwrap();
    fs::write(
        &manifest,
        text.replace(
            &format!("\"schema_version\": {SCHEMA_VERSION}"),
            "\"schema_version\": 99",
        ),
    )
    .unwrap();
    assert!(fs::read_to_string(&manifest).unwrap().contains("99"));

    let (index, stats) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    assert!(stats.full_rebuild);
    assert_eq!(stats.reindexed, 2);
    assert_eq!(index.section_count(), 3);
    assert_eq!(
        read_manifest(&idx.0).unwrap().schema_version,
        SCHEMA_VERSION
    );
}

/// Returns one vector per text, but of two different widths.
struct RaggedEmbedder;

impl Embedder for RaggedEmbedder {
    fn model_id(&self) -> String {
        "ragged".into()
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .enumerate()
            .map(|(i, _)| vec![1.0; i + 1])
            .collect())
    }
}

/// Returns fewer vectors than it was given texts.
struct ShortEmbedder;

impl Embedder for ShortEmbedder {
    fn model_id(&self) -> String {
        "short".into()
    }

    fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(vec![vec![1.0]])
    }
}

#[test]
fn a_misbehaving_embedder_is_an_index_error() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let ragged = Index::open_and_sync(&idx, &bundle, Some(&RaggedEmbedder)).unwrap_err();
    assert!(matches!(ragged, Error::Index { .. }));
    assert!(ragged.to_string().contains("differing widths"));

    let short = Index::open_and_sync(&idx, &bundle, Some(&ShortEmbedder)).unwrap_err();
    assert_eq!(
        short.to_string(),
        "index: embedder returned 1 vectors for 3 sections"
    );
}

#[test]
fn model_change_forces_full_rebuild() {
    let (dir, bundle) = fixture();
    let idx = IndexDir(dir.path().join("idx"));
    let (first, _) = Index::open_and_sync(&idx, &bundle, Some(&FakeEmbedder)).unwrap();
    drop(first);

    let (index, stats) = Index::open_and_sync(&idx, &bundle, None).unwrap();
    assert!(stats.full_rebuild);
    assert!(stats.lexical_only);
    assert_eq!(stats.reindexed, 2);
    assert_eq!(index.section_count(), 3);
    // No embedder, so no vectors — and the manifest says so.
    assert_eq!(index.vector_count(), 0);
    assert_eq!(index.model_id(), None);
}
