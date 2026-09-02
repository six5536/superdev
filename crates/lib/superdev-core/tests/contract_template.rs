//! contract_template.rs — the template format contract bound to the token
//! vocabulary and the shipped set the binary substitutes.
//!
//! The last hand-written definition on file: contract-008's tokens and
//! template set are authored blocks, so a test compares them to the source
//! (ADR-036). Moved here from `contract_files.rs` when the config and format
//! contracts became source includes (ADR-042, P024 S7); the template contract
//! follows in slice 9, and this file goes with it.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../..", path]
        .iter()
        .collect()
}

/// The first fenced block carrying `tag`, without its markers.
///
/// Scanned a line at a time, so a CRLF checkout reads as its LF twin: a
/// fence is a fence whatever ends the line (I040). The closing marker is the
/// one that opened the block, so a ```` ```` ```` block may contain ``` ``` ````.
fn fenced_block(text: &str, tag: &str) -> Option<String> {
    let mut lines = text.lines();
    let marker = loop {
        let trimmed = lines.next()?.trim_start();
        let ticks = trimmed.len() - trimmed.trim_start_matches('`').len();
        if ticks >= 3 && trimmed[ticks..].trim() == tag {
            break trimmed[..ticks].to_string();
        }
    };
    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with(&marker) {
            return Some(body.join("\n"));
        }
        body.push(line);
    }
    None
}

/// The first fenced block tagged `tag` in the contract at `path`.
fn block(path: &str, tag: &str) -> String {
    let text = std::fs::read_to_string(repo(path)).expect("the contract is on file");
    fenced_block(&text, tag).unwrap_or_else(|| panic!("{path} carries no {tag} block"))
}

/// The template format contract, whose surface is a token vocabulary and a
/// shipped set rather than a file a reader parses.
const TEMPLATE_CONTRACT: &str =
    "knowledge/contracts/public/active/contract-008-text-format-template.md";

/// Every `{{superdev:…}}` token in `text`, in the order written.
fn tokens_in(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = text;
    while let Some(at) = rest.find("{{superdev:") {
        rest = &rest[at..];
        let Some(end) = rest.find("}}") else { break };
        out.insert(rest[..end + 2].to_string());
        rest = &rest[end + 2..];
    }
    out
}

/// Covers I035 criteria 4 and 12 (I038): the token vocabulary the template
/// format contract declares is the one the binary substitutes.
///
/// The implemented side is read from the source rather than listed here, so a
/// sixth constant is caught: a list written in the test would only ever agree
/// with itself.
#[test]
fn every_substitution_token_is_declared_and_every_declared_token_is_real() {
    let source = std::fs::read_to_string(repo("crates/lib/superdev-core/src/templates.rs"))
        .expect("the template engine is on file");
    let implemented: BTreeSet<String> = source
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("pub const TOKEN_")?;
            let value = rest.split_once('=')?.1.trim();
            Some(value.trim_matches(|c| c == '"' || c == ';').to_string())
        })
        .collect();
    assert!(
        implemented.len() >= 5,
        "the token constants were not found in the source: {implemented:?}"
    );

    let declared = tokens_in(&block(TEMPLATE_CONTRACT, "text"));
    let undeclared: Vec<&String> = implemented.difference(&declared).collect();
    assert!(
        undeclared.is_empty(),
        "DEFECT — the binary substitutes tokens the contract does not declare: {undeclared:?}"
    );
    let unbuilt: Vec<&String> = declared.difference(&implemented).collect();
    assert!(
        unbuilt.is_empty(),
        "PENDING — the contract declares tokens the binary does not substitute: {unbuilt:?}"
    );
}

/// Covers I035 criteria 4 and 12 (I038): the contract carries one section per
/// shipped template, which is what it promises the sections do.
#[test]
fn every_shipped_template_has_its_section_and_every_section_a_template() {
    let shipped: BTreeSet<String> = superdev_core::templates::shipped()
        .iter()
        .map(|t| t.name.to_string())
        .collect();
    assert!(
        !shipped.is_empty(),
        "the binary ships no template, so this binds nothing"
    );

    let text = std::fs::read_to_string(repo(TEMPLATE_CONTRACT)).expect("the contract is on file");
    let documented: BTreeSet<String> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("### Template: "))
        .map(|name| name.trim().to_string())
        .collect();

    let undocumented: Vec<&String> = shipped.difference(&documented).collect();
    assert!(
        undocumented.is_empty(),
        "DEFECT — the binary ships templates the contract does not describe: {undocumented:?}"
    );
    let unbuilt: Vec<&String> = documented.difference(&shipped).collect();
    assert!(
        unbuilt.is_empty(),
        "PENDING — the contract describes templates the binary does not ship: {unbuilt:?}"
    );
}
