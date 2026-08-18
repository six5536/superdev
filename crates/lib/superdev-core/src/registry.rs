//! registry.rs — default providers and versions, tested together, baked into
//! this binary. The binary's own version is the blueprint version.

use crate::capability::Capability;

/// Why a pinned version is locked to the registry default: what this binary
/// carries as the version's provenance. A version the binary cannot vouch for
/// is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// The binary carries the artefact's checksum.
    Checksum,
    /// The binary carries the content itself.
    Embedded,
}

impl Provenance {
    /// The noun the refusal message names as the provenance.
    pub fn describe(self) -> &'static str {
        match self {
            Provenance::Checksum => "pinned checksum",
            Provenance::Embedded => "embedded content",
        }
    }
}

/// A version this binary pins. Every pin carries its provenance — a pin the
/// binary cannot vouch for is unrepresentable, which is why pinned versions
/// are locked to the registry default.
#[derive(Debug, Clone, Copy)]
pub struct Pinned {
    /// The pinned version string.
    pub version: &'static str,
    /// Why the version is locked.
    pub provenance: Provenance,
}

/// One (capability, provider) pair and the version superdev ships for it.
#[derive(Debug, Clone, Copy)]
pub struct RegistryEntry {
    /// Slot this entry fills.
    pub capability: Capability,
    /// Provider id.
    pub provider: &'static str,
    /// Pinned version, when superdev pins one (None = managed by the source).
    pub version: Option<Pinned>,
    /// False for slots whose provider does not exist yet.
    pub available: bool,
    /// The provider init picks when the user names none. Exactly one per capability.
    pub default: bool,
}

const ENTRIES: [RegistryEntry; 5] = [
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
        version: Some(Pinned {
            version: env!("CARGO_PKG_VERSION"),
            provenance: Provenance::Embedded,
        }),
        available: true,
        default: true,
    },
    RegistryEntry {
        capability: Capability::CodeIndex,
        provider: "codegraph",
        version: Some(Pinned {
            version: CODEGRAPH_VERSION,
            provenance: Provenance::Checksum,
        }),
        available: true,
        default: true,
    },
    RegistryEntry {
        capability: Capability::BashOutputFilter,
        provider: "rtk",
        version: Some(Pinned {
            version: RTK_VERSION,
            provenance: Provenance::Checksum,
        }),
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

const RTK_VERSION: &str = "0.45.0";

/// The pinned rtk release, one checksummed artefact per platform rtk
/// publishes (`<os>-<arch>`, mise's naming), as `(platform, url, checksum)`.
/// windows-arm64 has no upstream artefact and is deliberately absent — the
/// auto_env platform files skip the tool there.
///
/// Bump: change `RTK_VERSION`, then refresh every url and checksum together —
/// the checksum is the provenance, so a version without one cannot be
/// installed:
///
/// ```sh
/// curl -s https://api.github.com/repos/rtk-ai/rtk/releases/tags/v<version> \
///   | jq -r '.assets[] | "\(.name) \(.digest)"'
/// ```
pub const RTK_PLATFORMS: [(&str, &str, &str); 5] = [
    (
        "linux-arm64",
        "https://github.com/rtk-ai/rtk/releases/download/v0.45.0/rtk-aarch64-unknown-linux-gnu.tar.gz",
        "sha256:80a746dd305ef944ff50ef011ae4ce3878dd5ba88dfe35d859d05498191637c3",
    ),
    (
        "linux-x64",
        "https://github.com/rtk-ai/rtk/releases/download/v0.45.0/rtk-x86_64-unknown-linux-musl.tar.gz",
        "sha256:c4c036fbf181fc55ef329786c8c17e0d427972b053b825944d968a6aafef1ba4",
    ),
    (
        "macos-arm64",
        "https://github.com/rtk-ai/rtk/releases/download/v0.45.0/rtk-aarch64-apple-darwin.tar.gz",
        "sha256:064151cfc2d50b24d810b06a0af2e41b9c945e83534e4c438c3d3eae607fc3f4",
    ),
    (
        "macos-x64",
        "https://github.com/rtk-ai/rtk/releases/download/v0.45.0/rtk-x86_64-apple-darwin.tar.gz",
        "sha256:9ea02f889d5a2779e4fb700df4587824303c5a57cda22e903e30058079fca0ef",
    ),
    (
        "windows-x64",
        "https://github.com/rtk-ai/rtk/releases/download/v0.45.0/rtk-x86_64-pc-windows-msvc.zip",
        "sha256:34cea9009a8099acdaf85147b971d95f65efabfa63fb3aea7d3e2b73e6f517c3",
    ),
];

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
        assert!(entry_for(Capability::Knowledge, "flying").is_none());
        assert_eq!(providers_for(Capability::Knowledge), vec!["aokf"]);
    }

    /// A half-done bump — new version, stale urls — would install the old
    /// artefact under the new version's name; sorted platforms keep the
    /// generated pin from reordering between runs.
    fn assert_platforms_match_version(version: &str, platforms: &[(&str, &str, &str)]) {
        let tag = format!("/v{version}/");
        for (platform, url, checksum) in platforms {
            assert!(url.contains(&tag), "{platform}: {url}");
            let hex = checksum
                .strip_prefix("sha256:")
                .unwrap_or_else(|| panic!("{platform}: {checksum}"));
            assert_eq!(hex.len(), 64, "{platform}: {checksum}");
            assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
        }
        let mut sorted: Vec<&str> = platforms.iter().map(|(p, ..)| *p).collect();
        sorted.sort_unstable();
        let given: Vec<&str> = platforms.iter().map(|(p, ..)| *p).collect();
        assert_eq!(sorted, given);
    }

    #[test]
    fn every_codegraph_bundle_matches_the_pinned_version() {
        assert_platforms_match_version(CODEGRAPH_VERSION, &CODEGRAPH_PLATFORMS);
    }

    #[test]
    fn every_rtk_artefact_matches_the_pinned_version() {
        assert_platforms_match_version(RTK_VERSION, &RTK_PLATFORMS);
        // The gap is deliberate: rtk publishes no windows-arm64 artefact.
        assert!(RTK_PLATFORMS.iter().all(|(p, ..)| *p != "windows-arm64"));
    }

    #[test]
    fn skills_slot_ships_at_the_binary_version() {
        let skills = entries()
            .iter()
            .find(|e| e.capability == Capability::Skills)
            .unwrap();
        assert!(skills.available);
        assert_eq!(skills.provider, "superdev-skills");
        let pinned = skills.version.unwrap();
        assert_eq!(pinned.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(pinned.provenance, Provenance::Embedded);
    }
}
