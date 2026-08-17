//! manage.rs — the init/status/sync/update verbs: plan, print, apply.
//!
//! All the domain work happens in `superdev-core`; this module wires the
//! verbs to it, owns the printed output, and turns results into exit codes.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use superdev_core::action::Action;
use superdev_core::capability::Capability;
use superdev_core::component::{Claim, Ctx};
use superdev_core::components::codegraph::CODEGRAPH_INDEX_DIR;
use superdev_core::components::mattskills;
use superdev_core::components::pin;
use superdev_core::components::skillpack;
use superdev_core::engine::Planned;
use superdev_core::error::{Error, Result};
use superdev_core::lock::Lock;
use superdev_core::manifest::{CONFIG_PATH, Manifest};
use superdev_core::runner::{CommandRunner, SystemRunner};
use superdev_core::{components, engine, orphan, registry, report};

/// Provider name for repo-level actions no capability owns.
const REPO_PROVIDER: &str = "superdev";

/// Printed after a materialisation: the upstream setup skill is interactive,
/// so it is the one step superdev cannot run for the user.
const SETUP_HINT: &str =
    "workflows: run /setup-matt-pocock-skills in Claude Code to finish configuring";

/// The five capability-disable flags (kebab-case comes free from clap).
#[derive(clap::Args)]
pub struct InitArgs {
    /// Skip the prompting/workflow skills
    #[arg(long)]
    pub no_workflows: bool,
    /// Skip the frontend design workflows
    #[arg(long)]
    pub no_frontend: bool,
    /// Skip the superdev skill pack
    #[arg(long)]
    pub no_skills: bool,
    /// Skip the code index
    #[arg(long)]
    pub no_code_index: bool,
    /// Skip the knowledgebase scaffold
    #[arg(long)]
    pub no_knowledge: bool,
    /// Workflows provider (default: the registry default)
    #[arg(long, value_name = "ID", conflicts_with = "no_workflows")]
    pub workflows_provider: Option<String>,
}

impl InitArgs {
    fn disabled(&self) -> Vec<Capability> {
        let flags = [
            (self.no_workflows, Capability::Workflows),
            (self.no_frontend, Capability::Frontend),
            (self.no_skills, Capability::Skills),
            (self.no_code_index, Capability::CodeIndex),
            (self.no_knowledge, Capability::Knowledge),
        ];
        flags
            .into_iter()
            .filter_map(|(off, capability)| off.then_some(capability))
            .collect()
    }
}

/// Set the repo up: write the manifest, then apply the whole blueprint.
pub fn init(root: &Path, args: &InitArgs) -> Result<u8> {
    if !root.join(".git").exists() {
        return Err(Error::Manifest {
            message: "not a git repository — run `git init` first".into(),
        });
    }
    // The manifest, not the directory: the knowledge tools create
    // `.superdev/cache/` in repos that were never initialised.
    if root.join(CONFIG_PATH).is_file() {
        return Err(Error::Manifest {
            message: "already initialised — run `superdev sync`".into(),
        });
    }
    if let Some(id) = &args.workflows_provider
        && registry::entry_for(Capability::Workflows, id).is_none()
    {
        return Err(Error::Manifest {
            message: format!(
                "workflows provider must be one of: {}",
                registry::providers_for(Capability::Workflows).join(", ")
            ),
        });
    }
    let mut manifest = Manifest::default_for(superdev_core::version(), &args.disabled());
    if let (Some(id), Some(config)) = (
        &args.workflows_provider,
        manifest
            .capabilities
            .get_mut(Capability::Workflows.as_str()),
    ) {
        let entry = registry::entry_for(Capability::Workflows, id).expect("validated above");
        config.provider = entry.provider.to_string();
        config.version = entry.version.map(|p| p.version.to_string());
    }
    let mut adopted = adopt_existing_skills(root, &mut manifest);
    adopted.extend(adopt_existing_mattskills(root, &mut manifest));
    manifest.save(root)?;
    for line in &adopted {
        out(line)?;
    }
    let mut lock = Lock::default();
    let runner = SystemRunner;
    let (planned, _) = plan_all(root, &runner, &manifest, &lock)?;
    print_plan(&planned)?;
    let materialising = materialises(&planned);
    let outcome = apply_and_report(root, &runner, &manifest, &planned, &mut lock);
    if outcome.is_err() {
        // The manifest is written before the apply and deliberately kept: it is
        // what `sync` resumes from. Say so, rather than leave it unmentioned.
        // A failed print must not mask the failure that prompted it.
        let _ = out(&format!(
            "left in place: {CONFIG_PATH} — `superdev sync` can resume from it"
        ));
    } else if materialising {
        out(SETUP_HINT)?;
    }
    outcome
}

