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

    let first = fix_repo(dir.path(), &knowledge, &[]).unwrap();
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

    let second = fix_repo(dir.path(), &knowledge, &[]).unwrap();
    assert!(second.written.is_empty(), "{:?}", second.written);
    assert_tree(dir.path(), AFTER);
}

/// The point of the pass: the tree it leaves has nothing left to report.
#[test]
fn the_repaired_tree_validates_clean() {
    let dir = seed(BEFORE);
    let knowledge = dir.path().join("knowledge");
    fix_repo(dir.path(), &knowledge, &[]).unwrap();

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

    fix_repo(dir.path(), &knowledge, &[]).unwrap();
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

    fix_repo(dir.path(), &knowledge, &[]).unwrap();
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

    fix_repo(dir.path(), &dir.path().join("knowledge"), &[]).unwrap();

    for (path, was) in outside.iter().zip(before) {
        assert_eq!(std::fs::read_to_string(dir.path().join(path)).unwrap(), was);
    }
}

/// `--fix` covers the knowledge on the same condition the check reports its
/// findings: a path naming a file inside the knowledge repairs it — the
/// check reports that file's link findings, so the repair the findings
/// point at must be reachable — while a path naming something else repairs
/// nothing.
#[test]
fn a_path_inside_the_knowledge_covers_the_fix() {
    let dir = seed(BEFORE);
    let knowledge = dir.path().join("knowledge");

    let outside = [std::path::PathBuf::from("README.md")];
    let repair = fix_repo(dir.path(), &knowledge, &outside).unwrap();
    assert!(repair.written.is_empty(), "{:?}", repair.written);

    let inside = [std::path::PathBuf::from("knowledge/alpha.md")];
    let repair = fix_repo(dir.path(), &knowledge, &inside).unwrap();
    assert_eq!(
        repair.written,
        vec![
            "knowledge/alpha.md".to_string(),
            "knowledge/index.md".into(),
            "knowledge/sub/beta.md".into(),
        ],
        "the named run's repair is the bare run's"
    );
    assert_tree(dir.path(), AFTER);
}

/// A knowledge directory that is not there is not an error: `--fix` on a
/// repository without one has nothing to repair.
#[test]
fn a_missing_knowledge_directory_repairs_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let repair = fix_repo(dir.path(), &dir.path().join("knowledge"), &[]).unwrap();
    assert!(repair.written.is_empty());
}

/// D-8's fallback: a path a rename left behind still names a concept through
/// its stem, and the pass repairs it. A path naming nothing stays a broken
/// link, and an image stays an image.
#[test]
fn a_stale_path_is_repaired_through_its_stem() {
    let dir = seed(&[
        ("README.md", "readme\n"),
        (
            "knowledge/manifest.sokf.yaml",
            "sokf: \"0.4\"\nname: fixture\n",
        ),
        (
            "knowledge/alpha.md",
            "---\ntype: Note\nid: alpha\n---\n\n\
             Beta moved, so [beta](beta.md) resolves to nothing — but `beta` is still\n\
             its id. [Nothing](no-such-thing.md) names no concept, and\n\
             ![a diagram](diagram.png) is a picture.\n",
        ),
        (
            "knowledge/moved/beta.md",
            "---\ntype: Note\nid: beta\n---\n\nBody.\n",
        ),
    ]);
    let knowledge = dir.path().join("knowledge");

    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    let report = validate::validate(&bundle, dir.path());
    let messages: Vec<&str> = report.findings.iter().map(|f| f.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "body link names a concept by path: beta.md — write it as [sokf:beta]",
            "broken body link: no-such-thing.md",
            "broken body link: diagram.png",
        ],
        "the image is a broken link, never a concept named by path"
    );

    fix_repo(dir.path(), &knowledge, &[]).unwrap();
    let alpha = std::fs::read_to_string(knowledge.join("alpha.md")).unwrap();
    assert!(alpha.contains("[beta][sokf:beta]"), "{alpha}");
    assert!(alpha.contains("[Nothing](no-such-thing.md)"), "{alpha}");
    assert!(alpha.contains("![a diagram](diagram.png)"), "{alpha}");
    assert!(
        alpha.contains("[sokf:beta]: /knowledge/moved/beta.md"),
        "{alpha}"
    );
}

/// `--fix` covers the knowledge on the same condition the check does: a run
/// told to look at one directory elsewhere repairs nothing.
#[test]
fn a_named_path_that_is_not_the_knowledge_repairs_nothing() {
    let dir = seed(BEFORE);
    let knowledge = dir.path().join("knowledge");
    let before = std::fs::read_to_string(knowledge.join("alpha.md")).unwrap();

    let elsewhere = vec![dir.path().join("README.md")];
    let repair = fix_repo(dir.path(), &knowledge, &elsewhere).unwrap();
    assert!(repair.written.is_empty(), "{:?}", repair.written);
    assert_eq!(
        std::fs::read_to_string(knowledge.join("alpha.md")).unwrap(),
        before
    );

    // Naming the knowledge itself covers it, as it does for the check.
    let named = vec![knowledge.clone()];
    assert!(
        !fix_repo(dir.path(), &knowledge, &named)
            .unwrap()
            .written
            .is_empty()
    );
}

