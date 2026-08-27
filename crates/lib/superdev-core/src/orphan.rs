//! orphan.rs — what the lock records that no live claim covers.
//!
//! There are no migration scripts: the lock is what superdev applied, the
//! components' claims are what the blueprint wants now, and the difference
//! is the migration.

use std::collections::BTreeSet;
use std::path::Path;

use crate::action::Action;
use crate::component::Claim;
use crate::error::Result;
use crate::lock::{Lock, sha256_hex};

/// The orphan pass, computed against the lock. Planning only; the engine
/// runs the actions and the caller drops the released and gone keys.
#[derive(Debug, Default)]
pub struct OrphanPlan {
    /// Removals of superdev's own residue, in lock order.
    pub actions: Vec<Action>,
    /// Lock keys whose content the user changed: left in place, released.
    pub released: Vec<String>,
    /// Lock keys whose target is already gone: dropped silently.
    pub gone: Vec<String>,
}

impl OrphanPlan {
    /// One report line per released orphan.
    pub fn released_lines(&self) -> Vec<String> {
        self.released
            .iter()
            .map(|key| {
                format!(
                    "orphan: {key} changed since superdev wrote it — left in place, released from the lock"
                )
            })
            .collect()
    }
}

/// Every lock `files` entry no claim covers, classified by what is on disk.
/// Fails on an orphan it cannot read — the rule everywhere the engine
/// refuses to guess about content.
pub fn plan(root: &Path, lock: &Lock, claims: &[Claim]) -> Result<OrphanPlan> {
    let claimed: BTreeSet<String> = claims.iter().map(Claim::lock_key).collect();
    let mut plan = OrphanPlan::default();
    for (key, locked_hash) in &lock.files {
        if claimed.contains(key) {
            continue;
        }
        let claim = Claim::parse_key(key);
        match claim.read_current(root)? {
            None => plan.gone.push(key.clone()),
            Some(value) if sha256_hex(value.as_bytes()) == *locked_hash => {
                plan.actions.push(Action::Remove {
                    claim,
                    reason: "no longer in the blueprint".into(),
                });
            }
            Some(_) => plan.released.push(key.clone()),
        }
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::component::Claim;
    use crate::components::mise;
    use crate::lock::{Lock, sha256_hex};

    fn lock_with(entries: &[(&str, &str)]) -> Lock {
        let mut lock = Lock::default();
        for (key, content) in entries {
            lock.files
                .insert((*key).into(), sha256_hex(content.as_bytes()));
        }
        lock
    }

    #[test]
    fn claimed_entries_are_never_orphans() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("kept.txt"), "content").unwrap();
        let lock = lock_with(&[("kept.txt", "content")]);
        let claims = vec![Claim::File("kept.txt".into())];
        let plan = plan(dir.path(), &lock, &claims).unwrap();
        assert!(plan.actions.is_empty());
        assert!(plan.released.is_empty());
        assert!(plan.gone.is_empty());
    }

    #[test]
    fn each_shape_classifies_by_disk_state() {
        let dir = tempfile::tempdir().unwrap();
        // Unmodified file → removal. Modified file → released. Missing → gone.
        std::fs::write(dir.path().join("stale.txt"), "superdev's").unwrap();
        std::fs::write(dir.path().join("theirs.txt"), "edited").unwrap();
        // Unmodified pin → removal.
        let mise_toml = mise::set_pin("", "http:codegraph", "\"1.5.0\"").unwrap();
        std::fs::write(dir.path().join(".mise.toml"), &mise_toml).unwrap();
        let pin_value = mise::current_pin(&mise_toml, "http:codegraph")
            .unwrap()
            .unwrap();
        // Unmodified JSON key → removal.
        std::fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"superdev-sokf":{"command":"superdev"}}}"#,
        )
        .unwrap();
        let mcp_value: serde_json::Value =
            serde_json::from_str(r#"{"command":"superdev"}"#).unwrap();

        let mut lock = lock_with(&[
            ("stale.txt", "superdev's"),
            ("theirs.txt", "superdev's"),
            ("vanished.txt", "whatever"),
        ]);
        lock.files.insert(
            mise::pin_lock_key("http:codegraph"),
            sha256_hex(pin_value.as_bytes()),
        );
        lock.files.insert(
            ".mcp.json:mcpServers.superdev-sokf".into(),
            sha256_hex(mcp_value.to_string().as_bytes()),
        );

        let plan = plan(dir.path(), &lock, &[]).unwrap();
        assert_eq!(plan.released, vec!["theirs.txt".to_string()]);
        assert_eq!(plan.gone, vec!["vanished.txt".to_string()]);
        let descs: Vec<String> = plan.actions.iter().map(Action::describe).collect();
        assert!(
            descs.contains(&"remove stale.txt (no longer in the blueprint)".into()),
            "{descs:?}"
        );
        assert!(
            descs.contains(&"unpin http:codegraph in .mise.toml".into()),
            "{descs:?}"
        );
        assert!(
            descs.contains(&"remove mcpServers.superdev-sokf from .mcp.json".into()),
            "{descs:?}"
        );
        assert_eq!(
            plan.released_lines(),
            vec![
                "orphan: theirs.txt changed since superdev wrote it — left in place, released from the lock"
                    .to_string()
            ]
        );
    }

    #[test]
    fn an_absent_pin_or_key_in_a_present_file_is_gone() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mise.toml"), "[tools]\nnode = \"24\"\n").unwrap();
        std::fs::write(dir.path().join(".mcp.json"), r#"{"mcpServers":{}}"#).unwrap();
        let lock = lock_with(&[
            (".mise.toml:http:codegraph", "x"),
            (".mcp.json:mcpServers.superdev-sokf", "x"),
        ]);
        let plan = plan(dir.path(), &lock, &[]).unwrap();
        assert!(plan.actions.is_empty());
        assert_eq!(plan.gone.len(), 2);
    }

    #[test]
    fn an_unreadable_orphan_fails_loudly() {
        let dir = tempfile::tempdir().unwrap();
        // A directory where the locked file should be.
        std::fs::create_dir(dir.path().join("was-a-file.txt")).unwrap();
        let lock = lock_with(&[("was-a-file.txt", "content")]);
        assert!(plan(dir.path(), &lock, &[]).is_err());
        // Malformed shared files are errors too, never guesses.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mise.toml"), "[tools\n").unwrap();
        let lock = lock_with(&[(".mise.toml:http:codegraph", "x")]);
        assert!(plan(dir.path(), &lock, &[]).is_err());
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mcp.json"), "not json").unwrap();
        let lock = lock_with(&[(".mcp.json:mcpServers.superdev-sokf", "x")]);
        assert!(plan(dir.path(), &lock, &[]).is_err());
    }
}
