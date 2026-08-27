//! components/skills.rs — shared machinery for capabilities that ship
//! embedded skills into `.claude/skills/`: the path convention, the item
//! list, and init-time adoption. The skill pack and the SOKF component both
//! build on it, so the two cannot drift apart in behaviour.

use std::path::Path;

use crate::manifest::CONFIG_PATH;

use crate::content::{ContentSet, ItemKind, Owner};

use super::item::ManagedItem;

/// Every skill one owner provides, as owned files, skipping the custom ones.
///
/// A skill is its whole directory — SKILL.md, companions, harness configs —
/// so the `custom` list releases the directory, not a file.
pub(crate) fn skill_dir_items(
    content: &ContentSet,
    owner: Owner,
    custom: &[String],
) -> Vec<ManagedItem> {
    content
        .items_of(owner, ItemKind::Skill)
        .filter(|item| !custom.contains(&item.name))
        .flat_map(|item| {
            item.files.iter().map(move |(rel, file)| {
                let name = &item.name;
                ManagedItem::OwnedFile {
                    path: format!(".claude/skills/{name}/{rel}"),
                    content: file.clone(),
                    reason: format!("{name} skill"),
                }
            })
        })
        .collect()
}

/// Every skill name one owner provides, in name order.
pub(crate) fn skill_names(content: &ContentSet, owner: Owner) -> Vec<&str> {
    content
        .items_of(owner, ItemKind::Skill)
        .map(|item| item.name.as_str())
        .collect()
}

/// Each skill's identity file, for init-time adoption: (name, SKILL.md).
pub(crate) fn skill_identities(content: &ContentSet, owner: Owner) -> Vec<(&str, &str)> {
    content
        .items_of(owner, ItemKind::Skill)
        .map(|item| {
            let skill_md = item
                .files
                .iter()
                .find(|(rel, _)| rel == "SKILL.md")
                .expect("every skill directory carries a SKILL.md")
                .1
                .as_str();
            (item.name.as_str(), skill_md)
        })
        .collect()
}

/// Where a shipped skill lives in the managed repo.
pub(crate) fn skill_path(name: &str) -> String {
    format!(".claude/skills/{name}/SKILL.md")
}

/// Release, at adoption time, every shipped skill the repo already has under
/// its own name and with its own content. Overwriting those would replace
/// work superdev never wrote with a backup the user has to go looking for;
/// marking them custom keeps the file and hands the choice back. Returns the
/// lines to print. Only `init` calls this — later syncs honour the list as
/// written.
/// `custom` is the list to extend — a capability entry's, or the SOKF
/// table's — and `label` names the owner in the printed line. Taking the
/// list rather than the manifest is what lets a core component with no
/// capability entry use the same code.
pub(crate) fn adopt_existing(
    root: &Path,
    label: &str,
    custom: &mut Vec<String>,
    skills: &[(&str, &str)],
) -> Vec<String> {
    for (name, shipped) in skills {
        let existing = std::fs::read_to_string(root.join(skill_path(name)));
        // Identical content is superdev's own text already: nothing to keep.
        if existing.is_ok_and(|existing| existing != *shipped) {
            custom.push((*name).to_string());
        }
    }
    custom
        .iter()
        .map(|name| format!("{label}: kept your {name} — marked custom in {CONFIG_PATH}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{ContentSet, Item, ItemKind, Origin};

    /// Test plan case 14: a `custom` name releases an item whatever layer
    /// provided it. Ownership is the item's, not its provenance's — a pack
    /// skill the user has taken over is theirs on the same terms as a
    /// shipped one.
    #[test]
    fn a_custom_name_releases_a_pack_provided_item() {
        let owner = Owner::Knowledge;
        let item = |name: &str| Item {
            owner,
            kind: ItemKind::Skill,
            name: name.to_string(),
            files: vec![("SKILL.md".to_string(), format!("# {name}\n"))],
        };
        let from_pack = ContentSet::from_layers(
            vec![(
                vec![item("acme-review"), item("acme-plan")],
                Origin::Pack {
                    index: 0,
                    name: "./packs/acme".into(),
                },
            )],
            None,
        );

        let all = skill_dir_items(&from_pack, owner, &[]);
        assert_eq!(all.len(), 2, "both pack skills are written by default");

        let released = skill_dir_items(&from_pack, owner, &["acme-review".to_string()]);
        let paths: Vec<&str> = released
            .iter()
            .map(|item| match item {
                ManagedItem::OwnedFile { path, .. } => path.as_str(),
                _ => unreachable!("skills are owned files"),
            })
            .collect();
        assert_eq!(paths, [".claude/skills/acme-plan/SKILL.md"]);
    }
}
