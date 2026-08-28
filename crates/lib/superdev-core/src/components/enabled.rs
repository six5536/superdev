//! components/enabled.rs — the manifest-to-component resolution: which
//! provider implementation fills each enabled capability, in canonical apply
//! order, plus the core components no manifest entry governs.

use crate::capability::Capability;
use crate::component::Component;
use crate::components::{codegraph, plugin, skillpack, sokf};
use crate::error::{Error, Result};
use crate::manifest::Manifest;
use crate::registry;

/// Every mise `[tools]` key superdev pins. A repo carrying any of them needs
/// `mise install` before a provider command can resolve its tool.
pub const MANAGED_MISE_TOOLS: [&str; 1] = [codegraph::CODEGRAPH_MISE_TOOL];

/// Every component to plan, in canonical apply order: the enabled
/// capabilities resolved from the manifest's provider choices, then SOKF,
/// which is part of superdev and so appears unconditionally.
pub fn enabled(manifest: &Manifest) -> Result<Vec<Box<dyn Component>>> {
    // Validate first, so `component_for` only ever sees a pair the registry has.
    for (name, configs) in &manifest.capabilities {
        let capability = Capability::parse(name).expect("manifest rejects unknown capabilities");
        for config in configs {
            if registry::entry_for(capability, &config.provider).is_none() {
                return Err(Error::Manifest {
                    message: format!(
                        "{name} provider must be one of: {}",
                        registry::providers_for(capability).join(", ")
                    ),
                });
            }
        }
    }
    // One component per enabled (capability, provider) entry, in registry
    // order — a many slot's packs plan independently, side by side.
    let mut components: Vec<Box<dyn Component>> = Vec::new();
    for entry in registry::entries() {
        if manifest
            .configs(entry.capability)
            .iter()
            .any(|c| c.provider == entry.provider)
        {
            components.push(component_for(entry.capability, entry.provider));
        }
    }
    // Last, as the knowledge scaffold always was: it writes over the tree the
    // capabilities have finished setting up. No manifest entry gates it.
    components.push(Box::new(sokf::Sokf));
    Ok(components)
}

/// The component implementing a known (capability, provider) pair.
fn component_for(capability: Capability, provider: &str) -> Box<dyn Component> {
    match (capability, provider) {
        (Capability::Frontend, _) => Box::new(plugin::frontend_design()),
        (Capability::Skills, _) => Box::new(skillpack::SkillPack),
        (Capability::CodeIndex, _) => Box::new(codegraph::Codegraph),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_matches_what_apply_locks() {
        use crate::component::{Claim, Ctx};
        use crate::engine::{self, Planned};
        use crate::lock::Lock;
        use crate::runner::FakeRunner;
        use std::collections::BTreeSet;

        let manifest = Manifest::default_for(env!("CARGO_PKG_VERSION"), &[]);
        for component in enabled(&manifest).unwrap() {
            // The item-list components derive plan and owned from one list,
            // so their consistency is true by construction; only the
            // hand-written pairs need this simulation.
            if matches!(component.provider(), "knowledge" | "superdev-skills") {
                continue;
            }
            let dir = tempfile::tempdir().unwrap();
            let fake = FakeRunner::new();
            let empty = Lock::default();
            let ctx = Ctx {
                root: dir.path(),
                runner: &fake,
                manifest: &manifest,
                lock: &empty,
                content: crate::content::test_snapshot(),
            };
            let planned = vec![Planned {
                capability: component.capability(),
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
                content: crate::content::test_snapshot(),
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
        manifest.capabilities.get_mut("code-index").unwrap()[0].provider = "flying".into();
        // `err()`, not `unwrap_err()`: the Ok side holds trait objects that
        // cannot be Debug-printed.
        let err = enabled(&manifest).err().unwrap().to_string();
        assert!(
            err.contains("code-index provider must be one of: codegraph"),
            "{err}"
        );
        assert!(enabled(&Manifest::default_for("0.1.0", &[])).is_ok());
    }

    #[test]
    fn every_skills_entry_is_validated_and_resolved() {
        // A second entry naming an unregistered pack is refused with the
        // guided provider listing, even though the first entry is fine.
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest
            .capabilities
            .get_mut("skills")
            .unwrap()
            .push(crate::manifest::CapabilityConfig {
                provider: "another-pack".into(),
                version: None,
                custom: Vec::new(),
            });
        let err = enabled(&manifest).err().unwrap().to_string();
        assert!(
            err.contains("skills provider must be one of: superdev-skills"),
            "{err}"
        );
    }

    #[test]
    fn enabled_skips_disabled_capabilities() {
        let manifest = Manifest::default_for("0.1.0", &[Capability::CodeIndex]);
        let components = enabled(&manifest).unwrap();
        assert_eq!(components.len(), 3);
        assert!(
            components
                .iter()
                .all(|c| c.capability() != Some(Capability::CodeIndex))
        );
    }
}
