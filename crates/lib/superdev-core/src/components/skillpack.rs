//! components/skillpack.rs — the skills capability: superdev's own pack,
//! shipped as owned files in the managed repo. Claude Code loads project
//! skills from `.claude/skills/` natively, so there is nothing to install.

use crate::action::{Action, Ownership};
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;

macro_rules! asset {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $rel))
    };
}

/// The pack: (skill name, embedded SKILL.md).
pub const SKILLS: [(&str, &str); 4] = [
    ("aokf-maintain", asset!("skills/aokf-maintain/SKILL.md")),
    ("double-check", asset!("skills/double-check/SKILL.md")),
    ("humanise", asset!("skills/humanise/SKILL.md")),
    ("self-improve", asset!("skills/self-improve/SKILL.md")),
];

/// Where Claude Code reads hook registrations. Shared with the user's own
/// hooks, so only superdev's array element is managed.
pub const SETTINGS_PATH: &str = ".claude/settings.json";
/// The array the hook entry lives in.
pub const HOOK_POINTER: &str = "hooks.PostToolUse";
/// What identifies superdev's element among the user's.
pub const HOOK_MARKER: &str = "superdev aokf hook validate";
/// The registration itself: validate the bundle after an Edit/Write.
pub const HOOK_ELEMENT: &str = r#"{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}"#;

/// Release, at adoption time, every pack skill the repo already has under its
/// own name and with its own content. Overwriting those would replace work
/// superdev never wrote with a backup the user has to go looking for; marking
/// them custom keeps the file and hands the choice back. Returns the lines to
/// print. Only `init` calls this — later syncs honour the list as written.
pub(crate) fn adopt_existing(
    root: &std::path::Path,
    manifest: &mut crate::manifest::Manifest,
) -> Vec<String> {
    let Some(config) = manifest.capabilities.get_mut(Capability::Skills.as_str()) else {
        return Vec::new();
    };
    for (name, shipped) in SKILLS {
        let existing =
            std::fs::read_to_string(root.join(format!(".claude/skills/{name}/SKILL.md")));
        // Identical content is superdev's own text already: nothing to keep.
        if existing.is_ok_and(|existing| existing != shipped) {
            config.custom.push(name.to_string());
        }
    }
    config
        .custom
        .iter()
        .map(|name| {
            format!(
                "skills: kept your {name} — marked custom in {}",
                crate::manifest::CONFIG_PATH
            )
        })
        .collect()
}

/// The superdev skill pack provider.
pub struct SkillPack;

impl SkillPack {
    /// The hook action, unless the settings file already carries the exact
    /// desired element. Planning must stay empty when converged: `status`
    /// exits 1 on any planned action.
    fn hook_action(&self, ctx: &Ctx<'_>) -> Option<Action> {
        let desired: serde_json::Value =
            serde_json::from_str(HOOK_ELEMENT).expect("the hook element is valid JSON");
        let present = std::fs::read_to_string(ctx.root.join(SETTINGS_PATH))
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v["hooks"]["PostToolUse"].as_array().cloned())
            .is_some_and(|items| items.contains(&desired));
        (!present).then(|| Action::EnsureJsonArrayElement {
            path: SETTINGS_PATH.into(),
            pointer: HOOK_POINTER.into(),
            marker: HOOK_MARKER.into(),
            value_json: HOOK_ELEMENT.into(),
        })
    }
}

impl Component for SkillPack {
    fn capability(&self) -> Capability {
        Capability::Skills
    }

