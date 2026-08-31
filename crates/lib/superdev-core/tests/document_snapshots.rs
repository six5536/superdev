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

use superdev_core::validate::schema::document::{Document, SchemaSet, check_documents};

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
