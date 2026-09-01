//! contract_interfaces.rs — the internal interface contracts bound to the
//! code they describe.
//!
//! [ADR-036] obliges a project to bind its implemented interface to the
//! contract's declared surface, and leaves the mechanism to the project.
//! Nothing introspects a Rust API at runtime, so this binds by text: every
//! item an interface contract declares — a signature, a struct field, an
//! enum variant — must exist in the crate's production source, so it cannot
//! be renamed or retyped without its contract moving with it. What a
//! contract does not name, it does not bind.

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

/// What a declared line binds, or `None` when it declares nothing.
///
/// An item's head runs to the body, so `pub fn f(a: A) -> B;` and the same
/// line opening a block compare equal. A field binds whole, up to its
/// trailing comma; a variant binds by name, because its own fields bind as
/// fields. A line carrying an elision — `/* … */` — binds only what precedes
/// it, because the contract deliberately left the rest out.
fn bound(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // An elision ends the binding: what follows it is explicitly not bound.
    let trimmed = trimmed.split("/*").next().unwrap_or(trimmed).trim();
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
    if opens.iter().any(|o| trimmed.starts_with(o)) {
        return Some(flat(&trimmed[..head_end(trimmed)]));
    }
    // A field: `pub name: Type`, up to its trailing comma.
    if trimmed.starts_with("pub ") && trimmed.contains(':') {
        let text = flat(trimmed.trim_end_matches(','));
        return (text.len() > 3).then_some(text);
    }
    // A variant: an identifier opening a body, or standing alone. Its own
    // fields bind as fields, so the name is what this line adds.
    let name: String = trimmed
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let rest = trimmed[name.len()..].trim_start();
    let is_variant = name.starts_with(|c: char| c.is_ascii_uppercase())
        && (rest.is_empty() || rest.starts_with([',', '{', '(']));
    (is_variant && name.len() > 1).then_some(name)
}

/// Where an item's head ends: the body, the value or the terminator, at
/// bracket depth zero. `->` is an arrow, never a closing bracket.
fn head_end(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    for (i, c) in text.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' if i > 0 && bytes[i - 1] == b'-' => {}
            '>' | ')' | ']' => depth -= 1,
            '{' | ';' | '=' if depth == 0 => return i,
            _ => {}
        }
    }
    text.len()
}

/// Every line of every `rust` block in `text`, with wrapped lines joined so a
/// signature over three lines binds as one.
fn rust_lines(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut inside = false;
    let mut pending = String::new();
    let close = |pending: &mut String, out: &mut Vec<String>| {
        if !pending.is_empty() {
            out.push(std::mem::take(pending));
        }
    };
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```rust") {
            inside = true;
            continue;
        }
        if inside && trimmed.starts_with("```") {
            close(&mut pending, &mut out);
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        // A line whose brackets do not balance continues on the next.
        if pending.is_empty() {
            pending = line.to_string();
        } else {
            pending.push(' ');
            pending.push_str(line.trim());
        }
        if depth_of(&pending) <= 0 {
            close(&mut pending, &mut out);
        }
    }
    close(&mut pending, &mut out);
    out
}

/// A text's bracket depth, with `->` read as an arrow.
fn depth_of(text: &str) -> i32 {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    for (i, c) in text.char_indices() {
        match c {
            '(' | '<' => depth += 1,
            '>' if i > 0 && bytes[i - 1] == b'-' => {}
            ')' | '>' => depth -= 1,
            _ => {}
        }
    }
    depth
}

/// The crate's production source, flattened: test modules and comment lines
/// are dropped, so a declaration cannot be satisfied by a test fixture or by
/// prose in a comment.
fn production_source(dir: &Path, out: &mut String) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // `tests/` holds no production source.
            if path.file_name().is_some_and(|n| n == "tests") {
                continue;
            }
            production_source(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let mut in_tests = false;
            let mut depth = 0i32;
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("#[cfg(test)]") {
                    in_tests = true;
                    depth = 0;
                    continue;
                }
                if in_tests {
                    depth += line.matches('{').count() as i32;
                    depth -= line.matches('}').count() as i32;
                    if depth <= 0 && line.contains('}') {
                        in_tests = false;
                    }
                    continue;
                }
                if trimmed.starts_with("//") {
                    continue;
                }
                out.push_str(&flat(line));
                out.push(' ');
            }
        }
    }
}

/// Covers I035 criteria 4 and 8: every item, field and variant the internal
/// interface contracts declare exists in the production source, so a rename
/// or a retype fails until its contract moves with it.
#[test]
fn every_declared_signature_exists_in_the_source() {
    let mut haystack = String::new();
    production_source(&repo("crates"), &mut haystack);

    let dir = repo("knowledge/contracts/internal/active");
    let mut checked = 0;
    let mut unbuilt: BTreeSet<String> = BTreeSet::new();
    for entry in std::fs::read_dir(&dir).expect("the internal contracts are on file") {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        for line in rust_lines(&text) {
            let Some(bound) = bound(&line) else { continue };
            checked += 1;
            if !haystack.contains(&bound) {
                unbuilt.insert(format!("{name}: {bound}"));
            }
        }
    }
    assert!(
        checked >= 60,
        "the contracts declare far fewer items than expected: {checked}"
    );
    // ADR-038: a declared element the code lacks is a promise still
    // outstanding, not a defect.
    assert!(
        unbuilt.is_empty(),
        "PENDING — declared in a contract and absent from the production source:\n{}",
        unbuilt.into_iter().collect::<Vec<String>>().join("\n")
    );
}