    fn provider(&self) -> &'static str {
        "superdev-skills"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let config = ctx
            .config(Capability::Skills)
            .expect("planned only when enabled");
        super::pin::require_registry_default(ctx, Capability::Skills, "superdev-skills")?;
        let mut actions = Vec::new();
        for (name, content) in SKILLS {
            if config.custom.iter().any(|c| c == name) {
                continue;
            }
            let path = format!(".claude/skills/{name}/SKILL.md");
            let existing = std::fs::read_to_string(ctx.root.join(&path)).ok();
            if existing.as_deref() != Some(content) {
                actions.push(Action::WriteFile {
                    path,
                    content: content.to_string(),
                    ownership: Ownership::Owned,
                    reason: format!("{name} skill"),
                });
            }
        }
        actions.extend(self.hook_action(ctx));
        Ok(actions)
    }

    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim> {
        let custom = ctx
            .config(Capability::Skills)
            .map(|c| c.custom.as_slice())
            .unwrap_or_default();
        let mut claims: Vec<Claim> = SKILLS
            .iter()
            .filter(|(name, _)| !custom.iter().any(|c| c == name))
            .map(|(name, _)| Claim::File(format!(".claude/skills/{name}/SKILL.md")))
            .collect();
        claims.push(Claim::JsonKey {
            path: SETTINGS_PATH.into(),
            pointer: format!("{HOOK_POINTER}[{HOOK_MARKER}]"),
        });
        claims
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;

    fn ctx_parts() -> (Manifest, Lock) {
        (
            Manifest::default_for(env!("CARGO_PKG_VERSION"), &[]),
            Lock::default(),
        )
    }

    /// Write every skill and the exact hook entry, so nothing is planned.
    fn converge(root: &std::path::Path) {
        for (name, content) in SKILLS {
            let path = root.join(format!(".claude/skills/{name}/SKILL.md"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        std::fs::write(
            root.join(SETTINGS_PATH),
            format!(r#"{{"hooks":{{"PostToolUse":[{HOOK_ELEMENT}]}}}}"#),
        )
        .unwrap();
    }

    #[test]
    fn a_fresh_repo_plans_every_skill_and_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = SkillPack.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 5);
        let descs: Vec<String> = actions.iter().map(|a| a.describe()).collect();
        for (name, _) in SKILLS {
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!(".claude/skills/{name}/SKILL.md"))),
                "{descs:?}"
            );
        }
        assert!(
            descs
                .iter()
                .any(|d| d.contains("superdev aokf hook validate")),
            "{descs:?}"
        );
        assert!(fake.calls().is_empty(), "planning must run nothing");
    }

    #[test]
    fn a_converged_repo_plans_nothing() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(SkillPack.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn a_drifted_skill_is_rewritten_alone() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        std::fs::write(
            dir.path().join(".claude/skills/humanise/SKILL.md"),
            "edited",
        )
        .unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = SkillPack.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].describe().contains("humanise"));
    }

    #[test]
    fn a_custom_skill_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        std::fs::write(
            dir.path().join(".claude/skills/humanise/SKILL.md"),
            "mine now",
        )
        .unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().custom = vec!["humanise".into()];
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(SkillPack.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn an_unknown_custom_name_is_ignored_by_planning() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().custom = vec!["grill-me".into()];
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(SkillPack.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn a_foreign_version_pin_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().version = Some("9.9.9".into());
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(SkillPack.plan(&ctx).is_err());
    }

    #[test]
    fn a_stale_hook_entry_replans_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        converge(dir.path());
        // Same marker, older shape: must be replaced, so it must be planned.
        std::fs::write(
            dir.path().join(SETTINGS_PATH),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#,
        )
        .unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = SkillPack.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].describe().contains("hooks.PostToolUse"));
    }

    #[test]
    fn owned_omits_custom_skills_but_keeps_the_hook() {
        use crate::component::Claim;
        let dir = tempfile::tempdir().unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap().custom = vec!["humanise".into()];
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let keys: Vec<String> = SkillPack.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(!keys.iter().any(|k| k.contains("humanise")), "{keys:?}");
        assert!(keys.contains(&".claude/skills/double-check/SKILL.md".to_string()));
        assert!(keys.contains(
            &".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]".to_string()
        ));
    }

    #[test]
    fn reports_its_slot_and_provider() {
        assert_eq!(SkillPack.capability(), Capability::Skills);
        assert_eq!(SkillPack.provider(), "superdev-skills");
    }

    #[test]
    fn adoption_keeps_the_repos_own_skills_and_ignores_identical_ones() {
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, body: &str| {
            let path = dir.path().join(format!(".claude/skills/{name}/SKILL.md"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        };
        // Theirs, under one of our names.
        write("humanise", "# Ours, thanks\n");
        // Already superdev's own text: nothing of the user's to keep.
        let (_, shipped) = SKILLS
            .iter()
            .find(|(name, _)| *name == "double-check")
            .unwrap();
        write("double-check", shipped);

        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let lines = adopt_existing(dir.path(), &mut manifest);
        assert_eq!(manifest.capabilities["skills"].custom, ["humanise"]);
        assert_eq!(
            lines,
            vec![format!(
                "skills: kept your humanise — marked custom in {}",
                crate::manifest::CONFIG_PATH
            )]
        );

        // Nothing to adopt in an empty repo, or with skills disabled.
        let empty = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        assert!(adopt_existing(empty.path(), &mut manifest).is_empty());
        assert!(manifest.capabilities["skills"].custom.is_empty());
        let mut off = Manifest::default_for("0.1.0", &[crate::capability::Capability::Skills]);
        assert!(adopt_existing(dir.path(), &mut off).is_empty());
    }
}
