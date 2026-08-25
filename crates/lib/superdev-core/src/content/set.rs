//! content/set.rs — the resolved content every component reads from.

use std::collections::BTreeMap;

use super::item::{Item, ItemKind, Owner};

/// Which layer an item came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// The binary's embedded snapshot.
    Snapshot,
    /// A pack, by its manifest index and name.
    Pack {
        /// Position in the manifest's `[[packs]]` array.
        index: usize,
        /// The pack as the user named it.
        name: String,
    },
}

/// One item a later layer hid.
///
/// Reported only when both layers are packs: superseding the snapshot is the
/// ordinary case and passes unreported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shadowed {
    /// The hidden item's owner.
    pub owner: Owner,
    /// The hidden item's kind.
    pub kind: ItemKind,
    /// The hidden item's name.
    pub name: String,
    /// The layer whose item is in force.
    pub winner: Origin,
    /// The layer whose item it hid.
    pub loser: Origin,
}

/// Every item the layers resolved to. Built once per run, borrowed by `Ctx`.
pub struct ContentSet {
    /// Keyed by identity so lookup and `items_of` are both ordered.
    items: BTreeMap<(Owner, ItemKind, String), (Item, Origin)>,
    shadowed: Vec<Shadowed>,
    base: Option<Origin>,
}

impl ContentSet {
    /// The set one layer provides on its own.
    ///
    /// Layering, shadow collection and base replacement arrive with the
    /// resolver; until then a run has exactly one layer — the snapshot.
    pub(crate) fn from_layer(items: Vec<Item>, origin: Origin) -> ContentSet {
        ContentSet {
            items: items
                .into_iter()
                .map(|item| {
                    (
                        (item.owner, item.kind, item.name.clone()),
                        (item, origin.clone()),
                    )
                })
                .collect(),
            shadowed: Vec::new(),
            base: None,
        }
    }

    /// One item, or `None` when no layer provides it.
    pub fn item(&self, owner: Owner, kind: ItemKind, name: &str) -> Option<&Item> {
        self.entry(owner, kind, name).map(|(item, _)| item)
    }

    /// Every item of one kind, in name order.
    pub fn items_of(&self, owner: Owner, kind: ItemKind) -> impl Iterator<Item = &Item> {
        self.items
            .range((owner, kind, String::new())..)
            .take_while(move |((o, k, _), _)| *o == owner && *k == kind)
            .map(|(_, (item, _))| item)
    }

    /// Where an item came from, for reporting.
    pub fn origin(&self, owner: Owner, kind: ItemKind, name: &str) -> Option<&Origin> {
        self.entry(owner, kind, name).map(|(_, origin)| origin)
    }

    /// Pack-over-pack shadowing, for the report.
    pub fn shadowed(&self) -> &[Shadowed] {
        &self.shadowed
    }

    /// The entry that replaced the snapshot, when one did. ADR-004.
    pub fn base(&self) -> Option<&Origin> {
        self.base.as_ref()
    }

    /// Borrowing lookup that avoids cloning the name into an owned key.
    fn entry(&self, owner: Owner, kind: ItemKind, name: &str) -> Option<&(Item, Origin)> {
        self.items
            .range((owner, kind, name.to_string())..)
            .next()
            .filter(|((o, k, n), _)| *o == owner && *k == kind && n == name)
            .map(|(_, entry)| entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::Capability;

    fn item(owner: Owner, kind: ItemKind, name: &str) -> Item {
        Item {
            owner,
            kind,
            name: name.to_string(),
            files: vec![(String::new(), format!("{name} content"))],
        }
    }

    fn knowledge() -> Owner {
        Owner::Capability(Capability::Knowledge)
    }

    fn set() -> ContentSet {
        ContentSet::from_layer(
            vec![
                item(knowledge(), ItemKind::DocTemplate, "adr.md"),
                item(knowledge(), ItemKind::DocTemplate, "spec.md"),
                item(knowledge(), ItemKind::Skill, "frame"),
                item(Owner::Repo, ItemKind::AgentScaffold, "coding.md"),
            ],
            Origin::Snapshot,
        )
    }

    #[test]
    fn an_item_is_found_by_its_whole_identity() {
        let set = set();
        assert!(
            set.item(knowledge(), ItemKind::DocTemplate, "adr.md")
                .is_some()
        );
        // Right name, wrong kind: not the same item.
        assert!(set.item(knowledge(), ItemKind::Skill, "adr.md").is_none());
        // Right name and kind, wrong owner.
        assert!(set.item(Owner::Repo, ItemKind::Skill, "frame").is_none());
        assert!(set.item(knowledge(), ItemKind::Skill, "absent").is_none());
    }

    #[test]
    fn items_of_returns_one_kind_in_name_order() {
        let set = set();
        let names: Vec<&str> = set
            .items_of(knowledge(), ItemKind::DocTemplate)
            .map(|i| i.name.as_str())
            .collect();
        assert_eq!(names, ["adr.md", "spec.md"]);
        let skills: Vec<&str> = set
            .items_of(knowledge(), ItemKind::Skill)
            .map(|i| i.name.as_str())
            .collect();
        assert_eq!(skills, ["frame"], "kinds do not bleed into one another");
    }

    #[test]
    fn items_of_an_empty_kind_yields_nothing() {
        assert_eq!(
            set()
                .items_of(Owner::Repo, ItemKind::ProjectTemplate)
                .count(),
            0
        );
    }

    #[test]
    fn a_single_layer_reports_its_origin_and_shadows_nothing() {
        let set = set();
        assert_eq!(
            set.origin(knowledge(), ItemKind::Skill, "frame"),
            Some(&Origin::Snapshot)
        );
        assert!(set.shadowed().is_empty());
        assert!(set.base().is_none());
    }
}
