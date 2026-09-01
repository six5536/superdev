//! fragments.rs — the live repository's contract-style fragment and its
//! carriers: every contract-kind schema materializes the standard, so the
//! one read a contract writer is guaranteed to make carries it.

use std::path::PathBuf;

use superdev_core::sokf::included_text;

fn repo(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../..", path]
        .iter()
        .collect()
}

/// `text` with its line endings made uniform, for comparison against a
/// literal written with LF.
///
/// The comparison is where CRLF and LF are made the same — nothing normalises
/// on the way in, so what these tests read is what is on disk (I040). The
/// product needs none of this: its checks read a line at a time through
/// `validate::lines`, and a line is the same line whatever ends it.
fn same(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// The standard's source states the definition rules and the binding
/// obligation (ADR-033, ADR-036).
#[test]
fn the_fragment_states_the_definition_rules() {
    let text = std::fs::read_to_string(repo("knowledge/schemas/fragments/contract-style.md"))
        .expect("the fragment ships with the schema set");
    for rule in [
        "a contract defines its interface",
        "from the contract alone",
        "MUST describe and MUST NOT define",
        "MUST NOT state how the",
        "generating the surface from it or by a test",
        "MUST be proved\n  current",
        "MUST NOT\n  restate the ADR's reasoning",
    ] {
        assert!(same(&text).contains(rule), "missing: {rule}");
    }
}

/// Every contract-kind schema carries the fragment's current body between
/// its include markers, in the live tree and in the pack mirror.
#[test]
fn every_contract_schema_materializes_the_standard() {
    let source = same(
        &std::fs::read_to_string(repo("knowledge/schemas/fragments/contract-style.md")).unwrap(),
    );
    let body = source
        .splitn(3, "---\n")
        .nth(2)
        .expect("frontmatter closes");
    let content = included_text(body);
    assert!(!content.is_empty());

    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        let dir = repo(root);
        let mut seen = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if !name.starts_with("contract-") {
                continue;
            }
            seen += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            assert!(
                same(&text).contains(&content),
                "{root}/{name} does not carry the materialized standard"
            );
        }
        assert_eq!(seen, 16, "{root}: every contract kind carries it");
    }
}
