//! manage.rs — the init/status/sync/update verbs: load, call the core
//! pipeline, render its lines, and turn its facts into exit codes.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use superdev_core::capability::Capability;
use superdev_core::components::pin;
use superdev_core::error::{Error, Result};
use superdev_core::lock::Lock;
use superdev_core::manifest::{CONFIG_PATH, Manifest, TemplateRecord};
use superdev_core::pack;
use superdev_core::pipeline::{self, PlanMode, RepoPlan};
use superdev_core::runner::SystemRunner;
use superdev_core::{registry, report, templates};

use crate::template_select;

/// Printed after a materialisation: the upstream setup skill is interactive,
/// so it is the one step superdev cannot run for the user.
/// Printed at the end of every knowledge-enabled init: bootstrap is judgement
/// work the agent does after the mechanical scaffolding.
const BOOTSTRAP_HINT: &str = "knowledge: run /bootstrap in Claude Code to fill the bundle from existing docs and an owner interview";

/// The four capability-disable flags (kebab-case comes free from clap).
#[derive(clap::Args)]
pub struct InitArgs {
    /// Skip the frontend design workflows
    #[arg(long)]
    pub no_frontend: bool,
    /// Skip the superdev skill pack
    #[arg(long)]
    pub no_skills: bool,
    /// Skip the code index
    #[arg(long)]
    pub no_code_index: bool,
    /// Skip the bash output filter
    #[arg(long)]
    pub no_bash_output_filter: bool,
    /// Skip the knowledge scaffold
    #[arg(long)]
    pub no_knowledge: bool,
    #[arg(long, value_name = "NAME", help = crate::template_select::TEMPLATE_HELP)]
    pub template: Option<String>,
    /// Project name for template substitution (default: the directory name)
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,
}

impl InitArgs {
    fn disabled(&self) -> Vec<Capability> {
        let flags = [
            (self.no_frontend, Capability::Frontend),
            (self.no_skills, Capability::Skills),
            (self.no_code_index, Capability::CodeIndex),
            (self.no_bash_output_filter, Capability::BashOutputFilter),
            (self.no_knowledge, Capability::Knowledge),
        ];
        flags
            .into_iter()
            .filter_map(|(off, capability)| off.then_some(capability))
            .collect()
    }
}

