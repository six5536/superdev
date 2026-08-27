//! check.rs — the per-kind and cross-file checks.
//!
//! A behavioural port of the Node reference. Every finding's text is the
//! contract: `tests/format_parity.rs` holds the reference's output as goldens,
//! and the wording is compared verbatim.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use super::grammar::Grammar;
use super::read::{fence_map, prose_only};

/// Every tag the scanner reads, shared by the unit and core checks.
static TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<(/)?([A-Za-z_][\w-]*)((?:\s+[\w-]+\s*=\s*"[^"]*")*)\s*(/)?>"#).unwrap()
});

/// The core file: an H1, balanced block tags, and the block names every other
/// file may refer to.
///
/// Returns the block names it defines, which the reference collects into a
/// process-wide set and the port hands back instead — a validator that has to
/// be run twice in one process should not remember the first run's core.
pub fn check_core(text: &str, errs: &mut Vec<String>, g: &Grammar) -> BTreeSet<String> {
    static BLOCK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"<([a-z][a-z0-9_]*)(?:\s[^>]*)?>").unwrap());

    let k = &g.kinds.core;
    let mut blocks = BTreeSet::new();
    if k.collect_blocks {
        for c in BLOCK.captures_iter(text) {
            blocks.insert(c[1].to_string());
        }
    }

    let lines: Vec<&str> = text.split('\n').collect();
    let fenced = fence_map(&lines);
    if k.require_h1
        && !lines
            .iter()
            .enumerate()
            .any(|(i, l)| !fenced[i] && l.starts_with("# "))
    {
        errs.push("core: missing H1".to_string());
    }
    if !k.balanced_tags {
        return blocks;
    }

    let prose = prose_only(&lines, &fenced);
    let mut stack: Vec<String> = Vec::new();
    for m in TAG.captures_iter(&prose) {
        if m.get(4).is_some() {
            continue;
        }
        let name = m[2].to_string();
        if m.get(1).is_some() {
            if stack.last() != Some(&name) {
                errs.push(format!("core: unbalanced </{name}>"));
                return blocks;
            }
            stack.pop();
        } else {
            stack.push(name);
        }
    }
    if let Some(open) = stack.last() {
        errs.push(format!("core: unclosed <{open}>"));
    }
    blocks
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::format::parse_grammar;

    fn grammar() -> Grammar {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "../../..",
            ".agents/format/grammar.yaml",
        ]
        .iter()
        .collect();
        parse_grammar(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn the_live_core_passes_and_defines_its_blocks() {
        let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "../../..", ".agents/core.md"]
            .iter()
            .collect();
        let text = std::fs::read_to_string(path).unwrap();
        let mut errs = Vec::new();
        let blocks = check_core(&text, &mut errs, &grammar());
        assert!(errs.is_empty(), "{errs:?}");
        assert!(blocks.contains("workflow"), "{blocks:?}");
        assert!(blocks.contains("core_principles"), "{blocks:?}");
    }

    #[test]
    fn a_core_with_no_h1_and_an_unclosed_block_is_reported() {
        let mut errs = Vec::new();
        check_core(
            "<superdev>\n<workflow>\n</superdev>\n",
            &mut errs,
            &grammar(),
        );
        assert_eq!(errs, ["core: missing H1", "core: unbalanced </superdev>"]);
    }

    #[test]
    fn an_unclosed_block_at_the_end_is_reported() {
        let mut errs = Vec::new();
        check_core("# T\n\n<superdev>\n", &mut errs, &grammar());
        assert_eq!(errs, ["core: unclosed <superdev>"]);
    }
}
