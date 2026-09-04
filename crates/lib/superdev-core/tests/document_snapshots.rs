//! document_snapshots.rs — `validate::schema::document` over one fixture case
//! per rule, compared to a recorded report.
//!
//! Each `tests/fixtures/documents/<case>/` holds a `schemas/` directory and
//! the documents it governs; `<case>.golden.json` holds what the checks
//! report for them. The inline tests in `document.rs` pin behaviour — that a
//! missing section is found at all; these pin the wording a reader sees,
//! which is the part that has no other test and the part most likely to be
//! changed without thinking.
//!
//! Regenerate with:
//!
//! ```sh
//! UPDATE_GOLDENS=1 cargo test -p superdev-core --test document_snapshots
//! ```
//!
//! Then read the diff. A reworded message shows up as a wording diff; a
//! finding that appears or vanishes is a behaviour change, and wants the
//! argument any behaviour change wants.

use std::path::{Path, PathBuf};

use superdev_core::validate::schema::document::{
    Document, SchemaSet, check_declarations, check_documents, check_examples,
};

/// The fixture root.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/documents")
}

/// Every `.md` directly under `dir`, sorted, as (name, text).
fn read_dir(dir: &Path) -> Vec<(String, String)> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                std::fs::read_to_string(&p).unwrap(),
            )
        })
        .collect()
}

/// The frontmatter `type`, when the document carries one.
fn doc_type(text: &str) -> Option<String> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    rest[..end]
        .lines()
        .find_map(|l| l.strip_prefix("type:"))
        .map(|v| v.trim().to_string())
}

/// Check `case` and compare the report to its golden, or rewrite the golden
/// when `UPDATE_GOLDENS` is set.
fn snapshot(case: &str) {
    let dir = fixtures().join(case);
    let schemas = read_dir(&dir.join("schemas"));
    let docs = read_dir(&dir);

    let (set, mut findings) = SchemaSet::load(&schemas);
    // `validate` reports unreadable declarations through the grammar's own
    // schema check; this harness runs only the document layer, so it asks
    // for them explicitly.
    findings.extend(check_declarations(&schemas));
    // Each schema's example against the schema declaring it (ADR-024).
    findings.extend(check_examples(&schemas));
    // The types are held here so the documents can borrow them: a `Document`
    // borrows its type, and leaking one per case to satisfy the lifetime
    // would be a leak in the test rather than a fix to it.
    let types: Vec<Option<String>> = docs.iter().map(|(_, text)| doc_type(text)).collect();
    let candidates: Vec<Document<'_>> = docs
        .iter()
        .zip(&types)
        .map(|((name, text), doc_type)| Document {
            path: name,
            text,
            doc_type: doc_type.as_deref(),
        })
        .collect();
    findings.extend(check_documents(&candidates, &set));

    let ours = serde_json::json!({
        "passed": findings.iter().all(|f| !f.fatal),
        "findings": findings
            .iter()
            .map(|f| serde_json::json!({
                "severity": if f.fatal { "error" } else { "warning" },
                "file": f.file,
                "message": f.message,
            }))
            .collect::<Vec<_>>(),
    });

    let path = fixtures().join(format!("{case}.golden.json"));
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        let mut text = serde_json::to_string_pretty(&ours).unwrap();
        text.push('\n');
        std::fs::write(&path, text).unwrap();
        return;
    }
    let golden: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(ours, golden, "{case} does not match its golden");
}

#[test]
fn missing_section() {
    snapshot("missing-section");
}

#[test]
fn misordered_section() {
    snapshot("misordered-section");
}

#[test]
fn prohibited_section() {
    snapshot("prohibited-section");
}

#[test]
fn wrong_columns() {
    snapshot("wrong-columns");
}

#[test]
fn over_limit() {
    snapshot("over-limit");
}

#[test]
fn unknown_type() {
    snapshot("unknown-type");
}

#[test]
fn duplicate_type() {
    snapshot("duplicate-type");
}

#[test]
fn governs_nothing() {
    snapshot("governs-nothing");
}

#[test]
fn missing_content() {
    snapshot("missing-content");
}

#[test]
fn unknown_content_kind() {
    snapshot("unknown-content-kind");
}

#[test]
fn frontmatter_mismatch() {
    snapshot("frontmatter-mismatch");
}

#[test]
fn uncompilable_pattern() {
    snapshot("uncompilable-pattern");
}

#[test]
fn missing_required_key() {
    snapshot("missing-required-key");
}

#[test]
fn broken_example() {
    snapshot("broken-example");
}

/// A schema declaring variants (ADR-045): a kind `a` document lacking the
/// section tagged `[a]` fails, a kind `b` one passes with it out of order,
/// and a mis-declared schema reports each fault on the schema file.
#[test]
fn variant_rules() {
    snapshot("variant-rules");
}

/// A schema declaring one heading per variant (ADR-049): an unframed
/// document with plain criteria passes, a framed one with a keyless item
/// fails naming it, a framed one with the heading out of order fails, and a
/// schema whose two rules for a heading overlap, or of which one is
/// untagged, reports each pair on the schema file and binds nothing.
#[test]
fn per_variant_heading() {
    snapshot("per-variant-heading");
}

/// A schema declaring `item-key` (ADR-047): a malformed key, a missing one
/// and a repeat across sections each report on the document, the same key
/// in another document reports nothing, and a key with no capture group or
/// on a prose section reports on the schema.
#[test]
fn item_key() {
    snapshot("item-key");
}

/// A schema declaring `item-only-pattern` and `item-prohibited-pattern`
/// (ADR-047): a modal verb in prose, a table row, a numbered item and a
/// subsection heading under a bullet-list rule, and in a bullet under a
/// prose rule, each report the section and the line; a retired verb and a
/// two-verb item each report the item and the matched text; a `PENDING`
/// item and a `SHALL NOT` item report nothing; a prohibited pattern on a
/// prose section and an uncompilable bound report on the schema.
#[test]
fn item_bounds() {
    snapshot("item-bounds");
}

/// A schema declaring `nested` and `item-key-optional` (ADR-051): a nested
/// item with no key, no tag or a retired verb each report the nested item;
/// a promise with no criterion beneath it reports the promise; a key
/// repeated between a promise and a criterion reports both; a plain note
/// under the optional key reports nothing unless it matches the prohibited
/// pattern, and a keyed one is held to the pattern; a marker beyond the
/// declared depth or of the other kind is text; and a `nested` on a prose
/// section, a nested key with two captures and the flag with no key report
/// on the schema.
#[test]
fn nested_items() {
    snapshot("nested-items");
}
