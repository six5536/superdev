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
use superdev_core::components::skillpack;
use superdev_core::engine::Planned;
use superdev_core::error::{Error, Result};
use superdev_core::lock::Lock;
use superdev_core::manifest::{CONFIG_PATH, Manifest};
use superdev_core::runner::{CommandRunner, SystemRunner};
use superdev_core::{components, engine, orphan, registry, report};

/// Provider name for repo-level actions no capability owns.
const REPO_PROVIDER: &str = "superdev";

/// Capabilities whose version is this binary's to decide — a checksum baked
/// in beside the version, or content embedded in the binary itself — so
/// superdev can install the registry default and nothing else. Their
/// components refuse to plan any other pin.
const BINARY_PINNED: [Capability; 3] = [
    Capability::Workflows,
    Capability::CodeIndex,
    Capability::Skills,
];

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
    let mut manifest = Manifest::default_for(superdev_core::version(), &args.disabled());
    let adopted = adopt_existing_skills(root, &mut manifest);
    manifest.save(root)?;
    for line in &adopted {
        out(line)?;
    }
    let mut lock = Lock::default();
    let runner = SystemRunner;
    let (planned, _) = plan_all(root, &runner, &manifest, &lock)?;
    print_plan(&planned)?;
    let outcome = apply_and_report(root, &runner, &manifest, &planned, &mut lock);
    if outcome.is_err() {
        // The manifest is written before the apply and deliberately kept: it is
        // what `sync` resumes from. Say so, rather than leave it unmentioned.
        // A failed print must not mask the failure that prompted it.
        let _ = out(&format!(
            "left in place: {CONFIG_PATH} — `superdev sync` can resume from it"
        ));
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
    // In memory only — status never writes. Unpruned, a skill just marked
    // custom would read as an orphan and plan its own deletion.
    prune_custom_skills(&manifest, &mut lock);
    let runner = SystemRunner;
    let (planned, orphans) = plan_all(root, &runner, &plannable(&manifest), &lock)?;
    print_plan(&planned)?;
    for line in &behind {
        out(line)?;
    }
    for line in &custom_lines(&manifest) {
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
    if let Some((capability, pinned, default)) = checksum_pin_mismatch(&manifest) {
        return Err(Error::Manifest {
            message: format!(
                "{} is pinned {pinned} but this superdev only supports {default} — run `superdev update`",
                capability.as_str()
            ),
        });
    }
    let mut lock = Lock::load(root)?;
    // Before planning: a skill just marked custom still has its lock entry,
    // and unpruned an unmodified one would read as an orphan and be deleted —
    // the opposite of what marking it custom asked for.
    let mut lock_changed = prune_custom_skills(&manifest, &mut lock);
    let runner = SystemRunner;
    let (planned, orphans) = plan_all(root, &runner, &manifest, &lock)?;
    print_plan(&planned)?;
    for line in &behind_pins(&manifest) {
        out(line)?;
    }
    for line in &orphans.released_lines() {
        out(line)?;
    }
    if dry_run {
        return Ok(0);
    }
    // Released and gone orphans leave the lock without an action, and a
    // disabled capability's applied record goes with its files.
    for key in orphans.released.iter().chain(orphans.gone.iter()) {
        lock_changed |= lock.files.remove(key).is_some();
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
    stamp_blueprint(root, &manifest)
}

/// Move version pins to this binary's defaults (or to an explicit version),
/// then sync.
pub fn update(root: &Path, target: Option<&str>) -> Result<u8> {
    let mut manifest = load_manifest(root)?;
    match target {
        Some(target) => {
            let (capability, version) = parse_target(target)?;
            let config = manifest
                .capabilities
                .get_mut(capability.as_str())
                .ok_or_else(|| Error::Manifest {
                    message: format!("`{}` is not enabled", capability.as_str()),
                })?;
            config.version = version.or_else(|| default_version(capability));
        }
        None => {
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
fn parse_target(target: &str) -> Result<(Capability, Option<String>)> {
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
    // This binary decides these versions: it carries the checksum, or the
    // content itself. Any other version has no provenance — and no URLs. The
    // components refuse the same thing when planning.
    if version.is_some() && BINARY_PINNED.contains(&capability) {
        let default = default_version(capability).unwrap_or_default();
        return Err(Error::Manifest {
            message: format!(
                "{} version must match the registry default {default} — this binary is the provenance",
                capability.as_str()
            ),
        });
    }
    Ok((capability, version))
}

/// This binary's default version for `capability`, when it pins one.
fn default_version(capability: Capability) -> Option<String> {
    registry::default_entry(capability)
        .version
        .map(str::to_string)
}

/// The registry version for the provider the manifest names, when both exist.
fn registry_version(manifest: &Manifest, capability: Capability) -> Option<String> {
    let config = manifest.capabilities.get(capability.as_str())?;
    registry::entry_for(capability, &config.provider)?
        .version
        .map(str::to_string)
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
        let (Some(config), Some(default)) = (
            manifest.capabilities.get(capability.as_str()),
            registry_version(manifest, capability),
        ) else {
            continue;
        };
        let stale = if BINARY_PINNED.contains(&capability) {
            pin_mismatch(manifest, capability).is_some()
        } else {
            config
                .version
                .as_deref()
                .is_some_and(|pinned| is_behind(pinned, &default))
        };
        if stale {
            lines.push(format!(
                "{}: pinned {}, registry has {default} — run `superdev update`",
                capability.as_str(),
                config.version.as_deref().unwrap_or("(unset)")
            ));
        }
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

/// Remove released skills' hashes from the lock: a custom skill is the
/// user's file, and a stale hash would misread their next edit as drift
/// against superdev content. True when anything was removed.
fn prune_custom_skills(manifest: &Manifest, lock: &mut Lock) -> bool {
    let Some(config) = manifest.capabilities.get(Capability::Skills.as_str()) else {
        return false;
    };
    let mut pruned = false;
    for name in &config.custom {
        pruned |= lock
            .files
            .remove(&format!(".claude/skills/{name}/SKILL.md"))
            .is_some();
    }
    pruned
}

/// One line per skill released to the user, so custom state stays visible
/// without reading the manifest.
fn custom_lines(manifest: &Manifest) -> Vec<String> {
    manifest
        .capabilities
        .get(Capability::Skills.as_str())
        .map(|config| {
            config
                .custom
                .iter()
                .map(|name| format!("skills: {name} custom, unmanaged"))
                .collect()
        })
        .unwrap_or_default()
}

/// A checksum-pinned capability's pin and this binary's default, when the two
/// differ. Only the default has a checksum baked in beside it, so any other
/// pin — newer included — is one superdev cannot install.
fn pin_mismatch(manifest: &Manifest, capability: Capability) -> Option<(String, String)> {
    let config = manifest.capabilities.get(capability.as_str())?;
    let default = registry_version(manifest, capability)?;
    let pinned = config.version.clone();
    (pinned.as_deref() != Some(default.as_str()))
        .then(|| (pinned.unwrap_or_else(|| "(unset)".into()), default))
}

/// The first checksum-pinned capability pinned off this binary's default.
fn checksum_pin_mismatch(manifest: &Manifest) -> Option<(Capability, String, String)> {
    BINARY_PINNED.into_iter().find_map(|capability| {
        pin_mismatch(manifest, capability).map(|(pinned, default)| (capability, pinned, default))
    })
}

/// A copy of the manifest that can be planned: every checksum-pinned
/// capability back at the default. Every other pin is left alone — components
/// accept those as given.
fn plannable(manifest: &Manifest) -> Manifest {
    let mut plannable = manifest.clone();
    for capability in BINARY_PINNED {
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

/// Compare dotted versions component by component. A deliberate pin ahead of
/// the registry is not drift, so plain inequality will not do.
fn is_behind(pinned: &str, default: &str) -> bool {
    // Numbers order numerically; anything else falls back to text, so an odd
    // version still gets a stable answer instead of a panic.
    let key = |v: &str| {
        v.split('.')
            .map(|part| (part.parse::<u64>().ok(), part.to_string()))
            .collect::<Vec<_>>()
    };
    key(pinned) < key(default)
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
            .find(|(name, _)| *name == "grill-me")
            .unwrap();
        write("grill-me", shipped);

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
    fn parses_update_targets() {
        assert_eq!(
            parse_target("code-index").unwrap(),
            (Capability::CodeIndex, None)
        );
        assert!(parse_target("flying").is_err());
        // Both checksum-verified capabilities refuse a hand-picked version.
        assert!(parse_target("workflows@9.9.9").is_err());
        assert!(parse_target("code-index@9.9.9").is_err());
        assert!(parse_target("skills@9.9.9").is_err());
    }

    fn pin(manifest: &mut Manifest, capability: Capability, version: Option<&str>) {
        manifest
            .capabilities
            .get_mut(capability.as_str())
            .unwrap()
            .version = version.map(str::to_string);
    }

    #[test]
    fn any_checksum_pin_off_the_default_is_stale() {
        for capability in BINARY_PINNED {
            let name = capability.as_str();
            let default = default_version(capability).unwrap();
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
            assert_eq!(checksum_pin_mismatch(&manifest).unwrap().0, capability);
            pin(&mut manifest, capability, None);
            assert!(behind_pins(&manifest)[0].contains("pinned (unset)"));
        }
    }

    #[test]
    fn plannable_resets_every_checksum_pin() {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        for capability in BINARY_PINNED {
            pin(&mut manifest, capability, Some("1.0.0"));
        }
        let plannable = plannable(&manifest);
        assert!(checksum_pin_mismatch(&plannable).is_none());
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
        assert!(prune_custom_skills(&manifest, &mut lock));
        assert!(!lock.files.contains_key(".claude/skills/humanise/SKILL.md"));
        assert!(
            lock.files
                .contains_key(".claude/skills/double-check/SKILL.md")
        );
        // Nothing left to prune: reports no change.
        assert!(!prune_custom_skills(&manifest, &mut lock));

        assert_eq!(
            custom_lines(&manifest),
            vec![
                "skills: humanise custom, unmanaged".to_string(),
                "skills: grill-me custom, unmanaged".to_string(),
            ]
        );
        let no_skills = Manifest::default_for("0.1.0", &[Capability::Skills]);
        assert!(custom_lines(&no_skills).is_empty());
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

    #[test]
    fn only_older_pins_read_as_behind() {
        assert!(is_behind("1.4.9", "1.5.0"));
        assert!(!is_behind("1.5.0", "1.5.0"));
        assert!(!is_behind("9.9.9", "1.5.0"));
        assert!(is_behind("1.10", "1.10.0"));
    }
}
