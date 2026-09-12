//! Approval becomes usable only after exact commit publication and record persistence.

use super::{super::store::hash, *};

pub(super) fn approve(
    root: &Path,
    store: &Store,
    record: &mut WorkflowRecord,
    change: &ScopeChange,
    controller: &Controller<'_>,
) -> Result<()> {
    let ScopeChange::Approve {
        document: kind,
        expected_hash: expected,
        input,
        indexes,
    } = change
    else {
        return Err(refusal(
            "publication requires a typed document approval request",
        ));
    };
    let kind = *kind;
    require_scope(record)?;
    require_branch(root, record)?;
    verify_input(record, input, controller)?;
    let document = documents::document(root, record, kind)?;
    if document.hash != *expected {
        return Err(refusal(
            "document changed after the approval question; ask about its current revision",
        ));
    }
    let issue_hash = match kind {
        DocumentKind::Issue => None,
        DocumentKind::Plan => Some(documents::approved_issue(root, record)?.hash),
    };
    let paths = documents::indexes(root, &document, indexes)?;
    let prepared = git::prepare_document_publication(
        root,
        &paths,
        &format!("docs(workflow): publish {} for {}", document.id, record.id),
    )?;
    let committed = git::file_at_revision(root, &prepared.commit, &document.path)?;
    if hash(committed.as_bytes()) != document.hash {
        return Err(refusal(
            "prepared commit differs from the approved document bytes; inspect Git filters or concurrent edits",
        ));
    }
    record.pending_publication = Some(PendingPublication {
        document,
        input: input.clone(),
        parent: prepared.parent,
        commit: prepared.commit,
        branch: git::current_branch(root)?,
        paths,
        issue_hash,
    });
    store.save(record)?;
    recover(root, store, record)
}

pub(super) fn recover(root: &Path, store: &Store, record: &mut WorkflowRecord) -> Result<()> {
    let pending = record
        .pending_publication
        .clone()
        .ok_or_else(|| refusal("no pending publication to recover"))?;
    let kind = if pending.issue_hash.is_some() {
        DocumentKind::Plan
    } else {
        DocumentKind::Issue
    };
    let current = documents::document(root, record, kind)?;
    if current != pending.document {
        return Err(refusal(
            "pending publication document changed; inspect and restore the intended revision before recovery",
        ));
    }
    if let Some(expected) = &pending.issue_hash {
        let issue = documents::approved_issue(root, record)?;
        if issue.hash != *expected {
            return Err(refusal(
                "issue changed during plan publication; inspect the pending approval before recovery",
            ));
        }
    }
    let changed = git::changed_paths(root, &pending.parent, &pending.commit)?;
    if changed.iter().any(|path| !pending.paths.contains(path)) {
        return Err(refusal(
            "pending publication commit includes unrelated paths",
        ));
    }
    for path in &pending.paths {
        let committed = git::file_at_revision(root, &pending.commit, path)?;
        let current = documents::text(root, path)?;
        if committed != current {
            return Err(refusal(format!(
                "publication path `{path}` changed; inspect before recovery"
            )));
        }
    }
    git::finish_document_publication(
        root,
        &pending.parent,
        &pending.commit,
        &pending.branch,
        &pending.paths,
    )?;
    let approval = DocumentApproval {
        original: pending.document,
        input: pending.input,
        commit: pending.commit,
        compatible: Vec::new(),
        issue_hash: pending.issue_hash,
    };
    match kind {
        DocumentKind::Issue => {
            // A fresh issue approval can follow substantive changes. No plan
            // approval from the previous intent survives that replacement.
            record.issue_approval = Some(approval);
            record.plan_approval = None;
            record.scope_step = ScopeStep::WritePlan;
        }
        DocumentKind::Plan => {
            record.plan_approval = Some(approval);
            record.scope_step = ScopeStep::Handoff;
        }
    }
    record.pending_publication = None;
    store.save(record)
}
