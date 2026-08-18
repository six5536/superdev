//! pipeline.rs — the verb pipeline between manifest and engine: one plan
//! entry and one apply entry, shared by init, status, sync and update.
//!
//! The ordering rules live here, by construction: the custom prune runs
//! before planning (an unpruned just-released skill would read as an orphan),
//! and the orphan pass plans last (removals run after every component write).
//! The binary loads, calls, renders and turns facts into exit codes.

use std::io;
use std::path::Path;

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Ctx};
use crate::components::codegraph::CODEGRAPH_INDEX_DIR;
use crate::components::{mattskills, skillpack};
use crate::engine::Planned;
use crate::error::{Error, Result};
use crate::lock::Lock;
use crate::manifest::Manifest;
use crate::orphan::OrphanPlan;
use crate::registry::{self, Pinned};
use crate::runner::CommandRunner;
use crate::{components, engine, orphan, report};

/// Provider name for repo-level actions no capability owns.
const REPO_PROVIDER: &str = "superdev";

/// How the pipeline treats a manifest pinned off the registry default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanMode {
    /// Plan the version this binary can provide and let the behind lines
    /// carry the news — status must report a stale pin, not fail on it.
    Status,
    /// Refuse to plan: sync would have to act on the pin, and substituting
    /// the default silently is worse than stopping.
    Sync,
}

/// One planning pass over the repo: the actions, the orphan outcome, the
/// report lines, and the pruned lock the apply consumes.
pub struct RepoPlan {
    planned: Vec<Planned>,
    orphans: OrphanPlan,
    behind: Vec<String>,
    custom: Vec<String>,
    switch: Vec<String>,
    blueprint: Option<String>,
    /// The loaded lock with custom-released entries pruned in memory.
    lock: Lock,
    /// True when the prune removed anything — the lock needs saving even
    /// when nothing else is planned.
    lock_changed: bool,
}

impl RepoPlan {
    /// The planned entries, for rendering.
    pub fn planned(&self) -> &[Planned] {
        &self.planned
    }

    /// Prepend an init-only entry — the project template — ahead of every
    /// capability's, so its scaffolds exist before any component write. No
    /// other verb plans templates, which is what keeps them write-once.
    pub fn prepend(&mut self, entry: Planned) {
        self.planned.insert(0, entry);
    }

    /// Whether any entry carries an action.
    pub fn has_actions(&self) -> bool {
        self.planned.iter().any(|p| !p.actions.is_empty())
    }

    /// Whether the plan copies an upstream skill set into the repo.
    pub fn materialises(&self) -> bool {
        self.planned.iter().any(|p| {
            p.actions
                .iter()
                .any(|a| matches!(a, Action::MaterialiseSkills { .. }))
        })
    }

    /// One line per enabled capability pinned away from this binary's registry.
    pub fn behind_lines(&self) -> &[String] {
        &self.behind
    }

    /// One line per skill or workflow skill released to the user.
    pub fn custom_lines(&self) -> &[String] {
        &self.custom
    }

    /// What a workflows provider switch left behind.
    pub fn switch_lines(&self) -> &[String] {
        &self.switch
    }

    /// One line per orphan released because the user edited it.
    pub fn released_lines(&self) -> Vec<String> {
        self.orphans.released_lines()
    }

    /// The blueprint-version report: informational, never the exit code. A
    /// settled repo under a newer binary is not drift.
    pub fn blueprint_line(&self) -> Option<&str> {
        self.blueprint.as_deref()
    }
}

/// The result of an apply: the rendered report and what the run did.
pub struct ApplyOutcome {
    /// The rendered apply report; empty when nothing needed applying.
    pub report: String,
    /// False when the engine failed and unwound.
    pub ok: bool,
    /// True when the plan materialised an upstream skill set.
    pub materialised: bool,
}

