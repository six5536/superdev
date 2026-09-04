//! validate::fix — the repair pass behind `superdev validate --fix`.
//!
//! Four repairs, all mechanical, all derived from the same knowledge the
//! check reads: a document whose folder disagrees with its `lifecycle` is
//! moved into the folder that value names, a body link that names a concept
//! by path is rewritten to address it by `id` (SPEC §8), every include
//! block is refilled from the body of the concept or the region of the
//! repository file its marker names, and
//! every document's generated definition block is rewritten from the ids
//! its body cites (SPEC §9). The moves run first, so every definition block
//! is written against the path a document ends the pass at; links convert
//! before includes copy, so a copied body is the converted one and one pass
//! converges.
//!
//! Nothing here decides anything. The check names a fault and this undoes it,
//! so a file the check has nothing to say about is not written at all — which
//! is what makes a second run a no-op rather than a diff.
//!
//! The pass writes only inside the SOKF knowledge. It is never wired into the
//! hook: the hook fires after an edit, so a hook that repaired would rewrite
//! the file the agent is still working in.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::schema::document::SchemaSet;
use super::sokf::{
    ID_LABEL, Identities, canonical, definition_block, identities, link_path, render_block,
    resolve_target, scan_body, stem_id,
};
use super::{lifecycle, source};
use crate::error::{Error, Result};
use crate::sokf::bundle::{Bundle, load_bundle};
use crate::sokf::concept::{INCLUDE_OPEN, include_blocks};

/// What one repair pass changed.
#[derive(Debug, Default)]
pub struct Repair {
    /// Knowledge-relative paths rewritten, in path order; a moved document
    /// appears as `from -> to`.
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
    let root = canonical(&bundle.root).unwrap_or_else(|| bundle.root.clone());
    let mut repair = Repair::default();

    // Filing first: move each document into the folder its `lifecycle`
    // names, then reload, so the link repairs below read and write the
    // paths the documents end the pass at.
    let moves = lifecycle::moves(bundle, &schema_set(bundle)?);
    let moved;
    let bundle = if moves.is_empty() {
        bundle
    } else {
        for (from, to) in &moves {
            move_within(&root, &bundle.root.join(from), &bundle.root.join(to))?;
            repair.written.push(format!("{from} -> {to}"));
        }
        moved = load_bundle(&bundle.root)?;
        &moved
    };

    // The check's findings are the caller's business; here they are the
    // by-product of building the id maps the repairs read.
    let ids = identities(bundle, repo_root, &mut Vec::new());
    let mut documents: Vec<&str> = bundle
        .concepts
        .iter()
        .map(|c| c.path.as_str())
        .chain(bundle.indexes.iter().map(|(p, _)| p.as_str()))
        .collect();
    documents.sort_unstable();

    // Pass 1: convert links in memory. An include below copies its source's
    // converted body, so one run converges instead of chasing itself.
    let mut converted: Vec<(&str, String, usize, String)> = Vec::new();
    for path in documents {
        let file = bundle.root.join(path);
        let text = read(&file)?;
        let head = body_offset(&text);
        let directory = file
            .parent()
            .map_or_else(|| file.clone(), Path::to_path_buf);
        let body = convert_links(&text[head..], &directory, repo_root, &ids);
        converted.push((path, text, head, body));
    }

    // Include sources: each concept's converted body, by id — borrowed, not
    // cloned, and looked up through a path index rather than a scan.
    let by_path: HashMap<&str, usize> = converted
        .iter()
        .enumerate()
        .map(|(i, (p, ..))| (*p, i))
        .collect();
    let sources: HashMap<&str, &str> = bundle
        .concepts
        .iter()
        .filter_map(|c| {
            let id = c.id.as_deref()?;
            let i = *by_path.get(c.path.as_str())?;
            Some((id, converted[i].3.as_str()))
        })
        .collect();
    let concepts: HashSet<&str> = bundle.concepts.iter().map(|c| c.path.as_str()).collect();

