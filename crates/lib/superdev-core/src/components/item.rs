//! components/item.rs — the declarative item list behind the static
//! components. A component describes what it keeps in the repo as
//! [`ManagedItem`]s; one driver derives both `plan` (read, compare, emit)
//! and `owned` (collect the claims) from the same list, so the two cannot
//! disagree.
//!
//! Pins stay with [`super::pin`]; commands and genuinely dynamic state stay
//! hand-written in their components.

use std::path::Path;

use crate::action::{Action, Ownership};
use crate::component::Claim;
use crate::fsutil::has_line;

/// One declarative thing a component keeps in the repo.
pub(crate) enum ManagedItem {
    /// A superdev-owned file: rewritten whenever its content differs, hashed
    /// into the lock.
    OwnedFile {
        /// Repo-relative target path.
        path: String,
        /// Desired content.
        content: String,
        /// Short human reason, shown in plans.
        reason: String,
    },
    /// A write-once scaffold: planned only while absent, the user's from the
    /// moment it exists. Never claimed.
    Scaffold {
        /// Repo-relative target path.
        path: String,
        /// Initial content.
        content: String,
        /// Short human reason, shown in plans.
        reason: String,
    },
    /// A line guaranteed in a shared file, by exact whole-line match. Never
    /// hashed, never claimed — deleting it just means the next sync puts it
    /// back.
    EnsureLine {
        /// Repo-relative target path.
        path: String,
        /// Exact line to guarantee.
        line: String,
        /// Short human reason, shown in plans.
        reason: String,
    },
    /// A managed entry in a shared JSON file: one key, or — with a marker —
    /// one superdev-owned array element. Compared by parsed value, so
    /// reformatting the file is not drift; claimed by canonical value.
    JsonEntry {
        /// Repo-relative target path.
        path: String,
        /// Dotted key path.
        pointer: String,
        /// Substring identifying superdev's element in an array at `pointer`;
        /// `None` manages the key itself.
        marker: Option<String>,
        /// The desired value, as a JSON string.
        value_json: String,
    },
}

/// The diff for `items`, in list order: one action per item that is missing
/// or differs. Unreadable targets count as missing — the engine reports why
/// when it tries to write.
pub(crate) fn plan_items(root: &Path, items: &[ManagedItem]) -> Vec<Action> {
    let mut actions = Vec::new();
    for item in items {
        match item {
            ManagedItem::OwnedFile {
                path,
                content,
                reason,
            } => {
                let existing = std::fs::read_to_string(root.join(path)).ok();
                if existing.as_deref() != Some(content.as_str()) {
                    actions.push(Action::WriteFile {
                        path: path.clone(),
                        content: content.clone(),
                        ownership: Ownership::Owned,
                        reason: reason.clone(),
                    });
                }
            }
            ManagedItem::Scaffold {
                path,
                content,
                reason,
            } => {
                if std::fs::read_to_string(root.join(path)).is_err() {
                    actions.push(Action::WriteFile {
                        path: path.clone(),
                        content: content.clone(),
                        ownership: Ownership::Scaffold,
                        reason: reason.clone(),
                    });
                }
            }
            ManagedItem::EnsureLine { path, line, reason } => {
                let content = std::fs::read_to_string(root.join(path)).unwrap_or_default();
                if !has_line(&content, line) {
                    actions.push(Action::EnsureLine {
                        path: path.clone(),
                        line: line.clone(),
                        reason: reason.clone(),
                        append_note: None,
                    });
                }
            }
            ManagedItem::JsonEntry {
                path,
                pointer,
                marker,
                value_json,
            } => {
                if json_entry_missing(root, path, pointer, marker.as_deref(), value_json) {
                    actions.push(match marker {
                        None => Action::SetJsonKey {
                            path: path.clone(),
                            pointer: pointer.clone(),
                            value_json: value_json.clone(),
                        },
                        Some(marker) => Action::EnsureJsonArrayElement {
                            path: path.clone(),
                            pointer: pointer.clone(),
                            marker: marker.clone(),
                            value_json: value_json.clone(),
                        },
                    });
                }
            }
        }
    }
    actions
}

/// The claims `items` cover, in list order. Scaffolds and lines carry none.
pub(crate) fn claims(items: &[ManagedItem]) -> Vec<Claim> {
    items
        .iter()
        .filter_map(|item| match item {
            ManagedItem::OwnedFile { path, .. } => Some(Claim::File(path.clone())),
            ManagedItem::Scaffold { .. } | ManagedItem::EnsureLine { .. } => None,
            ManagedItem::JsonEntry {
                path,
                pointer,
                marker,
                ..
            } => Some(Claim::JsonKey {
                path: path.clone(),
                pointer: match marker {
                    None => pointer.clone(),
                    Some(marker) => format!("{pointer}[{marker}]"),
                },
            }),
        })
        .collect()
}

