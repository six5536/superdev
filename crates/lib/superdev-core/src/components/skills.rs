//! components/skills.rs — shared machinery for capabilities that ship
//! embedded skills into `.claude/skills/`: the path convention, the item
//! list, and init-time adoption. The skill pack and the aokf component both
//! build on it, so the two cannot drift apart in behaviour.

use std::path::Path;

use crate::capability::Capability;
use crate::manifest::{CONFIG_PATH, Manifest};

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
pub(crate) fn adopt_existing(
    root: &Path,
    capability: Capability,
    provider: &str,
    skills: &[(&str, &str)],
    manifest: &mut Manifest,
) -> Vec<String> {
    let Some(config) = manifest.config_of_mut(capability, provider) else {
        return Vec::new();
    };
    for (name, shipped) in skills {
        let existing = std::fs::read_to_string(root.join(skill_path(name)));
        // Identical content is superdev's own text already: nothing to keep.
        if existing.is_ok_and(|existing| existing != *shipped) {
            config.custom.push((*name).to_string());
        }
    }
    config
        .custom
        .iter()
        .map(|name| {
            format!(
                "{}: kept your {name} — marked custom in {CONFIG_PATH}",
                capability.as_str()
            )
        })
        .collect()
}
