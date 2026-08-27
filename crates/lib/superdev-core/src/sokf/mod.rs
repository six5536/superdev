//! sokf — the read side of the SOKF knowledge: parsing, the graph, the
//! search index, and the MCP server, over the format defined in
//! `.agents/sokf/SPEC.md`.
//!
//! The checks live in [`crate::validate`], which is where the SOKF half and
//! the schema half meet.

pub mod bundle;
pub mod concept;
pub mod embed;
pub mod graph;
pub mod index;
pub mod mcp;

pub use bundle::{Bundle, BundleManifest, load_bundle};
pub use concept::{Concept, Link, ParseError, Section, Source, Status, parse_concept};
pub use embed::{
    ApiEmbedder, Embedder, EmbeddingsConfig, LOCAL_MODEL, LOCAL_MODEL_REVISION, Model2VecEmbedder,
    embedder_from,
};
pub use graph::{Edge, Graph, UnknownId, inverse_rel};
pub use index::{Hit, Index, IndexDir, SCHEMA_VERSION, SearchOpts, SyncStats};
pub use mcp::SokfServer;
