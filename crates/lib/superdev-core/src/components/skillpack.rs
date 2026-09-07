//! components/skillpack.rs — the skills capability: superdev's own pack,
//! shipped as owned files in the managed repo. Claude Code loads project
//! skills from `.claude/skills/` natively, so there is nothing to install.

use std::path::Path;

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::content::{ContentSet, Owner};
use crate::error::Result;
use crate::manifest::Manifest;

use super::item::{self, ManagedItem};

/// The skills capability owns superdev's own pack: whatever the resolved
/// content carries under this owner.
pub(crate) const OWNER: Owner = Owner::Capability(Capability::Skills);

/// Release, at adoption time, every pack skill the repo already has under
/// its own name and with its own content. Returns the lines to print.
pub(crate) fn adopt_existing(
    root: &Path,
    content: &ContentSet,
    manifest: &mut Manifest,
) -> Vec<String> {
    let identities = super::skills::skill_identities(content, OWNER);
    let Some(config) = manifest.config_of_mut(Capability::Skills, "superdev-skills") else {
        return Vec::new();
    };
    super::skills::adopt_existing(
        root,
        Capability::Skills.as_str(),
        &mut config.custom,
        &identities,
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
    super::skills::skill_dir_items(ctx.content, OWNER, custom)
}

impl Component for SkillPack {
    fn capability(&self) -> Option<Capability> {
        Some(Capability::Skills)
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

    /// The pack's skills as the resolved content carries them.
    fn shipped() -> Vec<(&'static str, &'static str)> {
        crate::components::skills::skill_identities(crate::content::test_snapshot(), OWNER)
    }

    /// Write every skill, so nothing is planned.
    fn converge(root: &std::path::Path) {
        for (name, content) in shipped() {
            let path = root.join(format!(".claude/skills/{name}/SKILL.md"));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
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
            content: crate::content::test_snapshot(),
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
            content: crate::content::test_snapshot(),
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
            content: crate::content::test_snapshot(),
        };
        assert!(SkillPack.plan(&ctx).is_err());
    }
    #[test]
    fn reports_its_slot_and_provider() {
        assert_eq!(SkillPack.capability(), Some(Capability::Skills));
        assert_eq!(SkillPack.provider(), "superdev-skills");
    }
}
