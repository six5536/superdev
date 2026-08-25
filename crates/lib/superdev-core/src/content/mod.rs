//! content — what a pack provides, and how its tree names it.
//!
//! A pack declares no item list: the position of a file in the tree names the
//! item's owning capability, its kind and its name (ADR-003). The same rules
//! read the pack compiled into this binary and one fetched from a git source,
//! so the default path and a pinned pack take one code path rather than two.
//!
//! Depends on nothing but `std` and [`crate::capability`]: components read a
//! resolved [`ContentSet`], and nothing here knows how one is fetched.

mod item;
mod layout;
mod set;
mod snapshot;

pub use item::{Item, ItemKind, Owner};
pub use layout::items_from;
pub use set::{ContentSet, Origin, Shadowed};
pub(crate) use snapshot::items as snapshot_items;
#[cfg(test)]
pub(crate) use snapshot::test_snapshot;
pub use snapshot::{pack_manifest_source, snapshot};
