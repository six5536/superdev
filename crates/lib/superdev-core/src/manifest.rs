//! manifest.rs — .superdev/config.toml: what the repo wants.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::capability::Capability;
use crate::error::{Error, Result};
use crate::registry;

/// Repo-relative path of the manifest.
pub const CONFIG_PATH: &str = ".superdev/config.toml";

/// One enabled capability: which provider fills it, at which version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityConfig {
    /// Provider id (e.g. "codegraph").
    pub provider: String,
    /// Version pin; None when the source manages versions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// The manifest: blueprint version plus one table per enabled capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Blueprint (= superdev) version that wrote this file.
    pub blueprint: String,
    /// Enabled capabilities, keyed by kebab-case name. Absent = disabled.
    #[serde(flatten)]
    pub capabilities: BTreeMap<String, CapabilityConfig>,
}

impl Manifest {
    /// Registry defaults, minus `disabled` and any unavailable slot.
    pub fn default_for(blueprint: &str, disabled: &[Capability]) -> Manifest {
        let capabilities = registry::entries()
            .iter()
            .filter(|e| e.available && !disabled.contains(&e.capability))
            .map(|e| {
                (
                    e.capability.as_str().to_string(),
                    CapabilityConfig {
                        provider: e.provider.to_string(),
                        version: e.version.map(str::to_string),
                    },
                )
            })
            .collect();
        Manifest {
            blueprint: blueprint.to_string(),
            capabilities,
        }
    }

    /// Parse and validate manifest TOML.
    pub fn parse(s: &str) -> Result<Manifest> {
        let m: Manifest = toml_edit::de::from_str(s).map_err(|e| Error::Toml {
            path: CONFIG_PATH.into(),
            message: e.to_string(),
        })?;
        for name in m.capabilities.keys() {
            if Capability::parse(name).is_none() {
                return Err(Error::Manifest {
                    message: format!("unknown capability `{name}`"),
                });
            }
        }
        Ok(m)
    }

    /// Serialise to TOML (blueprint key first, then capability tables).
    pub fn to_toml(&self) -> String {
        toml_edit::ser::to_string_pretty(self).expect("manifest serialises")
    }

    /// Read from `<root>/.superdev/config.toml`.
    pub fn load(root: &Path) -> Result<Manifest> {
        let path = root.join(CONFIG_PATH);
        let s = fs::read_to_string(&path).map_err(|e| Error::Io {
            path: path.clone(),
            source: e,
        })?;
        Manifest::parse(&s)
    }

    /// Write to `<root>/.superdev/config.toml`, creating `.superdev/`.
    pub fn save(&self, root: &Path) -> Result<()> {
        let path = root.join(CONFIG_PATH);
        let dir = path.parent().expect("config path has a parent");
        fs::create_dir_all(dir).map_err(|e| Error::Io {
            path: dir.into(),
            source: e,
        })?;
        fs::write(&path, self.to_toml()).map_err(|e| Error::Io { path, source: e })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;

    #[test]
    fn default_manifest_round_trips() {
        let m = Manifest::default_for("0.1.0", &[Capability::CodeIndex]);
        assert!(!m.capabilities.contains_key("code-index"));
        assert!(!m.capabilities.contains_key("skills")); // unavailable slot never defaults on
        assert_eq!(
            m.capabilities["workflows"].version.as_deref(),
            Some("6.2.0")
        );
        let parsed = Manifest::parse(&m.to_toml()).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn spec_shape_parses() {
        let m = Manifest::parse(
            "blueprint = \"0.1.0\"\n\n[knowledge]\nprovider = \"aokf\"\n\n[code-index]\nprovider = \"codegraph\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        assert_eq!(
            m.capabilities["code-index"].version.as_deref(),
            Some("1.2.3")
        );
    }

    #[test]
    fn unknown_capability_is_rejected() {
        let err =
            Manifest::parse("blueprint = \"0.1.0\"\n[flying]\nprovider = \"x\"\n").unwrap_err();
        assert!(err.to_string().contains("flying"));
    }

    #[test]
    fn malformed_toml_names_the_config_path() {
        let err = Manifest::parse("blueprint =").unwrap_err();
        assert!(err.to_string().starts_with(CONFIG_PATH));
    }

    #[test]
    fn io_failures_surface_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = Manifest::load(dir.path()).unwrap_err();
        assert!(missing.to_string().contains("config.toml"));

        // A file where `.superdev/` should go makes the directory uncreatable.
        std::fs::write(dir.path().join(".superdev"), "").unwrap();
        let blocked = Manifest::default_for("0.1.0", &[])
            .save(dir.path())
            .unwrap_err();
        assert!(blocked.to_string().contains(".superdev"));
    }

    #[test]
    fn load_and_save_use_the_dot_dir() {
        let dir = tempfile::tempdir().unwrap();
        let m = Manifest::default_for("0.1.0", &[]);
        m.save(dir.path()).unwrap();
        assert!(dir.path().join(".superdev/config.toml").is_file());
        assert_eq!(Manifest::load(dir.path()).unwrap(), m);
    }
}
