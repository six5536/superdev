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
use crate::components::{aokf, skillpack};
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

    /// Whether any entry carries drift: a managed file, pin or entry that no
    /// longer matches the blueprint. `Run` actions are excluded because they
    /// provision external state — a code index, an installed tool — that no
    /// checkout carries and the lock never hashes. A run a real change
    /// triggers is planned beside the write that triggered it, so dropping
    /// runs here hides no drift.
    pub fn has_drift(&self) -> bool {
        self.planned
            .iter()
            .any(|p| p.actions.iter().any(|a| !matches!(a, Action::Run { .. })))
    }

    /// One line per enabled capability pinned away from this binary's registry.
    pub fn behind_lines(&self) -> &[String] {
        &self.behind
    }

    /// One line per skill or workflow skill released to the user.
    pub fn custom_lines(&self) -> &[String] {
        &self.custom
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
    let claims_by_component: Vec<(Capability, String, Vec<Claim>)> = components
        .iter()
        .map(|c| (c.capability(), c.provider().to_string(), c.owned(&ctx)))
        .collect();
    claim_collision(&claims_by_component)?;
    let mut claims: Vec<Claim> = claims_by_component
        .into_iter()
        .flat_map(|(_, _, claims)| claims)
        .collect();
    // The aggregator is repo-level: no component claims it, and without a
    // live claim its lock entry would read as an orphan every run.
    claims.push(Claim::File(AGGREGATOR_PATH.into()));
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
    }
    // Nothing writes or reads `owners` any more: clear whatever attribution a
    // pre-removal binary left behind, in one stroke, rather than waiting for
    // each file's rewrite — an entry on an up-to-date file would never see one.
    if !lock.owners.is_empty() {
        lock.owners.clear();
        lock_changed = true;
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
    // A pack removed from a many slot goes the same way as a disabled
    // capability: its record leaves with its files.
    for (name, records) in lock.components.iter_mut() {
        let Some(capability) = Capability::parse(name) else {
            continue;
        };
        let before = records.len();
        records.retain(|r| manifest.config_of(capability, &r.provider).is_some());
        lock_changed |= records.len() != before;
    }
    lock.components.retain(|_, records| !records.is_empty());
    if planned.iter().all(|p| p.actions.is_empty()) {
        if lock_changed {
            lock.save(root)?;
        }
        stamp_blueprint(root, manifest)?;
        return Ok(ApplyOutcome {
            report: String::new(),
            ok: true,
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
    })
}

/// Mark as custom, at init time, everything the repo already carries under a
/// name superdev would manage. Returns the lines to print.
pub fn adopt_existing(root: &Path, manifest: &mut Manifest) -> Vec<String> {
    let mut lines = skillpack::adopt_existing(root, manifest);
    lines.extend(aokf::adopt_existing(root, manifest));
    lines
}

/// The pin for the provider the manifest names, falling back to the default
/// entry when the capability is not enabled or names a provider the registry
/// lacks. None means the version floats. Callers pass single slots only, so
/// the first entry is the whole set; a many slot has no one selected pin.
pub fn selected_pin(manifest: &Manifest, capability: Capability) -> Option<Pinned> {
    manifest
        .configs(capability)
        .first()
        .and_then(|c| registry::entry_for(capability, &c.provider))
        .unwrap_or_else(|| registry::default_entry(capability))
        .version
}

/// The registry version for `provider` in `capability`, when the registry
/// carries that pair.
pub fn registry_version_of(capability: Capability, provider: &str) -> Option<String> {
    registry::entry_for(capability, provider)?
        .version
        .map(|p| p.version.to_string())
}

/// Refuse when two enabled components claim the same lock key — across
/// capabilities or between two packs in one many slot. Deliberate overrides
/// are intra-component, so a cross-component collision is always an
/// accident — silently picking a winner would oscillate across syncs. The
/// message carries the way out; providers are named only when the
/// capability alone cannot tell the two sides apart.
fn claim_collision(claims_by_component: &[(Capability, String, Vec<Claim>)]) -> Result<()> {
    let mut seen: std::collections::BTreeMap<String, (Capability, String)> =
        std::collections::BTreeMap::new();
    for (capability, provider, claims) in claims_by_component {
        for claim in claims {
            let key = claim.lock_key();
            if let Some((first_cap, first_provider)) = seen.get(&key)
                && !(first_cap == capability && first_provider == provider)
            {
                let (first, second) = if first_cap == capability {
                    (
                        format!("{} ({first_provider})", first_cap.as_str()),
                        format!("{} ({provider})", capability.as_str()),
                    )
                } else {
                    (
                        first_cap.as_str().to_string(),
                        capability.as_str().to_string(),
                    )
                };
                return Err(Error::Manifest {
                    message: format!(
                        "{first} and {second} both claim {key} — add its skill to one side's custom list, or upgrade superdev",
                    ),
                });
            }
            seen.insert(key, (*capability, provider.clone()));
        }
    }
    Ok(())
}

/// Where superdev's agent instructions live: the aggregator AGENTS.md
/// imports, owned and rewritten as the enabled set changes.
const AGGREGATOR_PATH: &str = ".agents/superdev.md";
/// The one line superdev keeps in the user's AGENTS.md.
const AGENTS_IMPORT_LINE: &str = "@.agents/superdev.md";
/// Reported once, when the line is appended to an AGENTS.md that already
/// existed — the repos migrating off the old superdev-written scaffold.
const AGENTS_TRIM_HINT: &str = "AGENTS.md is yours — superdev's guidance moved behind @.agents/superdev.md; \
     trim any old superdev-written sections";

/// The general agent rules every managed repo gets, write-once scaffolds
/// beside the aggregator: (path, content).
const RULE_SCAFFOLDS: [(&str, &str); 2] = [
    (
        ".agents/coding.md",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/agents/coding.md"
        )),
    ),
    (
        ".agents/prose.md",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/agents/prose.md"
        )),
    ),
];

