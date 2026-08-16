//! Component implementations and the helpers they share.

pub mod aokf;
pub mod codegraph;
pub mod mise;
pub mod plugin;
pub mod skillpack;

use crate::component::Component;
use crate::manifest::Manifest;

/// Every mise `[tools]` key superdev pins. A repo carrying any of them needs
/// `mise install` before a provider command can resolve its tool.
pub const MANAGED_MISE_TOOLS: [&str; 2] = [
    plugin::SUPERPOWERS_MISE_TOOL,
    codegraph::CODEGRAPH_MISE_TOOL,
];

/// Every enabled component, in canonical apply order.
pub fn enabled(manifest: &Manifest) -> Vec<Box<dyn Component>> {
    let all: Vec<Box<dyn Component>> = vec![
        Box::new(plugin::superpowers()),
        Box::new(plugin::frontend_design()),
        Box::new(skillpack::SkillPack),
        Box::new(codegraph::Codegraph),
        Box::new(aokf::Aokf),
    ];
    all.into_iter()
        .filter(|c| manifest.capabilities.contains_key(c.capability().as_str()))
        .collect()
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

        let manifest = Manifest::default_for(env!("CARGO_PKG_VERSION"), &[]);
        for component in enabled(&manifest) {
            let dir = tempfile::tempdir().unwrap();
            let fake = FakeRunner::new();
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
            let claimed: BTreeSet<String> =
                component.owned(&ctx).iter().map(Claim::lock_key).collect();
            let locked: BTreeSet<String> = lock.files.keys().cloned().collect();
            assert_eq!(claimed, locked, "{}", component.provider());
        }
    }

    #[test]
    fn enabled_skips_disabled_capabilities() {
        let manifest = Manifest::default_for("0.1.0", &[Capability::CodeIndex]);
        let components = enabled(&manifest);
        assert_eq!(components.len(), 4);
        assert!(
            components
                .iter()
                .all(|c| c.capability() != Capability::CodeIndex)
        );
    }
}
