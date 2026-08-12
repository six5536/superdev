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
