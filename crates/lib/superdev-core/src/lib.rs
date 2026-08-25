//! superdev core: the domain logic behind the `superdev` CLI.
//!
//! A managed repo is described by a manifest of [`capability`] slots, each
//! filled by a [`component`] that observes the repo and returns the
//! [`action`]s needed to match it. The [`engine`] is the only place those
//! actions run, and it rolls the run back when one fails.
//!
//! [`aokf`] is the other half, and reads rather than writes: it parses the
//! knowledge bundle the `knowledge` capability installs, validates it,
//! indexes it for hybrid search, and serves it to agents over MCP.
//!
//! The binary in `crates/app/superdev` parses arguments and calls in here;
//! the two release in lockstep.
// Under the nightly coverage job (cargo-llvm-cov sets `coverage_nightly`), enable
// the attribute used to exclude genuinely untestable glue from coverage. Inert on
// the stable toolchain used for normal builds and tests.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(missing_docs)]

pub mod action;
pub mod aokf;
pub mod capability;
pub mod component;
pub mod components;
pub mod content;
pub mod engine;
pub mod error;
pub(crate) mod fsutil;
pub(crate) mod json_edit;
pub mod lock;
pub mod manifest;
pub mod orphan;
pub mod pack;
pub mod pipeline;
pub mod registry;
pub mod report;
pub mod runner;
pub mod templates;

/// The crate version, as compiled in from the workspace `Cargo.toml`.
///
/// The binary reports this as its own version; the release pipeline checks it
/// against the tag.
///
/// ```
/// assert_eq!(superdev_core::version(), env!("CARGO_PKG_VERSION"));
/// ```
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn version_is_semver_with_an_optional_prerelease() {
        let version = version();
        // A release tag may carry a prerelease (`0.1.0-rc.1`) or build
        // metadata; what precedes either must still be MAJOR.MINOR.PATCH.
        let core = version
            .split(['-', '+'])
            .next()
            .expect("version is non-empty");
        let parts: Vec<&str> = core.split('.').collect();
        assert_eq!(parts.len(), 3, "expected MAJOR.MINOR.PATCH in {version}");
        assert!(
            parts
                .iter()
                .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())),
            "expected numeric version components in {version}"
        );
    }
}
