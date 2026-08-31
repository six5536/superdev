//! validate::lifecycle — the filing check: a document's `lifecycle` value,
//! and the folder that must carry it.
//!
//! A schema that declares a `lifecycle` enum puts its documents in scope.
//! For each, three things must hold: the value exists, the value is in the
//! enum, and the last path segment before the filename names it — that
//! folder is what makes the state visible in the tree. A document sitting
//! directly in its kind's base directory is *unfiled*; one sitting in a
//! folder named for another state is misfiled. Both are repaired by the fix
//! pass, which rewrites only that one segment — which is what lets contracts
//! keep their audience partition above it.

use super::schema::document::SchemaSet;
use super::sokf::Finding;
use crate::sokf::bundle::Bundle;

/// A filing finding fails the run: a committed tree has no unfiled or
/// misfiled document, and `--fix` repairs both, so nothing softer is needed.
const FATAL: bool = true;

/// One document in scope: its repo-relative path as the report spells it,
/// its bundle-relative path as the fix pass moves it, its type, and the
/// `lifecycle` its frontmatter carries.
pub struct Subject {
    /// Path as findings should name it (repo-relative).
    pub path: String,
    /// The frontmatter `type`, which names the governing schema.
    pub doc_type: String,
    /// The frontmatter `lifecycle`, when present.
    pub lifecycle: Option<String>,
}

/// Check every subject whose schema declares a `lifecycle` enum.
#[must_use]
pub fn check(subjects: &[Subject], set: &SchemaSet) -> Vec<Finding> {
    let mut findings = Vec::new();
    for subject in subjects {
        let Some(allowed) = set.lifecycle_enum(&subject.doc_type) else {
            continue;
        };
        let list = allowed.join(", ");
        let mut push = |message: String| {
            findings.push(Finding {
                path: subject.path.clone(),
                message,
                fatal: FATAL,
            });
        };
        let Some(value) = subject.lifecycle.as_deref() else {
            push(format!("missing `lifecycle` — one of: {list}"));
            continue;
        };
        if !allowed.iter().any(|a| a == value) {
            push(format!(
                "`lifecycle` value `{value}` is not in the schema's enum: {list}"
            ));
            continue;
        }
        match filing(&subject.path, value, allowed) {
            Filing::Filed => {}
            Filing::Misfiled { folder } => push(format!(
                "filed under `{folder}/` while `lifecycle` is `{value}` — \
                 `superdev validate --fix` moves it"
            )),
            Filing::Unfiled => push(format!(
                "unfiled: `lifecycle` `{value}` names folder `{value}/` — \
                 `superdev validate --fix` files it"
            )),
        }
    }
    findings
}

/// The moves the fix pass owes `bundle`: bundle-relative `(from, to)` pairs
/// for every concept whose folder disagrees with a valid `lifecycle` value.
/// A missing or out-of-enum value moves nothing — there is no folder to
/// derive.
#[must_use]
pub fn moves(bundle: &Bundle, set: &SchemaSet) -> Vec<(String, String)> {
    let mut moves = Vec::new();
    for concept in &bundle.concepts {
        let Some(allowed) = set.lifecycle_enum(&concept.kind) else {
            continue;
        };
        let Some(value) = concept.lifecycle.as_deref() else {
            continue;
        };
        if !allowed.iter().any(|a| a == value) {
            continue;
        }
        if let Some(target) = target(&concept.path, value, allowed) {
            moves.push((concept.path.clone(), target));
        }
    }
    moves
}

/// How a path stands against the folder its `lifecycle` value names.
enum Filing {
    Filed,
    Misfiled { folder: String },
    Unfiled,
}

fn filing(path: &str, value: &str, allowed: &[String]) -> Filing {
    let (folder, _) = split(path);
    if folder == value {
        Filing::Filed
    } else if allowed.iter().any(|a| a == folder) {
        Filing::Misfiled {
            folder: folder.to_string(),
        }
    } else {
        Filing::Unfiled
    }
}

