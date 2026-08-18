//! components/mattskills.rs — the workflows capability via mattpocock/skills,
//! materialised into the repo as owned files. A collaborator gets working
//! skills from git alone; nothing is installed at user level.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;
use crate::registry::{self, MATTSKILLS_CHECKSUM, MATTSKILLS_URL};

/// mise `[tools]` key for the pinned checkout.
pub const MATTSKILLS_MISE_TOOL: &str = "http:mattpocock-skills";

/// Checkout directories holding one skill directory each.
const SOURCE_DIRS: [&str; 2] = ["skills/engineering", "skills/productivity"];

/// Upstream skill names at the pinned version, for init adoption and custom
/// reporting. Refresh together with the version, url and checksum.
pub const MATTSKILLS_SKILLS: [&str; 25] = [
    "ask-matt",
    "code-review",
    "codebase-design",
    "diagnosing-bugs",
    "domain-modeling",
    "grill-with-docs",
    "implement",
    "improve-codebase-architecture",
    "prototype",
    "research",
    "resolving-merge-conflicts",
    "setup-matt-pocock-skills",
    "tdd",
    "to-spec",
    "to-tickets",
    "triage",
    "wayfinder",
    "wizard",
    "grill-me",
    "grilling",
    "handoff",
    "teach",
    "to-questionnaire",
    "wait-what",
    "writing-for-agents",
];

/// Release, at init time, every mattpocock-skills upstream skill directory
/// the repo already has. The checkout does not exist yet, so unlike the
/// skill pack's adoption there is no content to compare — any existing
/// directory counts, and the report says what to do if the user wants it
/// managed instead. Only `init` calls this — later syncs honour the list as
/// written. A no-op off the mattpocock-skills provider.
pub(crate) fn adopt_existing(
    root: &std::path::Path,
    manifest: &mut crate::manifest::Manifest,
) -> Vec<String> {
    let Some(config) = manifest
        .capabilities
        .get_mut(Capability::Workflows.as_str())
    else {
        return Vec::new();
    };
    if config.provider != "mattpocock-skills" {
        return Vec::new();
    }
    for name in MATTSKILLS_SKILLS {
        if root.join(format!(".claude/skills/{name}")).is_dir() {
            config.custom.push(name.to_string());
        }
    }
    config
        .custom
        .iter()
        .map(|name| {
            format!(
                "workflows: kept your {name} — marked custom in {}",
                crate::manifest::CONFIG_PATH
            )
        })
        .collect()
}

macro_rules! asset {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $rel))
    };
}

/// Embedded skill overrides, as (target path, content): superdev's own
/// version of an upstream skill, materialised in place of the checkout's.
/// Carried by this component, so they install exactly where this provider
/// is installed. The skill name (the path segment under `.claude/skills/`)
/// is what a `custom` entry releases.
pub(crate) const OVERRIDES: [(&str, &str); 2] = [
    (
        ".claude/skills/grilling/SKILL.md",
        asset!("overrides/mattpocock-skills/skills/grilling/SKILL.md"),
    ),
    (
        ".claude/skills/grilling/agents/openai.yaml",
        asset!("overrides/mattpocock-skills/skills/grilling/agents/openai.yaml"),
    ),
];

/// The skill name an override target belongs to, for `custom` matching.
fn override_skill(path: &str) -> &str {
    path.strip_prefix(".claude/skills/")
        .and_then(|rest| rest.split('/').next())
        .expect("override targets live under .claude/skills/")
}

/// The overrides a manifest still manages: `custom` releases the whole
/// skill, upstream and override alike.
fn live_overrides(custom: &[String]) -> Vec<(String, String)> {
    OVERRIDES
        .iter()
        .filter(|(path, _)| !custom.iter().any(|c| c == override_skill(path)))
        .map(|(path, content)| ((*path).to_string(), (*content).to_string()))
        .collect()
}

/// The mattpocock-skills provider.
pub struct MattSkills;

/// The `.mise.toml` value for the pinned release.
fn pin_value() -> String {
    let version = registry::entry_for(Capability::Workflows, "mattpocock-skills")
        .and_then(|e| e.version)
        .expect("registry pins mattpocock-skills")
        .version;
    format!(
        "{{ version = \"{version}\", url = \"{MATTSKILLS_URL}\", checksum = \"{MATTSKILLS_CHECKSUM}\", strip_components = 1 }}"
    )
}

impl Component for MattSkills {
    fn capability(&self) -> Capability {
        Capability::Workflows
    }

