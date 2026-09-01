//! contract_interfaces.rs — the internal interface contracts bound to the
//! code they describe.
//!
//! [ADR-036] obliges a project to bind its implemented interface to the
//! contract's declared surface, and leaves the mechanism to the project.
//! Nothing introspects a Rust API at runtime, so this binds by text: every
//! item an interface contract declares must exist in the crate source, so a
//! signature cannot be renamed or retyped without its contract moving with
//! it. What a contract does not name, it does not bind.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../..", path]
        .iter()
        .collect()
}

/// Whitespace collapsed, so a wrapped signature and a one-line one compare
/// equal.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// An item declaration's head: everything before the body, the value or the
/// terminator, so a contract may elide a body with `{ /* … */ }`.
fn head(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let opens = [
        "pub fn ",
        "pub const fn ",
        "pub async fn ",
        "pub struct ",
        "pub enum ",
        "pub trait ",
        "pub type ",
        "pub const ",
        "pub static ",
    ];
    if !opens.iter().any(|o| trimmed.starts_with(o)) {
        return None;
    }
    let cut = trimmed
        .find(['{', '=', ';'])
        .unwrap_or(trimmed.len());
    let head = flat(&trimmed[..cut]);
    (!head.is_empty()).then_some(head)
}

/// Every line of every `rust` block in `text`.
fn rust_lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```rust") {
            inside = true;
            continue;
        }
        if inside && trimmed.starts_with("```") {
            inside = false;
            continue;
        }
        if inside {
            out.push(line.to_string());
        }
    }
    out
}

/// Every `.rs` file under `dir`, flattened, as one haystack.
fn sources(dir: &Path, out: &mut String) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push_str(&flat(&std::fs::read_to_string(&path).unwrap_or_default()));
            out.push(' ');
        }
    }
}

/// Covers I035 criteria 4 and 8: every item the internal interface contracts
/// declare exists in the crate source, so a renamed or retyped signature
/// fails until its contract moves with it.
#[test]
fn every_declared_signature_exists_in_the_source() {
    let mut haystack = String::new();
    sources(&repo("crates"), &mut haystack);

    let dir = repo("knowledge/contracts/internal/active");
    let mut checked = 0;
    let mut missing: BTreeSet<String> = BTreeSet::new();
    for entry in std::fs::read_dir(&dir).expect("the internal contracts are on file") {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        for line in rust_lines(&text) {
            let Some(head) = head(&line) else { continue };
            checked += 1;
            if !haystack.contains(&head) {
                missing.insert(format!("{name}: {head}"));
            }
        }
    }
    assert!(checked >= 30, "the contracts declare items: {checked}");
    assert!(
        missing.is_empty(),
        "declared in a contract and absent from the source:\n{}",
        missing.into_iter().collect::<Vec<String>>().join("\n")
    );
}
