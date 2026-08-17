//! registry.rs — default providers and versions, tested together, baked into
//! this binary. The binary's own version is the blueprint version.

use crate::capability::Capability;

/// Source tarball for the pinned Superpowers release (mise `http` backend).
pub const SUPERPOWERS_URL: &str =
    "https://github.com/obra/superpowers/archive/refs/tags/v6.2.0.tar.gz";
/// sha256 of that tarball.
pub const SUPERPOWERS_CHECKSUM: &str =
    "sha256:468246a7b4981d4c014c2b58d9ee538700ffded075279d5810059cdc1abeb5f3";

/// Source tarball for the pinned mattpocock/skills release (mise `http` backend).
pub const MATTSKILLS_URL: &str =
    "https://github.com/mattpocock/skills/archive/refs/tags/v1.2.3.tar.gz";
/// sha256 of that tarball.
///
/// Bump: refresh the version, url, checksum and `MATTSKILLS_SKILLS` together,
/// then audit the new tarball's `skills/engineering` and `skills/productivity`
/// trees — every file must be UTF-8 text and non-executable, because
/// materialisation reads text and does not preserve modes. v1.2.3 was audited
/// clean at 76 files.
pub const MATTSKILLS_CHECKSUM: &str =
    "sha256:238fac54d0f53d3e2d0501c1b38c9c0e4e9bc26f6b057b53a7328ea15d43b66f";

/// One (capability, provider) pair and the version superdev ships for it.
#[derive(Debug, Clone, Copy)]
pub struct RegistryEntry {
    /// Slot this entry fills.
    pub capability: Capability,
    /// Provider id.
    pub provider: &'static str,
    /// Pinned version, when superdev pins one (None = managed by the source).
    pub version: Option<&'static str>,
    /// False for slots whose provider does not exist yet.
    pub available: bool,
    /// The provider init picks when the user names none. Exactly one per capability.
    pub default: bool,
}

const ENTRIES: [RegistryEntry; 6] = [
    RegistryEntry {
        capability: Capability::Workflows,
        provider: "superpowers",
        version: Some("6.2.0"),
        available: true,
        default: true,
    },
    RegistryEntry {
        capability: Capability::Workflows,
        provider: "mattpocock-skills",
        version: Some("1.2.3"),
        available: true,
        default: false,
    },
    RegistryEntry {
        capability: Capability::Frontend,
        provider: "frontend-design",
        version: None,
        available: true,
        default: true,
    },
    RegistryEntry {
        capability: Capability::Skills,
        provider: "superdev-skills",
        version: Some(env!("CARGO_PKG_VERSION")),
        available: true,
        default: true,
    },
    RegistryEntry {
        capability: Capability::CodeIndex,
        provider: "codegraph",
        version: Some(CODEGRAPH_VERSION),
        available: true,
        default: true,
    },
    RegistryEntry {
        capability: Capability::Knowledge,
        provider: "aokf",
        version: None,
        available: true,
        default: true,
    },
];

const CODEGRAPH_VERSION: &str = "1.5.0";

