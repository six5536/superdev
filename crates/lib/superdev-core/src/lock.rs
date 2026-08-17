//! lock.rs — .superdev/lock.toml: what superdev last applied.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

/// Repo-relative path of the lock file.
pub const LOCK_PATH: &str = ".superdev/lock.toml";

/// What one capability had applied last.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedComponent {
    /// Provider that was applied.
    pub provider: String,
    /// Version that was applied, when pinned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Last-applied state: how `status` tells deliberate user change from drift.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// Per-capability applied provider/version.
    #[serde(default)]
    pub components: BTreeMap<String, LockedComponent>,
    /// sha256 of superdev-owned content, keyed by repo-relative path
    /// (`.mise.toml:<tool>` for managed mise keys).
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    /// Which capability materialised each `files` entry, for entries copied
    /// from a provider checkout rather than embedded content. Everything
    /// else never appears here.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub owners: BTreeMap<String, String>,
}

/// Lowercase-hex sha256.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

impl Lock {
    /// Read from `<root>/.superdev/lock.toml`; a missing file is an empty lock.
    pub fn load(root: &Path) -> Result<Lock> {
        let path = root.join(LOCK_PATH);
        match fs::read_to_string(&path) {
            Ok(s) => toml_edit::de::from_str(&s).map_err(|e| Error::Toml {
                path,
                message: e.to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Lock::default()),
            Err(e) => Err(Error::Io { path, source: e }),
        }
    }

    /// Write to `<root>/.superdev/lock.toml`, creating `.superdev/`.
    pub fn save(&self, root: &Path) -> Result<()> {
        let path = root.join(LOCK_PATH);
        let dir = path.parent().expect("lock path has a parent");
        fs::create_dir_all(dir).map_err(|e| Error::Io {
            path: dir.into(),
            source: e,
        })?;
        let s = toml_edit::ser::to_string_pretty(self).expect("lock serialises");
        fs::write(&path, s).map_err(|e| Error::Io { path, source: e })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_known_vector() {
        // sha256("abc")
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn missing_lock_is_default() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(Lock::load(dir.path()).unwrap(), Lock::default());
    }

    #[test]
    fn lock_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let mut lock = Lock::default();
        lock.components.insert(
            "workflows".into(),
            LockedComponent {
                provider: "superpowers".into(),
                version: Some("6.2.0".into()),
            },
        );
        lock.files
            .insert(".agents/aokf/SPEC.md".into(), sha256_hex(b"spec"));
        lock.save(dir.path()).unwrap();
        assert_eq!(Lock::load(dir.path()).unwrap(), lock);
    }

    #[test]
    fn a_0_1_0_lock_reads_unchanged() {
        let toml = r#"[components.skills]
provider = "superdev-skills"
version = "0.1.0"

[files]
".agents/aokf/SPEC.md" = "aaaa"
".mise.toml:http:superpowers" = "bbbb"
".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]" = "cccc"
"#;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".superdev")).unwrap();
        std::fs::write(dir.path().join(LOCK_PATH), toml).unwrap();
        let lock = Lock::load(dir.path()).unwrap();
        assert_eq!(lock.components["skills"].provider, "superdev-skills");
        assert_eq!(lock.files[".mise.toml:http:superpowers"], "bbbb");
        assert_eq!(lock.files.len(), 3);
        assert!(lock.owners.is_empty());
    }

    #[test]
    fn owners_round_trip_and_stay_optional() {
        let dir = tempfile::tempdir().unwrap();
        let mut lock = Lock::default();
        lock.files
            .insert(".claude/skills/tdd/SKILL.md".into(), sha256_hex(b"x"));
        lock.owners
            .insert(".claude/skills/tdd/SKILL.md".into(), "workflows".into());
        lock.save(dir.path()).unwrap();
        assert_eq!(Lock::load(dir.path()).unwrap(), lock);
        // Without owners the table is absent entirely.
        let plain = Lock::default();
        plain.save(dir.path()).unwrap();
        let text = std::fs::read_to_string(dir.path().join(LOCK_PATH)).unwrap();
        assert!(!text.contains("owners"), "{text}");
    }

    #[test]
    fn io_and_parse_failures_surface_the_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".superdev")).unwrap();
        std::fs::write(dir.path().join(LOCK_PATH), "components =").unwrap();
        let malformed = Lock::load(dir.path()).unwrap_err();
        assert!(malformed.to_string().contains("lock.toml"));

        // A directory where the lock file should be: a read error that is not NotFound.
        let unreadable = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(unreadable.path().join(LOCK_PATH)).unwrap();
        let err = Lock::load(unreadable.path()).unwrap_err();
        assert!(err.to_string().contains("lock.toml"));

        // A file where `.superdev/` should go makes the directory uncreatable.
        let blocked = tempfile::tempdir().unwrap();
        std::fs::write(blocked.path().join(".superdev"), "").unwrap();
        let err = Lock::default().save(blocked.path()).unwrap_err();
        assert!(err.to_string().contains(".superdev"));
    }
}
