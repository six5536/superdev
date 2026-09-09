//! manifest.rs — .superdev/config.toml: what the repo wants.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::capability::{Capability, Cardinality};
use crate::error::{Error, Result};
use crate::registry;
use crate::sokf::embed::EmbeddingsConfig;

// The manifest's on-disk shape is the config contract's Definition
// (contract-004): the path, every table `parse` reads and `to_toml` writes,
// and the doc comment on each field, sit in the `config` region below and the
// contract includes them. `EmbeddingsConfig` sits in its own `config` region
// in `sokf/embed.rs`.
// sokf:begin config
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
    /// Skills released from management: superdev stops writing them and
    /// `status` reports them as custom. Honoured by `skills`, which writes
    /// into `.claude/skills/` alongside the SOKF knowledge.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,
}

/// `[knowledge]` — the SOKF knowledge's settings. A plain table, not a
/// capability: SOKF is part of superdev, so there is no provider to name and
/// no slot to leave empty.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeConfig {
    /// Set only in a manifest written before SOKF became core. Its presence
    /// is the whole reason the field exists: `parse` refuses such a manifest
    /// and names the edit, rather than silently reading a provider choice
    /// that no longer means anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Embedding provider for the search index. Absent = the local model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<EmbeddingsConfig>,
    /// SOKF skills released from management: superdev stops writing them and
    /// `status` reports them as custom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom: Vec<String>,
}

/// `[workflow]` — project-wide policy for the local SCOPE → BUILD → ACCEPT
/// workflow. Models, plans, and adapters may read but cannot override it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowConfig {
    /// Require an interactive human decision before local integration.
    pub human_acceptance_required: bool,
    /// Maximum final review correction cycles before BUILD pauses.
    pub max_final_correction_cycles: u32,
    /// Maximum batched correction/re-review cycles during SCOPE.
    #[serde(default = "default_max_scope_review_cycles")]
    pub max_scope_review_cycles: u32,
    /// Deadline for one isolated model role.
    #[serde(default = "default_isolated_role_timeout_seconds")]
    pub isolated_role_timeout_seconds: u32,
    /// Maximum isolated-role bytes exposed to parent model context.
    #[serde(default = "default_max_isolated_context_bytes")]
    pub max_isolated_context_bytes: u32,
    /// Maximum isolated-role lines exposed to parent model context.
    #[serde(default = "default_max_isolated_context_lines")]
    pub max_isolated_context_lines: u32,
    /// Maximum serialized review/checklist/question state.
    #[serde(default = "default_max_review_state_bytes")]
    pub max_review_state_bytes: u32,
    /// Maximum findings returned by one review.
    #[serde(default = "default_max_review_findings")]
    pub max_review_findings: u32,
    /// Maximum bytes retained in one isolated diagnostic artifact.
    #[serde(default = "default_max_isolated_artifact_bytes")]
    pub max_isolated_artifact_bytes: u32,
    /// Maximum isolated diagnostic artifacts retained per session.
    #[serde(default = "default_max_isolated_artifacts_per_session")]
    pub max_isolated_artifacts_per_session: u32,
    /// Maximum isolated diagnostic artifact age.
    #[serde(default = "default_isolated_artifact_retention_hours")]
    pub isolated_artifact_retention_hours: u32,
}

const fn default_max_scope_review_cycles() -> u32 {
    3
}
const fn default_isolated_role_timeout_seconds() -> u32 {
    1_200
}
const fn default_max_isolated_context_bytes() -> u32 {
    8_192
}
const fn default_max_isolated_context_lines() -> u32 {
    200
}
const fn default_max_review_state_bytes() -> u32 {
    262_144
}
const fn default_max_review_findings() -> u32 {
    100
}
const fn default_max_isolated_artifact_bytes() -> u32 {
    10_485_760
}
const fn default_max_isolated_artifacts_per_session() -> u32 {
    20
}
const fn default_isolated_artifact_retention_hours() -> u32 {
    24
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            human_acceptance_required: true,
            max_final_correction_cycles: 3,
            max_scope_review_cycles: default_max_scope_review_cycles(),
            isolated_role_timeout_seconds: default_isolated_role_timeout_seconds(),
            max_isolated_context_bytes: default_max_isolated_context_bytes(),
            max_isolated_context_lines: default_max_isolated_context_lines(),
            max_review_state_bytes: default_max_review_state_bytes(),
            max_review_findings: default_max_review_findings(),
            max_isolated_artifact_bytes: default_max_isolated_artifact_bytes(),
            max_isolated_artifacts_per_session: default_max_isolated_artifacts_per_session(),
            isolated_artifact_retention_hours: default_isolated_artifact_retention_hours(),
        }
    }
}

