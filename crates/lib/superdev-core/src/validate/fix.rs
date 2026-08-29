//! validate::fix — the repair pass behind `superdev validate --fix`.
//!
//! Two repairs, both mechanical, both derived from the same knowledge the
//! check reads: a body link that names a concept by path is rewritten to
//! address it by `id` (SPEC §8), and every document's generated definition
//! block is rewritten from the ids its body cites (SPEC §9).
//!
//! Nothing here decides anything. The check names a fault and this undoes it,
//! so a file the check has nothing to say about is not written at all — which
//! is what makes a second run a no-op rather than a diff.
//!
//! The pass writes only inside the SOKF knowledge. It is never wired into the
//! hook: the hook fires after an edit, so a hook that repaired would rewrite
//! the file the agent is still working in.

use std::collections::BTreeSet;
use std::path::Path;

use super::sokf::{
    ID_LABEL, Identities, canonical, definition_block, identities, link_path, render_block,
    resolve_target, scan_body,
};
use crate::error::{Error, Result};
use crate::sokf::bundle::Bundle;

/// What one repair pass changed.
#[derive(Debug, Default)]
pub struct Repair {
    /// Knowledge-relative paths rewritten, in path order.
    pub written: Vec<String>,
}

/// Repair every document in `bundle`, writing back the ones that changed.
///
/// `repo_root` is what a definition's path is rooted at (SPEC §9).
///
/// # Errors
///
/// A document cannot be read or written, or a repair resolved to a path
/// outside the SOKF knowledge — which is a bug, not a user error, and stops
/// the pass rather than writing.
pub fn fix(bundle: &Bundle, repo_root: &Path) -> Result<Repair> {
    // The check's findings are the caller's business; here they are the
    // by-product of building the id maps the repairs read.
    let ids = identities(bundle, repo_root, &mut Vec::new());
    let root = canonical(&bundle.root).unwrap_or_else(|| bundle.root.clone());

    let mut repair = Repair::default();
    let documents = bundle
        .concepts
        .iter()
        .map(|c| (c.path.as_str(), Some(c.body.as_str())))
        .chain(bundle.indexes.iter().map(|(p, _)| (p.as_str(), None)));
    let mut documents: Vec<(&str, Option<&str>)> = documents.collect();
    documents.sort_by_key(|(path, _)| *path);

    for (path, body) in documents {
        let file = bundle.root.join(path);
        let text = read(&file)?;
        // A concept's body is a verbatim suffix of its file, so the
        // frontmatter is everything before it and is never rewritten.
        let head = body.map_or(0, |body| text.len() - body.len());
        let fixed = format!(
            "{}{}",
            &text[..head],
            repair_body(&text[head..], &file, repo_root, &ids)
        );
        if fixed == text {
            continue;
        }
        write_within(&root, &file, &fixed)?;
        repair.written.push(path.to_string());
    }
    Ok(repair)
}

/// One document body, with its path links converted and its definition block
/// regenerated.
fn repair_body(body: &str, file: &Path, repo_root: &Path, ids: &Identities) -> String {
    let directory = file.parent().unwrap_or(file);
    let converted = convert_links(body, directory, repo_root, ids);
    let cited: BTreeSet<String> = scan_body(&converted).cited_ids();
    with_block(&converted, &render_block(&cited, &ids.repo_paths))
}

/// Every inline link that names a concept by path, rewritten to address it by
/// id.
///
/// Only an inline link is rewritten. A reference-style link carries its
/// destination in a definition the author wrote, and moving that is a
/// different edit from the one this pass makes; the check keeps naming it.
fn convert_links(body: &str, directory: &Path, repo_root: &Path, ids: &Identities) -> String {
    let mut edits: Vec<(std::ops::Range<usize>, String)> = Vec::new();
    for link in scan_body(body).links {
        if link.id.is_some() || !link.inline {
            continue;
        }
        let Some(file) = link_path(&link.dest) else {
            continue;
        };
        let Some(target) = resolve_target(file, directory, repo_root) else {
            continue;
        };
        let Some(id) = ids.by_path.get(&target) else {
            continue; // Not a concept: a source file or a README stays a path.
        };
        if let Some(text) = inline_text(body, &link.span) {
            edits.push((link.span.clone(), format!("[{text}][{ID_LABEL}{id}]")));
        }
    }

    let mut out = body.to_string();
    // Back to front, so an earlier edit does not move a later span.
    for (span, replacement) in edits.into_iter().rev() {
        out.replace_range(span, &replacement);
    }
    out
}

