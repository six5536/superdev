//! Component implementations and the helpers they share.

pub mod aokf;
pub mod codegraph;
pub mod mattskills;
pub mod mise;
pub mod plugin;
pub mod skillpack;

use crate::capability::Capability;
use crate::component::Component;
use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::registry;

/// Every mise `[tools]` key superdev pins. A repo carrying any of them needs
/// `mise install` before a provider command can resolve its tool.
pub const MANAGED_MISE_TOOLS: [&str; 3] = [
    plugin::SUPERPOWERS_MISE_TOOL,
    mattskills::MATTSKILLS_MISE_TOOL,
    codegraph::CODEGRAPH_MISE_TOOL,
];

/// Every enabled component, in canonical apply order, resolved from the
/// manifest's provider choices. Until this resolution existed, the
/// manifest's `provider` field was recorded but never read.
pub fn enabled(manifest: &Manifest) -> Result<Vec<Box<dyn Component>>> {
    // Validate first, so `component_for` only ever sees a pair the registry has.
    for (name, config) in &manifest.capabilities {
        let capability = Capability::parse(name).expect("manifest rejects unknown capabilities");
        if registry::entry_for(capability, &config.provider).is_none() {
            return Err(Error::Manifest {
                message: format!(
                    "{name} provider must be one of: {}",
                    registry::providers_for(capability).join(", ")
                ),
            });
        }
    }
    let mut components: Vec<Box<dyn Component>> = Vec::new();
    for entry in registry::entries() {
        let Some(config) = manifest.capabilities.get(entry.capability.as_str()) else {
            continue;
        };
        if config.provider != entry.provider {
            continue;
        }
        components.push(component_for(entry.capability, entry.provider));
    }
    Ok(components)
}

/// The component implementing a known (capability, provider) pair.
fn component_for(capability: Capability, provider: &str) -> Box<dyn Component> {
    match (capability, provider) {
        (Capability::Workflows, "superpowers") => Box::new(plugin::superpowers()),
        (Capability::Workflows, "mattpocock-skills") => Box::new(mattskills::MattSkills),
        (Capability::Frontend, _) => Box::new(plugin::frontend_design()),
        (Capability::Skills, _) => Box::new(skillpack::SkillPack),
        (Capability::CodeIndex, _) => Box::new(codegraph::Codegraph),
        (Capability::Knowledge, _) => Box::new(aokf::Aokf),
        _ => unreachable!("resolved from the registry"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;
    use crate::manifest::Manifest;

    #[test]
    fn owned_matches_what_apply_locks() {
        use crate::component::{Claim, Ctx};
        use crate::engine::{self, Planned};
        use crate::lock::Lock;
        use crate::runner::FakeRunner;
        use std::collections::BTreeSet;

        // The workflows component materialises from a checkout; only it reads this.
        let checkout = tempfile::tempdir().unwrap();
        for rel in [
            "skills/engineering/alpha/SKILL.md",
            "skills/engineering/beta/SKILL.md",
            "skills/productivity/gamma/SKILL.md",
        ] {
            let p = checkout.path().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, "skill body").unwrap();
        }

        let manifest = Manifest::default_for(env!("CARGO_PKG_VERSION"), &[]);
        for component in enabled(&manifest).unwrap() {
            let dir = tempfile::tempdir().unwrap();
            let fake = FakeRunner::new();
            fake.script(
                "mise where http:mattpocock-skills",
                crate::runner::Output {
                    status: 0,
                    stdout: format!("{}\n", checkout.path().display()),
                    stderr: String::new(),
                },
            );
            let empty = Lock::default();
            let ctx = Ctx {
                root: dir.path(),
                runner: &fake,
                manifest: &manifest,
                lock: &empty,
            };
            let planned = vec![Planned {
                capability: Some(component.capability()),
                provider: component.provider().to_string(),
                actions: component.plan(&ctx).unwrap(),
            }];
            let mut lock = Lock::default();
            let result = engine::apply(dir.path(), &fake, &manifest, &planned, &mut lock);
            assert!(result.ok, "{}: apply failed", component.provider());
            // `owned` reads the lock, as it does in production, where the lock
            // on disk is the previous apply's output.
            let after = Ctx {
                root: dir.path(),
                runner: &fake,
                manifest: &manifest,
                lock: &lock,
            };
            let claimed: BTreeSet<String> = component
                .owned(&after)
                .iter()
                .map(Claim::lock_key)
                .collect();
            let locked: BTreeSet<String> = lock.files.keys().cloned().collect();
            assert_eq!(claimed, locked, "{}", component.provider());
        }
    }

    #[test]
    fn enabled_rejects_an_unknown_provider() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("workflows").unwrap().provider = "flying".into();
        // `err()`, not `unwrap_err()`: the Ok side holds trait objects that
        // cannot be Debug-printed.
        let err = enabled(&manifest).err().unwrap().to_string();
        assert!(
            err.contains("workflows provider must be one of: superpowers"),
            "{err}"
        );
        assert!(enabled(&Manifest::default_for("0.1.0", &[])).is_ok());
    }

    #[test]
    fn the_workflows_provider_resolves_from_the_manifest() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.version = Some("1.2.3".into());
        let components = enabled(&manifest).unwrap();
        assert!(
            components
                .iter()
                .any(|c| c.provider() == "mattpocock-skills")
        );
        assert!(!components.iter().any(|c| c.provider() == "superpowers"));
    }

    #[test]
    fn enabled_skips_disabled_capabilities() {
        let manifest = Manifest::default_for("0.1.0", &[Capability::CodeIndex]);
        let components = enabled(&manifest).unwrap();
        assert_eq!(components.len(), 4);
        assert!(
            components
                .iter()
                .all(|c| c.capability() != Capability::CodeIndex)
        );
    }
}
