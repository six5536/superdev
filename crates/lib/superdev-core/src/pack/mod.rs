//! pack — where content comes from, and what it may contain.
//!
//! Owns the source a pack is resolved from, the identity that decides whether
//! an entry replaces the embedded pack or layers over it (ADR-004), and
//! `pack.toml` with the paths and keys a pack may not carry.
//!
//! Depends on [`crate::content`] for what a pack provides; `content` never
//! depends on this. Nothing here knows about components or capabilities.

mod manifest;
mod resolve;
mod source;

pub use manifest::{
    PACK_MANIFEST, PackManifest, REJECTED, REJECTED_BASENAME, SUPPORTED_FORMATS, check_path,
};
pub use resolve::{Resolution, ResolveMode, resolve};
pub use source::{DEFAULT_PACK, DefaultPack, PackSource};
