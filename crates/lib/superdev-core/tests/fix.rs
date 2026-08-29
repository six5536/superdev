//! fix.rs — one knowledge tree carrying every repairable fault, repaired by
//! `validate::fix_repo`, compared against the tree it should have become.
//!
//! The inline tests in `validate/fix.rs` pin the pieces. This pins the whole
//! pass over a tree: what it converts, what it leaves alone, what it writes,
//! and what a second run does — which is nothing.

use std::path::Path;

use superdev_core::validate::{self, fix_repo};

/// The tree as it arrives: path links to concepts, a stale definition block,
/// and links to files that are no concepts at all.
const BEFORE: &[(&str, &str)] = &[
    ("README.md", "The repository's own readme.\n"),
    (
        "knowledge/manifest.sokf.yaml",
        "sokf: \"0.4\"\nname: fixture\n",
    ),
    (
        "knowledge/index.md",
        "# Core\n\n* [Alpha](alpha.md) - the first.\n* [Beta](sub/beta.md) - the second.\n",
    ),
    (
        "knowledge/alpha.md",
        "---\ntype: Note\nid: alpha\nlinks:\n  - rel: depends-on\n    to: beta\n---\n\n\
         Alpha depends on [beta](sub/beta.md) and on [beta again](/knowledge/sub/beta.md).\n\n\
         It also reads [the readme](/README.md), which is no concept, and cites\n\
         [a page](https://example.com).\n\n\
         <!-- sokf:links -->\n[sokf:beta]: /knowledge/gone.md\n[sokf:stray]: /knowledge/stray.md\n",
    ),
    (
        "knowledge/sub/beta.md",
        "---\ntype: Note\nid: beta\n---\n\nBeta names [alpha][sokf:alpha] already, and needs a block.\n",
    ),
];

/// The tree the pass must leave behind.
const AFTER: &[(&str, &str)] = &[
    ("README.md", "The repository's own readme.\n"),
    (
        "knowledge/manifest.sokf.yaml",
        "sokf: \"0.4\"\nname: fixture\n",
    ),
    (
        "knowledge/index.md",
        "# Core\n\n* [Alpha][sokf:alpha] - the first.\n* [Beta][sokf:beta] - the second.\n\n\
         <!-- sokf:links -->\n[sokf:alpha]: /knowledge/alpha.md\n[sokf:beta]: /knowledge/sub/beta.md\n",
    ),
    (
        "knowledge/alpha.md",
        "---\ntype: Note\nid: alpha\nlinks:\n  - rel: depends-on\n    to: beta\n---\n\n\
         Alpha depends on [beta][sokf:beta] and on [beta again][sokf:beta].\n\n\
         It also reads [the readme](/README.md), which is no concept, and cites\n\
         [a page](https://example.com).\n\n\
         <!-- sokf:links -->\n[sokf:beta]: /knowledge/sub/beta.md\n",
    ),
    (
        "knowledge/sub/beta.md",
        "---\ntype: Note\nid: beta\n---\n\nBeta names [alpha][sokf:alpha] already, and needs a block.\n\n\
         <!-- sokf:links -->\n[sokf:alpha]: /knowledge/alpha.md\n",
    ),
];

fn seed(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (path, text) in files {
        let file = dir.path().join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, *text).unwrap();
    }
    dir
}

fn assert_tree(root: &Path, expected: &[(&str, &str)]) {
    for (path, text) in expected {
        let found = std::fs::read_to_string(root.join(path)).unwrap();
        assert_eq!(&found, text, "{path}");
    }
}

#[test]
fn the_pass_repairs_a_tree_and_then_leaves_it_alone() {
    let dir = seed(BEFORE);
    let knowledge = dir.path().join("knowledge");

    let first = fix_repo(dir.path(), &knowledge).unwrap();
    assert_eq!(
        first.written,
        vec![
            "knowledge/alpha.md".to_string(),
            "knowledge/index.md".into(),
            "knowledge/sub/beta.md".into(),
        ],
        "every faulty document, named the way the report names it"
    );
    assert_tree(dir.path(), AFTER);

    let second = fix_repo(dir.path(), &knowledge).unwrap();
    assert!(second.written.is_empty(), "{:?}", second.written);
    assert_tree(dir.path(), AFTER);
}

/// The point of the pass: the tree it leaves has nothing left to report.
#[test]
fn the_repaired_tree_validates_clean() {
    let dir = seed(BEFORE);
    let knowledge = dir.path().join("knowledge");
    fix_repo(dir.path(), &knowledge).unwrap();

    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    let report = validate::validate(&bundle, dir.path());
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
}

/// Resolution reads the id and never the block (SPEC §9): a tree with every
/// block deleted still resolves every link, and reports only the blocks.
#[test]
fn deleting_every_block_breaks_no_link() {
    let dir = seed(AFTER);
    let knowledge = dir.path().join("knowledge");
    for (path, text) in AFTER {
        let Some((body, _)) = text.split_once("\n<!-- sokf:links -->") else {
            continue;
        };
        std::fs::write(knowledge.join(path.trim_start_matches("knowledge/")), body).unwrap();
    }

    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    let report = validate::validate(&bundle, dir.path());
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.message.contains("superdev validate --fix")),
        "{:#?}",
        report.findings
    );
    assert!(!report.findings.is_empty(), "the blocks are reported");
    assert!(!report.passed(), "a stale block fails the run");

    fix_repo(dir.path(), &knowledge).unwrap();
    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    assert!(
        validate::validate(&bundle, dir.path()).findings.is_empty(),
        "the pass puts them back"
    );
}

/// A concept renamed without touching its `id` keeps every link that names
/// it: only the blocks go stale, and the pass clears them.
#[test]
fn renaming_a_concept_breaks_no_link() {
    let dir = seed(AFTER);
    let knowledge = dir.path().join("knowledge");
    std::fs::rename(knowledge.join("alpha.md"), knowledge.join("renamed.md")).unwrap();

    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    let report = validate::validate(&bundle, dir.path());
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.message.contains("superdev validate --fix")),
        "{:#?}",
        report.findings
    );

    assert!(!report.passed(), "a stale block fails the run");

    fix_repo(dir.path(), &knowledge).unwrap();
    let beta = std::fs::read_to_string(knowledge.join("sub/beta.md")).unwrap();
    assert!(
        beta.contains("[sokf:alpha]: /knowledge/renamed.md"),
        "{beta}"
    );
    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    assert!(
        validate::validate(&bundle, dir.path()).findings.is_empty(),
        "nothing left to say"
    );
}

/// The pass writes inside the SOKF knowledge and nowhere else (NFR-1).
#[test]
fn nothing_outside_the_knowledge_is_written() {
    let dir = seed(BEFORE);
    let outside = ["README.md"];
    let before: Vec<String> = outside
        .iter()
        .map(|p| std::fs::read_to_string(dir.path().join(p)).unwrap())
        .collect();

    fix_repo(dir.path(), &dir.path().join("knowledge")).unwrap();

    for (path, was) in outside.iter().zip(before) {
        assert_eq!(std::fs::read_to_string(dir.path().join(path)).unwrap(), was);
    }
}

/// A knowledge directory that is not there is not an error: `--fix` on a
/// repository without one has nothing to repair.
#[test]
fn a_missing_knowledge_directory_repairs_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let repair = fix_repo(dir.path(), &dir.path().join("knowledge")).unwrap();
    assert!(repair.written.is_empty());
}