    fn provider(&self) -> &'static str {
        "mattpocock-skills"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let config = ctx
            .config(Capability::Workflows)
            .expect("planned only when enabled");
        let mut actions = Vec::new();
        if let Some(pin) = super::pin::planned_pin(
            ctx,
            Capability::Workflows,
            "mattpocock-skills",
            MATTSKILLS_MISE_TOOL,
            &pin_value(),
        )? {
            actions.push(pin);
        }
        if refresh_due(ctx, config) {
            actions.push(Action::MaterialiseSkills {
                tool: MATTSKILLS_MISE_TOOL.into(),
                source_dirs: SOURCE_DIRS.iter().map(|d| (*d).to_string()).collect(),
                custom: config.custom.clone(),
                overrides: live_overrides(&config.custom),
            });
        }
        Ok(actions)
    }

    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim> {
        let mut claims = vec![Claim::MisePin(MATTSKILLS_MISE_TOOL.to_string())];
        claims.extend(
            ctx.lock
                .owners
                .iter()
                .filter(|(_, owner)| owner.as_str() == Capability::Workflows.as_str())
                .map(|(key, _)| Claim::File(key.clone())),
        );
        claims
    }
}

/// Whether the materialised set needs refreshing — answered from the lock,
/// the working tree and this binary's embedded overrides, so `status` needs
/// neither network nor checkout.
fn refresh_due(ctx: &Ctx<'_>, config: &crate::manifest::CapabilityConfig) -> bool {
    let applied = ctx.lock.components.get(Capability::Workflows.as_str());
    let recorded =
        applied.is_some_and(|a| a.provider == "mattpocock-skills" && a.version == config.version);
    let attributed: Vec<&String> = ctx
        .lock
        .owners
        .iter()
        .filter(|(_, owner)| owner.as_str() == Capability::Workflows.as_str())
        .map(|(key, _)| key)
        .collect();
    if !recorded || attributed.is_empty() {
        return true;
    }
    if attributed.into_iter().any(|key| {
        // A hand-edited lock can drop the hash; refresh rather than panic.
        let Some(locked) = ctx.lock.files.get(key) else {
            return true;
        };
        match std::fs::read_to_string(ctx.root.join(key)) {
            Ok(content) => crate::lock::sha256_hex(content.as_bytes()) != *locked,
            Err(_) => true,
        }
    }) {
        return true;
    }
    // Overrides drift against the binary, not the lock: an upgrade that
    // changes embedded content leaves file and lock agreeing with each
    // other, so compare the file against what this binary ships.
    live_overrides(&config.custom)
        .iter()
        .any(
            |(path, content)| match std::fs::read_to_string(ctx.root.join(path)) {
                Ok(existing) => existing != *content,
                Err(_) => true,
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::component::{Claim, Component, Ctx};
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;

    fn ctx_parts() -> (Manifest, Lock) {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.version = Some("1.2.3".into());
        (manifest, Lock::default())
    }

    #[test]
    fn a_fresh_repo_plans_pin_and_materialise() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let descs: Vec<String> = MattSkills
            .plan(&ctx)
            .unwrap()
            .iter()
            .map(|a| a.describe())
            .collect();
        assert!(
            descs
                .iter()
                .any(|d| d.contains("pin http:mattpocock-skills")),
            "{descs:?}"
        );
        assert!(
            descs.contains(
                &"materialise http:mattpocock-skills skills into .claude/skills/".to_string()
            ),
            "{descs:?}"
        );
        assert!(fake.calls().is_empty(), "planning must run nothing");
    }

    #[test]
    fn a_converged_repo_plans_nothing_and_owns_its_files() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, mut lock) = ctx_parts();
        std::fs::write(
            dir.path().join(".mise.toml"),
            crate::components::mise::set_pin("", MATTSKILLS_MISE_TOOL, &pin_value()).unwrap(),
        )
        .unwrap();
        let skill = dir.path().join(".claude/skills/tdd/SKILL.md");
        std::fs::create_dir_all(skill.parent().unwrap()).unwrap();
        std::fs::write(&skill, "tdd content").unwrap();
        // Converged includes the embedded overrides: file, lock and owners.
        for (path, content) in OVERRIDES {
            let p = dir.path().join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, content).unwrap();
        }
        lock.files.insert(
            ".claude/skills/tdd/SKILL.md".into(),
            crate::lock::sha256_hex(b"tdd content"),
        );
        lock.owners
            .insert(".claude/skills/tdd/SKILL.md".into(), "workflows".into());
        for (path, content) in OVERRIDES {
            lock.files
                .insert((*path).into(), crate::lock::sha256_hex(content.as_bytes()));
            lock.owners.insert((*path).into(), "workflows".into());
        }
        lock.components.insert(
            "workflows".into(),
            crate::lock::LockedComponent {
                provider: "mattpocock-skills".into(),
                version: Some("1.2.3".into()),
            },
        );
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(MattSkills.plan(&ctx).unwrap().is_empty());
        let keys: Vec<String> = MattSkills.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(keys.contains(&".mise.toml:http:mattpocock-skills".to_string()));
        assert!(keys.contains(&".claude/skills/tdd/SKILL.md".to_string()));

        // Drift in one materialised file replans the refresh.
        std::fs::write(&skill, "edited").unwrap();
        let actions = MattSkills.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].describe().contains("materialise"), "{actions:?}");
    }

    #[test]
    fn override_drift_replans_and_custom_releases_the_override() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, mut lock) = ctx_parts();
        std::fs::write(
            dir.path().join(".mise.toml"),
            crate::components::mise::set_pin("", MATTSKILLS_MISE_TOOL, &pin_value()).unwrap(),
        )
        .unwrap();
        lock.components.insert(
            "workflows".into(),
            crate::lock::LockedComponent {
                provider: "mattpocock-skills".into(),
                version: Some("1.2.3".into()),
            },
        );
        for (path, content) in OVERRIDES {
            let p = dir.path().join(path);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, content).unwrap();
            lock.files
                .insert((*path).into(), crate::lock::sha256_hex(content.as_bytes()));
            lock.owners.insert((*path).into(), "workflows".into());
        }
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(MattSkills.plan(&ctx).unwrap().is_empty());

        // A binary whose embedded override differs from the file replans,
        // even though file and lock agree with each other.
        let (path, _) = OVERRIDES[0];
        std::fs::write(dir.path().join(path), "older override").unwrap();
        lock.files
            .insert(path.into(), crate::lock::sha256_hex(b"older override"));
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = MattSkills.plan(&ctx).unwrap();
        assert_eq!(actions.len(), 1, "{actions:?}");
        let overrides = match &actions[0] {
            Action::MaterialiseSkills { overrides, .. } => overrides.clone(),
            other => panic!("{other:?}"),
        };
        assert!(overrides.iter().any(|(p, _)| p == path));

        // custom releases the whole skill: no override pairs ride the action,
        // and the released file no longer counts as drift.
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("workflows").unwrap().custom = vec!["grilling".into()];
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let overrides = MattSkills
            .plan(&ctx)
            .unwrap()
            .into_iter()
            .find_map(|a| match a {
                Action::MaterialiseSkills { overrides, .. } => Some(overrides),
                _ => None,
            })
            .expect("fresh lock replans the materialise");
        assert!(overrides.is_empty(), "{overrides:?}");
    }

    #[test]
    fn a_foreign_version_pin_is_rejected_and_custom_rides_the_action() {
        let dir = tempfile::tempdir().unwrap();
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("workflows").unwrap().version = Some("9.9.9".into());
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(MattSkills.plan(&ctx).is_err());

        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("workflows").unwrap().custom = vec!["grill-me".into()];
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let custom = MattSkills
            .plan(&ctx)
            .unwrap()
            .into_iter()
            .find_map(|a| match a {
                Action::MaterialiseSkills { custom, .. } => Some(custom),
                _ => None,
            });
        assert_eq!(custom.unwrap(), vec!["grill-me".to_string()]);
    }

    #[test]
    fn adoption_marks_existing_upstream_skill_dirs_custom() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude/skills/tdd")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/tdd/SKILL.md"), "mine").unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        let lines = adopt_existing(dir.path(), &mut manifest);
        assert_eq!(manifest.capabilities["workflows"].custom, ["tdd"]);
        assert_eq!(
            lines,
            vec!["workflows: kept your tdd — marked custom in .superdev/config.toml".to_string()]
        );
        // A superpowers manifest adopts nothing.
        let mut superpowers = Manifest::default_for("0.1.0", &[]);
        superpowers
            .capabilities
            .get_mut("workflows")
            .unwrap()
            .provider = "superpowers".into();
        assert!(adopt_existing(dir.path(), &mut superpowers).is_empty());
    }
}
