//! manifest.rs — .superdev/config.toml: what the repo wants.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::aokf::embed::EmbeddingsConfig;
use crate::capability::{Capability, Cardinality};
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
    /// Embedding provider for capabilities that index text. Absent = the
    /// bundled local model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<EmbeddingsConfig>,
    /// Skills released from management: superdev stops writing them and
    /// `status` reports them as custom. Honoured by `skills` and `knowledge`,
    /// which both write into `.claude/skills/`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,
}

/// The project template `init` seeded this repo from: provenance, not
/// management — no verb ever re-plans a template. Recording the token values
/// beside the name answers "what seeded this repo, with what".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateRecord {
    /// The shipped template's name.
    pub name: String,
    /// The value `{{superdev:project-name}}` substituted to.
    pub project_name: String,
    /// The value `{{superdev:project-slug}}` substituted to.
    pub project_slug: String,
    /// The binary version whose template content the repo last matched:
    /// stamped by `init`, restamped by the `template-update` skill after an
    /// update or adoption. Absent in manifests from before the field existed
    /// — they just lack the skill's "already up to date" short-circuit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// A capability's manifest shape as written: one table, or — for a many
/// slot — an array of tables, one per provider entry. Kept distinct through
/// parsing so the array form on a single slot can be refused, then
/// normalised to a list either way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum WrittenEntries {
    /// A single `[name]` table.
    One(CapabilityConfig),
    /// `[[name]]` array-of-tables entries.
    Many(Vec<CapabilityConfig>),
}

/// The manifest as TOML sees it — the on-disk shape `parse` validates and
/// `to_toml` renders. Field order is the serialised order.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WrittenManifest {
    blueprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    template: Option<TemplateRecord>,
    #[serde(flatten)]
    capabilities: BTreeMap<String, WrittenEntries>,
}

/// The manifest: blueprint version plus, per enabled capability, its
/// provider entries — one for a single slot, a set for a many slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Blueprint (= superdev) version that wrote this file.
    pub blueprint: String,
    /// The template `init` seeded this repo from, when one was chosen.
    pub template: Option<TemplateRecord>,
    /// Enabled capabilities, keyed by kebab-case name. Absent = disabled.
    /// Every list is non-empty; single slots hold exactly one entry.
    pub capabilities: BTreeMap<String, Vec<CapabilityConfig>>,
}

impl Manifest {
    /// Registry defaults, minus `disabled` and any unavailable slot.
    pub fn default_for(blueprint: &str, disabled: &[Capability]) -> Manifest {
        let capabilities = registry::entries()
            .iter()
            .filter(|e| e.available && e.default && !disabled.contains(&e.capability))
            .map(|e| {
                (
                    e.capability.as_str().to_string(),
                    vec![CapabilityConfig {
                        provider: e.provider.to_string(),
                        version: e.version.map(|p| p.version.to_string()),
                        embeddings: None,
                        custom: Vec::new(),
                    }],
                )
            })
            .collect();
        Manifest {
            blueprint: blueprint.to_string(),
            template: None,
            capabilities,
        }
    }

