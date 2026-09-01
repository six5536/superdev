//! content/item.rs — what a pack provides, and the identity it supersedes on.

use crate::capability::Capability;

/// What materialises an item: a capability's component, superdev's own SOKF
/// component, or nothing in particular for the repo-level kinds.
///
/// Part of an item's identity because the SOKF component and the skills
/// capability both write into `.claude/skills/` and their `custom` lists are
/// name-guarded: a name in one list must never release the other's file.
/// ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    /// Materialised by one capability's component.
    Capability(Capability),
    /// Materialised by the SOKF component, which fills no slot.
    Knowledge,
    /// Repo-level: written outside any component's claim.
    Repo,
}

/// The kinds of content a pack may carry, each named by where it sits under
/// its owner directory. ADR-003.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ItemKind {
    /// `<owner>/skills/<name>/**` — owned files under `.claude/skills/`.
    Skill,
    /// `knowledge/concepts/<name>` — a write-once bundle scaffold. `<name>`
    /// is any entry directly under `concepts/`, file or directory, because
    /// the bundle ships scaffolds that are not one `.md` each. ADR-010.
    KnowledgeSkeleton,
    /// `knowledge/schemas/<name>.md` — an owned document schema.
    DocSchema,
    /// `knowledge/schemas/fragments/<name>.md` — an owned fragment, the
    /// authored home of content other documents materialize through an
    /// include block. Ships with the schema set. ADR-027.
    Fragment,
    /// `agents/<name>.md` — a write-once general-rules scaffold.
    AgentScaffold,
    /// `projects/<name>/**` — write-once repo scaffolds, token-substituted.
    ProjectTemplate,
}

/// One item and every file it owns, paths relative to the item's own root.
///
/// `(owner, kind, name)` is the identity a later layer supersedes on.
///
/// `name` is what the kind's path pattern calls `<name>`: the directory name
/// where the entry is a directory, and the file name without the `.md` the
/// pattern spells out — `agents/<name>.md` names `coding`, not `coding.md`.
/// A knowledge skeleton is the exception the pattern already states, since
/// `knowledge/concepts/<name>` admits any entry (ADR-010), so its name carries
/// whatever extension the entry has. A single-file item carries one file whose
/// relative path is empty: the item root *is* the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Which capability materialises it, or `Repo`.
    pub owner: Owner,
    /// What kind of content it is.
    pub kind: ItemKind,
    /// The entry's name in the pack tree.
    pub name: String,
    /// (path relative to the item root, content), in path order. A single-file
    /// item has exactly one entry, with an empty path.
    pub files: Vec<(String, String)>,
}
