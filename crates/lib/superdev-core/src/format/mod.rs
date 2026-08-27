//! format — the superdev-format checks: skills, schemas and the core file,
//! validated against the grammar that defines the language they are written
//! in.
//!
//! The AOKF side of the validator ([`crate::aokf`]) checks the knowledge
//! bundle against the AOKF spec. This side checks a wider set — the bundle's
//! schemas, but also `.claude/skills/` and `.agents/` — against a grammar
//! carried as data. One command runs both and reports once.

pub mod check;
pub mod grammar;
pub mod read;

pub use grammar::Grammar;

/// Read a grammar from YAML.
///
/// # Errors
/// Returns the deserialisation error, which names the offending key: the
/// types are `deny_unknown_fields`, so a typo in the grammar fails here
/// rather than silently switching a rule off.
pub fn parse_grammar(yaml: &str) -> Result<Grammar, serde_yaml_ng::Error> {
    serde_yaml_ng::from_str(yaml)
}