/// Report drift without changing anything. 0 in sync, 1 with work to do.
pub fn status(root: &Path) -> Result<u8> {
    let manifest = load_manifest(root)?;
    // Reporting a stale checksum-verified pin is half the job of `status`, and
    // those components refuse to plan one, so plan the version this binary can
    // provide and let the hint line carry the news.
    let behind = behind_pins(&manifest);
    let mut lock = Lock::load(root)?;
    // In memory only — status never writes. Unpruned, a skill or workflow
    // just marked custom would read as an orphan and plan its own deletion.
    prune_custom(&manifest, &mut lock);
    let runner = SystemRunner;
    let (planned, orphans) = plan_all(root, &runner, &plannable(&manifest), &lock)?;
    print_plan(&planned)?;
    for line in &behind {
        out(line)?;
    }
    for line in &custom_lines(&manifest) {
        out(line)?;
    }
    for line in &switch_lines(&manifest, &lock) {
        out(line)?;
    }
    for line in &orphans.released_lines() {
        out(line)?;
    }
    if let Some(line) = blueprint_line(&manifest) {
        out(&line)?;
    }
    Ok(u8::from(has_actions(&planned) || !behind.is_empty()))
}

/// Re-apply the blueprint so the repo matches the manifest.
pub fn sync(root: &Path, dry_run: bool) -> Result<u8> {
    let manifest = load_manifest(root)?;
    // Unlike `status`, sync would have to act on the pin. Substituting the
    // default silently is worse than stopping.
    if let Some((capability, pinned, default)) = locked_pin_mismatch(&manifest) {
        return Err(Error::Manifest {
            message: format!(
                "{} is pinned {pinned} but this superdev only supports {default} — run `superdev update`",
                capability.as_str()
            ),
        });
    }
    let mut lock = Lock::load(root)?;
    // Before planning: a skill or workflow just marked custom still has its
    // lock entry, and unpruned an unmodified one would read as an orphan and
    // be deleted — the opposite of what marking it custom asked for.
    let mut lock_changed = prune_custom(&manifest, &mut lock);
    let runner = SystemRunner;
    let (planned, orphans) = plan_all(root, &runner, &manifest, &lock)?;
    print_plan(&planned)?;
    let materialising = materialises(&planned);
    for line in &behind_pins(&manifest) {
        out(line)?;
    }
    for line in &orphans.released_lines() {
        out(line)?;
    }
    for line in &switch_lines(&manifest, &lock) {
        out(line)?;
    }
    if dry_run {
        return Ok(0);
    }
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
    if !has_actions(&planned) {
        if lock_changed {
            lock.save(root)?;
        }
        return stamp_blueprint(root, &manifest);
    }
    apply_and_report(root, &runner, &manifest, &planned, &mut lock)?;
    if materialising {
        out(SETUP_HINT)?;
    }
    stamp_blueprint(root, &manifest)
}

