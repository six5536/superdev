//! capability.rs — the functionality slots superdev manages.

/// A functionality slot in a managed repo. Flags and manifest keys name
/// capabilities, never the tools behind them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// Superpowers prompting/workflow skills.
    Workflows,
    /// Frontend design workflows (Anthropic plugin).
    Frontend,
    /// superdev's own skill pack plugin (slot; sub-project 3).
    Skills,
    /// Pre-indexed code knowledge graph.
    CodeIndex,
    /// The AOKF knowledgebase (native).
    Knowledge,
}

impl Capability {
    /// Every capability, in canonical apply order: plugins first, then the
    /// code index, then the knowledge scaffold.
    pub const ALL: [Capability; 5] = [
        Capability::Workflows,
        Capability::Frontend,
        Capability::Skills,
        Capability::CodeIndex,
        Capability::Knowledge,
    ];

    /// Kebab-case name used in the manifest, the lock, and CLI flags.
    pub fn as_str(self) -> &'static str {
        match self {
            Capability::Workflows => "workflows",
            Capability::Frontend => "frontend",
            Capability::Skills => "skills",
            Capability::CodeIndex => "code-index",
            Capability::Knowledge => "knowledge",
        }
    }

    /// Inverse of [`Capability::as_str`].
    pub fn parse(s: &str) -> Option<Capability> {
        Capability::ALL.into_iter().find(|c| c.as_str() == s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_names() {
        for c in Capability::ALL {
            assert_eq!(Capability::parse(c.as_str()), Some(c));
        }
        assert_eq!(Capability::parse("code-index"), Some(Capability::CodeIndex));
        assert_eq!(Capability::parse("nope"), None);
    }
}