    /// The entries for `capability`; empty when disabled.
    pub fn configs(&self, capability: Capability) -> &[CapabilityConfig] {
        self.capabilities
            .get(capability.as_str())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// The entry `provider` fills in `capability`, when enabled.
    pub fn config_of(&self, capability: Capability, provider: &str) -> Option<&CapabilityConfig> {
        self.configs(capability)
            .iter()
            .find(|c| c.provider == provider)
    }

    /// Mutable entry `provider` fills in `capability`, when enabled.
    pub fn config_of_mut(
        &mut self,
        capability: Capability,
        provider: &str,
    ) -> Option<&mut CapabilityConfig> {
        self.configs_mut(capability)
            .iter_mut()
            .find(|c| c.provider == provider)
    }

    /// Mutable entries for `capability`; empty when disabled.
    pub fn configs_mut(&mut self, capability: Capability) -> &mut [CapabilityConfig] {
        self.capabilities
            .get_mut(capability.as_str())
            .map(Vec::as_mut_slice)
            .unwrap_or(&mut [])
    }

    /// Whether `capability` is enabled at all.
    pub fn enabled(&self, capability: Capability) -> bool {
        self.capabilities.contains_key(capability.as_str())
    }

    /// Parse and validate manifest TOML.
    pub fn parse(s: &str) -> Result<Manifest> {
        let written: WrittenManifest = toml_edit::de::from_str(s).map_err(|e| Error::Toml {
            path: CONFIG_PATH.into(),
            message: e.to_string(),
        })?;
        let mut capabilities = BTreeMap::new();
        for (name, entries) in written.capabilities {
            if name == "workflows" {
                return Err(Error::Manifest {
                    message: "the workflows capability was removed — delete the [workflows] \
                              table (moving any custom names to [knowledge]); its skill set \
                              now ships with the knowledge capability. superpowers users: \
                              `claude plugin install superpowers`"
                        .into(),
                });
            }
            let Some(capability) = Capability::parse(&name) else {
                return Err(Error::Manifest {
                    message: format!("unknown capability `{name}`"),
                });
            };
            let configs = match entries {
                WrittenEntries::One(config) => vec![config],
                WrittenEntries::Many(_) if capability.cardinality() == Cardinality::Single => {
                    return Err(Error::Manifest {
                        message: format!("{name} holds one provider — use a single [{name}] table"),
                    });
                }
                WrittenEntries::Many(configs) if configs.is_empty() => {
                    return Err(Error::Manifest {
                        message: format!(
                            "{name} lists no entries — add a provider entry or delete the key"
                        ),
                    });
                }
                WrittenEntries::Many(configs) => configs,
            };
            let mut seen = std::collections::BTreeSet::new();
            for config in &configs {
                if !seen.insert(config.provider.as_str()) {
                    return Err(Error::Manifest {
                        message: format!(
                            "{name} lists provider `{}` more than once — each pack appears once",
                            config.provider
                        ),
                    });
                }
            }
            capabilities.insert(name, configs);
        }
        Ok(Manifest {
            blueprint: written.blueprint,
            template: written.template,
            capabilities,
        })
    }

    /// Serialise to TOML (blueprint key first, then capability tables). A
    /// many slot with one entry keeps the single-table shape — the array
    /// form appears only from two entries up, so no manifest changes shape
    /// on a rewrite.
    pub fn to_toml(&self) -> String {
        let capabilities = self
            .capabilities
            .iter()
            .map(|(name, configs)| {
                let entries = match configs.as_slice() {
                    [only] => WrittenEntries::One(only.clone()),
                    _ => WrittenEntries::Many(configs.clone()),
                };
                (name.clone(), entries)
            })
            .collect();
        let written = WrittenManifest {
            blueprint: self.blueprint.clone(),
            template: self.template.clone(),
            capabilities,
        };
        toml_edit::ser::to_string_pretty(&written).expect("manifest serialises")
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
        assert_eq!(
            m.capabilities["skills"][0].version.as_deref(),
            Some(env!("CARGO_PKG_VERSION"))
        );
        let parsed = Manifest::parse(&m.to_toml()).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn a_single_skills_table_is_a_one_entry_set_and_keeps_its_shape() {
        let toml = "blueprint = \"0.1.0\"\n\n[skills]\nprovider = \"superdev-skills\"\nversion = \"0.1.0\"\n";
        let m = Manifest::parse(toml).unwrap();
        assert_eq!(m.capabilities["skills"].len(), 1);
        assert_eq!(m.capabilities["skills"][0].provider, "superdev-skills");
        // A rewrite keeps the single-table shape, not the array form.
        let rendered = m.to_toml();
        assert!(rendered.contains("[skills]"), "{rendered}");
        assert!(!rendered.contains("[[skills]]"), "{rendered}");
    }

    #[test]
    fn multiple_skills_entries_parse_and_render_as_the_array_form() {
        let toml = "blueprint = \"0.1.0\"\n\n\
                    [[skills]]\nprovider = \"superdev-skills\"\nversion = \"0.1.0\"\n\n\
                    [[skills]]\nprovider = \"another-pack\"\nversion = \"1.2.0\"\n";
        let m = Manifest::parse(toml).unwrap();
        let providers: Vec<&str> = m.capabilities["skills"]
            .iter()
            .map(|c| c.provider.as_str())
            .collect();
        assert_eq!(providers, ["superdev-skills", "another-pack"]);
        let rendered = m.to_toml();
        assert!(rendered.contains("[[skills]]"), "{rendered}");
        assert_eq!(Manifest::parse(&rendered).unwrap(), m);
    }

    #[test]
    fn a_one_entry_array_behaves_as_the_single_table() {
        let array =
            Manifest::parse("blueprint = \"0.1.0\"\n[[skills]]\nprovider = \"superdev-skills\"\n")
                .unwrap();
        let table =
            Manifest::parse("blueprint = \"0.1.0\"\n[skills]\nprovider = \"superdev-skills\"\n")
                .unwrap();
        assert_eq!(array, table);
    }

    #[test]
    fn a_duplicated_provider_is_refused() {
        let err = Manifest::parse(
            "blueprint = \"0.1.0\"\n\
             [[skills]]\nprovider = \"superdev-skills\"\n\
             [[skills]]\nprovider = \"superdev-skills\"\n",
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("skills lists provider `superdev-skills` more than once"),
            "{err}"
        );
    }

    #[test]
    fn the_array_form_on_a_single_slot_is_refused() {
        let err = Manifest::parse("blueprint = \"0.1.0\"\n[[knowledge]]\nprovider = \"aokf\"\n")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("knowledge holds one provider — use a single [knowledge] table"),
            "{err}"
        );
    }

    #[test]
    fn an_empty_entry_list_is_refused() {
        let err = Manifest::parse("blueprint = \"0.1.0\"\nskills = []\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("skills lists no entries"), "{err}");
    }

    #[test]
    fn accessors_cover_enabled_disabled_and_per_provider_lookup() {
        let mut m = Manifest::default_for("0.1.0", &[Capability::CodeIndex]);
        assert!(m.enabled(Capability::Skills));
        assert!(!m.enabled(Capability::CodeIndex));
        assert!(m.configs(Capability::CodeIndex).is_empty());
        assert_eq!(m.configs(Capability::Skills).len(), 1);
        assert_eq!(
            m.config_of(Capability::Skills, "superdev-skills")
                .unwrap()
                .provider,
            "superdev-skills"
        );
        assert!(m.config_of(Capability::Skills, "other").is_none());
        m.configs_mut(Capability::Skills)[0].version = Some("9.9.9".into());
        assert_eq!(
            m.configs(Capability::Skills)[0].version.as_deref(),
            Some("9.9.9")
        );
        assert!(m.configs_mut(Capability::CodeIndex).is_empty());
    }

    #[test]
    fn a_workflows_table_gets_the_guided_error() {
        let err = Manifest::parse(
            "blueprint = \"0.1.0\"\n\n[workflows]\nprovider = \"mattpocock-skills\"\n",
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("the workflows capability was removed"),
            "{err}"
        );
        assert!(
            err.contains("moving any custom names to [knowledge]"),
            "{err}"
        );
        assert!(err.contains("claude plugin install superpowers"), "{err}");
    }

    #[test]
    fn spec_shape_parses() {
        let m = Manifest::parse(
            "blueprint = \"0.1.0\"\n\n[knowledge]\nprovider = \"aokf\"\n\n[code-index]\nprovider = \"codegraph\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        assert_eq!(
            m.capabilities["code-index"][0].version.as_deref(),
            Some("1.2.3")
        );
    }

    #[test]
    fn embeddings_survive_a_round_trip_and_stay_optional() {
        let mut m = Manifest::default_for("0.1.0", &[]);
        assert!(!m.to_toml().contains("embeddings"));
        m.capabilities.get_mut("knowledge").unwrap()[0].embeddings = Some(EmbeddingsConfig {
            provider: "openai".into(),
            model: "text-embedding-3-small".into(),
        });
        assert_eq!(Manifest::parse(&m.to_toml()).unwrap(), m);
    }

    #[test]
    fn custom_skills_survive_a_round_trip_and_stay_optional() {
        let mut m = Manifest::default_for("0.1.0", &[]);
        assert!(!m.to_toml().contains("custom"));
        m.capabilities.get_mut("skills").unwrap()[0].custom = vec!["humanise".into()];
        assert_eq!(Manifest::parse(&m.to_toml()).unwrap(), m);
    }

    #[test]
    fn template_record_survives_a_round_trip_and_stays_optional() {
        let mut m = Manifest::default_for("0.1.0", &[]);
        assert!(!m.to_toml().contains("template"));
        m.template = Some(TemplateRecord {
            name: "rust-npm".into(),
            project_name: "My Tool".into(),
            project_slug: "my-tool".into(),
            version: Some("0.2.0".into()),
        });
        let toml = m.to_toml();
        assert!(toml.contains("[template]"), "{toml}");
        assert!(toml.contains("project-name = \"My Tool\""), "{toml}");
        assert!(toml.contains("version = \"0.2.0\""), "{toml}");
        assert_eq!(Manifest::parse(&toml).unwrap(), m);

        // A table written before the version field existed still parses,
        // and a version-less record writes no version line.
        let old = toml.replace("version = \"0.2.0\"\n", "");
        let parsed = Manifest::parse(&old).unwrap();
        assert_eq!(parsed.template.as_ref().unwrap().version, None);
        assert!(!parsed.to_toml().contains("0.2.0"));
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
