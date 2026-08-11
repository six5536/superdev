//! registry.rs — default providers and versions, tested together, baked into
//! this binary. The binary's own version is the blueprint version.

use crate::capability::Capability;

/// Source tarball for the pinned Superpowers release (mise `http` backend).
pub const SUPERPOWERS_URL: &str =
    "https://github.com/obra/superpowers/archive/refs/tags/v6.2.0.tar.gz";
/// sha256 of that tarball.
pub const SUPERPOWERS_CHECKSUM: &str =
    "sha256:468246a7b4981d4c014c2b58d9ee538700ffded075279d5810059cdc1abeb5f3";

/// One capability's default provider and version.
#[derive(Debug, Clone, Copy)]
pub struct RegistryEntry {
    /// Slot this entry fills.
    pub capability: Capability,
    /// Default provider id.
    pub provider: &'static str,
    /// Pinned version, when superdev pins one (None = managed by the source).
    pub version: Option<&'static str>,
    /// False for slots whose provider does not exist yet.
    pub available: bool,
}

const ENTRIES: [RegistryEntry; 5] = [
    RegistryEntry {
        capability: Capability::Workflows,
        provider: "superpowers",
        version: Some("6.2.0"),
        available: true,
    },
    RegistryEntry {
        capability: Capability::Frontend,
        provider: "frontend-design",
        version: None,
        available: true,
    },
    RegistryEntry {
        capability: Capability::Skills,
        provider: "superdev-plugin",
        version: None,
        available: false,
    },
    RegistryEntry {
        capability: Capability::CodeIndex,
        provider: "codegraph",
        version: Some(CODEGRAPH_VERSION),
        available: true,
    },
    RegistryEntry {
        capability: Capability::Knowledge,
        provider: "aokf",
        version: None,
        available: true,
    },
];

// codegraph ships as the scoped npm package `@colbymchenry/codegraph`; the
// unscoped `codegraph` on npm is an unrelated 2024 placeholder.
const CODEGRAPH_VERSION: &str = "1.5.0";

/// The registry, in canonical apply order.
pub fn entries() -> &'static [RegistryEntry; 5] {
    &ENTRIES
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;

    #[test]
    fn covers_every_capability_once() {
        let entries = entries();
        for c in Capability::ALL {
            assert_eq!(entries.iter().filter(|e| e.capability == c).count(), 1);
        }
    }

    #[test]
    fn skills_slot_is_unavailable() {
        let skills = entries()
            .iter()
            .find(|e| e.capability == Capability::Skills)
            .unwrap();
        assert!(!skills.available);
    }
}