/// The link text of an inline `[text](dest)`, given the whole link's span.
///
/// Found from the right: the span ends at the closing `)`, and the last
/// `](` before it separates text from destination. A destination naming a
/// markdown file cannot contain that pair, and a span this does not fit is
/// left alone rather than guessed at.
fn inline_text<'a>(body: &'a str, span: &std::ops::Range<usize>) -> Option<&'a str> {
    let whole = body.get(span.clone())?;
    if !whole.starts_with('[') || !whole.ends_with(')') {
        return None;
    }
    let separator = whole.rfind("](")?;
    Some(&whole[1..separator])
}

/// A body's text with `block` as its definition block, replacing whatever
/// block it had.
///
/// A body already carrying that block is returned untouched, down to the
/// whitespace at its foot: a file this pass has nothing to repair is a file
/// it must not rewrite.
fn with_block(body: &str, block: &str) -> String {
    let end = definition_block(body).map_or(body.len(), |b| b.start);
    if &body[end..] == block {
        return body.to_string();
    }
    let head = body[..end].trim_end();
    match (head.is_empty(), block.is_empty()) {
        (true, _) => block.to_string(),
        (false, true) => format!("{head}\n"),
        (false, false) => format!("{head}\n\n{block}"),
    }
}

fn read(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Write `text` to `path`, refusing anything outside `root`.
///
/// The bound is checked against the resolved spelling of both sides, so a
/// symlink out of the knowledge directory is refused rather than followed.
fn write_within(root: &Path, path: &Path, text: &str) -> Result<()> {
    let resolved = canonical(path).unwrap_or_else(|| path.to_path_buf());
    if !resolved.starts_with(root) {
        return Err(Error::Io {
            path: path.to_path_buf(),
            source: std::io::Error::other(format!(
                "refusing to write outside the SOKF knowledge at {}",
                root.display()
            )),
        });
    }
    std::fs::write(path, text).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sokf::load_bundle;
    use crate::validate::sokf::BLOCK_MARKER;

    /// Everything the pass has to say about `body`, given no concepts at all.
    fn block_of(body: &str) -> String {
        with_block(body, &format!("{BLOCK_MARKER}\n[{ID_LABEL}a]: /k/a.md\n"))
    }

    #[test]
    fn a_block_replaces_the_one_before_it() {
        let once = block_of("Text.\n");
        assert_eq!(once, "Text.\n\n<!-- sokf:links -->\n[sokf:a]: /k/a.md\n");
        assert_eq!(block_of(&once), once, "the second pass changes nothing");
    }

    #[test]
    fn an_empty_block_leaves_the_body_alone() {
        assert_eq!(with_block("Text.\n", ""), "Text.\n");
        assert_eq!(with_block("", ""), "");
    }

    #[test]
    fn link_text_is_read_from_the_closing_parenthesis() {
        let body = "See [a [nested] label](x.md).";
        let span = 4..body.find(')').unwrap() + 1;
        assert_eq!(inline_text(body, &span), Some("a [nested] label"));
        assert_eq!(inline_text(body, &(0..3)), None);
    }

    /// The whole pass over a small knowledge: a path link converts, a link to
    /// a file that is no concept does not, and running twice writes once.
    #[test]
    fn the_pass_converts_and_settles() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "readme\n").unwrap();
        let knowledge = dir.path().join("knowledge");
        std::fs::create_dir_all(knowledge.join("sub")).unwrap();
        std::fs::write(
            knowledge.join("manifest.sokf.yaml"),
            "sokf: \"0.4\"\nname: t\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("a.md"),
            "---\ntype: T\nid: alpha\n---\nSee [beta](sub/beta.md) and [readme](/README.md).\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("sub/beta.md"),
            "---\ntype: T\nid: beta\n---\nx\n",
        )
        .unwrap();
        std::fs::write(knowledge.join("index.md"), "* [beta](sub/beta.md)\n").unwrap();

        let bundle = load_bundle(&knowledge).unwrap();
        let repair = fix(&bundle, dir.path()).unwrap();
        assert_eq!(repair.written, vec!["a.md".to_string(), "index.md".into()]);

        let a = std::fs::read_to_string(knowledge.join("a.md")).unwrap();
        assert_eq!(
            a,
            "---\ntype: T\nid: alpha\n---\nSee [beta][sokf:beta] and [readme](/README.md).\n\n<!-- sokf:links -->\n[sokf:beta]: /knowledge/sub/beta.md\n"
        );

        let bundle = load_bundle(&knowledge).unwrap();
        assert!(
            fix(&bundle, dir.path()).unwrap().written.is_empty(),
            "the pass is idempotent"
        );
    }

    #[test]
    fn a_write_outside_the_knowledge_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("knowledge");
        std::fs::create_dir_all(&root).unwrap();
        let outside = dir.path().join("elsewhere.md");
        std::fs::write(&outside, "x\n").unwrap();
        let error = write_within(&root, &outside, "y\n").unwrap_err();
        assert!(error.to_string().contains("refusing to write outside"));
        assert_eq!(std::fs::read_to_string(&outside).unwrap(), "x\n");
    }
}
