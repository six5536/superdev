//! Identifier and path priority is distinct from hybrid relevance.
use std::fs;

use super::*;
use crate::sokf::{bundle::load_bundle, embed::FakeEmbedder};

fn fixture(ambiguous: bool) -> (tempfile::TempDir, Index) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("manuals");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("contract.md"), "---\ntype: Contract\nid: contract-002-cli\nlifecycle: deprecated\ntags: [cli]\n---\nThe normative interface.\n").unwrap();
    fs::write(root.join("note.md"), "---\ntype: Note\nid: commentary\n---\ncontract-002 contract-002-cli contract-002 contract-002-cli discussion\n").unwrap();
    if ambiguous {
        fs::write(
            root.join("other.md"),
            "---\ntype: Contract\nid: contract-002-other\n---\nAnother interface.\n",
        )
        .unwrap();
    }
    let bundle = load_bundle(&root).unwrap();
    let (index, _) = Index::open_and_sync(
        &IndexDir(dir.path().join("index")),
        &bundle,
        Some(&FakeEmbedder),
    )
    .unwrap();
    (dir, index)
}

/// contract-003-api-sokf P_exact-address-priority and P_settled-down-ranked.
#[test]
fn a_named_contract_beats_discussion_even_when_deprecated_and_outside_lexical_candidates() {
    let (_dir, index) = fixture(false);
    for query in ["contract-002", "what does `contract-002-cli` promise?"] {
        let hits = index
            .search(
                query,
                Some(&FakeEmbedder),
                &SearchOpts {
                    limit: 1,
                    ..SearchOpts::default()
                },
            )
            .unwrap();
        assert_eq!(hits[0].concept_id.as_deref(), Some("contract-002-cli"));
        assert_eq!(hits[0].match_kind, MatchKind::Identifier);
    }
}

#[test]
fn a_physical_path_matches_in_a_custom_knowledge_directory() {
    let (dir, index) = fixture(false);
    for query in [
        "contract.md".to_string(),
        "locate manuals/contract.md".to_string(),
        dir.path().join("manuals/contract.md").display().to_string(),
    ] {
        let hits = index
            .search(
                &query,
                None,
                &SearchOpts {
                    limit: 1,
                    ..SearchOpts::default()
                },
            )
            .unwrap();
        assert_eq!(hits[0].concept_id.as_deref(), Some("contract-002-cli"));
        assert_eq!(hits[0].match_kind, MatchKind::Path);
    }
}

/// contract-003-api-sokf AC_ambiguous-shorthand.
#[test]
fn shorthand_is_unambiguous_and_identifiers_have_token_boundaries() {
    let (_dir, index) = fixture(true);
    for query in [
        "contract-002",
        "contract-0020",
        "contract-002-cli-extra",
        "other/manuals/contract.md",
    ] {
        assert!(
            super::direct::matches(query, &index.identities, &index.bundle_root).is_empty(),
            "{query}"
        );
    }
    let matches = super::direct::matches("contract-002-cli", &index.identities, &index.bundle_root);
    assert_eq!(matches.get("contract.md"), Some(&MatchKind::Identifier));
}

/// contract-003-api-sokf AC_single-word-id.
#[test]
fn single_word_ids_do_not_override_natural_language_queries() {
    let (_dir, index) = fixture(false);
    assert!(
        super::direct::matches(
            "commentary on module rules",
            &index.identities,
            &index.bundle_root
        )
        .is_empty()
    );
    assert_eq!(
        super::direct::matches("commentary", &index.identities, &index.bundle_root).get("note.md"),
        Some(&MatchKind::Identifier)
    );
}

/// contract-003-api-sokf AC_complete-path.
#[test]
fn paths_preserve_spaces_and_punctuation_instead_of_matching_fragments() {
    let root = std::path::Path::new("/tmp/project with spaces/manuals");
    let identities = std::collections::BTreeMap::from([
        ("guides/rust+api notes.md".to_string(), None),
        ("notes.md".to_string(), Some("tail".to_string())),
        ("(notes).md".to_string(), None),
    ]);
    for query in [
        "guides/rust+api notes.md",
        "./guides/rust+api notes.md",
        "/tmp/project with spaces/manuals/guides/rust+api notes.md",
        "locate `manuals/guides/rust+api notes.md`",
        "locate \"manuals/guides/rust+api notes.md\"",
    ] {
        let matches = super::direct::matches(query, &identities, root);
        assert_eq!(
            matches,
            std::collections::BTreeMap::from([("guides/rust+api notes.md", MatchKind::Path)]),
            "{query}"
        );
    }
    for query in ["(notes).md", "locate `(notes).md`", "locate notes.md?"] {
        let expected = if query.ends_with('?') {
            "notes.md"
        } else {
            "(notes).md"
        };
        assert_eq!(
            super::direct::matches(query, &identities, root),
            std::collections::BTreeMap::from([(expected, MatchKind::Path)])
        );
    }
    // A nonexistent path must not promote its tail as a different document.
    assert!(super::direct::matches("guides/not+notes.md", &identities, root).is_empty());
}

#[cfg(windows)]
#[test]
fn physical_windows_paths_use_native_separators() {
    let root = std::path::Path::new(r"C:\project with spaces\manuals");
    let identities = std::collections::BTreeMap::from([("guides/file.md".to_string(), None)]);
    assert_eq!(
        super::direct::matches(
            r"C:\project with spaces\manuals\guides\file.md",
            &identities,
            root
        ),
        std::collections::BTreeMap::from([("guides/file.md", MatchKind::Path)])
    );
}

/// contract-003-api-sokf AC_query-recovery-bounded.
#[test]
fn recovered_syntax_notes_are_distinct_and_capped() {
    let (_dir, index) = fixture(false);
    let report = index
        .search_report(
            "one:contract-002 two:contract-002 three:contract-002 four:contract-002",
            None,
            &SearchOpts::default(),
        )
        .unwrap();
    assert_eq!(report.warnings.len(), 3);
    assert_eq!(
        report
            .warnings
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3
    );
    assert_eq!(
        report.hits[0].concept_id.as_deref(),
        Some("contract-002-cli")
    );
}

/// contract-003-api-sokf P_filters-before-fusion.
#[test]
fn every_explicit_filter_still_excludes_a_direct_match() {
    let (_dir, index) = fixture(false);
    for opts in [
        SearchOpts {
            kinds: vec!["Note".into()],
            ..SearchOpts::default()
        },
        SearchOpts {
            tags: vec!["other".into()],
            ..SearchOpts::default()
        },
        SearchOpts {
            lifecycle: vec!["active".into()],
            ..SearchOpts::default()
        },
    ] {
        let hits = index
            .search("contract-002", Some(&FakeEmbedder), &opts)
            .unwrap();
        assert!(
            hits.iter()
                .all(|hit| hit.concept_id.as_deref() != Some("contract-002-cli"))
        );
    }
}