/// Move version pins to this binary's defaults (or to an explicit version),
/// then sync.
pub fn update(root: &Path, target: Option<&str>, provider: Option<&str>) -> Result<u8> {
    let mut manifest = load_manifest(root)?;
    match (target, provider) {
        (None, Some(_)) => {
            return Err(Error::Manifest {
                message: "--provider needs a capability target".into(),
            });
        }
        (Some(target), provider) => {
            let (capability, version) = parse_target(&manifest, target)?;
            if let Some(id) = provider
                && registry::entry_for(capability, id).is_none()
            {
                return Err(Error::Manifest {
                    message: format!(
                        "{} provider must be one of: {}",
                        capability.as_str(),
                        registry::providers_for(capability).join(", ")
                    ),
                });
            }
            let fallback = registry_version(&manifest, capability);
            let config = manifest
                .capabilities
                .get_mut(capability.as_str())
                .ok_or_else(|| Error::Manifest {
                    message: format!("`{}` is not enabled", capability.as_str()),
                })?;
            if let Some(id) = provider {
                let entry = registry::entry_for(capability, id).expect("validated above");
                config.provider = entry.provider.to_string();
                config.version = entry.version.map(|p| p.version.to_string());
            } else {
                config.version = version.or(fallback);
            }
        }
        (None, None) => {
            for capability in Capability::ALL {
                let version = registry_version(&manifest, capability);
                if let Some(config) = manifest.capabilities.get_mut(capability.as_str()) {
                    config.version = version;
                }
            }
        }
    }
    manifest.save(root)?;
    sync(root, false)
}

/// Split `<capability>[@<version>]`, rejecting what cannot be pinned by hand.
fn parse_target(manifest: &Manifest, target: &str) -> Result<(Capability, Option<String>)> {
    let (name, version) = match target.split_once('@') {
        Some((name, version)) => (name, Some(version.to_string())),
        None => (target, None),
    };
    let capability = Capability::parse(name).ok_or_else(|| Error::Manifest {
        message: format!("unknown capability `{name}`"),
    })?;
    if version.as_deref() == Some("") {
        return Err(Error::Manifest {
            message: format!("`{target}` names no version"),
        });
    }
    // A registry-pinned version is this binary's to decide: the pin carries
    // its provenance, so any other version cannot be verified. The components
    // refuse the same thing when planning.
    if version.is_some()
        && let Some(pinned) = selected_pin(manifest, capability)
    {
        return Err(Error::Manifest {
            message: pin::refusal_message(capability, pinned),
        });
    }
    Ok((capability, version))
}

/// The pin for the provider the manifest names, falling back to the default
/// entry when the capability is not enabled or names a provider the registry
/// lacks. None means the version floats.
fn selected_pin(manifest: &Manifest, capability: Capability) -> Option<registry::Pinned> {
    manifest
        .capabilities
        .get(capability.as_str())
        .and_then(|c| registry::entry_for(capability, &c.provider))
        .unwrap_or_else(|| registry::default_entry(capability))
        .version
}

/// The registry version for the provider the manifest names, when both exist.
fn registry_version(manifest: &Manifest, capability: Capability) -> Option<String> {
    let config = manifest.capabilities.get(capability.as_str())?;
    registry::entry_for(capability, &config.provider)?
        .version
        .map(|p| p.version.to_string())
}

/// The manifest, with the missing-file case named for what it means.
fn load_manifest(root: &Path) -> Result<Manifest> {
    if !root.join(CONFIG_PATH).is_file() {
        return Err(Error::Manifest {
            message: "not initialised — run `superdev init`".into(),
        });
    }
    Manifest::load(root)
}

