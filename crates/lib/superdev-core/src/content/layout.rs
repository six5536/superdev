//! content/layout.rs — the rules that turn a pack tree into items.
//!
//! A pack declares nothing: where a file sits names the item's owner, kind
//! and name (ADR-003). The same rules read the embedded snapshot and a
//! fetched pack, so both take one code path.

use std::collections::BTreeMap;

use crate::capability::Capability;

use super::item::{Item, ItemKind, Owner};

/// Where an item's files land, relative to the item root, for one path.
struct Position {
    owner: Owner,
    kind: ItemKind,
    name: String,
    /// Path under the item's root; empty when the item *is* this file.
    rel: String,
}

/// Every item a pack tree provides, in identity order, each file in path
/// order.
///
/// `files` is (pack-relative path, content); a path matching no rule is not
/// an item and is skipped. `pack.toml` and the capability instruction files
/// are the expected cases — the first is metadata, and the second describe a
/// version the binary pins or a format the compiled validator enforces, so
/// they move with the binary rather than with content.
pub fn items_from<'a>(files: impl IntoIterator<Item = (&'a str, &'a str)>) -> Vec<Item> {
    let mut grouped: BTreeMap<(Owner, ItemKind, String), Vec<(String, String)>> = BTreeMap::new();
    for (path, content) in files {
        let Some(position) = classify(path) else {
            continue;
        };
        grouped
            .entry((position.owner, position.kind, position.name))
            .or_default()
            .push((position.rel, content.to_string()));
    }
    grouped
        .into_iter()
        .map(|((owner, kind, name), mut files)| {
            files.sort();
            Item {
                owner,
                kind,
                name,
                files,
            }
        })
        .collect()
}