/// The path a document belongs at, or `None` when it is already there. Only
/// the last segment before the filename is written: replaced when it names
/// another state, inserted when the document is unfiled.
fn target(path: &str, value: &str, allowed: &[String]) -> Option<String> {
    let (dir, file) = path.rsplit_once('/').unwrap_or(("", path));
    let (folder, _) = split(path);
    if folder == value {
        return None;
    }
    let base = if allowed.iter().any(|a| a == folder) {
        // In a state folder, just the wrong one: its parent is the base.
        dir.rsplit_once('/').map_or("", |(base, _)| base)
    } else {
        dir
    };
    Some(if base.is_empty() {
        format!("{value}/{file}")
    } else {
        format!("{base}/{value}/{file}")
    })
}

/// A path's last directory segment before the filename, and the filename.
/// Empty when the file sits at the root.
fn split(path: &str) -> (&str, &str) {
    let (dir, file) = path.rsplit_once('/').unwrap_or(("", path));
    let folder = dir.rsplit_once('/').map_or(dir, |(_, last)| last);
    (folder, file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::schema::document::SchemaSet;

    /// A schema whose documents carry `lifecycle: open | done | wontfix`.
    const ISSUE_SCHEMA: &str = "---\ntype: Schema\n---\n````yaml\nfrontmatter:\n  type:\n    const: BugReport\n  lifecycle:\n    enum: [open, done, wontfix]\n````\n";

    fn set() -> SchemaSet {
        let files = vec![(
            "schemas/bug-report.md".to_string(),
            ISSUE_SCHEMA.to_string(),
        )];
        let (set, findings) = SchemaSet::load(&files);
        assert!(findings.is_empty());
        set
    }

    fn subject(path: &str, lifecycle: Option<&str>) -> Subject {
        Subject {
            path: path.to_string(),
            doc_type: "BugReport".to_string(),
            lifecycle: lifecycle.map(str::to_string),
        }
    }

    #[test]
    fn a_filed_document_with_a_valid_value_passes() {
        let findings = check(
            &[subject("issues/open/issue-001-bug-x.md", Some("open"))],
            &set(),
        );
        assert!(findings.is_empty());
    }

    #[test]
    fn a_missing_value_names_the_enum() {
        let findings = check(&[subject("issues/open/issue-001-bug-x.md", None)], &set());
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("open, done, wontfix"));
        assert!(findings[0].fatal, "a filing finding fails the run");
    }

    #[test]
    fn a_value_outside_the_enum_names_value_and_enum() {
        let findings = check(
            &[subject("issues/open/issue-001-bug-x.md", Some("closed"))],
            &set(),
        );
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("`closed`"));
        assert!(findings[0].message.contains("open, done, wontfix"));
    }

    #[test]
    fn a_misfiled_document_is_reported_with_its_folder() {
        let findings = check(
            &[subject("issues/done/issue-001-bug-x.md", Some("open"))],
            &set(),
        );
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("`done/`"));
    }

    #[test]
    fn an_unfiled_document_is_reported_with_its_target() {
        let findings = check(
            &[subject("issues/issue-001-bug-x.md", Some("open"))],
            &set(),
        );
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("unfiled"));
        assert!(findings[0].message.contains("`open/`"));
    }

    #[test]
    fn a_type_with_no_lifecycle_enum_is_out_of_scope() {
        let out = Subject {
            path: "architecture.md".to_string(),
            doc_type: "Reference".to_string(),
            lifecycle: None,
        };
        assert!(check(&[out], &set()).is_empty());
    }

    #[test]
    fn targets_replace_a_state_folder_and_append_for_unfiled() {
        let allowed: Vec<String> = ["open", "done", "wontfix"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(
            target("issues/done/i.md", "open", &allowed).as_deref(),
            Some("issues/open/i.md")
        );
        assert_eq!(
            target("issues/i.md", "open", &allowed).as_deref(),
            Some("issues/open/i.md")
        );
        assert_eq!(
            target("contracts/public/c.md", "open", &allowed).as_deref(),
            Some("contracts/public/open/c.md")
        );
        assert_eq!(target("issues/open/i.md", "open", &allowed), None);
        assert_eq!(
            target("i.md", "open", &allowed).as_deref(),
            Some("open/i.md")
        );
    }
}