/// True when the file does not already carry the desired entry. Compares the
/// parsed values, so reformatting or reordering the file is not drift. An
/// unreadable or malformed file counts as missing: the engine reports why.
fn json_entry_missing(
    root: &Path,
    path: &str,
    pointer: &str,
    marker: Option<&str>,
    value_json: &str,
) -> bool {
    let wanted: serde_json::Value =
        serde_json::from_str(value_json).expect("the item's value literal is valid JSON");
    let Some(at_pointer) = std::fs::read_to_string(root.join(path))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|root| {
            pointer
                .split('.')
                .try_fold(&root, |value, segment| value.get(segment))
                .cloned()
        })
    else {
        return true;
    };
    match marker {
        None => at_pointer != wanted,
        Some(_) => !at_pointer
            .as_array()
            .is_some_and(|items| items.contains(&wanted)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(path: &str, content: &str) -> ManagedItem {
        ManagedItem::OwnedFile {
            path: path.into(),
            content: content.into(),
            reason: "test".into(),
        }
    }

    #[test]
    fn each_kind_plans_when_missing_and_skips_when_satisfied() {
        let dir = tempfile::tempdir().unwrap();
        let items = vec![
            owned("owned.txt", "wanted"),
            ManagedItem::Scaffold {
                path: "scaffold.txt".into(),
                content: "seed".into(),
                reason: "test".into(),
            },
            ManagedItem::EnsureLine {
                path: ".gitignore".into(),
                line: ".superdev/cache/".into(),
                reason: "test".into(),
            },
            ManagedItem::JsonEntry {
                path: ".mcp.json".into(),
                pointer: "mcpServers.superdev-aokf".into(),
                marker: None,
                value_json: r#"{"command":"superdev"}"#.into(),
            },
            ManagedItem::JsonEntry {
                path: ".claude/settings.json".into(),
                pointer: "hooks.PostToolUse".into(),
                marker: Some("superdev aokf".into()),
                value_json: r#"{"matcher":"superdev aokf"}"#.into(),
            },
        ];
        // Empty repo: every item is planned, in list order.
        let descs: Vec<String> = plan_items(dir.path(), &items)
            .iter()
            .map(Action::describe)
            .collect();
        assert_eq!(descs.len(), 5, "{descs:?}");
        assert!(descs[0].starts_with("write owned.txt"), "{descs:?}");
        assert!(descs[4].starts_with("ensure .claude/settings.json"));

        // Satisfy each one; nothing is planned.
        std::fs::write(dir.path().join("owned.txt"), "wanted").unwrap();
        std::fs::write(dir.path().join("scaffold.txt"), "user edited").unwrap();
        // The line matches whole lines only, so a superstring is not enough —
        // the shared predicate the engine also applies.
        std::fs::write(
            dir.path().join(".gitignore"),
            "prefix.superdev/cache/suffix\n",
        )
        .unwrap();
        assert_eq!(
            plan_items(dir.path(), &items).len(),
            3,
            "superstring is not the line"
        );
        std::fs::write(dir.path().join(".gitignore"), ".superdev/cache/\n").unwrap();
        // Reformatted and reordered JSON is not drift: parsed compare.
        std::fs::write(
            dir.path().join(".mcp.json"),
            "{\n  \"mcpServers\": {\"superdev-aokf\": {\"command\": \"superdev\"}, \"theirs\": {}}\n}",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            r#"{"hooks":{"PostToolUse":[{"matcher":"superdev aokf"},{"matcher":"user"}]}}"#,
        )
        .unwrap();
        assert!(plan_items(dir.path(), &items).is_empty());

        // Drift in the owned file replans it alone; the touched scaffold stays
        // the user's.
        std::fs::write(dir.path().join("owned.txt"), "edited").unwrap();
        let planned = plan_items(dir.path(), &items);
        assert_eq!(planned.len(), 1);
        assert!(planned[0].describe().starts_with("write owned.txt"));
    }

    #[test]
    fn claims_cover_the_lockable_kinds_only() {
        let items = vec![
            owned("a.txt", "x"),
            ManagedItem::Scaffold {
                path: "s.txt".into(),
                content: "x".into(),
                reason: "test".into(),
            },
            ManagedItem::EnsureLine {
                path: "f".into(),
                line: "l".into(),
                reason: "test".into(),
            },
            ManagedItem::JsonEntry {
                path: ".mcp.json".into(),
                pointer: "mcpServers.superdev-aokf".into(),
                marker: None,
                value_json: "{}".into(),
            },
            ManagedItem::JsonEntry {
                path: ".claude/settings.json".into(),
                pointer: "hooks.PostToolUse".into(),
                marker: Some("marker".into()),
                value_json: "{}".into(),
            },
        ];
        let keys: Vec<String> = claims(&items).iter().map(Claim::lock_key).collect();
        assert_eq!(
            keys,
            vec![
                "a.txt".to_string(),
                ".mcp.json:mcpServers.superdev-aokf".to_string(),
                ".claude/settings.json:hooks.PostToolUse[marker]".to_string(),
            ]
        );
    }
}