/// Plan the whole repo: prune the custom-released lock entries in memory,
/// plan the repo-level lines and every component, and put the orphan pass
/// last so removals run after every component write — a rename whose write
/// fails rolls back before anything is deleted.
pub fn plan_repo(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    lock: &Lock,
    mode: PlanMode,
) -> Result<RepoPlan> {
    // The report lines describe the manifest as written; Status planning
    // alone runs against the plannable copy below.
    let behind = behind_pins(manifest);
    let custom = custom_lines(manifest);
    let blueprint = blueprint_line(manifest);
    let plannable_manifest;
    let manifest = match mode {
        PlanMode::Status => {
            plannable_manifest = plannable(manifest);
            &plannable_manifest
        }
        PlanMode::Sync => {
            if let Some((capability, pinned, default)) = locked_pin_mismatch(manifest) {
                return Err(Error::Manifest {
                    message: format!(
                        "{} is pinned {pinned} but this superdev only supports {default} — run `superdev update`",
                        capability.as_str()
                    ),
                });
            }
            manifest
        }
    };
    let mut lock = lock.clone();
    // Before planning: a skill or workflow just marked custom still has its
    // lock entry, and unpruned an unmodified one would read as an orphan and
    // be deleted — the opposite of what marking it custom asked for.
    let lock_changed = prune_custom(manifest, &mut lock);
    let components = components::enabled(manifest)?;
    let ctx = Ctx {
        root,
        runner,
        manifest,
        lock: &lock,
    };
    let mut planned = Vec::new();
    planned.extend(repo_entry(root, manifest)?);
    planned.extend(engine::plan(&components, &ctx)?);
    let claims_by_capability: Vec<(Capability, Vec<Claim>)> = components
        .iter()
        .map(|c| (c.capability(), c.owned(&ctx)))
        .collect();
    claim_collision(&claims_by_capability)?;
    let claims: Vec<Claim> = claims_by_capability
        .into_iter()
        .flat_map(|(_, claims)| claims)
        .collect();
    let orphans = orphan::plan(root, &lock, &claims)?;
    if !orphans.actions.is_empty() {
        planned.push(Planned {
            capability: None,
            provider: REPO_PROVIDER.into(),
            actions: orphans.actions.clone(),
        });
    }
    Ok(RepoPlan {
        behind,
        custom,
        switch: switch_lines(manifest, &lock),
        blueprint,
        planned,
        orphans,
        lock,
        lock_changed,
    })
}

/// Apply a plan: reconcile the lock's released, gone and disabled keys, then
/// either save the changed lock and stamp the blueprint (nothing planned) or
/// run the engine, keep the lock only on success, and stamp. The lock and
/// stamp settle before the caller prints `report`: a reader that closes
/// stdout early (`sync | head`) must not leave applied changes without
/// their lock entries. The cost is narrow: a stamp failure after a
/// successful apply surfaces as the error and drops the rendered report.
pub fn apply_repo(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    plan: RepoPlan,
) -> Result<ApplyOutcome> {
    let materialised = plan.materialises();
    let RepoPlan {
        planned,
        orphans,
        mut lock,
        mut lock_changed,
        ..
    } = plan;
    // Released and gone orphans leave the lock without an action, and a
    // disabled capability's applied record goes with its files.
    for key in orphans.released.iter().chain(orphans.gone.iter()) {
        lock_changed |= lock.files.remove(key).is_some();
        lock.owners.remove(key);
    }
    let disabled: Vec<String> = lock
        .components
        .keys()
        .filter(|name| !manifest.capabilities.contains_key(*name))
        .cloned()
        .collect();
    for name in disabled {
        lock.components.remove(&name);
        lock_changed = true;
    }
    if planned.iter().all(|p| p.actions.is_empty()) {
        if lock_changed {
            lock.save(root)?;
        }
        stamp_blueprint(root, manifest)?;
        return Ok(ApplyOutcome {
            report: String::new(),
            ok: true,
            materialised: false,
        });
    }
    let result = engine::apply(root, runner, manifest, &planned, &mut lock);
    if result.ok {
        lock.save(root)?;
        stamp_blueprint(root, manifest)?;
    }
    Ok(ApplyOutcome {
        report: report::render_apply(&result),
        ok: result.ok,
        materialised,
    })
}

/// Mark as custom, at init time, everything the repo already carries under a
/// name superdev would manage. Returns the lines to print.
pub fn adopt_existing(root: &Path, manifest: &mut Manifest) -> Vec<String> {
    let mut lines = skillpack::adopt_existing(root, manifest);
    lines.extend(mattskills::adopt_existing(root, manifest));
    lines
}