/// The include tree: a style source, a host with an empty include block, and
/// a second includer already carrying a stale copy.
const INCLUDES: &[(&str, &str)] = &[
    (
        "knowledge/manifest.sokf.yaml",
        "sokf: \"0.4\"\nname: fixture\n",
    ),
    (
        "knowledge/style.md",
        "---\ntype: Note\nid: style\n---\n\nCite [alpha][sokf:alpha].\n\n\
         <!-- sokf:links -->\n[sokf:alpha]: /knowledge/alpha.md\n",
    ),
    (
        "knowledge/alpha.md",
        "---\ntype: Note\nid: alpha\n---\n\nThe first.\n",
    ),
    (
        "knowledge/host.md",
        "---\ntype: Note\nid: host\n---\n\nIntro.\n\n\
         <!-- sokf:include style -->\n<!-- /sokf:include -->\n",
    ),
    (
        "knowledge/other.md",
        "---\ntype: Note\nid: other\n---\n\n\
         <!-- sokf:include style -->\nAn old copy.\n<!-- /sokf:include -->\n",
    ),
];

/// An empty include block is filled with the source's body — its definition
/// block excluded, its citations joining the host's own block — a stale copy
/// is refreshed, and a second run writes nothing.
#[test]
fn include_blocks_are_materialized_and_then_left_alone() {
    let dir = seed(INCLUDES);
    let knowledge = dir.path().join("knowledge");

    let first = fix_repo(dir.path(), &knowledge, &[]).unwrap();
    assert_eq!(
        first.written,
        vec!["knowledge/host.md".to_string(), "knowledge/other.md".into()]
    );
    let host = std::fs::read_to_string(knowledge.join("host.md")).unwrap();
    assert_eq!(
        host,
        "---\ntype: Note\nid: host\n---\n\nIntro.\n\n\
         <!-- sokf:include style -->\nCite [alpha][sokf:alpha].\n<!-- /sokf:include -->\n\n\
         <!-- sokf:links -->\n[sokf:alpha]: /knowledge/alpha.md\n"
    );
    let other = std::fs::read_to_string(knowledge.join("other.md")).unwrap();
    assert!(other.contains("Cite [alpha][sokf:alpha]."), "{other}");
    assert!(!other.contains("An old copy."), "{other}");

    let second = fix_repo(dir.path(), &knowledge, &[]).unwrap();
    assert!(second.written.is_empty(), "{:?}", second.written);

    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    let report = validate::validate(&bundle, dir.path());
    assert!(report.findings.is_empty(), "{:#?}", report.findings);
}

/// Editing the source stales every includer; the next pass rewrites them all.
#[test]
fn editing_the_source_rewrites_every_includer() {
    let dir = seed(INCLUDES);
    let knowledge = dir.path().join("knowledge");
    fix_repo(dir.path(), &knowledge, &[]).unwrap();

    std::fs::write(
        knowledge.join("style.md"),
        "---\ntype: Note\nid: style\n---\n\nNew rules.\n",
    )
    .unwrap();
    let repair = fix_repo(dir.path(), &knowledge, &[]).unwrap();
    assert_eq!(
        repair.written,
        vec!["knowledge/host.md".to_string(), "knowledge/other.md".into()]
    );
    for name in ["host.md", "other.md"] {
        let text = std::fs::read_to_string(knowledge.join(name)).unwrap();
        assert!(text.contains("New rules."), "{name}: {text}");
        assert!(!text.contains("Cite [alpha]"), "{name}: {text}");
    }
}

/// A marker fault elsewhere in the file does not freeze the repairs the
/// check promises: the stale well-formed block is refilled, and only the
/// faulty marker stays for the author.
#[test]
fn a_marker_fault_does_not_freeze_the_other_repairs() {
    let dir = seed(&[
        (
            "knowledge/manifest.sokf.yaml",
            "sokf: \"0.4\"\nname: fixture\n",
        ),
        (
            "knowledge/style.md",
            "---\ntype: Note\nid: style\n---\n\nThe rules.\n",
        ),
        (
            "knowledge/host.md",
            "---\ntype: Note\nid: host\n---\n\n\
             <!-- sokf:include style -->\nAn old copy.\n<!-- /sokf:include -->\n\n\
             A stray close below.\n\n<!-- /sokf:include -->\n",
        ),
    ]);
    let knowledge = dir.path().join("knowledge");

    let repair = fix_repo(dir.path(), &knowledge, &[]).unwrap();
    assert_eq!(repair.written, vec!["knowledge/host.md".to_string()]);
    let host = std::fs::read_to_string(knowledge.join("host.md")).unwrap();
    assert!(host.contains("The rules."), "{host}");
    assert!(!host.contains("An old copy."), "{host}");

    // The stray marker is the author's; the check still reports it, and a
    // second fix changes nothing.
    let bundle = superdev_core::sokf::load_bundle(&knowledge).unwrap();
    let report = validate::validate(&bundle, dir.path());
    assert_eq!(report.findings.len(), 1, "{:#?}", report.findings);
    assert!(report.findings[0].message.contains("no open marker"));
    let second = fix_repo(dir.path(), &knowledge, &[]).unwrap();
    assert!(second.written.is_empty(), "{:?}", second.written);
}