impl WorkflowConfig {
    fn validate(&self) -> Result<()> {
        let positive = [
            (
                "max_final_correction_cycles",
                self.max_final_correction_cycles,
            ),
            ("max_scope_review_cycles", self.max_scope_review_cycles),
            (
                "isolated_role_timeout_seconds",
                self.isolated_role_timeout_seconds,
            ),
            (
                "max_isolated_context_bytes",
                self.max_isolated_context_bytes,
            ),
            (
                "max_isolated_context_lines",
                self.max_isolated_context_lines,
            ),
            ("max_review_state_bytes", self.max_review_state_bytes),
            ("max_review_findings", self.max_review_findings),
            (
                "max_isolated_artifact_bytes",
                self.max_isolated_artifact_bytes,
            ),
            (
                "max_isolated_artifacts_per_session",
                self.max_isolated_artifacts_per_session,
            ),
            (
                "isolated_artifact_retention_hours",
                self.isolated_artifact_retention_hours,
            ),
        ];
        for (field, value) in positive {
            if value == 0 {
                return Err(Error::Manifest {
                    message: format!("workflow.{field} must be a positive integer"),
                });
            }
        }
        let ceilings = [
            (
                "max_isolated_context_bytes",
                self.max_isolated_context_bytes,
                51_200,
            ),
            (
                "max_isolated_context_lines",
                self.max_isolated_context_lines,
                2_000,
            ),
            (
                "max_review_state_bytes",
                self.max_review_state_bytes,
                1_048_576,
            ),
            ("max_review_findings", self.max_review_findings, 1_000),
            (
                "max_isolated_artifact_bytes",
                self.max_isolated_artifact_bytes,
                104_857_600,
            ),
            (
                "max_isolated_artifacts_per_session",
                self.max_isolated_artifacts_per_session,
                100,
            ),
            (
                "isolated_artifact_retention_hours",
                self.isolated_artifact_retention_hours,
                168,
            ),
        ];
        for (field, value, ceiling) in ceilings {
            if value > ceiling {
                return Err(Error::Manifest {
                    message: format!("workflow.{field} must not exceed {ceiling}"),
                });
            }
        }
        Ok(())
    }
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

/// One content pack the repo wants. Order in the manifest is layer order.
///
/// A `[[packs]]` array rather than a capability table: an absent capability
/// means disabled, but an absent pack list means the pack compiled into the
/// binary — a different thing, and one no capability table can say. ADR-001.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackEntry {
    /// Where the pack comes from, as the user wrote it.
    pub source: String,
    /// Git revision — tag, branch or commit sha. Absent for a path source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
}

/// The manifest as TOML sees it — the on-disk shape `parse` validates and
/// `to_toml` renders. Field order is the serialised order.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WrittenManifest {
    /// The superdev version last applied; `init` writes it.
    blueprint: String,
    /// `[template]` — the project template `init` seeded the repo from.
    /// Absent when the repo was never seeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    template: Option<TemplateRecord>,
    /// `[[packs]]` — the content packs to layer, in layer order. Absent means
    /// the pack embedded in the binary.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    packs: Vec<PackEntry>,
    /// Named ahead of the flatten so `[knowledge]` lands here rather than in
    /// the capability map. Always written: a fresh repo should be able to see
    /// the table it may put `custom` and `embeddings` in.
    #[serde(default)]
    knowledge: KnowledgeConfig,
    /// `[workflow]` — safe local workflow policy. Older manifests may omit
    /// it; rewrites materialize the safe defaults.
    #[serde(default)]
    workflow: WorkflowConfig,
    /// `[<capability>]` — one table per enabled capability, keyed by its
    /// kebab-case name; `[[<capability>]]` for a slot that takes several
    /// providers. An absent table means the capability is disabled.
    #[serde(flatten)]
    capabilities: BTreeMap<String, WrittenEntries>,
}
// sokf:end config