/// The pin for the provider the manifest names, falling back to the default
/// entry when the capability is not enabled or names a provider the registry
/// lacks. None means the version floats.
pub fn selected_pin(manifest: &Manifest, capability: Capability) -> Option<Pinned> {
    manifest
        .capabilities
        .get(capability.as_str())
        .and_then(|c| registry::entry_for(capability, &c.provider))
        .unwrap_or_else(|| registry::default_entry(capability))
        .version
}

/// The registry version for the provider the manifest names, when both exist.
pub fn registry_version(manifest: &Manifest, capability: Capability) -> Option<String> {
    let config = manifest.capabilities.get(capability.as_str())?;
    registry::entry_for(capability, &config.provider)?
        .version
        .map(|p| p.version.to_string())
}

/// Refuse when two capabilities claim the same lock key. Deliberate
/// overrides are intra-component, so a cross-component collision is always
/// an accident — silently picking a winner would oscillate across syncs.
/// The message carries the way out.
fn claim_collision(claims_by_capability: &[(Capability, Vec<Claim>)]) -> Result<()> {
    let mut seen: std::collections::BTreeMap<String, Capability> =
        std::collections::BTreeMap::new();
    for (capability, claims) in claims_by_capability {
        for claim in claims {
            let key = claim.lock_key();
            if let Some(first) = seen.get(&key)
                && first != capability
            {
                return Err(Error::Manifest {
                    message: format!(
                        "{} and {} both claim {key} — add its skill to one side's custom list, or upgrade superdev",
                        first.as_str(),
                        capability.as_str()
                    ),
                });
            }
            seen.insert(key, *capability);
        }
    }
    Ok(())
}

/// The ignore lines no capability owns: superdev's machine state, and the
/// code index when that capability is on. Neither belongs in git.
fn repo_entry(root: &Path, manifest: &Manifest) -> Result<Option<Planned>> {
    let path = root.join(".gitignore");
    let existing = match std::fs::read_to_string(&path) {
        Ok(existing) => existing,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(source) => return Err(Error::Io { path, source }),
    };
    let mut wanted = vec![(".superdev/cache/".to_string(), "ignore machine state")];
    if manifest
        .capabilities
        .contains_key(Capability::CodeIndex.as_str())
    {
        wanted.push((format!("{CODEGRAPH_INDEX_DIR}/"), "ignore the code index"));
    }
    let actions: Vec<Action> = wanted
        .into_iter()
        .filter(|(line, _)| !crate::fsutil::has_line(&existing, line))
        .map(|(line, reason)| Action::EnsureLine {
            path: ".gitignore".into(),
            line,
            reason: reason.to_string(),
        })
        .collect();
    if actions.is_empty() {
        return Ok(None);
    }
    Ok(Some(Planned {
        capability: None,
        provider: REPO_PROVIDER.into(),
        actions,
    }))
}

/// Remove released skills' and workflows' hashes from the lock: a custom
/// name is the user's file, and a stale hash would misread their next edit
/// as drift against superdev content. True when anything was removed.
fn prune_custom(manifest: &Manifest, lock: &mut Lock) -> bool {
    let mut pruned = false;
    if let Some(config) = manifest.capabilities.get(Capability::Skills.as_str()) {
        for name in &config.custom {
            let key = format!(".claude/skills/{name}/SKILL.md");
            pruned |= lock.files.remove(&key).is_some();
            lock.owners.remove(&key);
        }
    }
    if let Some(config) = manifest.capabilities.get(Capability::Workflows.as_str()) {
        for name in &config.custom {
            let prefix = format!(".claude/skills/{name}/");
            // Attribution decides, not the path: the skill pack writes under
            // the same directory, and a pack name listed here must not
            // release the pack's file.
            let keys: Vec<String> = lock
                .files
                .keys()
                .filter(|key| {
                    key.starts_with(&prefix)
                        && lock.owners.get(*key).map(String::as_str)
                            == Some(Capability::Workflows.as_str())
                })
                .cloned()
                .collect();
            for key in keys {
                pruned |= lock.files.remove(&key).is_some();
                lock.owners.remove(&key);
            }
        }
    }
    pruned
}

