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
pub mod engine;
pub mod error;
pub mod lock;
pub mod manifest;
pub mod orphan;
pub mod pipeline;
pub mod registry;
pub mod report;
pub mod runner;

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
    fn version_has_three_dotted_components() {
        assert_eq!(version().split('.').count(), 3);
    }
}