/// The item one pack-relative path belongs to, or `None` when the path is not
/// content.
///
/// The six positions are spelled out rather than derived from a general
/// `<owner>/<kind>/<name>` rule: only two capabilities carry content, and the
/// repo-level kinds have no owner directory at all. An unlisted position is
/// not an item, which is what keeps the capability instruction files out.
fn classify(path: &str) -> Option<Position> {
    let segments: Vec<&str> = path.split('/').collect();
    let position = |owner, kind, name: &str, rel: &[&str]| Position {
        owner,
        kind,
        name: name.to_string(),
        rel: rel.join("/"),
    };
    let knowledge = Owner::Knowledge;
    match segments.as_slice() {
        ["knowledge", "skills", name, rest @ ..] if !rest.is_empty() => {
            Some(position(knowledge, ItemKind::Skill, name, rest))
        }
        // A concept entry is a file or a directory: the bundle ships
        // `manifest.sokf.yaml` and the `plans/` and `specs/` indexes, which
        // are not one `.md` each. ADR-010.
        ["knowledge", "concepts", name] => {
            Some(position(knowledge, ItemKind::KnowledgeSkeleton, name, &[]))
        }
        ["knowledge", "concepts", name, rest @ ..] if !rest.is_empty() => {
            Some(position(knowledge, ItemKind::KnowledgeSkeleton, name, rest))
        }
        ["knowledge", "templates", file] if let Some(name) = file.strip_suffix(".md") => {
            Some(position(knowledge, ItemKind::DocTemplate, name, &[]))
        }
        ["skills", name, rest @ ..] if !rest.is_empty() => Some(position(
            Owner::Capability(Capability::Skills),
            ItemKind::Skill,
            name,
            rest,
        )),
        ["agents", file] if let Some(name) = file.strip_suffix(".md") => {
            Some(position(Owner::Repo, ItemKind::AgentScaffold, name, &[]))
        }
        ["projects", name, rest @ ..] if !rest.is_empty() => {
            Some(position(Owner::Repo, ItemKind::ProjectTemplate, name, rest))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(items: &[Item], owner: Owner, kind: ItemKind) -> Vec<&str> {
        items
            .iter()
            .filter(|i| i.owner == owner && i.kind == kind)
            .map(|i| i.name.as_str())
            .collect()
    }

    #[test]
    fn each_position_names_its_owner_kind_and_name() {
        let items = items_from([
            ("knowledge/skills/frame/SKILL.md", "frame"),
            ("knowledge/concepts/api-contracts.md", "concept"),
            ("knowledge/templates/adr.md", "template"),
            ("skills/double-check/SKILL.md", "pack skill"),
            ("agents/coding.md", "rules"),
            ("projects/rust-npm/README.md", "project"),
        ]);
        let knowledge = Owner::Knowledge;
        assert_eq!(names(&items, knowledge, ItemKind::Skill), ["frame"]);
        assert_eq!(
            names(&items, knowledge, ItemKind::KnowledgeSkeleton),
            ["api-contracts.md"]
        );
        assert_eq!(names(&items, knowledge, ItemKind::DocTemplate), ["adr"]);
        assert_eq!(
            names(
                &items,
                Owner::Capability(Capability::Skills),
                ItemKind::Skill
            ),
            ["double-check"]
        );
        assert_eq!(
            names(&items, Owner::Repo, ItemKind::AgentScaffold),
            ["coding"]
        );
        assert_eq!(
            names(&items, Owner::Repo, ItemKind::ProjectTemplate),
            ["rust-npm"]
        );
    }

    #[test]
    fn a_skill_is_one_item_holding_its_whole_directory() {
        let items = items_from([
            ("knowledge/skills/handoff/SKILL.md", "skill"),
            ("knowledge/skills/handoff/agents/openai.yaml", "harness"),
        ]);
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].files,
            [
                ("SKILL.md".to_string(), "skill".to_string()),
                ("agents/openai.yaml".to_string(), "harness".to_string()),
            ]
        );
    }

    #[test]
    fn a_single_file_item_carries_one_file_with_an_empty_path() {
        let items = items_from([("agents/coding.md", "rules")]);
        assert_eq!(items[0].files, [(String::new(), "rules".to_string())]);
    }

    /// ADR-010: `concepts/` admits a directory, and it supersedes as one unit.
    #[test]
    fn a_concept_directory_is_one_item_not_one_per_file() {
        let items = items_from([
            ("knowledge/concepts/plans/index.md", "plans"),
            ("knowledge/concepts/specs/index.md", "specs"),
            ("knowledge/concepts/manifest.sokf.yaml", "manifest"),
        ]);
        let knowledge = Owner::Knowledge;
        assert_eq!(
            names(&items, knowledge, ItemKind::KnowledgeSkeleton),
            ["manifest.sokf.yaml", "plans", "specs"]
        );
        let plans = items
            .iter()
            .find(|i| i.name == "plans")
            .expect("plans item");
        assert_eq!(plans.files, [("index.md".to_string(), "plans".to_string())]);
    }

    /// The instruction files and the SOKF spec move with the binary, so they
    /// sit in the tree without being content. `pack.toml` is metadata.
    #[test]
    fn paths_matching_no_rule_are_not_items() {
        let items = items_from([
            ("pack.toml", "format = 1"),
            ("sokf/agents/sokf.md", "instructions"),
            ("sokf/agents/sokf/SPEC.md", "spec"),
            ("codegraph/codegraph.md", "instructions"),
            ("rtk/rtk.md", "instructions"),
            ("knowledge/concepts", "a file where a directory belongs"),
            ("knowledge/templates/not-markdown.txt", "wrong extension"),
            ("agents/nested/deeper.md", "too deep"),
        ]);
        assert!(items.is_empty(), "unexpected items: {items:?}");
    }

    #[test]
    fn two_owners_may_ship_a_skill_of_the_same_name() {
        let items = items_from([
            ("knowledge/skills/review/SKILL.md", "knowledge's"),
            ("skills/review/SKILL.md", "the pack's"),
        ]);
        assert_eq!(items.len(), 2, "owner is part of the identity");
        assert!(items.iter().all(|i| i.name == "review"));
    }
}
