//! validate::source — what an include block must carry (SPEC §9): a
//! concept's body, or the named region of a repository file rendered as a
//! fenced block. The check and the repair both read it here, so the two
//! cannot disagree about the content, nor about when a block carries it.
//!
//! Nothing inside the file is parsed. The markers are found by substring in
//! whatever comment syntax the file uses, and the region's bytes move into
//! the block as they are.

use std::path::Path;

use super::sokf::canonical;
use crate::sokf::concept::{IncludeTarget, carries_include_markers, included_text};

/// What the block for `target` must carry: the body of the concept `lookup`
/// answers for, or the file's region rendered by [`render`].
///
/// # Errors
///
/// The finding's text, which is the repair's reason to leave the block as
/// written: the id names no concept, the concept itself carries an include
/// block (includes do not nest), or the source does not render.
pub(crate) fn expected<'a>(
    target: &IncludeTarget,
    lookup: impl Fn(&str) -> Option<&'a str>,
    repo_root: &Path,
) -> Result<String, String> {
    match target {
        IncludeTarget::Concept(id) => {
            let Some(body) = lookup(id) else {
                return Err(format!("include block names no concept: `{id}`"));
            };
            let content = included_text(body);
            if carries_include_markers(&content) {
                return Err(format!(
                    "include block names `{id}`, which itself carries an include block; includes do not nest"
                ));
            }
            Ok(content)
        }
        IncludeTarget::Source { path, region } => render(repo_root, path, region.as_deref()),
    }
}

/// Whether a block's content is `expected`: the same lines, whatever ends
/// them, so a CRLF checkout of the host or of the source reads as its LF
/// twin (I040); surrounding blank lines do not count.
#[must_use]
pub(crate) fn carries(content: &str, expected: &str) -> bool {
    crate::fsutil::lines(content.trim()) == crate::fsutil::lines(expected)
}

/// The marker opening a region; the region's name follows.
const BEGIN: &str = "sokf:begin ";

/// The marker closing a region; the region's name follows.
const END: &str = "sokf:end ";

/// The line a generator writes among a file's leading lines, carried into
/// the block so a reader sees the file was itself rendered.
const GENERATED_BY: &str = "sokf:generated-by";

/// Extensions whose conventional fence tag differs from the extension.
const TAGS: [(&str, &str); 4] = [
    ("rs", "rust"),
    ("yml", "yaml"),
    ("ts", "typescript"),
    ("py", "python"),
];

/// The content an include block naming `path` — `/`-rooted at `repo_root` —
/// and `region` must carry: the region's lines, the whole file when `region`
/// is `None`, fenced and tagged by the file's extension, with the file's
/// `sokf:generated-by` line first when it has one.
///
/// Regions sharing a name concatenate in file order. A marker names a
/// region by the whole name: `sokf:begin cli` opens `cli` and not `cli-v2`.
///
/// # Errors
///
/// The finding's text, naming the path and the region: the path does not
/// exist, resolves outside the repository or cannot be read; the file
/// carries no region of that name, or opens one it never closes.
pub(crate) fn render(repo_root: &Path, path: &str, region: Option<&str>) -> Result<String, String> {
    let what = match region {
        Some(region) => format!("{path}#{region}"),
        None => path.to_string(),
    };
    let fault = |problem: String| format!("include `{what}`: {problem}");

    let Some(file) = canonical(&repo_root.join(path.trim_start_matches('/'))) else {
        return Err(fault("the path does not exist".to_string()));
    };
    let root = canonical(repo_root).unwrap_or_else(|| repo_root.to_path_buf());
    if !file.starts_with(&root) {
        return Err(fault(
            "the path resolves outside the repository".to_string(),
        ));
    }
    let text = std::fs::read_to_string(&file)
        .map_err(|e| fault(format!("the file cannot be read: {e}")))?;
    let lines: Vec<&str> = text.lines().collect();

    let mut body: Vec<&str> = match region {
        Some(name) => region_lines(&lines, name).map_err(fault)?,
        None => lines.clone(),
    };
    if let Some(generated) = generated_by(&lines)
        && !body.contains(&generated)
    {
        body.insert(0, generated);
    }

    let fence = "`".repeat(fence_width(&body));
    let tag = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            TAGS.iter()
                .find(|(from, _)| *from == e)
                .map_or(e, |(_, to)| to)
        })
        .unwrap_or_default();
    Ok(format!("{fence}{tag}\n{}\n{fence}", body.join("\n")))
}

