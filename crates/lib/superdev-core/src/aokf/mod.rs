//! AOKF: parsing, validation, indexing, and the MCP server for the
//! knowledge bundle format defined in `.agents/aokf/SPEC.md`.

pub mod bundle;
pub mod concept;
pub mod graph;
pub mod validate;

pub use bundle::{Bundle, BundleManifest, load_bundle};
pub use concept::{Concept, Link, ParseError, Section, Source, Status, parse_concept};
pub use graph::{Edge, Graph, UnknownId, inverse_rel};
pub use validate::{Finding, Report, validate};
