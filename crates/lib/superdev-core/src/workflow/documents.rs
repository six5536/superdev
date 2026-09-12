//! Exact canonical document revisions and conservative publication boundaries.

use std::{io::Read, path::Path};

use cap_std::{ambient_authority, fs::Dir};

use super::{
    git,
    local::*,
    request::DocumentKind,
    store::{Store, hash, refusal},
};
use crate::{
    error::{Error, Result},
    sokf::parse_concept,
};

pub(super) fn validate_id(kind: DocumentKind, id: &str) -> Result<()> {
    let valid = id
        .strip_prefix(&format!("{}-", kind.prefix()))
        .and_then(|tail| tail.split_once('-'))
        .is_some_and(|(number, slug)| {
            number.len() == 3
                && number.bytes().all(|byte| byte.is_ascii_digit())
                && !slug.is_empty()
                && slug
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        });
    if !valid {
        return Err(refusal(format!(
            "invalid {} identity `{id}`",
            kind.prefix()
        )));
    }
    Ok(())
}

pub(super) fn reserve(
    root: &Path,
    store: &Store,
    kind: DocumentKind,
    id: &str,
    owner: Option<&str>,
) -> Result<()> {
    validate_id(kind, id)?;
    git::require_unique_record_number(root, kind.prefix(), id)?;
    let number_prefix = &id[..kind.prefix().len() + 5];
    // Check every local branch, not only HEAD: SCOPE may have returned to the
    // default branch while another plan's number remains reserved elsewhere.
    for branch in git::local_branches(root)? {
        for path in git::paths_at_revision(root, &branch)? {
            if path.starts_with("knowledge/")
                && let Some(stem) = Path::new(&path).file_stem().and_then(|name| name.to_str())
                && stem.starts_with(number_prefix)
                && stem != id
            {
                return Err(refusal(format!(
                    "document number is reserved by `{stem}` on `{branch}`"
                )));
            }
        }
    }
    for record in store.list()? {
        if owner == Some(record.id.as_str()) {
            continue;
        }
        let existing = match kind {
            DocumentKind::Issue => Some(record.issue.as_str()),
            DocumentKind::Plan => record.plan.as_deref(),
        };
        if existing.is_some_and(|existing| existing.starts_with(number_prefix)) {
            return Err(refusal(format!(
                "document identity is reserved by local workflow `{}`; resume or select a new identity",
                record.id
            )));
        }
    }
    Ok(())
}

pub(super) fn text(root: &Path, relative: &str) -> Result<String> {
    let directory =
        Dir::open_ambient_dir(root, ambient_authority()).map_err(|source| Error::Io {
            path: root.into(),
            source,
        })?;
    let file = directory.open(relative).map_err(|source| Error::Io {
        path: root.join(relative),
        source,
    })?;
    let mut text = String::new();
    file.take(4 * 1024 * 1024 + 1)
        .read_to_string(&mut text)
        .map_err(|source| Error::Io {
            path: root.join(relative),
            source,
        })?;
    if text.len() > 4 * 1024 * 1024 {
        return Err(refusal("workflow document exceeds the size limit"));
    }
    Ok(text)
}

pub(super) fn document(
    root: &Path,
    record: &WorkflowRecord,
    kind: DocumentKind,
) -> Result<DocumentRevision> {
    let id = match kind {
        DocumentKind::Issue => &record.issue,
        DocumentKind::Plan => record
            .plan
            .as_ref()
            .ok_or_else(|| refusal("workflow has no selected plan"))?,
    };
    validate_id(kind, id)?;
    let path = format!("knowledge/{}s/open/{id}.md", kind.prefix());
    let text = text(root, &path)?;
    let concept = parse_concept(&path, &text).map_err(|error| refusal(error.message))?;
    let expected_type = match kind {
        DocumentKind::Issue => "Issue",
        DocumentKind::Plan => "Plan",
    };
    if concept.id.as_deref() != Some(id)
        || concept.lifecycle.as_deref() != Some("open")
        || concept.raw["type"].as_str() != Some(expected_type)
    {
        return Err(refusal(
            "workflow document identity, type, or open lifecycle does not match",
        ));
    }
    if kind == DocumentKind::Plan {
        let issues: Vec<_> = concept
            .links
            .iter()
            .filter(|link| link.rel.as_deref() == Some("implements"))
            .collect();
        if issues.len() != 1 || issues[0].to.as_deref() != Some(record.issue.as_str()) {
            return Err(refusal("plan must implement exactly the selected issue"));
        }
    }
    Ok(DocumentRevision {
        id: id.clone(),
        path,
        hash: hash(text.as_bytes()),
    })
}

pub(super) fn approved_issue(root: &Path, record: &WorkflowRecord) -> Result<DocumentRevision> {
    let issue = document(root, record, DocumentKind::Issue)?;
    if !record
        .issue_approval
        .as_ref()
        .is_some_and(|approval| approval.covers(&issue))
    {
        return Err(refusal(
            "issue approval is absent or suspended by changed bytes; assess the diff or request fresh approval",
        ));
    }
    Ok(issue)
}

pub(super) fn approved_scope(root: &Path, record: &WorkflowRecord) -> Result<()> {
    if record.pending_publication.is_some() {
        return Err(refusal("recover the pending publication before execution"));
    }
    approved_issue(root, record)?;
    let plan = document(root, record, DocumentKind::Plan)?;
    let issue_approval = record
        .issue_approval
        .as_ref()
        .expect("approved issue was checked");
    let valid = record.plan_approval.as_ref().is_some_and(|approval| {
        approval.covers(&plan)
            && approval.issue_hash.as_ref().is_some_and(|hash| {
                *hash == issue_approval.original.hash || issue_approval.compatible.contains(hash)
            })
    });
    if !valid {
        return Err(refusal(
            "plan approval is absent, stale, or bound to a different issue revision",
        ));
    }
    Ok(())
}

pub(super) fn indexes(
    root: &Path,
    document: &DocumentRevision,
    indexes: &[String],
) -> Result<Vec<String>> {
    let folder = if document.id.starts_with("issue-") {
        "issues"
    } else {
        "plans"
    };
    let allowed = [
        format!("knowledge/{folder}/index.md"),
        format!("knowledge/{folder}/open/index.md"),
    ];
    let mut paths = vec![document.path.clone()];
    for index in indexes {
        if !allowed.contains(index) || paths.contains(index) {
            return Err(refusal(
                "publication contains an unrelated or duplicate index path",
            ));
        }
        // Generated index changes must concern this document only. Refuse a
        // mixed index rather than committing another draft's metadata.
        let old = git::file_at_revision(root, "HEAD", index)?;
        let new = text(root, index)?;
        let marker = format!("[sokf:{}]", document.id);
        let without_document = |text: &str| {
            text.lines()
                .filter(|line| !line.contains(&marker))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        if without_document(&old) != without_document(&new) {
            return Err(refusal(
                "generated index contains unrelated changes; separate them before document approval",
            ));
        }
        paths.push(index.clone());
    }
    Ok(paths)
}