    // Pass 2: materialize include blocks (concepts carry them; an index has
    // no frontmatter and no include), regenerate the definition block, and
    // write what changed.
    for (path, text, head, body) in &converted {
        let body = if concepts.contains(path) {
            materialize(body, &sources, repo_root)
        } else {
            body.clone()
        };
        let cited: BTreeSet<String> = scan_body(&body).cited_ids();
        let fixed = format!(
            "{}{}",
            &text[..*head],
            with_block(&body, &render_block(&cited, &ids.repo_paths))
        );
        // A line at a time, matching the check: a document whose only
        // difference from the repair is what ends its lines needs no repair,
        // so the pass does not rewrite a CRLF file into an LF one (I040).
        if crate::fsutil::lines(&fixed) == crate::fsutil::lines(text) {
            continue;
        }
        write_within(&root, &bundle.root.join(path), &fixed)?;
        repair.written.push((*path).to_string());
    }
    Ok(repair)
}

/// Every include block refilled from its source: a concept's converted body,
/// or a repository file's region, read through [`source::expected`] as the
/// check reads it (SPEC §9). A block the source cannot fill — an id naming
/// no concept, a concept that itself carries an include block, a path or
/// region that resolves to nothing — is left as written, as is every marker
/// fault; the check reports those. A well-formed block is repaired even
/// beside a faulty marker, so the check's "run `superdev validate --fix`"
/// stays true whatever else is wrong in the file.
fn materialize(body: &str, sources: &HashMap<&str, &str>, repo_root: &Path) -> String {
    if !body.contains(INCLUDE_OPEN) {
        return body.to_string();
    }
    let (blocks, _faults) = include_blocks(body);
    let mut out = String::with_capacity(body.len());
    let mut cursor = 0;
    for block in blocks {
        let lookup = |id: &str| sources.get(id).copied();
        let Ok(content) = source::expected(&block.target, lookup, repo_root) else {
            continue;
        };
        // A block the check accepts is not rewritten.
        if source::carries(&body[block.content_start..block.content_end], &content) {
            continue;
        }
        out.push_str(&body[cursor..block.content_start]);
        if !content.is_empty() {
            out.push_str(&content);
            out.push('\n');
        }
        cursor = block.content_end;
    }
    out.push_str(&body[cursor..]);
    out
}

/// Where a document's body starts: after the frontmatter's closing `---`, or
/// at byte 0 when there is none.
///
/// Read from the text this pass is about to rewrite, not from the copy the
/// bundle parsed, so a file edited between the load and the write is repaired
/// as it now stands or not at all — never sliced against a length it no
/// longer has. The split matches `parse_concept`'s, which is what keeps the
/// frontmatter out of every rewrite.
fn body_offset(text: &str) -> usize {
    let is_fence = |line: &str| line.trim_end_matches(['\n', '\r']) == "---";
    let mut lines = text.split_inclusive('\n');
    let Some(first) = lines.next().filter(|line| is_fence(line)) else {
        return 0;
    };
    let mut offset = first.len();
    for line in lines {
        offset += line.len();
        if is_fence(line) {
            return offset;
        }
    }
    0
}