/// Set the repo up: write the manifest, then apply the whole blueprint —
/// with the chosen project template's scaffolds, if any, ahead of it.
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
    let dir_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    let selection = template_select::choose(
        args.template.as_deref(),
        args.name.as_deref(),
        template_select::is_tty(),
        dir_name,
        &template_select::TerminalPrompter,
    )?;
    let mut manifest = Manifest::default_for(superdev_core::version(), &args.disabled());
    if let Some(selection) = &selection {
        manifest.template = Some(TemplateRecord {
            name: selection.template.name.to_string(),
            project_name: selection.tokens.name.clone(),
            project_slug: selection.tokens.slug.clone(),
            version: Some(superdev_core::version().to_string()),
        });
    }
    let adopted = pipeline::adopt_existing(root, &mut manifest);
    manifest.save(root)?;
    for line in &adopted {
        out(line)?;
    }
    let runner = SystemRunner;
    let mut plan = pipeline::plan_repo(root, &runner, &manifest, &Lock::default(), PlanMode::Sync)?;
    if let Some(selection) = &selection {
        let (entry, kept) = templates::plan(root, selection.template, &selection.tokens);
        for line in &kept {
            out(line)?;
        }
        if !entry.actions.is_empty() {
            plan.prepend(entry);
        }
    }
    print_plan(&plan)?;
    match pipeline::apply_repo(root, &runner, &manifest, plan) {
        Ok(outcome) if outcome.ok => {
            print_block(&outcome.report)?;
            if manifest.enabled(Capability::Knowledge) {
                out(BOOTSTRAP_HINT)?;
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

/// The `template` subcommands: read-only views of the shipped templates.
/// Grown for the template-update skill — `render` gives it the current
/// content to compare a repo against, and the printed token lines save it
/// re-deriving slug rules it does not own.
#[derive(clap::Subcommand)]
pub enum TemplateCommand {
    /// List the shipped project templates
    List,
    /// Write a template's token-substituted tree into an empty directory
    Render {
        /// Template to render (see `template list`)
        template: String,
        /// Project name the tokens substitute to
        #[arg(long, value_name = "NAME")]
        name: String,
        /// Directory to write into — created if absent, must be empty
        #[arg(long, value_name = "DIR")]
        dir: PathBuf,
    },
}

pub fn template(cmd: &TemplateCommand) -> Result<u8> {
    match cmd {
        TemplateCommand::List => {
            for t in templates::shipped() {
                out(&format!("{} — {}", t.name, t.description))?;
            }
            Ok(0)
        }
        TemplateCommand::Render {
            template,
            name,
            dir,
        } => template_render(template, name, dir),
    }
}

fn template_render(template: &str, name: &str, dir: &Path) -> Result<u8> {
    let template = templates::find(template).ok_or_else(|| Error::Manifest {
        message: format!(
            "template must be one of: {}",
            template_select::shipped_names()
        ),
    })?;
    fs_err(dir, std::fs::create_dir_all(dir))?;
    // Refuse leftovers: a stale file in the target would read as part of
    // this render to whoever diffs against it.
    let mut entries = fs_err(dir, std::fs::read_dir(dir))?;
    if entries.next().is_some() {
        return Err(Error::Io {
            path: dir.into(),
            source: io::Error::other("directory is not empty — render into a fresh one"),
        });
    }
    let tokens = templates::Tokens::for_name(name);
    let files = templates::render(template, &tokens);
    for (path, content) in &files {
        let target = dir.join(path);
        if let Some(parent) = target.parent() {
            fs_err(parent, std::fs::create_dir_all(parent))?;
        }
        fs_err(&target, std::fs::write(&target, content))?;
    }
    out(&format!(
        "rendered template {} into {} ({} files)",
        template.name,
        dir.display(),
        files.len()
    ))?;
    // The [template] keys, verbatim — whoever records provenance copies
    // these lines rather than re-deriving the slug.
    out(&format!("project-name = {:?}", tokens.name))?;
    out(&format!("project-slug = {:?}", tokens.slug))?;
    out(&format!("project-ident = {:?}", tokens.ident()))?;
    out(&format!("project-compact = {:?}", tokens.compact()))?;
    out(&format!("project-pascal = {:?}", tokens.pascal()))?;
    Ok(0)
}

/// Wrap an fs result with the path it concerned.
fn fs_err<T>(path: &Path, result: io::Result<T>) -> Result<T> {
    result.map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
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
/// `drift_only` keeps the report whole but narrows the exit code to drift,
/// leaving provisioning runs out of it.
pub fn status(root: &Path, drift_only: bool) -> Result<u8> {
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
    for line in plan.content_lines() {
        out(line)?;
    }
    for line in &plan.released_lines() {
        out(line)?;
    }
    if let Some(line) = plan.blueprint_line() {
        out(line)?;
    }
    // A stale pin is version drift either way; `--drift` narrows only the
    // action half, so a gate can ask "does this tree match the blueprint?"
    // without a checkout's unprovisioned external state answering for it.
    let outstanding = if drift_only {
        plan.has_drift()
    } else {
        plan.has_actions()
    };
    Ok(u8::from(outstanding || !plan.behind_lines().is_empty()))
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
    if dry_run {
        return Ok(0);
    }
    let outcome = pipeline::apply_repo(root, &runner, &manifest, plan)?;
    print_block(&outcome.report)?;
    if !outcome.ok {
        return Err(apply_failed());
    }
    Ok(0)
}

/// Bring pins current, then sync.
///
/// A capability's pin moves to this binary's default, or to an explicit
/// version. The pack's moves further: the untargeted form asks the default
/// source for its newest release and takes that, which is how content reaches
/// a repo whose binary has not changed (ADR-009). It is the one place
/// superdev reaches the network without being asked to fetch something.
///
/// The newest release *this binary can read*: the pack is resolved before the
/// pin naming it is written, and a release this binary would refuse leaves the
/// pin where it was with the reason reported. Written first, such a pin would
/// be unreachable by any superdev command — this saves the manifest before the
/// `sync` below validates anything, and `update` never moves a pin backwards
/// (ADR-013).
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
            if !manifest.enabled(capability) {
                return Err(Error::Manifest {
                    message: format!("`{}` is not enabled", capability.as_str()),
                });
            }
            if let Some(id) = provider {
                let entry = registry::entry_for(capability, id).expect("validated above");
                let configs = manifest.configs_mut(capability);
                // A provider switch replaces the slot's one entry; a slot
                // holding several packs has no single entry to switch.
                if configs.len() > 1 {
                    return Err(Error::Manifest {
                        message: format!(
                            "{} holds several packs — edit {CONFIG_PATH} to change them",
                            capability.as_str()
                        ),
                    });
                }
                configs[0].provider = entry.provider.to_string();
                configs[0].version = entry.version.map(|p| p.version.to_string());
            } else {
                for config in manifest.configs_mut(capability) {
                    config.version = version
                        .clone()
                        .or_else(|| pipeline::registry_version_of(capability, &config.provider));
                }
            }
        }
        (None, None) => {
            for capability in Capability::ALL {
                for config in manifest.configs_mut(capability) {
                    config.version = pipeline::registry_version_of(capability, &config.provider);
                }
            }
            // Only the untargeted form moves the pack pin: `update knowledge`
            // is a narrow request, and this is the one place superdev reaches
            // the network without being asked to fetch something. ADR-009.
            // The lock resolution reads, so a pin can be proved against the
            // cache before it is written. ADR-013.
            let lock = Lock::load(root)?;
            for line in pack::update_pins(&SystemRunner, root, &mut manifest, &lock) {
                out(&line)?;
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
    fn init_refuses_reruns_and_non_git_dirs() {
        let args = InitArgs {
            no_frontend: false,
            no_skills: false,
            no_code_index: false,
            no_bash_output_filter: false,
            no_knowledge: false,
            template: None,
            name: None,
        };
        // Both guards fire before anything is planned or run.
        let plain = tempfile::tempdir().unwrap();
        let err = init(plain.path(), &args).unwrap_err().to_string();
        assert!(err.contains("git"), "{err}");
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        Manifest::default_for("0.1.0", &[])
            .save(dir.path())
            .unwrap();
        let err = init(dir.path(), &args).unwrap_err().to_string();
        assert!(err.contains("sync"), "{err}");
    }

    #[test]
    fn update_provider_rules() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for(superdev_core::version(), &[]);
        manifest.save(dir.path()).unwrap();
        let err = update(dir.path(), None, Some("aokf"))
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("--provider needs a capability target"),
            "{err}"
        );
        let err = update(dir.path(), Some("knowledge"), Some("flying"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("knowledge provider must be one of"), "{err}");
    }

    #[test]
    fn parses_update_targets() {
        let manifest = Manifest::default_for("0.1.0", &[]);
        assert_eq!(
            parse_target(&manifest, "code-index").unwrap(),
            (Capability::CodeIndex, None)
        );
        assert!(parse_target(&manifest, "flying").is_err());
        assert!(parse_target(&manifest, "workflows").is_err());
        // Every registry-pinned capability refuses a hand-picked version.
        let err = parse_target(&manifest, "code-index@9.9.9")
            .unwrap_err()
            .to_string();
        assert!(err.contains("run `superdev update code-index`"), "{err}");
        assert!(parse_target(&manifest, "skills@9.9.9").is_err());
    }
}
