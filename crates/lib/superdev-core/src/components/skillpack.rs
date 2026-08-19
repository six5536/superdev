//! components/skillpack.rs — the skills capability: superdev's own pack,
//! shipped as owned files in the managed repo. Claude Code loads project
//! skills from `.claude/skills/` natively, so there is nothing to install.

use std::path::Path;

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;
use crate::manifest::Manifest;

use super::item::{self, ManagedItem};

macro_rules! asset {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $rel))
    };
}

/// The pack: (skill name, embedded SKILL.md). The knowledge-lifecycle
/// skills live with the aokf component, not here.
pub(crate) const SKILLS: [(&str, &str); 4] = [
    ("double-check", asset!("skills/double-check/SKILL.md")),
    ("humanise", asset!("skills/humanise/SKILL.md")),
    ("self-improve", asset!("skills/self-improve/SKILL.md")),
    ("template-update", asset!("skills/template-update/SKILL.md")),
];

/// Release, at adoption time, every pack skill the repo already has under
/// its own name and with its own content. Returns the lines to print.
pub(crate) fn adopt_existing(root: &Path, manifest: &mut Manifest) -> Vec<String> {
    super::skills::adopt_existing(
        root,
        Capability::Skills,
        "superdev-skills",
        &SKILLS,
        manifest,
    )
}

/// The superdev skill pack provider.
pub struct SkillPack;

/// Everything the pack keeps in the repo: each non-custom skill as an owned
/// file.
fn items(ctx: &Ctx<'_>) -> Vec<ManagedItem> {
    let custom = ctx
        .config(Capability::Skills, "superdev-skills")
        .map(|c| c.custom.as_slice())
        .unwrap_or_default();
    super::skills::skill_items(&SKILLS, custom)
}

impl Component for SkillPack {
    fn capability(&self) -> Capability {
        Capability::Skills
    }

    fn provider(&self) -> &'static str {
        "superdev-skills"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        super::pin::require_registry_default(ctx, Capability::Skills, "superdev-skills")?;
        Ok(item::plan_items(ctx.root, &items(ctx)))
    }

    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim> {
        item::claims(&items(ctx))
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

    /// Write every skill, so nothing is planned.
    fn converge(root: &std::path::Path) {
        for (name, content) in SKILLS {
            let path = root.join(format!(".claude/skills/{name}/SKILL.md"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
    }

    #[test]
    fn a_fresh_repo_plans_every_skill() {
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
        assert_eq!(actions.len(), 4);
        let descs: Vec<String> = actions.iter().map(|a| a.describe()).collect();
        for (name, _) in SKILLS {
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!(".claude/skills/{name}/SKILL.md"))),
                "{descs:?}"
            );
        }
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
        manifest.capabilities.get_mut("skills").unwrap()[0].custom = vec!["humanise".into()];
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
        manifest.capabilities.get_mut("skills").unwrap()[0].custom = vec!["grill-me".into()];
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
        manifest.capabilities.get_mut("skills").unwrap()[0].version = Some("9.9.9".into());
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
    fn owned_omits_custom_skills() {
        use crate::component::Claim;
        let dir = tempfile::tempdir().unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("skills").unwrap()[0].custom = vec!["humanise".into()];
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
        assert_eq!(manifest.capabilities["skills"][0].custom, ["humanise"]);
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
        assert!(manifest.capabilities["skills"][0].custom.is_empty());
        let mut off = Manifest::default_for("0.1.0", &[crate::capability::Capability::Skills]);
        assert!(adopt_existing(dir.path(), &mut off).is_empty());
    }
}