/// One line per skill or workflow skill released to the user, so custom
/// state stays visible without reading the manifest. Flags a custom name
/// that names no shipped skill, since marking it custom has no effect.
fn custom_lines(manifest: &Manifest) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(config) = manifest.capabilities.get(Capability::Skills.as_str()) {
        for name in &config.custom {
            lines.push(
                if skillpack::SKILLS.iter().any(|(known, _)| known == name) {
                    format!("skills: {name} custom, unmanaged")
                } else {
                    format!("skills: custom names unknown skill '{name}' — no effect")
                },
            );
        }
    }
    if let Some(config) = manifest.capabilities.get(Capability::Workflows.as_str())
        && config.provider == "mattpocock-skills"
    {
        for name in &config.custom {
            lines.push(if mattskills::MATTSKILLS_SKILLS.contains(&name.as_str()) {
                format!("workflows: {name} custom, unmanaged")
            } else {
                format!("workflows: custom names unknown skill '{name}' — no effect")
            });
        }
    }
    lines
}

/// When the lock's applied workflows provider differs from the manifest's:
/// what the switch left behind. Empty when the lock has no workflows record,
/// or when the recorded provider already matches.
fn switch_lines(manifest: &Manifest, lock: &Lock) -> Vec<String> {
    let Some(applied) = lock.components.get(Capability::Workflows.as_str()) else {
        return Vec::new();
    };
    let Some(config) = manifest.capabilities.get(Capability::Workflows.as_str()) else {
        return Vec::new();
    };
    if applied.provider == config.provider {
        return Vec::new();
    }
    let mut lines = Vec::new();
    if manifest
        .capabilities
        .contains_key(Capability::Knowledge.as_str())
    {
        lines.push(
            "workflows: update the .agents import in AGENTS.md for the new provider".to_string(),
        );
    }
    if applied.provider == "superpowers" {
        lines.push(
            "workflows: superpowers plugin left installed — `claude plugin uninstall superpowers` removes it"
                .to_string(),
        );
    }
    lines
}

/// One line per enabled capability pinned away from this binary's registry.
fn behind_pins(manifest: &Manifest) -> Vec<String> {
    let mut lines = Vec::new();
    for capability in Capability::ALL {
        // Every registry-pinned version is locked to the default, so stale
        // means mismatched — there is no is-it-older question to ask.
        let Some((pinned, default)) = pin_mismatch(manifest, capability) else {
            continue;
        };
        lines.push(format!(
            "{}: pinned {pinned}, registry has {default} — run `superdev update`",
            capability.as_str()
        ));
    }
    lines
}

/// A registry-locked capability's pin and this binary's default, when the two
/// differ. Only the default has provenance — a checksum baked in beside it,
/// or the content itself — so any other pin, newer included, is one superdev
/// cannot install.
fn pin_mismatch(manifest: &Manifest, capability: Capability) -> Option<(String, String)> {
    let config = manifest.capabilities.get(capability.as_str())?;
    let default = registry_version(manifest, capability)?;
    let pinned = config.version.clone();
    (pinned.as_deref() != Some(default.as_str()))
        .then(|| (pinned.unwrap_or_else(|| "(unset)".into()), default))
}

/// The first registry-locked capability pinned off this binary's default.
fn locked_pin_mismatch(manifest: &Manifest) -> Option<(Capability, String, String)> {
    Capability::ALL.into_iter().find_map(|capability| {
        pin_mismatch(manifest, capability).map(|(pinned, default)| (capability, pinned, default))
    })
}

/// A copy of the manifest that can be planned: every registry-locked
/// capability back at the default. Unpinned capabilities are left alone —
/// components accept those as given.
fn plannable(manifest: &Manifest) -> Manifest {
    let mut plannable = manifest.clone();
    for capability in Capability::ALL {
        // No entry means an unknown provider; leave the pin and let the
        // resolution error say so.
        let Some(version) = registry_version(manifest, capability) else {
            continue;
        };
        if let Some(config) = plannable.capabilities.get_mut(capability.as_str()) {
            config.version = Some(version);
        }
    }
    plannable
}