/// The lines of every region named `name`, in file order, markers excluded.
fn region_lines<'a>(lines: &[&'a str], name: &str) -> Result<Vec<&'a str>, String> {
    let mut body = Vec::new();
    let mut found = false;
    let mut inside = false;
    for line in lines {
        if inside && marks(line, END, name) {
            inside = false;
        } else if inside {
            body.push(*line);
        } else if marks(line, BEGIN, name) {
            inside = true;
            found = true;
        }
    }
    if inside {
        return Err(format!("region `{name}` opens and never closes"));
    }
    if !found {
        return Err(format!("the file carries no region `{name}`"));
    }
    Ok(body)
}

/// Whether `line` carries `marker` followed by the whole of `name`: the name
/// ends where the line does or at whitespace, so a name that prefixes
/// another does not open it.
fn marks(line: &str, marker: &str, name: &str) -> bool {
    line.match_indices(marker).any(|(at, _)| {
        let rest = &line[at + marker.len()..];
        rest.strip_prefix(name)
            .is_some_and(|after| after.chars().next().is_none_or(char::is_whitespace))
    })
}

/// The file's `sokf:generated-by` line, when one sits among its leading
/// lines — those before the first blank line.
fn generated_by<'a>(lines: &[&'a str]) -> Option<&'a str> {
    lines
        .iter()
        .take_while(|line| !line.trim().is_empty())
        .find(|line| line.contains(GENERATED_BY))
        .copied()
}

