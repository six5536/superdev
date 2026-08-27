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

/// A capability's lock shape as written: one record table for a single
/// entry, an array of tables from two up — mirroring the manifest so the
/// single case keeps its `[components.<name>]` shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum WrittenRecords {
    /// A single `[components.<name>]` table.
    One(LockedComponent),
    /// `[[components.<name>]]` entries, one per provider.
    Many(Vec<LockedComponent>),
}

/// (De)serialise the components map through [`WrittenRecords`].
mod records_serde {
    use super::{BTreeMap, LockedComponent, WrittenRecords};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        map: &BTreeMap<String, Vec<LockedComponent>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let written: BTreeMap<&String, WrittenRecords> = map
            .iter()
            .map(|(name, records)| {
                let entry = match records.as_slice() {
                    [only] => WrittenRecords::One(only.clone()),
                    _ => WrittenRecords::Many(records.clone()),
                };
                (name, entry)
            })
            .collect();
        written.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<String, Vec<LockedComponent>>, D::Error> {
        let written: BTreeMap<String, WrittenRecords> = Deserialize::deserialize(deserializer)?;
        Ok(written
            .into_iter()
            .map(|(name, entry)| {
                let records = match entry {
                    WrittenRecords::One(record) => vec![record],
                    WrittenRecords::Many(records) => records,
                };
                (name, records)
            })
            .collect())
    }
}

/// One resolved pack, recorded so a later run can prove it got the same bytes,
/// and so a dropped entry's files become orphans by the existing rule.
/// Per-file hashes stay in the lock's existing `files` map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackLock {
    /// The source as the manifest wrote it.
    pub source: String,
    /// The normalised comparison key every spelling of one source shares.
    pub identity: String,
    /// The revision resolved, for a git source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// What a fetched pack was verified against, checked on every later run.
    ///
    /// Absent for a path source: a directory is read afresh every run, so
    /// there are no pinned bytes to verify against, and a value recorded here
    /// would be rewritten by every commit touching the pack and read by
    /// nothing. Absent exactly when `rev` is. ADR-016.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    /// The `format` the pack's own manifest declared.
    pub format: u32,
}

/// Last-applied state: how `status` tells deliberate user change from drift.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    /// The packs the last apply resolved, in manifest order. Empty when no
    /// pack was named, which is the default path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<PackLock>,
    /// Applied provider/version records, keyed by capability — one per
    /// enabled (capability, provider) entry.
    #[serde(default, with = "records_serde")]
    pub components: BTreeMap<String, Vec<LockedComponent>>,
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
    /// A lock an earlier binary wrote carries no `[[packs]]`; the new field
    /// must not appear in it on the next save.
    #[test]
    fn a_lock_without_packs_round_trips_byte_identically() {
        let written = "[components.skills]\nprovider = \"superdev-skills\"\n\n[files]\n\"knowledge/index.md\" = \"abc\"\n";
        let lock: Lock = toml_edit::de::from_str(written).unwrap();
        assert!(lock.packs.is_empty());
        assert_eq!(toml_edit::ser::to_string_pretty(&lock).unwrap(), written);
    }

    #[test]
    fn a_lock_with_packs_round_trips() {
        let written = concat!(
            "[[packs]]\n",
            "source = \"github:six5536/superdev\"\n",
            "identity = \"github.com/six5536/superdev\"\n",
            "rev = \"assets-v1.4.0\"\n",
            "digest = \"sha256:9f2a\"\n",
            "format = 1\n\n",
            "[components.skills]\nprovider = \"superdev-skills\"\n\n",
            "[files]\n\"knowledge/index.md\" = \"abc\"\n",
        );
        let lock: Lock = toml_edit::de::from_str(written).unwrap();
        assert_eq!(
            lock.packs,
            [PackLock {
                source: "github:six5536/superdev".into(),
                identity: "github.com/six5536/superdev".into(),
                rev: Some("assets-v1.4.0".into()),
                digest: Some("sha256:9f2a".into()),
                format: 1,
            }]
        );
        assert_eq!(toml_edit::ser::to_string_pretty(&lock).unwrap(), written);
    }

    /// A path source has no rev, and the key must stay out of the file
    /// rather than appear empty.
    #[test]
    fn a_pack_lock_without_a_rev_omits_the_key() {
        let lock = Lock {
            packs: vec![PackLock {
                source: "./packs/acme".into(),
                identity: "/repo/packs/acme".into(),
                rev: None,
                digest: Some("sha256:0000".into()),
                format: 1,
            }],
            ..Lock::default()
        };
        let written = toml_edit::ser::to_string_pretty(&lock).unwrap();
        assert!(!written.contains("rev"), "{written}");
    }

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
            "code-index".into(),
            vec![LockedComponent {
                provider: "codegraph".into(),
                version: Some("1.5.0".into()),
            }],
        );
        lock.files
            .insert(".agents/sokf/SPEC.md".into(), sha256_hex(b"spec"));
        lock.save(dir.path()).unwrap();
        assert_eq!(Lock::load(dir.path()).unwrap(), lock);
    }

    #[test]
    fn a_many_slot_locks_one_record_per_provider() {
        let dir = tempfile::tempdir().unwrap();
        let mut lock = Lock::default();
        lock.components.insert(
            "skills".into(),
            vec![
                LockedComponent {
                    provider: "superdev-skills".into(),
                    version: Some("0.1.0".into()),
                },
                LockedComponent {
                    provider: "another-pack".into(),
                    version: Some("1.2.0".into()),
                },
            ],
        );
        lock.save(dir.path()).unwrap();
        let text = std::fs::read_to_string(dir.path().join(LOCK_PATH)).unwrap();
        assert!(text.contains("[[components.skills]]"), "{text}");
        assert_eq!(Lock::load(dir.path()).unwrap(), lock);
        // One record keeps the single-table shape old locks already carry.
        lock.components.get_mut("skills").unwrap().pop();
        lock.save(dir.path()).unwrap();
        let text = std::fs::read_to_string(dir.path().join(LOCK_PATH)).unwrap();
        assert!(text.contains("[components.skills]"), "{text}");
        assert!(!text.contains("[[components.skills]]"), "{text}");
        assert_eq!(Lock::load(dir.path()).unwrap(), lock);
    }

    #[test]
    fn a_0_1_0_lock_reads_unchanged() {
        let toml = r#"[components.skills]
provider = "superdev-skills"
version = "0.1.0"

[files]
".agents/sokf/SPEC.md" = "aaaa"
".mise.toml:http:codegraph" = "bbbb"
".claude/settings.json:hooks.PostToolUse[superdev hook validate]" = "cccc"
"#;
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".superdev")).unwrap();
        std::fs::write(dir.path().join(LOCK_PATH), toml).unwrap();
        let lock = Lock::load(dir.path()).unwrap();
        assert_eq!(lock.components["skills"][0].provider, "superdev-skills");
        assert_eq!(lock.files[".mise.toml:http:codegraph"], "bbbb");
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