/// Every component's plan, behind the repo-level entry, with the orphan pass
/// last.
fn plan_all(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    lock: &Lock,
) -> Result<(Vec<Planned>, orphan::OrphanPlan)> {
    let components = components::enabled(manifest)?;
    let ctx = Ctx {
        root,
        runner,
        manifest,
        lock,
    };
    let mut planned = Vec::new();
    planned.extend(repo_entry(root, manifest)?);
    planned.extend(engine::plan(&components, &ctx)?);
    let claims: Vec<Claim> = components.iter().flat_map(|c| c.owned(&ctx)).collect();
    let orphans = orphan::plan(root, lock, &claims)?;
    // Last, so removals run after every component write: a rename whose
    // write fails rolls back before anything is deleted.
    if !orphans.actions.is_empty() {
        planned.push(Planned {
            capability: None,
            provider: REPO_PROVIDER.into(),
            actions: orphans.actions.clone(),
        });
    }
    Ok((planned, orphans))
}

/// The ignore lines no capability owns: superdev's machine state, and the
/// code index when that capability is on. Neither belongs in git.
fn repo_entry(root: &Path, manifest: &Manifest) -> Result<Option<Planned>> {
    let path = root.join(".gitignore");
    let existing = match fs::read_to_string(&path) {
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
        // Same rule the engine applies: an exact whole-line match counts.
        .filter(|(line, _)| !existing.lines().any(|l| l == line))
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

/// Release, at adoption time, every pack skill the repo already has under its
/// own name and with its own content. Overwriting those would replace work
/// superdev never wrote with a backup the user has to go looking for; marking
/// them custom keeps the file and hands the choice back. Returns the lines to
/// print. Only `init` calls this — later syncs honour the list as written.
fn adopt_existing_skills(root: &Path, manifest: &mut Manifest) -> Vec<String> {
    let Some(config) = manifest.capabilities.get_mut(Capability::Skills.as_str()) else {
        return Vec::new();
    };
    for (name, shipped) in skillpack::SKILLS {
        let existing = fs::read_to_string(root.join(format!(".claude/skills/{name}/SKILL.md")));
        // Identical content is superdev's own text already: nothing to keep.
        if existing.is_ok_and(|existing| existing != shipped) {
            config.custom.push(name.to_string());
        }
    }
    config
        .custom
        .iter()
        .map(|name| format!("skills: kept your {name} — marked custom in {CONFIG_PATH}"))
        .collect()
}

/// Release, at init time, every mattpocock-skills upstream skill directory
/// the repo already has. The checkout does not exist yet, so unlike
/// `adopt_existing_skills` there is no content to compare — any existing
/// directory counts, and the report says what to do if the user wants it
/// managed instead. Only `init` calls this — later syncs honour the list as
/// written. A no-op off the mattpocock-skills provider.
fn adopt_existing_mattskills(root: &Path, manifest: &mut Manifest) -> Vec<String> {
    let Some(config) = manifest
        .capabilities
        .get_mut(Capability::Workflows.as_str())
    else {
        return Vec::new();
    };
    if config.provider != "mattpocock-skills" {
        return Vec::new();
    }
    for name in mattskills::MATTSKILLS_SKILLS {
        if root.join(format!(".claude/skills/{name}")).is_dir() {
            config.custom.push(name.to_string());
        }
    }
    config
        .custom
        .iter()
        .map(|name| format!("workflows: kept your {name} — marked custom in {CONFIG_PATH}"))
        .collect()
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

/// The blueprint-version report: informational, never the exit code. A
/// settled repo under a newer binary is not drift.
fn blueprint_line(manifest: &Manifest) -> Option<String> {
    (manifest.blueprint != superdev_core::version()).then(|| {
        format!(
            "blueprint {}, binary {} — sync will update it",
            manifest.blueprint,
            superdev_core::version()
        )
    })
}

/// Record this binary's version as the blueprint last applied. Rewrites
/// config.toml only when the value changes.
fn stamp_blueprint(root: &Path, manifest: &Manifest) -> Result<u8> {
    if manifest.blueprint != superdev_core::version() {
        let mut manifest = manifest.clone();
        manifest.blueprint = superdev_core::version().to_string();
        manifest.save(root)?;
    }
    Ok(0)
}

fn has_actions(planned: &[Planned]) -> bool {
    planned.iter().any(|p| !p.actions.is_empty())
}

/// Whether the plan copies an upstream skill set into the repo.
fn materialises(planned: &[Planned]) -> bool {
    planned.iter().any(|p| {
        p.actions
            .iter()
            .any(|a| matches!(a, Action::MaterialiseSkills { .. }))
    })
}

/// Apply, print the report, and keep the lock only when the run succeeded.
fn apply_and_report(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    planned: &[Planned],
    lock: &mut Lock,
) -> Result<u8> {
    let result = engine::apply(root, runner, manifest, planned, lock);
    // Save before printing: a reader that closes stdout (`sync | head`) ends the
    // run early, and the applied changes must not outlive their lock entries.
    if result.ok {
        lock.save(root)?;
    }
    print_block(&report::render_apply(&result))?;
    if !result.ok {
        return Err(Error::Manifest {
            message: "apply failed — see report above".into(),
        });
    }
    Ok(0)
}

fn print_plan(planned: &[Planned]) -> Result<()> {
    let rendered = report::render_plan(planned);
    if rendered.is_empty() {
        return out("nothing to do");
    }
    print_block(&rendered)
}

/// Print a rendered block, dropping the trailing newline [`out`] adds back.
fn print_block(rendered: &str) -> Result<()> {
    let rendered = rendered.trim_end_matches('\n');
    if rendered.is_empty() {
        return Ok(());
    }
    out(rendered)
}

/// The one stdout path, so `main` can keep BrokenPipe a success.
fn out(s: &str) -> Result<()> {
    writeln!(io::stdout(), "{s}").map_err(|source| Error::Io {
        path: "-".into(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Core's own fake is test-only inside that crate, so planning here needs
    /// its own seam: every command succeeds emptily.
    struct QuietRunner;

    impl CommandRunner for QuietRunner {
        fn run(
            &self,
            _program: &str,
            _args: &[String],
            _cwd: &Path,
        ) -> Result<superdev_core::runner::Output> {
            Ok(superdev_core::runner::Output {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
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
        let (_, shipped) = skillpack::SKILLS
            .iter()
            .find(|(name, _)| *name == "double-check")
            .unwrap();
        write("double-check", shipped);

        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let lines = adopt_existing_skills(dir.path(), &mut manifest);
        assert_eq!(manifest.capabilities["skills"].custom, ["humanise"]);
        assert_eq!(
            lines,
            vec![format!(
                "skills: kept your humanise — marked custom in {CONFIG_PATH}"
            )]
        );

        // Nothing to adopt in an empty repo, or with skills disabled.
        let empty = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        assert!(adopt_existing_skills(empty.path(), &mut manifest).is_empty());
        assert!(manifest.capabilities["skills"].custom.is_empty());
        let mut off = Manifest::default_for("0.1.0", &[Capability::Skills]);
        assert!(adopt_existing_skills(dir.path(), &mut off).is_empty());
    }

    #[test]
    fn init_args_carry_a_validated_workflows_provider() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let args = InitArgs {
            no_workflows: false,
            no_frontend: true,
            no_skills: true,
            no_code_index: true,
            no_knowledge: true,
            workflows_provider: Some("flying".into()),
        };
        let err = init(dir.path(), &args).unwrap_err().to_string();
        assert!(err.contains("workflows provider must be one of"), "{err}");
        assert!(
            !dir.path().join(CONFIG_PATH).exists(),
            "nothing written on error"
        );
    }

    #[test]
    fn update_provider_rules() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for(superdev_core::version(), &[]);
        manifest.save(dir.path()).unwrap();
        let err = update(dir.path(), None, Some("superpowers"))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("--provider needs a capability target"),
            "{err}"
        );
        let err = update(dir.path(), Some("workflows"), Some("flying"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("workflows provider must be one of"), "{err}");
    }

    #[test]
    fn parses_update_targets() {
        let manifest = Manifest::default_for("0.1.0", &[]);
        assert_eq!(
            parse_target(&manifest, "code-index").unwrap(),
            (Capability::CodeIndex, None)
        );
        assert!(parse_target(&manifest, "flying").is_err());
        // Every registry-pinned capability refuses a hand-picked version.
        let err = parse_target(&manifest, "workflows@9.9.9")
            .unwrap_err()
            .to_string();
        assert!(err.contains("run `superdev update workflows`"), "{err}");
        assert!(parse_target(&manifest, "code-index@9.9.9").is_err());
        assert!(parse_target(&manifest, "skills@9.9.9").is_err());
    }

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
        // Pins with no checksum beside them are left exactly as written.
        assert_eq!(
            plannable.capabilities["knowledge"].version,
            manifest.capabilities["knowledge"].version
        );
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
    fn adoption_marks_existing_upstream_skill_dirs_custom() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude/skills/tdd")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/tdd/SKILL.md"), "mine").unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "mattpocock-skills".into();
        let lines = adopt_existing_mattskills(dir.path(), &mut manifest);
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
        assert!(adopt_existing_mattskills(dir.path(), &mut superpowers).is_empty());
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
            superdev_core::lock::LockedComponent {
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
            superdev_core::lock::LockedComponent {
                provider: "mattpocock-skills".into(),
                version: Some("1.2.3".into()),
            },
        );
        assert!(switch_lines(&manifest, &same).is_empty());
    }

    #[test]
    fn plan_all_puts_the_orphan_entry_last_and_reports_released() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        let manifest = Manifest::default_for(superdev_core::version(), &[]);
        let mut lock = Lock::default();
        // An unmodified leftover and a user-edited one, under no live claim.
        std::fs::write(dir.path().join("stale.txt"), "superdev's").unwrap();
        lock.files.insert(
            "stale.txt".into(),
            superdev_core::lock::sha256_hex(b"superdev's"),
        );
        std::fs::write(dir.path().join("theirs.txt"), "edited").unwrap();
        lock.files.insert(
            "theirs.txt".into(),
            superdev_core::lock::sha256_hex(b"superdev's"),
        );
        let fake = QuietRunner;
        let (planned, orphans) = plan_all(dir.path(), &fake, &manifest, &lock).unwrap();
        let last = planned.last().unwrap();
        assert!(last.capability.is_none());
        assert!(
            last.actions
                .iter()
                .any(|a| a.describe().contains("remove stale.txt")),
            "{:?}",
            last.actions
        );
        assert_eq!(orphans.released, vec!["theirs.txt".to_string()]);
    }

    #[test]
    fn the_blueprint_line_reports_only_a_difference() {
        let mut manifest = Manifest::default_for(superdev_core::version(), &[]);
        assert_eq!(blueprint_line(&manifest), None);
        manifest.blueprint = "0.0.1".into();
        assert_eq!(
            blueprint_line(&manifest),
            Some(format!(
                "blueprint 0.0.1, binary {} — sync will update it",
                superdev_core::version()
            ))
        );
    }

    #[test]
    fn stamping_rewrites_only_a_stale_blueprint() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.0.1", &[]);
        manifest.save(dir.path()).unwrap();
        assert_eq!(stamp_blueprint(dir.path(), &manifest).unwrap(), 0);
        let stamped = Manifest::load(dir.path()).unwrap();
        assert_eq!(stamped.blueprint, superdev_core::version());
        // Already current: the file is left untouched. Marked with a comment
        // `save` would drop, since mtime here cannot resolve two writes a
        // microsecond apart.
        let path = dir.path().join(CONFIG_PATH);
        let before = format!("{}# untouched\n", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, &before).unwrap();
        assert_eq!(stamp_blueprint(dir.path(), &stamped).unwrap(), 0);
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after, "no rewrite when the value is current");
    }
}
