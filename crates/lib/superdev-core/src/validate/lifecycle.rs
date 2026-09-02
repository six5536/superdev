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
//!
//! A contract's id carries its `kind` as well, as the third segment of
//! `contract-{nnn}-{kind}-{slug}` (ADR-043); the two must agree, and a
//! disagreement is reported naming both. Nothing repairs it: which of the
//! two is wrong is the author's call.

use super::schema::document::SchemaSet;
use super::sokf::Finding;
use crate::sokf::bundle::Bundle;

/// A filing finding fails the run: a committed tree has no unfiled or
/// misfiled document, and `--fix` repairs both, so nothing softer is needed.
const FATAL: bool = true;

/// The type whose id names its `kind`: `contract-{nnn}-{kind}-{slug}`.
const KINDED_TYPE: &str = "Contract";

/// One document in scope: its repo-relative path as the report spells it,
/// its type, and the `lifecycle`, `id` and `kind` its frontmatter carries.
pub struct Subject {
    /// Path as findings should name it (repo-relative).
    pub path: String,
    /// The frontmatter `type`, which names the governing schema.
    pub doc_type: String,
    /// The frontmatter `lifecycle`, when present.
    pub lifecycle: Option<String>,
    /// The frontmatter `id`, when present.
    pub id: Option<String>,
    /// The frontmatter `kind`, when present.
    pub kind: Option<String>,
}

/// Check every subject: the `kind` rule on a contract, and the `lifecycle`
/// rules on every document whose schema declares the enum.
#[must_use]
pub fn check(subjects: &[Subject], set: &SchemaSet) -> Vec<Finding> {
    let mut findings = Vec::new();
    for subject in subjects {
        let mut push = |message: String| {
            findings.push(Finding {
                path: subject.path.clone(),
                message,
                fatal: FATAL,
            });
        };
        // An absent `id` or `kind`, and an id with no third segment, are the
        // schema's frontmatter findings; only a disagreement is this check's.
        if subject.doc_type == KINDED_TYPE
            && let (Some(id), Some(kind)) = (subject.id.as_deref(), subject.kind.as_deref())
            && let Some(segment) = id_kind(id)
            && segment != kind
        {
            push(format!(
                "`kind` is `{kind}`, and the id `{id}` names `{segment}` in its third \
                 segment — the two must agree"
            ));
        }
        let Some(allowed) = set.lifecycle_enum(&subject.doc_type) else {
            continue;
        };
        let list = allowed.join(", ");
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

/// The `kind` a contract id names: its third `-`-separated segment, `None`
/// when the id is too short to have one.
fn id_kind(id: &str) -> Option<&str> {
    id.split('-').nth(2)
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
            id: None,
            kind: None,
        }
    }

    /// A contract filed as active, with the id and `kind` given.
    fn contract(id: &str, kind: Option<&str>) -> Subject {
        Subject {
            path: format!("contracts/public/active/{id}.md"),
            doc_type: "Contract".to_string(),
            lifecycle: Some("active".to_string()),
            id: Some(id.to_string()),
            kind: kind.map(str::to_string),
        }
    }

    /// Covers I049 criterion 11: the id's third segment must equal `kind`,
    /// and a disagreement names both.
    #[test]
    fn a_contract_whose_kind_and_id_segment_disagree_is_reported_naming_both() {
        let findings = check(&[contract("contract-001-cli-widget", Some("api"))], &set());
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("`kind` is `api`"));
        assert!(
            findings[0]
                .message
                .contains("`contract-001-cli-widget` names `cli`")
        );
        assert!(findings[0].fatal, "a filing finding fails the run");
    }

    #[test]
    fn a_contract_whose_kind_and_id_segment_agree_passes() {
        let findings = check(&[contract("contract-001-cli-widget", Some("cli"))], &set());
        assert!(findings.is_empty());
    }

    /// An absent `kind` and a too-short id are the schema's findings, not
    /// this check's; a `kind` on a type that carries none in its id is not
    /// read.
    #[test]
    fn the_kind_rule_reports_only_a_disagreement_on_a_contract() {
        assert!(check(&[contract("contract-001-cli-widget", None)], &set()).is_empty());
        assert!(check(&[contract("contract-001", Some("cli"))], &set()).is_empty());
        let other = Subject {
            kind: Some("api".to_string()),
            id: Some("issue-001-bug-x".to_string()),
            ..subject("issues/open/issue-001-bug-x.md", Some("open"))
        };
        assert!(check(&[other], &set()).is_empty());
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
            id: None,
            kind: None,
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
