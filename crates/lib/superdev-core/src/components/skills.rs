//! components/skills.rs — shared machinery for capabilities that ship
//! embedded skills into `.claude/skills/`: the path convention, the item
//! list, and init-time adoption. The skill pack and the aokf component both
//! build on it, so the two cannot drift apart in behaviour.

use std::path::Path;

use crate::capability::Capability;
use crate::manifest::{CONFIG_PATH, Manifest};

use super::item::ManagedItem;

/// One shipped skill directory: its name and every file, as (path relative
/// to the directory, content).
pub(crate) type SkillFiles = (&'static str, &'static [(&'static str, &'static str)]);

/// Each non-custom skill directory as owned files, in shipped order.
pub(crate) fn skill_dir_items(skills: &[SkillFiles], custom: &[String]) -> Vec<ManagedItem> {
    skills
        .iter()
        .filter(|(name, _)| !custom.iter().any(|c| c == name))
        .flat_map(|(name, files)| {
            files
                .iter()
                .map(move |(rel, content)| ManagedItem::OwnedFile {
                    path: format!(".claude/skills/{name}/{rel}"),
                    content: (*content).to_string(),
                    reason: format!("{name} skill"),
                })
        })
        .collect()
}

/// Where a shipped skill lives in the managed repo.
pub(crate) fn skill_path(name: &str) -> String {
    format!(".claude/skills/{name}/SKILL.md")
}

/// Each non-custom skill as an owned file, in shipped order.
pub(crate) fn skill_items(skills: &[(&str, &str)], custom: &[String]) -> Vec<ManagedItem> {
    skills
        .iter()
        .filter(|(name, _)| !custom.iter().any(|c| c == name))
        .map(|(name, content)| ManagedItem::OwnedFile {
            path: skill_path(name),
            content: (*content).to_string(),
            reason: format!("{name} skill"),
        })
        .collect()
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
    skills: &[(&str, &str)],
    manifest: &mut Manifest,
) -> Vec<String> {
    let Some(config) = manifest.capabilities.get_mut(capability.as_str()) else {
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
