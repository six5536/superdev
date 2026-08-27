//! capability.rs — the functionality slots superdev manages.

/// How many providers a capability holds at once. Declared here, in the
/// blueprint, so a manifest cannot turn an exclusive slot plural.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    /// One provider, exclusively — alternatives compete for the slot.
    Single,
    /// A set of providers, additively — entries are packs, not rivals.
    Many,
}

/// A functionality slot in a managed repo. Flags and manifest keys name
/// capabilities, never the tools behind them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// Frontend design workflows (Anthropic plugin).
    Frontend,
    /// superdev's own skill pack, shipped as owned repo files.
    Skills,
    /// Pre-indexed code knowledge graph.
    CodeIndex,
    /// Command-output filtering before it reaches agent context.
    BashOutputFilter,
}

impl Capability {
    /// Every capability, in canonical apply order: plugins first, then the
    /// code index. The SOKF knowledge is not here — it is part of superdev,
    /// not a slot a provider fills.
    pub const ALL: [Capability; 4] = [
        Capability::Frontend,
        Capability::Skills,
        Capability::CodeIndex,
        Capability::BashOutputFilter,
    ];

    /// Kebab-case name used in the manifest, the lock, and CLI flags.
    pub fn as_str(self) -> &'static str {
        match self {
            Capability::Frontend => "frontend",
            Capability::Skills => "skills",
            Capability::CodeIndex => "code-index",
            Capability::BashOutputFilter => "bash-output-filter",
        }
    }

    /// Inverse of [`Capability::as_str`].
    pub fn parse(s: &str) -> Option<Capability> {
        Capability::ALL.into_iter().find(|c| c.as_str() == s)
    }

    /// How many providers this slot holds. Skill packs are additive, so
    /// skills is the many slot; everything else is exclusive.
    pub fn cardinality(self) -> Cardinality {
        match self {
            Capability::Skills => Cardinality::Many,
            _ => Cardinality::Single,
        }
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

    #[test]
    fn skills_is_the_only_many_slot() {
        for c in Capability::ALL {
            let expected = if c == Capability::Skills {
                Cardinality::Many
            } else {
                Cardinality::Single
            };
            assert_eq!(c.cardinality(), expected, "{c:?}");
        }
    }
}