/// A fence wide enough that no line of `body` closes it: one more backtick
/// than the longest run opening a line, and never fewer than three.
fn fence_width(body: &[&str]) -> usize {
    body.iter()
        .map(|line| line.trim_start().bytes().take_while(|b| *b == b'`').count() + 1)
        .max()
        .unwrap_or(0)
        .max(3)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// A repository root carrying `files`, each `/`-rooted at it. The root
    /// sits one directory below the temporary directory, so a `..` has
    /// somewhere to land.
    fn repo(files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        std::fs::create_dir_all(&root).unwrap();
        for (path, text) in files {
            let file = root.join(path.trim_start_matches('/'));
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        }
        (dir, root)
    }

    const MAIN: &str = "use clap::Parser;\n\n// sokf:begin cli\n/// The tool.\n#[derive(Parser)]\nstruct Cli {}\n// sokf:end cli\n\nfn main() {}\n";

    /// Covers I049 criteria 2 and 3: the region between the markers, fenced
    /// and tagged `rust` for a `.rs` file.
    #[test]
    fn a_region_renders_fenced_and_tagged_by_extension() {
        let (_dir, root) = repo(&[("/src/main.rs", MAIN)]);
        let block = render(&root, "/src/main.rs", Some("cli")).unwrap();
        assert_eq!(
            block,
            "```rust\n/// The tool.\n#[derive(Parser)]\nstruct Cli {}\n```"
        );
    }

    /// Covers criterion 3: regions sharing a name concatenate in file order.
    #[test]
    fn regions_of_one_name_concatenate_in_file_order() {
        let text = "# sokf:begin api\nfirst\n# sokf:end api\nbetween\n# sokf:begin api\nsecond\n# sokf:end api\n";
        let (_dir, root) = repo(&[("/schema.yml", text)]);
        let block = render(&root, "/schema.yml", Some("api")).unwrap();
        assert_eq!(block, "```yaml\nfirst\nsecond\n```");
    }

    /// Covers criterion 2: no `#` renders the whole file.
    #[test]
    fn no_region_renders_the_whole_file() {
        let (_dir, root) = repo(&[("/src/main.rs", MAIN)]);
        let block = render(&root, "/src/main.rs", None).unwrap();
        assert_eq!(block, format!("```rust\n{}\n```", MAIN.trim_end()));
    }

    /// Covers criterion 2: the tag is the extension, mapped where the
    /// conventional tag differs, bare when there is none.
    #[test]
    fn the_tag_is_the_extension_mapped_or_verbatim_or_bare() {
        let (_dir, root) = repo(&[
            ("/api.proto", "syntax = \"proto3\";\n"),
            ("/ci.yml", "on: push\n"),
            ("/app.ts", "export {};\n"),
            ("/run.py", "pass\n"),
            ("/Makefile", "all:\n"),
        ]);
        for (path, tag) in [
            ("/api.proto", "proto"),
            ("/ci.yml", "yaml"),
            ("/app.ts", "typescript"),
            ("/run.py", "python"),
            ("/Makefile", ""),
        ] {
            let block = render(&root, path, None).unwrap();
            assert!(block.starts_with(&format!("```{tag}\n")), "{path}: {block}");
        }
    }

    /// Covers criterion 6: a `sokf:generated-by` line among the file's
    /// leading lines is the block's first line, unchanged — and once.
    #[test]
    fn a_generated_by_line_leads_the_block() {
        let text = "-- sokf:generated-by scripts/render-schema.sh\n-- Do not edit.\n\n-- sokf:begin tables\nCREATE TABLE t (id INT);\n-- sokf:end tables\n";
        let (_dir, root) = repo(&[("/db/schema.sql", text)]);
        let region = render(&root, "/db/schema.sql", Some("tables")).unwrap();
        assert_eq!(
            region,
            "```sql\n-- sokf:generated-by scripts/render-schema.sh\nCREATE TABLE t (id INT);\n```"
        );
        let whole = render(&root, "/db/schema.sql", None).unwrap();
        assert_eq!(whole.matches("sokf:generated-by").count(), 1, "{whole}");
        assert!(whole.starts_with("```sql\n-- sokf:generated-by"), "{whole}");
    }

    /// A `generated-by` line below the leading lines is not a header.
    #[test]
    fn a_generated_by_line_after_a_blank_line_is_not_carried() {
        let text = "x\n\n# sokf:generated-by nothing\n# sokf:begin r\ny\n# sokf:end r\n";
        let (_dir, root) = repo(&[("/f.toml", text)]);
        assert_eq!(
            render(&root, "/f.toml", Some("r")).unwrap(),
            "```toml\ny\n```"
        );
    }

    /// Covers criterion 7: content that is valid in no language renders
    /// byte for byte, so nothing parsed it — and a fence inside it does not
    /// close the block.
    #[test]
    fn a_region_renders_byte_for_byte() {
        let text = "// sokf:begin junk\n}{ not ( code ``` ` \"\n\t  odd\t whitespace  \n```\n// sokf:end junk\n";
        let (_dir, root) = repo(&[("/weird.rs", text)]);
        let block = render(&root, "/weird.rs", Some("junk")).unwrap();
        assert_eq!(
            block,
            "````rust\n}{ not ( code ``` ` \"\n\t  odd\t whitespace  \n```\n````"
        );
    }

    /// Covers criterion 5: each failure names the path, the region and
    /// which it is.
    #[test]
    fn a_missing_path_an_escape_and_a_missing_region_are_each_named() {
        let (_dir, root) = repo(&[("/src/main.rs", MAIN)]);
        std::fs::write(_dir.path().join("outside-of-repo.txt"), "x\n").unwrap();

        assert_eq!(
            render(&root, "/src/gone.rs", Some("cli")).unwrap_err(),
            "include `/src/gone.rs#cli`: the path does not exist"
        );
        assert_eq!(
            render(&root, "/../outside-of-repo.txt", None).unwrap_err(),
            "include `/../outside-of-repo.txt`: the path resolves outside the repository"
        );
        assert_eq!(
            render(&root, "/src/main.rs", Some("server")).unwrap_err(),
            "include `/src/main.rs#server`: the file carries no region `server`"
        );
    }

    /// Covers criterion 5: a symlink out of the repository is outside it.
    #[test]
    #[cfg(unix)]
    fn a_symlink_out_of_the_repository_is_outside_it() {
        let elsewhere = tempfile::tempdir().unwrap();
        std::fs::write(elsewhere.path().join("secret.rs"), "x\n").unwrap();
        let (_dir, root) = repo(&[]);
        std::os::unix::fs::symlink(elsewhere.path().join("secret.rs"), root.join("linked.rs"))
            .unwrap();
        assert_eq!(
            render(&root, "/linked.rs", None).unwrap_err(),
            "include `/linked.rs`: the path resolves outside the repository"
        );
    }

    #[test]
    fn a_region_that_never_closes_is_named() {
        let (_dir, root) = repo(&[("/a.rs", "// sokf:begin open\nx\n")]);
        assert_eq!(
            render(&root, "/a.rs", Some("open")).unwrap_err(),
            "include `/a.rs#open`: region `open` opens and never closes"
        );
    }

    /// A marker names a region by its whole name, in any comment syntax.
    #[test]
    fn a_marker_matches_the_whole_name_in_any_comment_syntax() {
        let text = "<!-- sokf:begin cli-v2 -->\nother\n<!-- sokf:end cli-v2 -->\n<!-- sokf:begin cli -->\nmine\n<!-- sokf:end cli -->\n";
        let (_dir, root) = repo(&[("/doc.html", text)]);
        assert_eq!(
            render(&root, "/doc.html", Some("cli")).unwrap(),
            "```html\nmine\n```"
        );
    }

    #[test]
    fn a_directory_cannot_be_read() {
        let (_dir, root) = repo(&[("/src/main.rs", MAIN)]);
        let error = render(&root, "/src", None).unwrap_err();
        assert!(
            error.starts_with("include `/src`: the file cannot be read: "),
            "{error}"
        );
    }
}