/// The blueprint-version report line, when the manifest is stale.
fn blueprint_line(manifest: &Manifest) -> Option<String> {
    (manifest.blueprint != crate::version()).then(|| {
        format!(
            "blueprint {}, binary {} — sync will update it",
            manifest.blueprint,
            crate::version()
        )
    })
}

/// Record this binary's version as the blueprint last applied. Rewrites
/// config.toml only when the value changes.
fn stamp_blueprint(root: &Path, manifest: &Manifest) -> Result<()> {
    if manifest.blueprint != crate::version() {
        let mut manifest = manifest.clone();
        manifest.blueprint = crate::version().to_string();
        manifest.save(root)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::{LockedComponent, sha256_hex};
    use crate::runner::FakeRunner;

    /// Every capability whose default entry is registry-locked, derived so
    /// no test re-encodes the list the registry owns.
    fn locked_capabilities() -> Vec<Capability> {
        let locked: Vec<Capability> = registry::entries()
            .iter()
            .filter(|e| e.default && e.version.is_some())
            .map(|e| e.capability)
            .collect();
        assert!(!locked.is_empty());
        locked
    }

    fn pin(manifest: &mut Manifest, capability: Capability, version: Option<&str>) {
        manifest
            .capabilities
            .get_mut(capability.as_str())
            .unwrap()
            .version = version.map(str::to_string);
    }

    #[test]
    fn any_locked_pin_off_the_default_is_stale() {
        for capability in locked_capabilities() {
            let name = capability.as_str();
            let default = registry::default_entry(capability)
                .version
                .unwrap()
                .version
                .to_string();
            let mut manifest = Manifest::default_for("0.1.0", &[]);
            assert_eq!(pin_mismatch(&manifest, capability), None);
            assert!(behind_pins(&manifest).is_empty());

            pin(&mut manifest, capability, Some("1.0.0"));
            assert_eq!(
                pin_mismatch(&manifest, capability),
                Some(("1.0.0".to_string(), default.clone()))
            );
            assert_eq!(
                behind_pins(&manifest),
                vec![format!(
                    "{name}: pinned 1.0.0, registry has {default} — run `superdev update`"
                )]
            );

            // A newer pin is not "behind", but superdev still cannot install it.
            pin(&mut manifest, capability, Some("9.9.9"));
            assert!(pin_mismatch(&manifest, capability).is_some());
            assert_eq!(locked_pin_mismatch(&manifest).unwrap().0, capability);
            pin(&mut manifest, capability, None);
            assert!(behind_pins(&manifest)[0].contains("pinned (unset)"));
        }
    }

    #[test]
    fn plannable_resets_every_locked_pin() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        for capability in locked_capabilities() {
            pin(&mut manifest, capability, Some("1.0.0"));
        }
        let plannable = plannable(&manifest);
        assert!(locked_pin_mismatch(&plannable).is_none());
        // Pins with no provenance beside them are left exactly as written.
        assert_eq!(
            plannable.capabilities["knowledge"].version,
            manifest.capabilities["knowledge"].version
        );
    }

    #[test]
    fn status_mode_plans_the_default_and_reports_behind() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for(crate::version(), &[]);
        pin(&mut manifest, Capability::Skills, Some("0.0.1"));
        let fake = FakeRunner::new();
        // Sync refuses the stale pin outright.
        let err = match plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Sync,
        ) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("a stale locked pin must refuse to plan"),
        };
        assert!(err.contains("only supports"), "{err}");
        // Status plans the version this binary can provide, and the behind
        // lines describe the manifest as written — not the plannable copy.
        let plan = plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Status,
        )
        .unwrap();
        assert!(plan.has_actions());
        assert_eq!(plan.behind_lines().len(), 1);
        assert!(plan.behind_lines()[0].starts_with("skills: pinned 0.0.1"));
    }

    /// Every planned action description, flattened for substring asserts.
    fn plan_descs(plan: &RepoPlan) -> Vec<String> {
        plan.planned()
            .iter()
            .flat_map(|p| p.actions.iter().map(|a| a.describe()))
            .collect()
    }

    #[test]
    fn workflows_provider_selects_materialise_or_plugin_flow() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let fake = FakeRunner::new();

        // The default provider copies files in; nothing is installed at user
        // level, so no plugin action may appear.
        let manifest = Manifest::default_for(crate::version(), &[]);
        let plan = plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Sync,
        )
        .unwrap();
        let descs = plan_descs(&plan);
        assert!(
            descs
                .iter()
                .any(|d| d.contains("materialise http:mattpocock-skills")),
            "{descs:?}"
        );
        assert!(
            descs
                .iter()
                .any(|d| d.contains("pin http:mattpocock-skills"))
        );
        assert!(
            !descs
                .iter()
                .any(|d| d.contains("plugin install superpowers")),
            "{descs:?}"
        );

        // The secondary provider installs its plugin and materialises nothing.
        let mut manifest = Manifest::default_for(crate::version(), &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "superpowers".into();
        workflows.version = Some("6.2.0".into());
        let plan = plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Sync,
        )
        .unwrap();
        let descs = plan_descs(&plan);
        assert!(descs.iter().any(|d| d.contains("pin http:superpowers")));
        assert!(
            descs
                .iter()
                .any(|d| d.contains("plugin install superpowers@superpowers-dev")),
            "{descs:?}"
        );
        assert!(
            !descs.iter().any(|d| d.contains("materialise")),
            "{descs:?}"
        );
    }

    #[test]
    fn a_cross_capability_claim_collision_refuses_with_the_way_out() {
        let a = (
            Capability::Skills,
            vec![Claim::File(".claude/skills/grilling/SKILL.md".into())],
        );
        let b = (
            Capability::Workflows,
            vec![Claim::File(".claude/skills/grilling/SKILL.md".into())],
        );
        let err = claim_collision(&[a, b]).unwrap_err().to_string();
        assert!(
            err.contains("skills and workflows both claim .claude/skills/grilling/SKILL.md"),
            "{err}"
        );
        assert!(err.contains("custom list"), "{err}");

        // The same capability claiming a key twice is not a collision, and
        // distinct keys never are.
        let dup = (
            Capability::Skills,
            vec![Claim::File("a.txt".into()), Claim::File("a.txt".into())],
        );
        let other = (Capability::Workflows, vec![Claim::File("b.txt".into())]);
        assert!(claim_collision(&[dup, other]).is_ok());
    }

    #[test]
    fn custom_skills_are_pruned_from_the_lock_and_reported() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("skills").unwrap().custom =
            vec!["humanise".into(), "grill-me".into()];
        let mut lock = Lock::default();
        lock.files
            .insert(".claude/skills/humanise/SKILL.md".into(), "hash-a".into());
        lock.files.insert(
            ".claude/skills/double-check/SKILL.md".into(),
            "hash-b".into(),
        );
        assert!(prune_custom(&manifest, &mut lock));
        assert!(!lock.files.contains_key(".claude/skills/humanise/SKILL.md"));
        assert!(
            lock.files
                .contains_key(".claude/skills/double-check/SKILL.md")
        );
        // Nothing left to prune: reports no change.
        assert!(!prune_custom(&manifest, &mut lock));

        assert_eq!(
            custom_lines(&manifest),
            vec![
                "skills: humanise custom, unmanaged".to_string(),
                "skills: custom names unknown skill 'grill-me' — no effect".to_string(),
            ]
        );
        let no_skills = Manifest::default_for("0.1.0", &[Capability::Skills]);
        assert!(custom_lines(&no_skills).is_empty());
    }

    #[test]
    fn workflows_custom_entries_prune_files_and_owners() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.custom = vec!["tdd".into(), "humanise".into()];
        let mut lock = Lock::default();
        // A skill-pack file: same directory, no workflows attribution.
        lock.files
            .insert(".claude/skills/humanise/SKILL.md".into(), "h".into());
        for key in [
            ".claude/skills/tdd/SKILL.md",
            ".claude/skills/tdd/refs/A.md",
        ] {
            lock.files.insert(key.into(), "h".into());
            lock.owners.insert(key.into(), "workflows".into());
        }
        lock.files
            .insert(".claude/skills/wizard/SKILL.md".into(), "h".into());
        lock.owners
            .insert(".claude/skills/wizard/SKILL.md".into(), "workflows".into());
        assert!(prune_custom(&manifest, &mut lock));
        assert!(!lock.files.keys().any(|k| k.contains("/tdd/")));
        assert!(!lock.owners.keys().any(|k| k.contains("/tdd/")));
        assert!(lock.files.contains_key(".claude/skills/wizard/SKILL.md"));
        // The pack owns humanise, so the workflows custom list cannot release it.
        assert!(lock.files.contains_key(".claude/skills/humanise/SKILL.md"));
    }

    #[test]
    fn custom_lines_and_switch_lines_report() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        workflows.custom = vec!["tdd".into(), "flying".into()];
        manifest.capabilities.get_mut("skills").unwrap().custom =
            vec!["humanise".into(), "grill-me".into()];
        let lines = custom_lines(&manifest);
        assert!(lines.contains(&"workflows: tdd custom, unmanaged".to_string()));
        assert!(
            lines.contains(
                &"workflows: custom names unknown skill 'flying' — no effect".to_string()
            )
        );
        assert!(lines.contains(&"skills: humanise custom, unmanaged".to_string()));
        assert!(
            lines
                .contains(&"skills: custom names unknown skill 'grill-me' — no effect".to_string())
        );

        let mut lock = Lock::default();
        lock.components.insert(
            "workflows".into(),
            LockedComponent {
                provider: "superpowers".into(),
                version: Some("6.2.0".into()),
            },
        );
        let lines = switch_lines(&manifest, &lock);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("claude plugin uninstall superpowers")),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("update the .agents import")),
            "{lines:?}"
        );
        // No switch, no lines.
        assert!(switch_lines(&manifest, &Lock::default()).is_empty());
        let mut same = Lock::default();
        same.components.insert(
            "workflows".into(),
            LockedComponent {
                provider: "mattpocock-skills".into(),
                version: Some("1.2.3".into()),
            },
        );
        assert!(switch_lines(&manifest, &same).is_empty());
    }

    #[test]
    fn plan_repo_puts_the_orphan_entry_last_and_reports_released() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let manifest = Manifest::default_for(crate::version(), &[]);
        let mut lock = Lock::default();
        // An unmodified leftover and a user-edited one, under no live claim.
        std::fs::write(dir.path().join("stale.txt"), "superdev's").unwrap();
        lock.files
            .insert("stale.txt".into(), sha256_hex(b"superdev's"));
        std::fs::write(dir.path().join("theirs.txt"), "edited").unwrap();
        lock.files
            .insert("theirs.txt".into(), sha256_hex(b"superdev's"));
        let fake = FakeRunner::new();
        let plan = plan_repo(dir.path(), &fake, &manifest, &lock, PlanMode::Sync).unwrap();
        let last = plan.planned().last().unwrap();
        assert!(last.capability.is_none());
        assert!(
            last.actions
                .iter()
                .any(|a| a.describe().contains("remove stale.txt")),
            "{:?}",
            last.actions
        );
        assert_eq!(plan.released_lines().len(), 1);
        assert!(plan.released_lines()[0].contains("theirs.txt"));
    }

    #[test]
    fn the_blueprint_line_reports_only_a_difference() {
        let mut manifest = Manifest::default_for(crate::version(), &[]);
        assert_eq!(blueprint_line(&manifest), None);
        manifest.blueprint = "0.0.1".into();
        assert_eq!(
            blueprint_line(&manifest),
            Some(format!(
                "blueprint 0.0.1, binary {} — sync will update it",
                crate::version()
            ))
        );
    }

    #[test]
    fn stamping_rewrites_only_a_stale_blueprint() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.0.1", &[]);
        manifest.save(dir.path()).unwrap();
        stamp_blueprint(dir.path(), &manifest).unwrap();
        let stamped = Manifest::load(dir.path()).unwrap();
        assert_eq!(stamped.blueprint, crate::version());
        // Already current: the file is left untouched. Marked with a comment
        // `save` would drop, since mtime here cannot resolve two writes a
        // microsecond apart.
        let path = dir.path().join(crate::manifest::CONFIG_PATH);
        let before = format!("{}# untouched\n", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, &before).unwrap();
        stamp_blueprint(dir.path(), &stamped).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after, "no rewrite when the value is current");
    }
}