/// Every inline link that names a concept by path, rewritten to address it by
/// id.
///
/// Only an inline link is rewritten. A reference-style link carries its
/// destination in a definition the author wrote, and moving that is a
/// different edit from the one this pass makes; the check names it with the
/// label to write. An image is skipped for good: it names a picture, never a
/// concept, and the check asks nothing of one.
fn convert_links(body: &str, directory: &Path, repo_root: &Path, ids: &Identities) -> String {
    let mut edits: Vec<(std::ops::Range<usize>, String)> = Vec::new();
    for link in scan_body(body).links {
        if link.id.is_some() || !link.inline || link.image {
            continue;
        }
        let Some(file) = link_path(&link.dest) else {
            continue;
        };
        // The target's own `id` where the path resolves; its stem where the
        // path resolves to nothing but the stem names a concept, which is a
        // link a rename left behind (D-8). Anything else — a source file, a
        // README, a path naming nothing — stays as written.
        let Some(id) = resolve_target(file, directory, repo_root)
            .and_then(|target| ids.by_path.get(&target).cloned())
            .or_else(|| stem_id(file, ids))
        else {
            continue;
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

/// The schema set the filing repair reads, from the bundle's own
/// `schemas/` concepts, re-read whole because the contract parser wants the
/// full file. A bundle shipping no schemas yields an empty set, and the
/// pass moves nothing. Load findings are dropped here: the check reports
/// them; this is not the place to report them twice.
fn schema_set(bundle: &Bundle) -> Result<SchemaSet> {
    let mut files: Vec<(String, String)> = Vec::new();
    for concept in &bundle.concepts {
        if concept.path.starts_with("schemas/") {
            files.push((
                concept.path.clone(),
                read(&bundle.root.join(&concept.path))?,
            ));
        }
    }
    Ok(SchemaSet::load(&files).0)
}

/// `path` resolved for a containment check, whether or not it exists yet, or
/// `None` when it cannot be resolved — which the callers treat as a refusal.
///
/// A path that exists resolves outright. Every destination of a refile does
/// not, because refiling is what creates it, so the nearest existing ancestor
/// is resolved — settling every symlink in the part that is there, which is
/// what a bare `starts_with` against a canonical root gets wrong wherever the
/// root is reached through one: macOS `/var` and `/tmp` both are (I039).
///
/// A `..` in the part that does not exist is refused rather than appended.
/// `canonicalize` cannot resolve it, and a lexical `starts_with` cannot see
/// through it, so `root/gone/../../elsewhere` would pass a prefix check and
/// land outside the root once the filesystem resolved it. `file_name` returns
/// `None` for such a component, which is what ends the walk here — nothing
/// superdev writes needs one.
fn resolved(path: &Path) -> Option<PathBuf> {
    if let Some(exact) = canonical(path) {
        return Some(exact);
    }
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let mut here = path.to_path_buf();
    let base = loop {
        if let Some(base) = canonical(&here) {
            break base;
        }
        // `None` for `..`, `.` and a filesystem root: nothing to append safely.
        tail.push(here.file_name()?.to_owned());
        if !here.pop() {
            return None;
        }
    };
    let mut out = base;
    for name in tail.iter().rev() {
        out.push(name);
    }
    Some(out)
}

/// Rename `from` to `to`, creating the state folder, refusing either side
/// outside `root`.
fn move_within(root: &Path, from: &Path, to: &Path) -> Result<()> {
    for path in [from, to] {
        if !resolved(path).is_some_and(|r| r.starts_with(root)) {
            return Err(Error::Io {
                path: path.to_path_buf(),
                source: std::io::Error::other(format!(
                    "refusing to move outside the SOKF knowledge at {}",
                    root.display()
                )),
            });
        }
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::rename(from, to).map_err(|source| Error::Io {
        path: from.to_path_buf(),
        source,
    })
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
    if !resolved(path).is_some_and(|r| r.starts_with(root)) {
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

    /// The split matches `parse_concept`'s, which is what keeps a rewrite off
    /// the frontmatter.
    #[test]
    fn the_body_starts_after_the_frontmatter() {
        for (text, want) in [
            ("---\ntype: T\n---\nbody\n", "body\n"),
            ("---\r\ntype: T\r\n---\r\nbody\r\n", "body\r\n"),
            ("---\ntype: T\n---\n", ""),
            ("no frontmatter\n", "no frontmatter\n"),
            ("---\nunterminated\n", "---\nunterminated\n"),
            ("", ""),
        ] {
            assert_eq!(&text[body_offset(text)..], want, "{text:?}");
        }
    }

    /// Covers I049 criterion 2: a source include is filled with the
    /// renderer's block, a concept include with the concept's body as before,
    /// and one that resolves to nothing is left as written for the check.
    #[test]
    fn materialize_fills_a_source_include_and_leaves_a_concept_include_as_it_was() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("src/main.rs"),
            "// sokf:begin cli\nstruct Cli {}\n// sokf:end cli\n",
        )
        .unwrap();
        let sources: HashMap<&str, &str> = HashMap::from([("style", "The rules.\n")]);
        let body = "<!-- sokf:include /src/main.rs#cli -->\n<!-- /sokf:include -->\n\n\
                    <!-- sokf:include style -->\nOld.\n<!-- /sokf:include -->\n\n\
                    <!-- sokf:include /src/gone.rs -->\nkept\n<!-- /sokf:include -->\n";
        let out = materialize(body, &sources, dir.path());
        assert_eq!(
            out,
            "<!-- sokf:include /src/main.rs#cli -->\n```rust\nstruct Cli {}\n```\n<!-- /sokf:include -->\n\n\
             <!-- sokf:include style -->\nThe rules.\n<!-- /sokf:include -->\n\n\
             <!-- sokf:include /src/gone.rs -->\nkept\n<!-- /sokf:include -->\n"
        );
        assert_eq!(
            materialize(&out, &sources, dir.path()),
            out,
            "a filled block is not rewritten"
        );
    }

    /// A CRLF document (I040): a stale concept include and a stale source
    /// include are refilled with the open marker's line still ending where
    /// it did, so the block is still an include afterwards; a block that is
    /// current apart from its line ends is left alone.
    #[test]
    fn materialize_keeps_the_marker_line_whole_on_a_crlf_document() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("src/main.rs"),
            "// sokf:begin cli\r\nstruct Cli {}\r\n// sokf:end cli\r\n",
        )
        .unwrap();
        let sources: HashMap<&str, &str> = HashMap::from([("style", "The rules.\r\n")]);
        let body = "<!-- sokf:include /src/main.rs#cli -->\r\n```rust\r\nold\r\n```\r\n<!-- /sokf:include -->\r\n\r\n\
                    <!-- sokf:include style -->\r\nOld.\r\n<!-- /sokf:include -->\r\n";
        let out = materialize(body, &sources, dir.path());
        let (blocks, faults) = include_blocks(&out);
        assert!(faults.is_empty(), "{out:?}: {faults:?}");
        assert_eq!(blocks.len(), 2, "{out:?}");
        assert!(
            out.starts_with("<!-- sokf:include /src/main.rs#cli -->\r\n```rust\n"),
            "{out:?}"
        );
        assert!(
            out.contains("<!-- sokf:include style -->\r\nThe rules.\n<!-- /sokf:include -->\r\n"),
            "{out:?}"
        );
        assert_eq!(
            crate::fsutil::lines(&out[blocks[0].content_start..blocks[0].content_end]),
            ["```rust", "struct Cli {}", "```", ""]
        );

        let current = "<!-- sokf:include /src/main.rs#cli -->\r\n```rust\r\nstruct Cli {}\r\n```\r\n<!-- /sokf:include -->\r\n";
        assert_eq!(
            materialize(current, &sources, dir.path()),
            current,
            "a block current apart from its line ends is not rewritten"
        );
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

    /// The filing repair moves a misfiled and an unfiled document, and the
    /// definition blocks citing them are rewritten against the paths they
    /// end at — one pass, one clean tree.
    #[test]
    fn the_pass_files_by_lifecycle_before_repairing_links() {
        let dir = tempfile::tempdir().unwrap();
        let knowledge = dir.path().join("knowledge");
        std::fs::create_dir_all(knowledge.join("schemas")).unwrap();
        std::fs::create_dir_all(knowledge.join("issues/done")).unwrap();
        std::fs::write(
            knowledge.join("manifest.sokf.yaml"),
            "sokf: \"0.4\"\nname: t\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("schemas/issue.md"),
            "---\ntype: Schema\nid: schema-issue\n---\n````yaml\nfrontmatter:\n  type:\n    const: Issue\n  lifecycle:\n    enum: [open, done, wontfix]\n````\n",
        )
        .unwrap();
        // Misfiled: sits in done/ while open.
        std::fs::write(
            knowledge.join("issues/done/issue-001-a.md"),
            "---\ntype: Issue\nid: issue-001-a\nlifecycle: open\n---\nx\n",
        )
        .unwrap();
        // Unfiled: sits in the base directory.
        std::fs::write(
            knowledge.join("issues/issue-002-b.md"),
            "---\ntype: Issue\nid: issue-002-b\nlifecycle: done\n---\nx\n",
        )
        .unwrap();
        // Cites both, so its definition block must name the moved paths.
        std::fs::write(
            knowledge.join("citing.md"),
            "---\ntype: T\nid: citing\n---\nSee [a][sokf:issue-001-a] and [b][sokf:issue-002-b].\n\n<!-- sokf:links -->\n[sokf:issue-001-a]: /knowledge/issues/done/issue-001-a.md\n[sokf:issue-002-b]: /knowledge/issues/issue-002-b.md\n",
        )
        .unwrap();

        let bundle = load_bundle(&knowledge).unwrap();
        let repair = fix(&bundle, dir.path()).unwrap();
        assert!(
            repair
                .written
                .contains(&"issues/done/issue-001-a.md -> issues/open/issue-001-a.md".to_string())
        );
        assert!(
            repair
                .written
                .contains(&"issues/issue-002-b.md -> issues/done/issue-002-b.md".to_string())
        );
        assert!(knowledge.join("issues/open/issue-001-a.md").is_file());
        assert!(knowledge.join("issues/done/issue-002-b.md").is_file());

        let citing = std::fs::read_to_string(knowledge.join("citing.md")).unwrap();
        assert!(citing.contains("[sokf:issue-001-a]: /knowledge/issues/open/issue-001-a.md"));
        assert!(citing.contains("[sokf:issue-002-b]: /knowledge/issues/done/issue-002-b.md"));

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

    /// Covers I039: a refile lands when the knowledge root is reached
    /// through a symlink. The destination does not exist until the move
    /// makes it, so a guard that falls back to the raw spelling compares it
    /// against a resolved root and refuses its own work. Unix-only because
    /// the repro is a symlink; the defect it pins is what reddened macOS.
    #[test]
    #[cfg(unix)]
    fn a_refile_under_a_symlinked_root_is_not_refused() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        let knowledge = real.join("knowledge");
        std::fs::create_dir_all(knowledge.join("schemas")).unwrap();
        std::fs::create_dir_all(knowledge.join("issues/done")).unwrap();
        std::fs::write(
            knowledge.join("manifest.sokf.yaml"),
            "sokf: \"0.4\"\nname: t\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("schemas/issue.md"),
            "---\ntype: Schema\nid: schema-issue\n---\n````yaml\nfrontmatter:\n  type:\n    const: Issue\n  lifecycle:\n    enum: [open, done, wontfix]\n````\n",
        )
        .unwrap();
        std::fs::write(
            knowledge.join("issues/done/issue-001-a.md"),
            "---\ntype: Issue\nid: issue-001-a\nlifecycle: open\n---\nx\n",
        )
        .unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let via_link = link.join("knowledge");
        let bundle = load_bundle(&via_link).unwrap();
        let repair = fix(&bundle, &link).unwrap();

        assert!(
            repair
                .written
                .contains(&"issues/done/issue-001-a.md -> issues/open/issue-001-a.md".to_string()),
            "the refile was refused through the symlink: {:?}",
            repair.written
        );
        assert!(real.join("knowledge/issues/open/issue-001-a.md").is_file());
    }

    /// A `..` after a component that does not exist escapes a lexical prefix
    /// check: `canonicalize` cannot resolve any of it, and `root/gone/../../x`
    /// literally begins with `root` while the filesystem lands it outside.
    /// Both guards refuse it rather than resolving it.
    #[test]
    fn a_dotdot_past_a_missing_component_cannot_escape_the_knowledge() {
        let dir = tempfile::tempdir().unwrap();
        let root = canonical(&dir.path().join("knowledge")).unwrap_or_else(|| {
            std::fs::create_dir_all(dir.path().join("knowledge")).unwrap();
            canonical(&dir.path().join("knowledge")).unwrap()
        });
        let inside = root.join("a.md");
        std::fs::write(&inside, "x\n").unwrap();

        for escape in [
            "gone/../../escaped.md",
            "sub/../../escaped.md",
            "../escaped.md",
            "gone/../../../escaped.md",
        ] {
            let target = root.join(escape);
            assert!(
                !resolved(&target).is_some_and(|r| r.starts_with(&root)),
                "{escape} resolved to something inside the knowledge"
            );
            assert!(
                move_within(&root, &inside, &target)
                    .is_err_and(|e| e.to_string().contains("refusing to move outside")),
                "{escape} was not refused by move_within"
            );
            assert!(
                write_within(&root, &target, "y\n")
                    .is_err_and(|e| e.to_string().contains("refusing to write outside")),
                "{escape} was not refused by write_within"
            );
            assert!(
                !dir.path().join("escaped.md").exists(),
                "{escape} wrote outside the knowledge"
            );
        }
        assert!(inside.is_file(), "the source survived every refusal");
    }

    /// Covers I039: the guard still refuses a destination outside the
    /// knowledge, whether or not that destination exists — resolving the
    /// nearest existing ancestor must not become a way out.
    #[test]
    fn a_move_to_a_path_outside_the_knowledge_is_still_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("knowledge");
        std::fs::create_dir_all(&root).unwrap();
        let inside = root.join("a.md");
        std::fs::write(&inside, "x\n").unwrap();

        for outside in [dir.path().join("gone.md"), dir.path().join("no/such.md")] {
            let error = move_within(&root, &inside, &outside).unwrap_err();
            assert!(
                error.to_string().contains("refusing to move outside"),
                "{} was not refused",
                outside.display()
            );
        }
        assert!(
            inside.is_file(),
            "the refused move left the source in place"
        );
    }
}