/// The fenced aggregator: the general rules, then one import per enabled
/// capability that ships an instruction file. Imports are sibling-relative,
/// so they resolve from the aggregator's own directory.
fn aggregator_content(manifest: &Manifest) -> String {
    let mut out = String::from(
        "<superdev-system>\n\
         This repository is managed by superdev.\n\
         superdev is a collection of capabilities, described in the following files:\n\
         @coding.md\n\
         @prose.md\n",
    );
    if manifest.enabled(Capability::Knowledge) {
        out.push_str("@aokf.md\n");
    }
    if manifest.enabled(Capability::CodeIndex) {
        out.push_str("@codegraph.md\n");
    }
    if manifest.enabled(Capability::BashOutputFilter) {
        out.push_str("@rtk.md\n");
    }
    out.push_str("</superdev-system>\n");
    out
}

/// One file's content, absent as an empty string; other errors propagate.
fn read_or_empty(path: std::path::PathBuf) -> Result<String> {
    match std::fs::read_to_string(&path) {
        Ok(existing) => Ok(existing),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(Error::Io { path, source }),
    }
}

/// The repo-level entry no capability owns: the `.gitignore` lines, the
/// ensured AGENTS.md import, and the instructions aggregator it points at.
fn repo_entry(root: &Path, manifest: &Manifest) -> Result<Option<Planned>> {
    let gitignore = read_or_empty(root.join(".gitignore"))?;
    let mut wanted = vec![(".superdev/cache/".to_string(), "ignore machine state")];
    if manifest.enabled(Capability::CodeIndex) {
        wanted.push((format!("{CODEGRAPH_INDEX_DIR}/"), "ignore the code index"));
    }
    let mut actions: Vec<Action> = wanted
        .into_iter()
        .filter(|(line, _)| !crate::fsutil::has_line(&gitignore, line))
        .map(|(line, reason)| Action::EnsureLine {
            path: ".gitignore".into(),
            line,
            reason: reason.to_string(),
            append_note: None,
        })
        .collect();
    if !crate::fsutil::has_line(&read_or_empty(root.join("AGENTS.md"))?, AGENTS_IMPORT_LINE) {
        actions.push(Action::EnsureLine {
            path: "AGENTS.md".into(),
            line: AGENTS_IMPORT_LINE.into(),
            reason: "make agents read superdev's instructions".into(),
            append_note: Some(AGENTS_TRIM_HINT.into()),
        });
    }
    let aggregator = aggregator_content(manifest);
    if read_or_empty(root.join(AGGREGATOR_PATH))? != aggregator {
        actions.push(Action::WriteFile {
            path: AGGREGATOR_PATH.into(),
            content: aggregator,
            ownership: crate::action::Ownership::Owned,
            reason: "superdev's agent instructions".into(),
        });
    }
    for (path, content) in RULE_SCAFFOLDS {
        // Write-once: the rules are the user's to adapt from the moment they
        // exist, so only an absent file is planned.
        if !root.join(path).is_file() {
            actions.push(Action::WriteFile {
                path: path.into(),
                content: content.into(),
                ownership: crate::action::Ownership::Scaffold,
                reason: "general agent rules".into(),
            });
        }
    }
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
    // Name-guarded per capability: two capabilities ship into
    // `.claude/skills/`, so an unknown name in one's list must not release
    // the other's file.
    for (capability, provider, shipped) in [
        (
            Capability::Skills,
            "superdev-skills",
            skillpack::SKILLS
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>(),
        ),
        (Capability::Knowledge, "aokf", aokf::skill_names().collect()),
    ] {
        let Some(config) = manifest.config_of(capability, provider) else {
            continue;
        };
        for name in &config.custom {
            if !shipped.contains(&name.as_str()) {
                continue;
            }
            // Release the whole skill directory: a knowledge skill is its
            // directory, and the pack's directories hold only SKILL.md.
            let prefix = format!(".claude/skills/{name}/");
            let keys: Vec<String> = lock
                .files
                .keys()
                .filter(|key| key.starts_with(&prefix))
                .cloned()
                .collect();
            for key in keys {
                pruned |= lock.files.remove(&key).is_some();
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
    for (capability, provider, shipped) in [
        (
            Capability::Skills,
            "superdev-skills",
            skillpack::SKILLS
                .iter()
                .map(|(name, _)| *name)
                .collect::<Vec<_>>(),
        ),
        (Capability::Knowledge, "aokf", aokf::skill_names().collect()),
    ] {
        let Some(config) = manifest.config_of(capability, provider) else {
            continue;
        };
        let cap = capability.as_str();
        for name in &config.custom {
            lines.push(if shipped.contains(&name.as_str()) {
                format!("{cap}: {name} custom, unmanaged")
            } else {
                format!("{cap}: custom names unknown skill '{name}' — no effect")
            });
        }
    }
    lines
}

/// One line per enabled entry pinned away from this binary's registry. The
/// provider is named only when the slot holds more than one entry — a
/// single-entry line reads as before.
fn behind_pins(manifest: &Manifest) -> Vec<String> {
    let mut lines = Vec::new();
    for capability in Capability::ALL {
        let many = manifest.configs(capability).len() > 1;
        for (provider, pinned, default) in pin_mismatches(manifest, capability) {
            let label = if many {
                format!("{} ({provider})", capability.as_str())
            } else {
                capability.as_str().to_string()
            };
            lines.push(format!(
                "{label}: pinned {pinned}, registry has {default} — run `superdev update`"
            ));
        }
    }
    lines
}

/// Per entry: the provider, its pin and this binary's default, for every
/// registry-locked entry pinned off that default. Every registry-pinned
/// version is locked to the default, so stale means mismatched — there is no
/// is-it-older question to ask; only the default has provenance, so any
/// other pin, newer included, is one superdev cannot install.
fn pin_mismatches(manifest: &Manifest, capability: Capability) -> Vec<(String, String, String)> {
    manifest
        .configs(capability)
        .iter()
        .filter_map(|config| {
            let default = registry_version_of(capability, &config.provider)?;
            let pinned = config.version.clone();
            (pinned.as_deref() != Some(default.as_str())).then(|| {
                (
                    config.provider.clone(),
                    pinned.unwrap_or_else(|| "(unset)".into()),
                    default,
                )
            })
        })
        .collect()
}

/// The first registry-locked entry pinned off this binary's default.
fn locked_pin_mismatch(manifest: &Manifest) -> Option<(Capability, String, String)> {
    Capability::ALL.into_iter().find_map(|capability| {
        pin_mismatches(manifest, capability)
            .into_iter()
            .next()
            .map(|(_, pinned, default)| (capability, pinned, default))
    })
}

/// A copy of the manifest that can be planned: every registry-locked
/// capability back at the default. Unpinned capabilities are left alone —
/// components accept those as given.
fn plannable(manifest: &Manifest) -> Manifest {
    let mut plannable = manifest.clone();
    for capability in Capability::ALL {
        for config in plannable.configs_mut(capability) {
            // No entry means an unknown provider; leave the pin and let the
            // resolution error say so.
            if let Some(version) = registry_version_of(capability, &config.provider) {
                config.version = Some(version);
            }
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
    use crate::lock::sha256_hex;
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
        manifest.configs_mut(capability)[0].version = version.map(str::to_string);
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
            assert!(pin_mismatches(&manifest, capability).is_empty());
            assert!(behind_pins(&manifest).is_empty());

            pin(&mut manifest, capability, Some("1.0.0"));
            let provider = manifest.configs(capability)[0].provider.clone();
            assert_eq!(
                pin_mismatches(&manifest, capability),
                vec![(provider, "1.0.0".to_string(), default.clone())]
            );
            assert_eq!(
                behind_pins(&manifest),
                vec![format!(
                    "{name}: pinned 1.0.0, registry has {default} — run `superdev update`"
                )]
            );

            // A newer pin is not "behind", but superdev still cannot install it.
            pin(&mut manifest, capability, Some("9.9.9"));
            assert!(!pin_mismatches(&manifest, capability).is_empty());
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
            plannable.capabilities["knowledge"][0].version,
            manifest.capabilities["knowledge"][0].version
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

    /// The `--drift` gate's contract: a run provisions external state no
    /// checkout carries, so it is work to do without being drift. Every run
    /// a real change triggers is planned beside that change, which is what
    /// makes dropping runs from the exit code safe.
    #[test]
    fn a_provisioning_run_is_work_to_do_but_not_drift() {
        // Every capability disabled, and the repo entry's own files already
        // in place: a settled tree, so the plan starts empty and the asserts
        // below speak only about what the test prepends.
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for(crate::version(), &Capability::ALL);
        std::fs::create_dir_all(dir.path().join(".agents")).unwrap();
        std::fs::write(
            dir.path().join(".agents/superdev.md"),
            aggregator_content(&manifest),
        )
        .unwrap();
        for scaffold in [".agents/coding.md", ".agents/prose.md"] {
            std::fs::write(dir.path().join(scaffold), "the user's now\n").unwrap();
        }
        std::fs::write(dir.path().join(".gitignore"), ".superdev/cache/\n").unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "@.agents/superdev.md\n").unwrap();
        let fake = FakeRunner::new();
        let mut plan = plan_repo(
            dir.path(),
            &fake,
            &manifest,
            &Lock::default(),
            PlanMode::Status,
        )
        .unwrap();
        assert!(!plan.has_actions(), "descs: {:?}", plan_descs(&plan));

        plan.prepend(Planned {
            capability: Some(Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::Run {
                program: "mise".into(),
                args: vec!["exec".into()],
                purpose: "build the code index".into(),
                undo: None,
                optional: false,
            }],
        });
        assert!(plan.has_actions(), "a run is still work to do");
        assert!(!plan.has_drift(), "a run alone is not drift");

        plan.prepend(Planned {
            capability: Some(Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::WriteFile {
                path: ".agents/codegraph.md".into(),
                content: "x".into(),
                ownership: crate::action::Ownership::Owned,
                reason: "code-index instructions".into(),
            }],
        });
        assert!(plan.has_drift(), "a managed file is drift");
    }

    /// Every planned action description, flattened for substring asserts.
    fn plan_descs(plan: &RepoPlan) -> Vec<String> {
        plan.planned()
            .iter()
            .flat_map(|p| p.actions.iter().map(|a| a.describe()))
            .collect()
    }

    #[test]
    fn a_pack_dropped_from_the_manifest_loses_its_lock_record() {
        use crate::lock::LockedComponent;

        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        // The manifest keeps one pack; the lock still records two.
        let manifest = Manifest::default_for(crate::version(), &[]);
        let mut lock = Lock::default();
        lock.components.insert(
            "skills".into(),
            vec![
                LockedComponent {
                    provider: "superdev-skills".into(),
                    version: Some(crate::version().to_string()),
                },
                LockedComponent {
                    provider: "another-pack".into(),
                    version: Some("1.2.0".into()),
                },
            ],
        );
        let plan = RepoPlan {
            planned: Vec::new(),
            orphans: OrphanPlan::default(),
            behind: Vec::new(),
            custom: Vec::new(),
            blueprint: None,
            lock,
            lock_changed: false,
        };
        assert!(apply_repo(dir.path(), &fake, &manifest, plan).unwrap().ok);
        let saved = Lock::load(dir.path()).unwrap();
        let providers: Vec<&str> = saved.components["skills"]
            .iter()
            .map(|r| r.provider.as_str())
            .collect();
        // The dropped pack's record went; the kept pack's stayed. Its files
        // go the generic way: claims no longer cover them, so the orphan
        // pass classifies them like any other orphan.
        assert_eq!(providers, ["superdev-skills"]);
    }

    #[test]
    fn aggregator_imports_track_the_enabled_set() {
        let all = aggregator_content(&Manifest::default_for("0.1.0", &[]));
        assert!(all.starts_with("<superdev-system>\n"), "{all}");
        assert!(all.ends_with("</superdev-system>\n"), "{all}");
        assert!(all.contains("@aokf.md"), "{all}");
        assert!(all.contains("@codegraph.md"), "{all}");
        assert!(all.contains("@rtk.md"), "{all}");
        let partial = aggregator_content(&Manifest::default_for(
            "0.1.0",
            &[Capability::Knowledge, Capability::BashOutputFilter],
        ));
        assert!(!partial.contains("@aokf.md"), "{partial}");
        assert!(!partial.contains("@rtk.md"), "{partial}");
        assert!(partial.contains("@codegraph.md"), "{partial}");
        // The general rules are not capability-gated: every managed repo
        // imports them, even with every instruction-shipping capability off.
        let none = aggregator_content(&Manifest::default_for(
            "0.1.0",
            &[Capability::Knowledge, Capability::CodeIndex],
        ));
        assert!(none.contains("@coding.md"), "{none}");
        assert!(none.contains("@prose.md"), "{none}");
        assert!(
            !none.contains("@aokf.md") && !none.contains("@codegraph.md"),
            "{none}"
        );
    }

    #[test]
    fn repo_entry_plans_the_import_line_and_the_aggregator_once() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let entry = repo_entry(dir.path(), &manifest).unwrap().unwrap();
        let descs: Vec<String> = entry.actions.iter().map(Action::describe).collect();
        assert!(
            descs
                .iter()
                .any(|d| d.contains("ensure AGENTS.md contains `@.agents/superdev.md`")),
            "{descs:?}"
        );
        assert!(
            descs
                .iter()
                .any(|d| d.contains("write .agents/superdev.md")),
            "{descs:?}"
        );
        for (path, _) in RULE_SCAFFOLDS {
            assert!(
                descs.iter().any(|d| d.contains(path)),
                "{path} missing from {descs:?}"
            );
        }
        // A settled repo replans nothing — and the rule scaffolds count as
        // settled whatever their content, because they are the user's.
        std::fs::write(
            dir.path().join(".gitignore"),
            ".superdev/cache/\n.codegraph/\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("AGENTS.md"),
            "# Mine\n@.agents/superdev.md\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join(".agents")).unwrap();
        std::fs::write(
            dir.path().join(AGGREGATOR_PATH),
            aggregator_content(&manifest),
        )
        .unwrap();
        for (path, _) in RULE_SCAFFOLDS {
            std::fs::write(dir.path().join(path), "adapted by the user\n").unwrap();
        }
        assert!(repo_entry(dir.path(), &manifest).unwrap().is_none());
    }

    /// The knowledge capability plans every aokf-carried skill file, so a
    /// `[knowledge] custom` name can release any of them.
    #[test]
    fn knowledge_plans_the_full_carried_skill_set() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let fake = FakeRunner::new();
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
        for name in aokf::skill_names() {
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!(".claude/skills/{name}/SKILL.md"))),
                "{name} missing from the plan"
            );
        }
    }

    #[test]
    fn a_cross_capability_claim_collision_refuses_with_the_way_out() {
        let a = (
            Capability::Skills,
            "superdev-skills".to_string(),
            vec![Claim::File(".claude/skills/grilling/SKILL.md".into())],
        );
        let b = (
            Capability::Knowledge,
            "aokf".to_string(),
            vec![Claim::File(".claude/skills/grilling/SKILL.md".into())],
        );
        let err = claim_collision(&[a, b]).unwrap_err().to_string();
        assert!(
            err.contains("skills and knowledge both claim .claude/skills/grilling/SKILL.md"),
            "{err}"
        );
        assert!(err.contains("custom list"), "{err}");

        // The same component claiming a key twice is not a collision, and
        // distinct keys never are.
        let dup = (
            Capability::Skills,
            "superdev-skills".to_string(),
            vec![Claim::File("a.txt".into()), Claim::File("a.txt".into())],
        );
        let other = (
            Capability::Knowledge,
            "aokf".to_string(),
            vec![Claim::File("b.txt".into())],
        );
        assert!(claim_collision(&[dup, other]).is_ok());
    }

    #[test]
    fn two_packs_in_one_slot_colliding_name_both_providers() {
        let a = (
            Capability::Skills,
            "superdev-skills".to_string(),
            vec![Claim::File(".claude/skills/humanise/SKILL.md".into())],
        );
        let b = (
            Capability::Skills,
            "another-pack".to_string(),
            vec![Claim::File(".claude/skills/humanise/SKILL.md".into())],
        );
        let err = claim_collision(&[a, b]).unwrap_err().to_string();
        assert!(
            err.contains(
                "skills (superdev-skills) and skills (another-pack) both claim \
                 .claude/skills/humanise/SKILL.md"
            ),
            "{err}"
        );
        assert!(err.contains("custom list"), "{err}");
    }

    #[test]
    fn custom_skills_are_pruned_from_the_lock_and_reported() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("skills").unwrap()[0].custom =
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
    fn knowledge_custom_entries_release_the_whole_directory() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("knowledge").unwrap()[0].custom = vec!["tdd".into()];
        let mut lock = Lock::default();
        // A skill-pack file, untouched by the knowledge custom list.
        lock.files
            .insert(".claude/skills/humanise/SKILL.md".into(), "h".into());
        for key in [
            ".claude/skills/tdd/SKILL.md",
            ".claude/skills/tdd/refs/A.md",
        ] {
            lock.files.insert(key.into(), "h".into());
        }
        lock.files
            .insert(".claude/skills/wizard/SKILL.md".into(), "h".into());
        assert!(prune_custom(&manifest, &mut lock));
        assert!(!lock.files.keys().any(|k| k.contains("/tdd/")));
        assert!(lock.files.contains_key(".claude/skills/wizard/SKILL.md"));
        assert!(lock.files.contains_key(".claude/skills/humanise/SKILL.md"));
        // Nothing left to prune: reports no change.
        assert!(!prune_custom(&manifest, &mut lock));
    }

    #[test]
    fn knowledge_custom_lines_cover_every_carried_skill() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("knowledge").unwrap()[0].custom =
            vec!["tdd".into(), "flying".into()];
        let lines = custom_lines(&manifest);
        assert!(lines.contains(&"knowledge: tdd custom, unmanaged".to_string()));
        assert!(
            lines.contains(
                &"knowledge: custom names unknown skill 'flying' — no effect".to_string()
            )
        );
        // Every carried skill is a known custom name.
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("knowledge").unwrap()[0].custom =
            aokf::skill_names().map(String::from).collect();
        for line in custom_lines(&manifest) {
            assert!(line.ends_with("custom, unmanaged"), "{line}");
        }
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