/// The pinned codegraph release, one self-contained bundle per mise platform
/// (`<os>-<arch>`), as `(platform, url, checksum)`. The bundles vendor their
/// own Node, so a managed repo needs no node of its own — which the npm
/// package, whose shim is `#!/usr/bin/env node`, did.
///
/// Bump: change `CODEGRAPH_VERSION`, then refresh every url and checksum
/// together — the checksum is the provenance, so a version without one cannot
/// be installed:
///
/// ```sh
/// curl -s https://api.github.com/repos/colbymchenry/codegraph/releases/tags/v<version> \
///   | jq -r '.assets[] | "\(.name) \(.digest)"'
/// ```
pub const CODEGRAPH_PLATFORMS: [(&str, &str, &str); 6] = [
    (
        "linux-arm64",
        "https://github.com/colbymchenry/codegraph/releases/download/v1.5.0/codegraph-linux-arm64.tar.gz",
        "sha256:9f17750aedf45d51f68caae39ed21d6e2a7290b2326e5c53f95a165918ebd1d8",
    ),
    (
        "linux-x64",
        "https://github.com/colbymchenry/codegraph/releases/download/v1.5.0/codegraph-linux-x64.tar.gz",
        "sha256:2ba65e87a1210b706bb1e67d5e48b5fc4a1935e43dbb3fb5f31c5597840d2e58",
    ),
    (
        "macos-arm64",
        "https://github.com/colbymchenry/codegraph/releases/download/v1.5.0/codegraph-darwin-arm64.tar.gz",
        "sha256:cf5ee435a6e44d097b2f98f2b7b8b9422bb1094844404efed82519c5da1af2cf",
    ),
    (
        "macos-x64",
        "https://github.com/colbymchenry/codegraph/releases/download/v1.5.0/codegraph-darwin-x64.tar.gz",
        "sha256:0a0ccc29bf7da9d10be1458d89d7e15c55927ae24cd95e9fa3de4bdfea059dde",
    ),
    (
        "windows-arm64",
        "https://github.com/colbymchenry/codegraph/releases/download/v1.5.0/codegraph-win32-arm64.zip",
        "sha256:de125e792b5eed7dee8def2ab9bd7e762f372012f75f595e59d3b0c8714b0d55",
    ),
    (
        "windows-x64",
        "https://github.com/colbymchenry/codegraph/releases/download/v1.5.0/codegraph-win32-x64.zip",
        "sha256:d6798622b4f44ee6757c94335f437ee27a9ff7d3537b554cb6a2b3baf11bc4a1",
    ),
];

/// The registry, in canonical apply order.
pub fn entries() -> &'static [RegistryEntry] {
    &ENTRIES
}

/// The entry `init` uses for `capability` when no provider is named.
pub fn default_entry(capability: Capability) -> &'static RegistryEntry {
    ENTRIES
        .iter()
        .find(|e| e.capability == capability && e.default)
        .expect("every capability has a default entry")
}

/// The entry for a (capability, provider) pair, when the registry has one.
pub fn entry_for(capability: Capability, provider: &str) -> Option<&'static RegistryEntry> {
    ENTRIES
        .iter()
        .find(|e| e.capability == capability && e.provider == provider)
}

/// Valid provider ids for `capability`, in registry order.
pub fn providers_for(capability: Capability) -> Vec<&'static str> {
    ENTRIES
        .iter()
        .filter(|e| e.capability == capability && e.available)
        .map(|e| e.provider)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;

    #[test]
    fn one_default_per_capability_and_lookups_resolve() {
        for c in Capability::ALL {
            assert_eq!(
                entries()
                    .iter()
                    .filter(|e| e.capability == c && e.default)
                    .count(),
                1,
                "{c:?}"
            );
        }
        assert_eq!(default_entry(Capability::Workflows).provider, "superpowers");
        assert_eq!(
            entry_for(Capability::Workflows, "superpowers")
                .unwrap()
                .version,
            Some("6.2.0")
        );
        assert!(entry_for(Capability::Workflows, "flying").is_none());
        assert_eq!(providers_for(Capability::Knowledge), vec!["aokf"]);
    }

    #[test]
    fn every_codegraph_bundle_matches_the_pinned_version() {
        // A half-done bump — new version, stale urls — would install the old
        // bundle under the new version's name.
        let tag = format!("/v{CODEGRAPH_VERSION}/");
        for (platform, url, checksum) in CODEGRAPH_PLATFORMS {
            assert!(url.contains(&tag), "{platform}: {url}");
            let hex = checksum
                .strip_prefix("sha256:")
                .unwrap_or_else(|| panic!("{platform}: {checksum}"));
            assert_eq!(hex.len(), 64, "{platform}: {checksum}");
            assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
        }
        // Sorted, so the generated pin never reorders between runs.
        let mut sorted = CODEGRAPH_PLATFORMS.map(|(p, ..)| p);
        sorted.sort_unstable();
        assert_eq!(sorted, CODEGRAPH_PLATFORMS.map(|(p, ..)| p));
    }

    #[test]
    fn skills_slot_ships_at_the_binary_version() {
        let skills = entries()
            .iter()
            .find(|e| e.capability == Capability::Skills)
            .unwrap();
        assert!(skills.available);
        assert_eq!(skills.provider, "superdev-skills");
        assert_eq!(skills.version, Some(env!("CARGO_PKG_VERSION")));
    }
}
