//! manage.rs — the init/status/sync/update verbs: load, call the core
//! pipeline, render its lines, and turn its facts into exit codes.

use std::io::{self, Write};
use std::path::Path;

use superdev_core::capability::Capability;
use superdev_core::components::pin;
use superdev_core::error::{Error, Result};
use superdev_core::lock::Lock;
use superdev_core::manifest::{CONFIG_PATH, Manifest};
use superdev_core::pipeline::{self, PlanMode, RepoPlan};
use superdev_core::runner::SystemRunner;
use superdev_core::{registry, report};

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
    let adopted = pipeline::adopt_existing(root, &mut manifest);
    manifest.save(root)?;
    for line in &adopted {
        out(line)?;
    }
    let runner = SystemRunner;
    let plan = pipeline::plan_repo(root, &runner, &manifest, &Lock::default(), PlanMode::Sync)?;
    print_plan(&plan)?;
    match pipeline::apply_repo(root, &runner, &manifest, plan) {
        Ok(outcome) if outcome.ok => {
            print_block(&outcome.report)?;
            if outcome.materialised {
                out(SETUP_HINT)?;
            }
            Ok(0)
        }
        Ok(outcome) => {
            print_block(&outcome.report)?;
            left_in_place();
            Err(apply_failed())
        }
        Err(e) => {
            left_in_place();
            Err(e)
        }
    }
}

/// The manifest is written before the apply and deliberately kept: it is
/// what `sync` resumes from. Say so, rather than leave it unmentioned.
/// A failed print must not mask the failure that prompted it.
fn left_in_place() {
    let _ = out(&format!(
        "left in place: {CONFIG_PATH} — `superdev sync` can resume from it"
    ));
}

fn apply_failed() -> Error {
    Error::Manifest {
        message: "apply failed — see report above".into(),
    }
}

/// Report drift without changing anything. 0 in sync, 1 with work to do.
pub fn status(root: &Path) -> Result<u8> {
    let manifest = load_manifest(root)?;
    let lock = Lock::load(root)?;
    let runner = SystemRunner;
    let plan = pipeline::plan_repo(root, &runner, &manifest, &lock, PlanMode::Status)?;
    print_plan(&plan)?;
    for line in plan.behind_lines() {
        out(line)?;
    }
    for line in plan.custom_lines() {
        out(line)?;
    }
    for line in plan.switch_lines() {
        out(line)?;
    }
    for line in &plan.released_lines() {
        out(line)?;
    }
    if let Some(line) = plan.blueprint_line() {
        out(line)?;
    }
    Ok(u8::from(
        plan.has_actions() || !plan.behind_lines().is_empty(),
    ))
}

/// Re-apply the blueprint so the repo matches the manifest.
pub fn sync(root: &Path, dry_run: bool) -> Result<u8> {
    let manifest = load_manifest(root)?;
    let lock = Lock::load(root)?;
    let runner = SystemRunner;
    let plan = pipeline::plan_repo(root, &runner, &manifest, &lock, PlanMode::Sync)?;
    print_plan(&plan)?;
    for line in plan.behind_lines() {
        out(line)?;
    }
    for line in &plan.released_lines() {
        out(line)?;
    }
    for line in plan.switch_lines() {
        out(line)?;
    }
    if dry_run {
        return Ok(0);
    }
    let outcome = pipeline::apply_repo(root, &runner, &manifest, plan)?;
    print_block(&outcome.report)?;
    if !outcome.ok {
        return Err(apply_failed());
    }
    if outcome.materialised {
        out(SETUP_HINT)?;
    }
    Ok(0)
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
            let fallback = pipeline::registry_version(&manifest, capability);
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
                let version = pipeline::registry_version(&manifest, capability);
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
        && let Some(pinned) = pipeline::selected_pin(manifest, capability)
    {
        return Err(Error::Manifest {
            message: pin::refusal_message(capability, pinned),
        });
    }
    Ok((capability, version))
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

fn print_plan(plan: &RepoPlan) -> Result<()> {
    let rendered = report::render_plan(plan.planned());
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
}
