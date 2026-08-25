//! Core library for {{superdev:project-name}}. All logic lives here, behind
//! interfaces the binary and the tests share.

#![warn(missing_docs)]

/// This crate's version, as the binary reports it.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Placeholder behaviour so the seeded workspace builds and tests green.
pub fn greeting() -> String {
    format!("{{superdev:project-name}} {}", version())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_carries_the_version() {
        assert!(greeting().contains(version()));
    }
}
