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
    fn skills_slot_is_unavailable() {
        let skills = entries()
            .iter()
            .find(|e| e.capability == Capability::Skills)
            .unwrap();
        assert!(!skills.available);
    }
}