/// The manifest: blueprint version plus, per enabled capability, its
/// provider entries — one for a single slot, a set for a many slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Blueprint (= superdev) version that wrote this file.
    pub blueprint: String,
    /// The template `init` seeded this repo from, when one was chosen.
    pub template: Option<TemplateRecord>,
    /// The content packs to layer, in layer order. Empty when the manifest
    /// carries no `[[packs]]`, which resolves from the pack compiled into
    /// the binary — never "disabled". ADR-001.
    pub packs: Vec<PackEntry>,
    /// The SOKF knowledge's settings. Always present: SOKF is core, so there
    /// is no state in which the table means "off".
    pub knowledge: KnowledgeConfig,
    /// Project-wide workflow acceptance and retry policy.
    pub workflow: WorkflowConfig,
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
                        custom: Vec::new(),
                    }],
                )
            })
            .collect();
        Manifest {
            blueprint: blueprint.to_string(),
            template: None,
            // Written out rather than left absent: an absent `[[packs]]`
            // resolves the same way, but a fresh repo should be able to see
            // what its content is pinned to, and edit it, without first
            // learning that the empty case means something.
            packs: vec![PackEntry {
                source: crate::pack::DEFAULT_PACK.source.to_string(),
                rev: Some(crate::pack::DEFAULT_PACK.rev.to_string()),
            }],
            knowledge: KnowledgeConfig::default(),
            workflow: WorkflowConfig::default(),
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
        let document = s
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| Error::Toml {
                path: CONFIG_PATH.into(),
                message: e.to_string(),
            })?;
        let written: WrittenManifest = toml_edit::de::from_str(s).map_err(|e| Error::Toml {
            path: CONFIG_PATH.into(),
            message: e.to_string(),
        })?;
        if !document.contains_key("workflow") && !blueprint_predates_workflow(&written.blueprint) {
            return Err(Error::Manifest {
                message: "[workflow] is required after the 0.2.0 blueprint migration".into(),
            });
        }
        // A manifest from before SOKF became core: `[knowledge]` named a
        // provider for a slot that no longer exists. Refused by name, so the
        // reader is told the edit rather than left with a provider choice
        // that is silently ignored.
        if written.knowledge.provider.is_some() {
            return Err(Error::Manifest {
                message: "[knowledge] is no longer a capability — SOKF is part of superdev. \
                          Delete the `provider` line; keep the table for `custom` and \
                          `embeddings`"
                    .into(),
            });
        }
        written.workflow.validate()?;
        let mut capabilities = BTreeMap::new();
        for (name, entries) in written.capabilities {
            if name == "workflows" {
                return Err(Error::Manifest {
                    message: "the workflows capability was removed — delete the [workflows] \
                              table (moving any custom names to [knowledge]); its skill set \
                              now ships with the SOKF knowledge. superpowers users: \
                              `claude plugin install superpowers`"
                        .into(),
                });
            }
            if name == "bash-output-filter" {
                return Err(Error::Manifest {
                    message: "the bash-output-filter capability was removed — delete the \
                              [bash-output-filter] table; the next sync then removes \
                              .miserc.toml, mise.unix.toml, mise.windows-x64.toml, \
                              .agents/rtk.md and the rtk PreToolUse hook"
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
            packs: written.packs,
            knowledge: written.knowledge,
            workflow: written.workflow,
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
            packs: self.packs.clone(),
            knowledge: self.knowledge.clone(),
            workflow: self.workflow.clone(),
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

fn blueprint_predates_workflow(version: &str) -> bool {
    let mut parts = version.split('.').map(|part| part.parse::<u64>());
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(Ok(major)), Some(Ok(minor)), Some(Ok(_)), None) => (major, minor) < (0, 2),
        _ => false,
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
        let err =
            Manifest::parse("blueprint = \"0.1.0\"\n[[code-index]]\nprovider = \"codegraph\"\n")
                .unwrap_err()
                .to_string();
        assert!(
            err.contains("code-index holds one provider — use a single [code-index] table"),
            "{err}"
        );
    }

    /// A manifest from before SOKF became core names a provider for a slot
    /// that no longer exists. Refused by name, so the reader is told what to
    /// edit rather than having the line silently ignored.
    #[test]
    fn a_pre_sokf_knowledge_capability_is_refused_by_name() {
        let err = Manifest::parse("blueprint = \"0.1.0\"\n\n[knowledge]\nprovider = \"aokf\"\n")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("[knowledge] is no longer a capability"),
            "{err}"
        );
        assert!(err.contains("Delete the `provider` line"), "{err}");
        // The table itself is fine — it is the provider line that is not.
        let ok = Manifest::parse("blueprint = \"0.1.0\"\n\n[knowledge]\ncustom = [\"maintain\"]\n")
            .unwrap();
        assert_eq!(ok.knowledge.custom, ["maintain"]);
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
    fn a_bash_output_filter_table_gets_the_guided_error() {
        let err =
            Manifest::parse("blueprint = \"0.1.0\"\n\n[bash-output-filter]\nprovider = \"rtk\"\n")
                .unwrap_err()
                .to_string();
        assert!(
            err.contains("the bash-output-filter capability was removed"),
            "{err}"
        );
        assert!(err.contains(".agents/rtk.md"), "{err}");
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
            "blueprint = \"0.1.0\"\n\n[code-index]\nprovider = \"codegraph\"\nversion = \"1.2.3\"\n",
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
        m.knowledge.embeddings = Some(EmbeddingsConfig {
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

    /// An older manifest may omit workflow policy; parsing applies safe
    /// defaults and the next rewrite materializes them.
    #[test]
    fn an_older_manifest_gains_safe_workflow_defaults_on_rewrite() {
        let written = "blueprint = \"0.1.0\"\n\n[knowledge]\ncustom = [\"maintain\"]\n";
        let manifest = Manifest::parse(written).unwrap();
        assert!(
            manifest.packs.is_empty(),
            "absent means empty, never disabled"
        );
        assert_eq!(manifest.workflow, WorkflowConfig::default());
        assert_eq!(
            manifest.to_toml(),
            concat!(
                "blueprint = \"0.1.0\"\n\n[knowledge]\ncustom = [\"maintain\"]\n\n",
                "[workflow]\n",
                "human_acceptance_required = true\n",
                "max_final_correction_cycles = 3\n",
                "max_scope_review_cycles = 3\n",
                "isolated_role_timeout_seconds = 1200\n",
                "max_isolated_context_bytes = 8192\n",
                "max_isolated_context_lines = 200\n",
                "max_review_state_bytes = 262144\n",
                "max_review_findings = 100\n",
                "max_isolated_artifact_bytes = 10485760\n",
                "max_isolated_artifacts_per_session = 20\n",
                "isolated_artifact_retention_hours = 24\n",
            )
        );
    }

    #[test]
    fn a_current_manifest_cannot_drop_workflow_policy() {
        let error = Manifest::parse("blueprint = \"0.2.0\"\n[knowledge]\n").unwrap_err();
        assert!(error.to_string().contains("[workflow] is required"));
    }

    #[test]
    fn a_manifest_with_packs_round_trips_and_keeps_layer_order() {
        let written = concat!(
            "blueprint = \"0.1.0\"\n\n",
            "[[packs]]\n",
            "source = \"github:six5536/superdev\"\n",
            "rev = \"assets-v1.4.0\"\n\n",
            "[[packs]]\n",
            "source = \"./packs/acme\"\n\n",
            "[knowledge]\ncustom = [\"maintain\"]\n",
        );
        let manifest = Manifest::parse(written).unwrap();
        assert_eq!(
            manifest.packs,
            [
                PackEntry {
                    source: "github:six5536/superdev".into(),
                    rev: Some("assets-v1.4.0".into()),
                },
                PackEntry {
                    source: "./packs/acme".into(),
                    rev: None,
                },
            ],
            "manifest order is layer order"
        );
        assert_eq!(
            manifest.to_toml(),
            format!(
                "{written}\n[workflow]\nhuman_acceptance_required = true\nmax_final_correction_cycles = 3\nmax_scope_review_cycles = 3\nisolated_role_timeout_seconds = 1200\nmax_isolated_context_bytes = 8192\nmax_isolated_context_lines = 200\nmax_review_state_bytes = 262144\nmax_review_findings = 100\nmax_isolated_artifact_bytes = 10485760\nmax_isolated_artifacts_per_session = 20\nisolated_artifact_retention_hours = 24\n"
            )
        );
    }

    #[test]
    fn workflow_retry_limits_must_be_positive() {
        let written = "blueprint = \"0.2.0\"\n[workflow]\nhuman_acceptance_required = true\nmax_final_correction_cycles = 0\n";
        let err = Manifest::parse(written).unwrap_err();
        assert!(
            err.to_string().contains("max_final_correction_cycles"),
            "{err}"
        );
    }

    /// `packs` is a top-level array, not a capability table: it must not be
    /// mistaken for one on the way in (ADR-001).
    #[test]
    fn packs_is_not_read_as_a_capability() {
        let manifest = Manifest::parse(
            "blueprint = \"0.1.0\"\n\n[[packs]]\nsource = \"./p\"\n\n[skills]\nprovider = \"superdev-skills\"\n",
        )
        .unwrap();
        assert_eq!(manifest.capabilities.keys().collect::<Vec<_>>(), ["skills"]);
        assert_eq!(manifest.packs.len(), 1);
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
