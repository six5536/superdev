//! engine.rs — plan every component, apply the result, roll back on failure.
//!
//! Every side effect is journalled as it happens, so the first failure can
//! unwind the run instead of leaving the repo half-changed.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::action::{Action, Ownership};
use crate::capability::Capability;
use crate::component::{Component, Ctx};
use crate::components::mise;
use crate::error::{Error, Result};
use crate::lock::{Lock, LockedComponent, sha256_hex};
use crate::manifest::Manifest;
use crate::runner::CommandRunner;

/// Where backups of overwritten files go, under the repo root.
const BACKUP_DIR: &str = ".superdev/cache/backup";
/// Argument form resolved to a mise tool's install path.
const MISE_WHERE: &str = "{mise-where:";
/// Skip reason for a removal target that is already absent. Named, because
/// reconciliation reads it back to tell a swept file from a released one.
const ALREADY_GONE: &str = "already gone";

/// One component's planned changes.
#[derive(Debug, Clone)]
pub struct Planned {
    /// None for repo-level actions not owned by a capability (e.g. .gitignore).
    pub capability: Option<Capability>,
    /// Provider that produced these actions.
    pub provider: String,
    /// Actions, in apply order.
    pub actions: Vec<Action>,
}

/// What became of one action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    /// Done, with an optional note the user should see.
    Applied {
        /// Anything surprising about the change, e.g. a clobbered user edit.
        note: Option<String>,
    },
    /// Nothing to do, with the reason.
    Skipped(String),
    /// Failed, with the error text. Ends the run.
    Failed(String),
}

/// One planned entry's outcomes.
#[derive(Debug, Clone)]
pub struct ComponentReport {
    /// `capability (provider)`, or `repo (provider)` for repo-level entries.
    pub label: String,
    /// (action description, outcome) pairs. Mise pins come first, because they
    /// are applied as one grouped edit before any entry runs, and the entry
    /// that contributed the first pin also carries the `mise trust` and `mise
    /// install` that follow them — or, when no pin edit was needed, the first
    /// entry that runs a command does. The rest are in action order.
    pub outcomes: Vec<(String, ActionOutcome)>,
}

/// The result of one apply run.
#[derive(Debug, Clone)]
pub struct ApplyResult {
    /// One report per planned entry, in the order given.
    pub reports: Vec<ComponentReport>,
    /// Action descriptions undone after a failure.
    pub reverted: Vec<String>,
    /// Side effects that could not be undone.
    pub not_reverted: Vec<String>,
    /// False when any action failed.
    pub ok: bool,
}

/// Run every component's plan. Pure aside from component observation.
pub fn plan(components: &[Box<dyn Component>], ctx: &Ctx<'_>) -> Result<Vec<Planned>> {
    components
        .iter()
        .map(|c| {
            Ok(Planned {
                capability: Some(c.capability()),
                provider: c.provider().to_string(),
                actions: c.plan(ctx)?,
            })
        })
        .collect()
}

/// Apply planned actions. On failure, unwind this run's journal in reverse
/// (best-effort) and return `ok = false`. Mutates `lock` only for fully
/// applied components; the caller saves it only when ok. `manifest` supplies
/// the capability versions recorded into the lock.
pub fn apply(
    root: &Path,
    runner: &dyn CommandRunner,
    manifest: &Manifest,
    planned: &[Planned],
    lock: &mut Lock,
) -> ApplyResult {
    let mut session = Session::new(root, runner, planned, lock);
    let mut ok = session.apply_pins(planned);
    if ok {
        for (index, entry) in planned.iter().enumerate() {
            if !session.apply_entry(index, entry, manifest, lock) {
                ok = false;
                break;
            }
        }
    }
    let (reverted, not_reverted) = if ok {
        (Vec::new(), Vec::new())
    } else {
        session.unwind()
    };
    ApplyResult {
        reports: session.reports,
        reverted,
        not_reverted,
        ok,
    }
}

/// One journalled side effect, and how to take it back.
enum Undo {
    RestoreFile { path: String, prior: Option<String> },
    RunCommand { program: String, args: Vec<String> },
}

/// The lock changes one entry accumulates while its actions run, applied
/// together when the entry completes.
struct LockEffects<'e> {
    /// (key, hash) pairs to insert into `files`.
    written: &'e mut Vec<(String, String)>,
    /// Keys to attribute to the entry's capability in `owners`.
    attributed: &'e mut Vec<String>,
    /// Keys to drop from both.
    removed: &'e mut Vec<String>,
}

/// State carried through a single apply run.
struct Session<'a> {
    root: &'a Path,
    runner: &'a dyn CommandRunner,
    /// One timestamp per run, so a run's backups sit together.
    stamp: u64,
    journal: Vec<Undo>,
    /// Side effects with no undo, reported when a later failure unwinds.
    irreversible: Vec<String>,
    /// Locked hashes as of the start of the run, for user-edit detection.
    prior_hashes: BTreeMap<String, String>,
    /// Lock attribution as of the start of the run, for reconciliation.
    prior_owners: BTreeMap<String, String>,
    /// Lock keys earlier entries wrote in this run. Removals were planned
    /// against the pre-run state, so a later one must not drop them.
    written_keys: BTreeSet<String>,
    /// Pin hashes each entry earned, held until the entry completes.
    pins: Vec<Vec<(String, String)>>,
    reports: Vec<ComponentReport>,
}

impl<'a> Session<'a> {
    fn new(
        root: &'a Path,
        runner: &'a dyn CommandRunner,
        planned: &[Planned],
        lock: &Lock,
    ) -> Session<'a> {
        Session {
            root,
            runner,
            stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            journal: Vec::new(),
            irreversible: Vec::new(),
            prior_hashes: lock.files.clone(),
            prior_owners: lock.owners.clone(),
            written_keys: BTreeSet::new(),
            pins: vec![Vec::new(); planned.len()],
            reports: planned
                .iter()
                .map(|p| ComponentReport {
                    label: match p.capability {
                        Some(c) => format!("{} ({})", c.as_str(), p.provider),
                        None => format!("repo ({})", p.provider),
                    },
                    outcomes: Vec::new(),
                })
                .collect(),
        }
    }

    fn record(&mut self, entry: usize, action: &Action, outcome: ActionOutcome) {
        self.reports[entry]
            .outcomes
            .push((action.describe(), outcome));
    }

    /// Apply every `SetMisePin` as one edit, then trust and install.
    /// Providers run their own commands afterwards, so the tools must exist.
    ///
    /// With no pin edit planned, the tools may still be missing — a fresh
    /// clone commits `.mise.toml` but installs nothing — so
    /// [`Session::install_committed_pins`] covers that case.
    fn apply_pins(&mut self, planned: &[Planned]) -> bool {
        let pins: Vec<(usize, &Action, &str, &str)> = planned
            .iter()
            .enumerate()
            .flat_map(|(index, p)| {
                p.actions.iter().filter_map(move |a| match a {
                    Action::SetMisePin { tool, value_toml } => {
                        Some((index, a, tool.as_str(), value_toml.as_str()))
                    }
                    _ => None,
                })
            })
            .collect();
        let Some(&(first, first_action, _, _)) = pins.first() else {
            return self.install_committed_pins(planned);
        };
        let path = self.root.join(".mise.toml");
        let prior = match read_text(&path) {
            Ok(prior) => prior,
            Err(e) => {
                self.record(first, first_action, ActionOutcome::Failed(e.to_string()));
                return false;
            }
        };
        // Edit in memory first: a bad pin then leaves the file untouched.
        let mut content = prior.clone().unwrap_or_default();
        for &(entry, action, tool, value) in &pins {
            match mise::set_pin(&content, tool, value) {
                Ok(next) => content = next,
                Err(e) => {
                    self.record(entry, action, ActionOutcome::Failed(e.to_string()));
                    return false;
                }
            }
        }
        self.journal.push(Undo::RestoreFile {
            path: ".mise.toml".into(),
            prior,
        });
        if let Err(e) = write_file(&path, &content) {
            self.record(first, first_action, ActionOutcome::Failed(e.to_string()));
            return false;
        }
        for &(entry, action, tool, _) in &pins {
            // Hash the normalised value, so layout changes never read as drift.
            let value = mise::current_pin(&content, tool)
                .expect("content came from set_pin, so it parses")
                .expect("the pin was just set");
            self.pins[entry].push((mise::pin_lock_key(tool), sha256_hex(value.as_bytes())));
            self.record(entry, action, ActionOutcome::Applied { note: None });
        }
        // Install what this run pinned plus every managed pin the file already
        // carried: an untouched pin may still be missing on this machine.
        let mut tools: Vec<String> = pins.iter().map(|&(.., tool, _)| tool.to_string()).collect();
        tools.extend(managed_pins_in(&content));
        tools.sort_unstable();
        tools.dedup();
        self.install_pinned_tools(first, &tools)
    }

    /// Install the pins `.mise.toml` already carries, when a provider is about
    /// to run a command. On a fresh clone of a managed repo the committed pins
    /// match, so nothing is planned, yet no tool is installed on this machine.
    /// Skipped when nothing runs, or when no superdev-managed tool is pinned.
    fn install_committed_pins(&mut self, planned: &[Planned]) -> bool {
        let runs = |p: &&Planned| {
            p.actions
                .iter()
                .any(|a| matches!(a, Action::Run { .. } | Action::MaterialiseSkills { .. }))
        };
        let Some(entry) = planned.iter().position(|p| runs(&p)) else {
            return true;
        };
        let Ok(Some(content)) = read_text(&self.root.join(".mise.toml")) else {
            return true;
        };
        // An unparseable file pins nothing superdev can claim; leave it to the
        // component that reads it to report the problem.
        let pinned = managed_pins_in(&content);
        if pinned.is_empty() {
            return true;
        }
        self.install_pinned_tools(entry, &pinned)
    }

    /// Trust the config, then install — naming the tools. mise refuses to
    /// install from a config it has not been told to trust, and superdev
    /// writes managed pins into repos that have never trusted theirs.
    /// Trusting is idempotent, so it runs every time; a failure is treated
    /// exactly like a failed install.
    ///
    /// A bare `mise install` would install every tool the repo pins, tying
    /// superdev's success to toolchains it does not manage: one unrelated pin
    /// that cannot build on this machine would fail the whole apply.
    fn install_pinned_tools(&mut self, entry: usize, tools: &[String]) -> bool {
        if !self.run_mise(entry, &["trust".into()], "trust the repo's mise config") {
            return false;
        }
        let mut args = vec!["install".to_string()];
        args.extend(tools.iter().cloned());
        self.run_mise(entry, &args, "install the pinned tools")
    }

    /// Run a `mise` command the plan never contained, recording it against
    /// `entry` as if it had.
    fn run_mise(&mut self, entry: usize, args: &[String], purpose: &str) -> bool {
        let args = args.to_vec();
        let action = Action::Run {
            program: "mise".into(),
            args: args.clone(),
            purpose: purpose.into(),
            undo: None,
            optional: false,
        };
        let outcome = self.run_action("mise", &args, &None, false);
        let ok = !matches!(outcome, ActionOutcome::Failed(_));
        self.record(entry, &action, outcome);
        ok
    }

    /// Apply one entry's non-pin actions, then record what it applied.
    fn apply_entry(
        &mut self,
        index: usize,
        entry: &Planned,
        manifest: &Manifest,
        lock: &mut Lock,
    ) -> bool {
        let mut written = Vec::new();
        let mut attributed: Vec<String> = Vec::new();
        let mut removed: Vec<String> = Vec::new();
        for action in &entry.actions {
            let outcome = match action {
                // Pins were applied as one grouped edit before any entry ran.
                Action::SetMisePin { .. } => continue,
                Action::WriteFile {
                    path,
                    content,
                    ownership,
                    ..
                } => self.write_action(path, content, *ownership, &mut written),
                Action::EnsureLine { path, line, .. } => self.ensure_line(path, line),
                Action::SetJsonKey {
                    path,
                    pointer,
                    value_json,
                } => self.set_json_key(path, pointer, value_json, &mut written),
                Action::EnsureJsonArrayElement {
                    path,
                    pointer,
                    marker,
                    value_json,
                } => {
                    self.ensure_json_array_element(path, pointer, marker, value_json, &mut written)
                }
                Action::MaterialiseSkills {
                    tool,
                    source_dirs,
                    custom,
                } => self.materialise_skills(
                    entry.capability,
                    tool,
                    source_dirs,
                    custom,
                    LockEffects {
                        written: &mut written,
                        attributed: &mut attributed,
                        removed: &mut removed,
                    },
                ),
                Action::Run {
                    program,
                    args,
                    undo,
                    optional,
                    ..
                } => self.run_action(program, args, undo, *optional),
                Action::RemoveFile { path, .. } => self.remove_file(path, &mut removed),
                Action::RemoveMisePin { tool } => self.remove_mise_pin(tool, &mut removed),
                Action::RemoveJsonKey { path, pointer } => {
                    self.remove_json_key(path, pointer, &mut removed)
                }
            };
            let failed = matches!(outcome, ActionOutcome::Failed(_));
            self.record(index, action, outcome);
            if failed {
                return false;
            }
        }
        let keys: Vec<String> = std::mem::take(&mut self.pins[index])
            .into_iter()
            .chain(written)
            .map(|(key, hash)| {
                lock.files.insert(key.clone(), hash);
                key
            })
            .collect();
        // Before the removals, so a reconciled removal wins over an
        // attribution the same entry recorded.
        for key in attributed {
            if let Some(capability) = entry.capability {
                lock.owners.insert(key, capability.as_str().to_string());
            }
        }
        for key in removed {
            // An earlier entry rewrote this file in this same run: the removal
            // was planned against the old state and would strand the fresh
            // file, unlocked and unowned, until some later run noticed.
            if self.written_keys.contains(&key) {
                continue;
            }
            lock.files.remove(&key);
            lock.owners.remove(&key);
        }
        self.written_keys.extend(keys);
        if let Some(capability) = entry.capability {
            lock.components.insert(
                capability.as_str().to_string(),
                LockedComponent {
                    provider: entry.provider.clone(),
                    version: manifest
                        .capabilities
                        .get(capability.as_str())
                        .and_then(|c| c.version.clone()),
                },
            );
        }
        true
    }

    fn write_action(
        &mut self,
        path: &str,
        content: &str,
        ownership: Ownership,
        written: &mut Vec<(String, String)>,
    ) -> ActionOutcome {
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        if ownership == Ownership::Scaffold && existing.is_some() {
            return ActionOutcome::Skipped("exists".into());
        }
        let mut note = None;
        if let Some(old) = &existing {
            let backup = self
                .root
                .join(BACKUP_DIR)
                .join(self.stamp.to_string())
                .join(path);
            if let Err(e) = write_file(&backup, old) {
                return ActionOutcome::Failed(e.to_string());
            }
            if ownership == Ownership::Owned
                && self.prior_hashes.get(path) != Some(&sha256_hex(old.as_bytes()))
            {
                note = Some("overwrote a user-edited file (backed up)".to_string());
            }
        }
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: existing,
        });
        if let Err(e) = write_file(&full, content) {
            return ActionOutcome::Failed(e.to_string());
        }
        if ownership == Ownership::Owned {
            written.push((path.to_string(), sha256_hex(content.as_bytes())));
        }
        ActionOutcome::Applied { note }
    }

    fn ensure_line(&mut self, path: &str, line: &str) -> ActionOutcome {
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let mut next = existing.clone().unwrap_or_default();
        if next.lines().any(|l| l == line) {
            return ActionOutcome::Skipped("present".into());
        }
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: existing,
        });
        if !next.is_empty() && !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(line);
        next.push('\n');
        match write_file(&full, &next) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Remove an owned file the blueprint dropped. The lock key is released
    /// even when the removal is skipped: gone or user-changed, the file is no
    /// longer superdev's.
    fn remove_file(&mut self, path: &str, removed: &mut Vec<String>) -> ActionOutcome {
        removed.push(path.to_string());
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let Some(content) = existing else {
            return ActionOutcome::Skipped(ALREADY_GONE.into());
        };
        // Re-check at apply time: an edit between plan and apply is the
        // user's, and superdev takes back only what it wrote.
        if self.prior_hashes.get(path) != Some(&sha256_hex(content.as_bytes())) {
            return ActionOutcome::Skipped(
                "changed since superdev wrote it — left in place".into(),
            );
        }
        let backup = self
            .root
            .join(BACKUP_DIR)
            .join(self.stamp.to_string())
            .join(path);
        if let Err(e) = write_file(&backup, &content) {
            return ActionOutcome::Failed(e.to_string());
        }
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: Some(content),
        });
        match fs::remove_file(&full) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Copy a pinned checkout's skill directories into the repo, then
    /// reconcile: attributed entries the checkout no longer ships leave by
    /// the same rules as RemoveFile. One aggregate outcome carries the counts.
    fn materialise_skills(
        &mut self,
        capability: Option<Capability>,
        tool: &str,
        source_dirs: &[String],
        custom: &[String],
        effects: LockEffects<'_>,
    ) -> ActionOutcome {
        // Repo-level entries own nothing, so they have nothing to attribute —
        // and never materialise.
        let Some(capability) = capability else {
            return ActionOutcome::Failed("materialise needs an owning capability".into());
        };
        let args = vec!["where".to_string(), tool.to_string()];
        let checkout = match self.runner.run("mise", &args, self.root) {
            Ok(out) if out.status == 0 => std::path::PathBuf::from(out.stdout.trim()),
            Ok(out) => {
                return ActionOutcome::Failed(
                    Error::Command {
                        command: command_line("mise", &args),
                        status: Some(out.status),
                        stderr: out.stderr,
                    }
                    .to_string(),
                );
            }
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let mut wrote = 0usize;
        let mut kept = 0usize;
        let mut edited = 0usize;
        let mut fresh: Vec<String> = Vec::new();
        for source_dir in source_dirs {
            let dir = checkout.join(source_dir);
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(e) => {
                    return ActionOutcome::Failed(
                        Error::Io {
                            path: dir,
                            source: e,
                        }
                        .to_string(),
                    );
                }
            };
            for entry in entries {
                let Ok(entry) = entry else { continue };
                let path = entry.path();
                // A skill is a directory; loose files beside them are the
                // checkout's own, e.g. its README.
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if custom.contains(&name) {
                    continue;
                }
                let mut files = Vec::new();
                if let Err(e) = collect_files(&path, &mut files) {
                    return ActionOutcome::Failed(e.to_string());
                }
                for file in files {
                    let rel = file
                        .strip_prefix(&path)
                        .expect("collected under this skill directory");
                    let target = format!(
                        ".claude/skills/{name}/{}",
                        rel.display().to_string().replace('\\', "/")
                    );
                    let content = match read_text(&file) {
                        Ok(Some(content)) => content,
                        Ok(None) => continue,
                        Err(e) => return ActionOutcome::Failed(e.to_string()),
                    };
                    fresh.push(target.clone());
                    let existing = match read_text(&self.root.join(&target)) {
                        Ok(existing) => existing,
                        Err(e) => return ActionOutcome::Failed(e.to_string()),
                    };
                    // An unchanged file is still claimed, so a converged run
                    // keeps its lock entry and its attribution.
                    if existing.as_deref() == Some(content.as_str()) {
                        kept += 1;
                        effects
                            .written
                            .push((target.clone(), sha256_hex(content.as_bytes())));
                        effects.attributed.push(target);
                        continue;
                    }
                    let mut probe = Vec::new();
                    match self.write_action(&target, &content, Ownership::Owned, &mut probe) {
                        ActionOutcome::Applied { note } => {
                            wrote += 1;
                            edited += usize::from(note.is_some());
                        }
                        ActionOutcome::Failed(e) => return ActionOutcome::Failed(e),
                        ActionOutcome::Skipped(_) => unreachable!("owned writes never skip"),
                    }
                    effects.written.append(&mut probe);
                    effects.attributed.push(target);
                }
            }
        }
        // Reconcile: what this capability had materialised and the checkout
        // no longer ships leaves by the RemoveFile rules.
        let mut released = 0usize;
        let mut swept = 0usize;
        let stale: Vec<String> = self
            .prior_owners
            .iter()
            .filter(|(key, owner)| owner.as_str() == capability.as_str() && !fresh.contains(key))
            .map(|(key, _)| key.clone())
            .collect();
        for key in stale {
            match self.remove_file(&key, effects.removed) {
                ActionOutcome::Applied { .. } => swept += 1,
                ActionOutcome::Skipped(reason) if reason == ALREADY_GONE => {}
                ActionOutcome::Skipped(_) => released += 1,
                ActionOutcome::Failed(e) => return ActionOutcome::Failed(e),
            }
        }
        let mut note = format!("wrote {wrote}, kept {kept}, removed {swept}, released {released}");
        if edited > 0 {
            note.push_str(&format!("; overwrote {edited} user-edited (backed up)"));
        }
        ActionOutcome::Applied { note: Some(note) }
    }

    /// Remove a managed pin. Journals the whole file: removing a pin is a
    /// file rewrite.
    fn remove_mise_pin(&mut self, tool: &str, removed: &mut Vec<String>) -> ActionOutcome {
        let key = mise::pin_lock_key(tool);
        removed.push(key.clone());
        let path = self.root.join(".mise.toml");
        let existing = match read_text(&path) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let Some(content) = existing else {
            return ActionOutcome::Skipped(ALREADY_GONE.into());
        };
        let value = match mise::current_pin(&content, tool) {
            Ok(Some(value)) => value,
            Ok(None) => return ActionOutcome::Skipped(ALREADY_GONE.into()),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        if self.prior_hashes.get(&key) != Some(&sha256_hex(value.as_bytes())) {
            return ActionOutcome::Skipped(
                "changed since superdev wrote it — left in place".into(),
            );
        }
        let next = match mise::remove_pin(&content, tool) {
            Ok(next) => next.expect("the pin is present"),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        self.journal.push(Undo::RestoreFile {
            path: ".mise.toml".into(),
            prior: Some(content),
        });
        match write_file(&path, &next) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Remove a managed JSON key or array element. Journals the whole file.
    fn remove_json_key(
        &mut self,
        path: &str,
        pointer: &str,
        removed: &mut Vec<String>,
    ) -> ActionOutcome {
        let key = format!("{path}:{pointer}");
        removed.push(key.clone());
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let Some(content) = existing else {
            return ActionOutcome::Skipped(ALREADY_GONE.into());
        };
        let value = match json_value_at(path, &content, pointer) {
            Ok(Some(value)) => value,
            Ok(None) => return ActionOutcome::Skipped(ALREADY_GONE.into()),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        if self.prior_hashes.get(&key) != Some(&sha256_hex(value.as_bytes())) {
            return ActionOutcome::Skipped(
                "changed since superdev wrote it — left in place".into(),
            );
        }
        let (next, _) = match remove_json_pointer(path, &content, pointer) {
            Ok(edited) => edited.expect("the entry is present"),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: Some(content),
        });
        match write_file(&full, &next) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Merge one key into a JSON file, hashing the value into the lock the way
    /// a mise pin is hashed: superdev owns the key, not the file.
    fn set_json_key(
        &mut self,
        path: &str,
        pointer: &str,
        value_json: &str,
        written: &mut Vec<(String, String)>,
    ) -> ActionOutcome {
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        // Edit in memory first, so a malformed file is left as the user wrote it.
        let edited = edit_json_key(
            path,
            existing.as_deref().unwrap_or("{}"),
            pointer,
            value_json,
        );
        let (content, value) = match edited {
            Ok(edited) => edited,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: existing,
        });
        if let Err(e) = write_file(&full, &content) {
            return ActionOutcome::Failed(e.to_string());
        }
        // Hash the canonical value, so layout changes never read as drift.
        written.push((format!("{path}:{pointer}"), sha256_hex(value.as_bytes())));
        ActionOutcome::Applied { note: None }
    }

    /// Merge one array element into a JSON file. Superdev owns the element
    /// its marker finds — replaced in place, appended when absent — and the
    /// lock hashes the canonical element, not the file.
    fn ensure_json_array_element(
        &mut self,
        path: &str,
        pointer: &str,
        marker: &str,
        value_json: &str,
        written: &mut Vec<(String, String)>,
    ) -> ActionOutcome {
        let full = self.root.join(path);
        let existing = match read_text(&full) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        // Edit in memory first, so a malformed file is left as the user wrote it.
        let edited = edit_json_array_element(
            path,
            existing.as_deref().unwrap_or("{}"),
            pointer,
            marker,
            value_json,
        );
        let (content, value) = match edited {
            Ok(edited) => edited,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior: existing,
        });
        if let Err(e) = write_file(&full, &content) {
            return ActionOutcome::Failed(e.to_string());
        }
        written.push((
            format!("{path}:{pointer}[{marker}]"),
            sha256_hex(value.as_bytes()),
        ));
        ActionOutcome::Applied { note: None }
    }

    fn run_action(
        &mut self,
        program: &str,
        args: &[String],
        undo: &Option<(String, Vec<String>)>,
        optional: bool,
    ) -> ActionOutcome {
        let args = match self.resolve_args(args) {
            Ok(args) => args,
            // The failure is mise's, not the action's program.
            Err(e) => return missing_or_failed("mise", e, optional),
        };
        match self.runner.run(program, &args, self.root) {
            Err(e) => missing_or_failed(program, e, optional),
            Ok(out) if out.status != 0 => ActionOutcome::Failed(
                Error::Command {
                    command: command_line(program, &args),
                    status: Some(out.status),
                    stderr: out.stderr,
                }
                .to_string(),
            ),
            Ok(_) => {
                match undo {
                    Some((program, args)) => self.journal.push(Undo::RunCommand {
                        program: program.clone(),
                        args: args.clone(),
                    }),
                    None => self
                        .irreversible
                        .push(format!("`{}` has no undo", command_line(program, &args))),
                }
                ActionOutcome::Applied { note: None }
            }
        }
    }

    /// Replace every `{mise-where:TOOL}` argument with the tool's install path.
    fn resolve_args(&self, args: &[String]) -> Result<Vec<String>> {
        args.iter()
            .map(|arg| {
                let Some(tool) = arg
                    .strip_prefix(MISE_WHERE)
                    .and_then(|a| a.strip_suffix('}'))
                else {
                    return Ok(arg.clone());
                };
                let args = vec!["where".to_string(), tool.to_string()];
                let out = self.runner.run("mise", &args, self.root)?;
                if out.status != 0 {
                    return Err(Error::Command {
                        command: command_line("mise", &args),
                        status: Some(out.status),
                        stderr: out.stderr,
                    });
                }
                Ok(out.stdout.trim().to_string())
            })
            .collect()
    }

    /// Take back this run's side effects, newest first.
    fn unwind(&mut self) -> (Vec<String>, Vec<String>) {
        let mut reverted = Vec::new();
        let mut not_reverted = Vec::new();
        while let Some(undo) = self.journal.pop() {
            match undo {
                Undo::RestoreFile {
                    path,
                    prior: Some(old),
                } => match write_file(&self.root.join(&path), &old) {
                    Ok(()) => reverted.push(format!("restored {path}")),
                    Err(e) => not_reverted.push(format!("{path} left changed: {e}")),
                },
                Undo::RestoreFile { path, prior: None } => {
                    match fs::remove_file(self.root.join(&path)) {
                        Ok(()) => reverted.push(format!("removed {path}")),
                        // Never created: the write is what failed.
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => not_reverted.push(format!("{path} left in place: {e}")),
                    }
                }
                Undo::RunCommand { program, args } => {
                    let line = command_line(&program, &args);
                    match self.runner.run(&program, &args, self.root) {
                        Ok(out) if out.status == 0 => reverted.push(format!("ran `{line}`")),
                        _ => not_reverted.push(format!("`{line}` did not undo cleanly")),
                    }
                }
            }
        }
        not_reverted.append(&mut self.irreversible);
        (reverted, not_reverted)
    }
}

/// The superdev-managed tools `.mise.toml` pins, in registry order. Naming
/// them is what keeps `mise install` off the repo's own toolchain.
fn managed_pins_in(content: &str) -> Vec<String> {
    crate::components::MANAGED_MISE_TOOLS
        .iter()
        .filter(|tool| matches!(mise::current_pin(content, tool), Ok(Some(_))))
        .map(|tool| (*tool).to_string())
        .collect()
}

/// A missing program is a skip for optional actions, a failure otherwise.
fn missing_or_failed(program: &str, error: Error, optional: bool) -> ActionOutcome {
    if optional && matches!(error, Error::Command { status: None, .. }) {
        ActionOutcome::Skipped(format!(
            "{program} not installed — run `superdev sync` once it is"
        ))
    } else {
        ActionOutcome::Failed(error.to_string())
    }
}

fn command_line(program: &str, args: &[String]) -> String {
    format!("{program} {}", args.join(" "))
        .trim_end()
        .to_string()
}

/// File content, or None when the file is absent. Anything unreadable — a
/// binary file at a target path included — is an error, never an overwrite.
pub(crate) fn read_text(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::Io {
            path: path.into(),
            source: e,
        }),
    }
}

/// Every file under `dir`, recursively, in sorted order.
fn collect_files(dir: &Path, into: &mut Vec<std::path::PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| Error::Io {
            path: dir.into(),
            source: e,
        })?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(&path, into)?;
        } else {
            into.push(path);
        }
    }
    Ok(())
}

/// Set one dotted key path in a JSON document, creating missing objects on the
/// way. Returns the file content to write and the canonical value text, which
/// is what the lock hashes.
///
/// Every other key survives; their order does not, because serde_json sorts
/// object keys on the way out.
fn edit_json_key(
    path: &str,
    json: &str,
    pointer: &str,
    value_json: &str,
) -> Result<(String, String)> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(value_json)
        .map_err(|e| bad(format!("invalid value `{value_json}`: {e}")))?;

    let mut segments: Vec<&str> = pointer.split('.').collect();
    let key = segments.pop().expect("split yields at least one segment");
    // Names the container the walk is standing in, for the error message.
    let mut container = "the root".to_string();
    let mut cursor = &mut root;
    for segment in segments {
        cursor = match cursor.as_object_mut() {
            Some(map) => map
                .entry(segment)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new())),
            None => return Err(bad(format!("{container} is not a JSON object"))),
        };
        container = format!("`{segment}`");
    }
    match cursor.as_object_mut() {
        Some(map) => map.insert(key.to_string(), value.clone()),
        None => return Err(bad(format!("{container} is not a JSON object"))),
    };

    let mut content = serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
    content.push('\n');
    Ok((content, value.to_string()))
}

/// Ensure the array at a dotted key path contains `value_json`: the first
/// element whose serialised form contains `marker` is replaced, else the
/// element is appended. Missing objects on the way — and the array itself —
/// are created. Returns the file content to write and the canonical element
/// text, which is what the lock hashes.
fn edit_json_array_element(
    path: &str,
    json: &str,
    pointer: &str,
    marker: &str,
    value_json: &str,
) -> Result<(String, String)> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let value: serde_json::Value = serde_json::from_str(value_json)
        .map_err(|e| bad(format!("invalid value `{value_json}`: {e}")))?;

    let mut container = "the root".to_string();
    let mut segment_name = "the root";
    let mut cursor = &mut root;
    for segment in pointer.split('.') {
        cursor = match cursor.as_object_mut() {
            Some(map) => map
                .entry(segment)
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new())),
            None => return Err(bad(format!("{container} is not a JSON object"))),
        };
        container = format!("`{segment}`");
        segment_name = segment;
    }
    // The walk mints an empty object for a missing final segment; the pointer
    // names an array, so turn that placeholder into one.
    if cursor.as_object().is_some_and(serde_json::Map::is_empty) {
        *cursor = serde_json::Value::Array(Vec::new());
    }
    let Some(items) = cursor.as_array_mut() else {
        return Err(bad(format!("`{segment_name}` is not a JSON array")));
    };
    match items
        .iter_mut()
        .find(|item| item.to_string().contains(marker))
    {
        Some(item) => *item = value.clone(),
        None => items.push(value.clone()),
    }

    let mut content = serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
    content.push('\n');
    Ok((content, value.to_string()))
}

/// Split a lock-style pointer into dotted segments and the optional trailing
/// `[marker]` naming an array element.
pub(crate) fn parse_pointer(pointer: &str) -> (Vec<&str>, Option<&str>) {
    match pointer.split_once('[') {
        Some((dotted, rest)) => (
            dotted.split('.').collect(),
            Some(rest.strip_suffix(']').unwrap_or(rest)),
        ),
        None => (pointer.split('.').collect(), None),
    }
}

/// The canonical value text at `pointer`: the object key's value, or the
/// array element whose serialised form contains the marker — the same rule
/// `edit_json_array_element` matches by. `Ok(None)` when absent.
pub(crate) fn json_value_at(path: &str, json: &str, pointer: &str) -> Result<Option<String>> {
    let root: serde_json::Value = serde_json::from_str(json).map_err(|e| Error::Toml {
        path: path.into(),
        message: e.to_string(),
    })?;
    let (segments, marker) = parse_pointer(pointer);
    let mut cursor = &root;
    for segment in segments {
        match cursor.get(segment) {
            Some(next) => cursor = next,
            None => return Ok(None),
        }
    }
    let value = match marker {
        None => Some(cursor),
        Some(marker) => cursor
            .as_array()
            .and_then(|items| items.iter().find(|item| item.to_string().contains(marker))),
    };
    Ok(value.map(ToString::to_string))
}

/// Remove the entry `pointer` names. Returns the new file content and the
/// removed canonical value; `Ok(None)` when absent. Empty parents stay.
pub(crate) fn remove_json_pointer(
    path: &str,
    json: &str,
    pointer: &str,
) -> Result<Option<(String, String)>> {
    let bad = |message: String| Error::Toml {
        path: path.into(),
        message,
    };
    let mut root: serde_json::Value = serde_json::from_str(json).map_err(|e| bad(e.to_string()))?;
    let (mut segments, marker) = parse_pointer(pointer);
    let last = if marker.is_none() {
        segments.pop()
    } else {
        None
    };
    let mut cursor = &mut root;
    for segment in segments {
        match cursor.get_mut(segment) {
            Some(next) => cursor = next,
            None => return Ok(None),
        }
    }
    let removed = match (last, marker) {
        (Some(key), None) => cursor.as_object_mut().and_then(|map| map.remove(key)),
        (_, Some(marker)) => cursor.as_array_mut().and_then(|items| {
            let index = items
                .iter()
                .position(|item| item.to_string().contains(marker))?;
            Some(items.remove(index))
        }),
        (None, None) => None,
    };
    Ok(removed.map(|value| {
        let mut content =
            serde_json::to_string_pretty(&root).expect("a parsed value re-serialises");
        content.push('\n');
        (content, value.to_string())
    }))
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::Io {
            path: parent.into(),
            source: e,
        })?;
    }
    fs::write(path, content).map_err(|e| Error::Io {
        path: path.into(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, Ownership};
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::{FakeRunner, Output};

    fn write_owned(path: &str) -> Action {
        Action::WriteFile {
            path: path.into(),
            content: "content".into(),
            ownership: Ownership::Owned,
            reason: "test".into(),
        }
    }

    #[test]
    fn applies_writes_and_updates_lock() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Knowledge),
            provider: "aokf".into(),
            actions: vec![write_owned("a/b.txt")],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a/b.txt")).unwrap(),
            "content"
        );
        assert!(lock.files.contains_key("a/b.txt"));
        assert_eq!(lock.components["knowledge"].provider, "aokf");
    }

    #[test]
    fn groups_mise_pins_and_installs_once() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![
            Planned {
                capability: Some(crate::capability::Capability::Workflows),
                provider: "superpowers".into(),
                actions: vec![Action::SetMisePin {
                    tool: "http:superpowers".into(),
                    value_toml: "\"6.2.0\"".into(),
                }],
            },
            Planned {
                capability: Some(crate::capability::Capability::CodeIndex),
                provider: "codegraph".into(),
                actions: vec![
                    Action::SetMisePin {
                        tool: "npm:codegraph".into(),
                        value_toml: "\"1.0.0\"".into(),
                    },
                    Action::Run {
                        program: "codegraph".into(),
                        args: vec!["init".into()],
                        purpose: "build the code index".into(),
                        undo: None,
                        optional: false,
                    },
                ],
            },
        ];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let mise = std::fs::read_to_string(dir.path().join(".mise.toml")).unwrap();
        assert!(mise.contains("http:superpowers"));
        assert!(mise.contains("npm:codegraph"));
        let calls = fake.calls();
        // One install, naming every pin this run wrote — and nothing else.
        assert_eq!(
            calls
                .iter()
                .filter(|c| c.starts_with("mise install"))
                .collect::<Vec<_>>(),
            vec!["mise install http:superpowers npm:codegraph"]
        );
        // Providers need their pinned tools installed before they run, and
        // mise will not install from a config it has not been told to trust.
        let trust = calls.iter().position(|c| c == "mise trust").unwrap();
        let install = calls
            .iter()
            .position(|c| c.starts_with("mise install"))
            .unwrap();
        let init = calls.iter().position(|c| c == "codegraph init").unwrap();
        assert_eq!(trust + 1, install, "calls: {calls:?}");
        assert!(
            install < init,
            "mise install must precede provider commands"
        );
        // Both pins are hashed into the lock under their mise keys.
        assert!(
            lock.files
                .contains_key(&crate::components::mise::pin_lock_key("http:superpowers"))
        );
    }

    #[test]
    fn committed_pins_are_installed_before_a_run() {
        let dir = tempfile::tempdir().unwrap();
        // A fresh clone: the pin is already committed, so nothing edits it.
        std::fs::write(
            dir.path().join(".mise.toml"),
            mise::set_pin("", "http:superpowers", "{ version = \"6.2.0\" }").unwrap(),
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "superpowers".into(),
            actions: vec![Action::Run {
                program: "claude".into(),
                args: vec!["plugin".into(), "install".into(), "superpowers".into()],
                purpose: "install".into(),
                undo: None,
                optional: true,
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let calls = fake.calls();
        let install = calls
            .iter()
            .position(|c| c == "mise install http:superpowers")
            .unwrap_or_else(|| panic!("no targeted `mise install` in {calls:?}"));
        let trust = calls
            .iter()
            .position(|c| c == "mise trust")
            .unwrap_or_else(|| panic!("no `mise trust` in {calls:?}"));
        let plugin = calls
            .iter()
            .position(|c| c == "claude plugin install superpowers")
            .unwrap();
        assert_eq!(trust + 1, install, "calls: {calls:?}");
        assert!(install < plugin, "calls: {calls:?}");
        // The entry that runs the command carries both steps in its report.
        let described: Vec<&str> = result.reports[0]
            .outcomes
            .iter()
            .map(|(d, _)| d.as_str())
            .collect();
        assert!(
            described
                .iter()
                .any(|d| d.contains("mise trust") && d.contains("trust the repo's mise config")),
            "outcomes: {described:?}"
        );
        assert!(
            described.iter().any(|d| d.contains("mise install")),
            "outcomes: {described:?}"
        );
    }

    #[test]
    fn install_never_names_the_repos_own_tools() {
        let dir = tempfile::tempdir().unwrap();
        // A repo with a rich toolchain of its own, one entry of which cannot
        // build on this machine. Installing it is not superdev's business.
        std::fs::write(
            dir.path().join(".mise.toml"),
            "[tools]\nnode = \"24.15\"\n\"cargo:cargo-ndk\" = '4.1.2'\n",
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "superpowers".into(),
            actions: vec![Action::SetMisePin {
                tool: "http:superpowers".into(),
                value_toml: "\"6.2.0\"".into(),
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let installs: Vec<String> = fake
            .calls()
            .into_iter()
            .filter(|c| c.starts_with("mise install"))
            .collect();
        assert_eq!(
            installs,
            vec!["mise install http:superpowers"],
            "superdev installs its own pins, never the repo's"
        );
        // The user's tools stay in the file, untouched and uninstalled.
        let mise = std::fs::read_to_string(dir.path().join(".mise.toml")).unwrap();
        assert!(mise.contains("cargo:cargo-ndk"));
    }

    #[test]
    fn a_failed_trust_stops_the_run_and_unwinds_the_pin_edit() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "mise trust",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "not trusted".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "superpowers".into(),
            actions: vec![
                Action::SetMisePin {
                    tool: "http:superpowers".into(),
                    value_toml: "\"6.2.0\"".into(),
                },
                Action::Run {
                    program: "claude".into(),
                    args: vec!["plugin".into(), "install".into(), "superpowers".into()],
                    purpose: "install".into(),
                    undo: None,
                    optional: false,
                },
            ],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        let calls = fake.calls();
        assert_eq!(calls, vec!["mise trust"], "install must not follow");
        // The pin edit is taken back, exactly as a failed install would.
        assert!(!dir.path().join(".mise.toml").exists());
        assert!(result.reverted.iter().any(|r| r.contains(".mise.toml")));
        assert!(matches!(
            result.reports[0].outcomes.last().unwrap(),
            (d, ActionOutcome::Failed(e)) if d.contains("mise trust") && e.contains("not trusted")
        ));
        assert!(lock.files.is_empty());
    }

    #[test]
    fn nothing_to_install_without_runs_or_managed_pins() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        // A run, but no `.mise.toml` at all.
        let run = Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![Action::Run {
                program: "claude".into(),
                args: vec!["plugin".into(), "list".into()],
                purpose: "list".into(),
                undo: None,
                optional: true,
            }],
        };
        assert!(
            apply(
                dir.path(),
                &fake,
                &manifest,
                std::slice::from_ref(&run),
                &mut lock
            )
            .ok
        );
        assert!(
            !fake
                .calls()
                .iter()
                .any(|c| c == "mise install" || c == "mise trust")
        );

        // A `.mise.toml` pinning only the user's own tools.
        std::fs::write(dir.path().join(".mise.toml"), "[tools]\nnode = \"24\"\n").unwrap();
        assert!(apply(dir.path(), &fake, &manifest, &[run], &mut lock).ok);
        assert!(
            !fake
                .calls()
                .iter()
                .any(|c| c == "mise install" || c == "mise trust")
        );

        // A managed pin, but nothing to run.
        std::fs::write(
            dir.path().join(".mise.toml"),
            mise::set_pin("", "npm:@colbymchenry/codegraph", "\"1.5.0\"").unwrap(),
        )
        .unwrap();
        let write_only = Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![write_owned("a.txt")],
        };
        assert!(apply(dir.path(), &fake, &manifest, &[write_only], &mut lock).ok);
        assert!(
            !fake
                .calls()
                .iter()
                .any(|c| c == "mise install" || c == "mise trust")
        );
    }

    #[test]
    fn failure_unwinds_earlier_writes() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "codegraph init",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "no node".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![
            Planned {
                capability: Some(crate::capability::Capability::Knowledge),
                provider: "aokf".into(),
                actions: vec![write_owned("created.txt")],
            },
            Planned {
                capability: Some(crate::capability::Capability::CodeIndex),
                provider: "codegraph".into(),
                actions: vec![Action::Run {
                    program: "codegraph".into(),
                    args: vec!["init".into()],
                    purpose: "index".into(),
                    undo: None,
                    optional: false,
                }],
            },
        ];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(
            !dir.path().join("created.txt").exists(),
            "created file must be deleted on unwind"
        );
        assert!(result.reverted.iter().any(|r| r.contains("created.txt")));
    }

    #[test]
    fn optional_run_with_missing_program_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.missing("claude");
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Frontend),
            provider: "frontend-design".into(),
            actions: vec![Action::Run {
                program: "claude".into(),
                args: vec![
                    "plugin".into(),
                    "install".into(),
                    "frontend-design@claude-code".into(),
                ],
                purpose: "install".into(),
                undo: None,
                optional: true,
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert!(matches!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Skipped(_)
        ));
    }

    #[test]
    fn mise_where_placeholder_is_resolved() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "mise where http:superpowers",
            Output {
                status: 0,
                stdout: "/tmp/sp\n".into(),
                stderr: String::new(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "superpowers".into(),
            actions: vec![Action::Run {
                program: "claude".into(),
                args: vec![
                    "plugin".into(),
                    "marketplace".into(),
                    "add".into(),
                    "{mise-where:http:superpowers}".into(),
                ],
                purpose: "register".into(),
                undo: None,
                optional: true,
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert!(
            fake.calls()
                .contains(&"claude plugin marketplace add /tmp/sp".to_string())
        );
    }

    #[test]
    fn malformed_mise_tools_key_fails_without_panicking() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mise.toml"), "tools = 3\n").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::SetMisePin {
                tool: "npm:codegraph".into(),
                value_toml: "\"1.0.0\"".into(),
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        let (_, outcome) = &result.reports[0].outcomes[0];
        let ActionOutcome::Failed(message) = outcome else {
            panic!("expected a failure, got {outcome:?}");
        };
        // The discriminating text: an Io error would also name the file.
        assert_eq!(message, ".mise.toml: `tools` is not a table");
        // The malformed file is left exactly as the user wrote it.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".mise.toml")).unwrap(),
            "tools = 3\n"
        );
        assert!(lock.files.is_empty());
    }

    /// The registration the aokf provider plans, used by the JSON tests.
    fn set_mcp_key() -> Action {
        Action::SetJsonKey {
            path: ".mcp.json".into(),
            pointer: "mcpServers.superdev-aokf".into(),
            value_json: r#"{"command":"superdev","args":["mcp","aokf"]}"#.into(),
        }
    }

    #[test]
    fn set_json_key_merges_and_preserves_other_servers() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".mcp.json"),
            "{\n  \"mcpServers\": { \"other\": { \"command\": \"othersrv\" } },\n  \"extra\": true\n}\n",
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Knowledge),
            provider: "aokf".into(),
            actions: vec![set_mcp_key()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let text = std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap();
        assert!(text.ends_with("}\n"), "{text}");
        let written: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(written["mcpServers"]["other"]["command"], "othersrv");
        assert_eq!(written["extra"], true);
        assert_eq!(
            written["mcpServers"]["superdev-aokf"]["command"],
            "superdev"
        );
        assert_eq!(written["mcpServers"]["superdev-aokf"]["args"][1], "aokf");
        assert!(
            lock.files
                .contains_key(".mcp.json:mcpServers.superdev-aokf"),
            "lock: {:?}",
            lock.files
        );
    }

    #[test]
    fn set_json_key_creates_the_file_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Knowledge),
            provider: "aokf".into(),
            actions: vec![set_mcp_key()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let written: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(
            written["mcpServers"]["superdev-aokf"],
            serde_json::json!({ "command": "superdev", "args": ["mcp", "aokf"] })
        );
    }

    #[test]
    fn malformed_mcp_json_fails_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mcp.json"), "not json\n").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Knowledge),
            provider: "aokf".into(),
            actions: vec![write_owned("created.txt"), set_mcp_key()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        let (description, outcome) = &result.reports[0].outcomes[1];
        assert_eq!(description, "set mcpServers.superdev-aokf in .mcp.json");
        let ActionOutcome::Failed(message) = outcome else {
            panic!("expected a failure, got {outcome:?}");
        };
        assert!(message.starts_with(".mcp.json:"), "{message}");
        // The user's file is left exactly as they wrote it, and the earlier
        // write in the same run is taken back.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
            "not json\n"
        );
        assert!(!dir.path().join("created.txt").exists());
        assert!(result.reverted.iter().any(|r| r.contains("created.txt")));
        assert!(lock.files.is_empty());
    }

    #[test]
    fn a_json_key_path_through_a_non_object_fails() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mcp.json"), "{ \"mcpServers\": 3 }").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Knowledge),
            provider: "aokf".into(),
            actions: vec![set_mcp_key()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        let (_, outcome) = &result.reports[0].outcomes[0];
        let ActionOutcome::Failed(message) = outcome else {
            panic!("expected a failure, got {outcome:?}");
        };
        assert_eq!(message, ".mcp.json: `mcpServers` is not a JSON object");
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
            "{ \"mcpServers\": 3 }"
        );

        // A file whose root is not an object at all.
        std::fs::write(dir.path().join(".mcp.json"), "[]").unwrap();
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        let (_, ActionOutcome::Failed(message)) = &result.reports[0].outcomes[0] else {
            panic!("expected a failure");
        };
        assert_eq!(message, ".mcp.json: the root is not a JSON object");
    }

    /// The hook registration the skills provider plans, used by the array tests.
    fn ensure_hook() -> Action {
        Action::EnsureJsonArrayElement {
            path: ".claude/settings.json".into(),
            pointer: "hooks.PostToolUse".into(),
            marker: "superdev aokf hook validate".into(),
            value_json: r#"{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}"#.into(),
        }
    }

    #[test]
    fn ensure_array_element_appends_and_preserves_user_entries() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            "{\n  \"hooks\": { \"PostToolUse\": [ { \"matcher\": \"Agent\", \"hooks\": [] } ] },\n  \"permissions\": { \"deny\": [] }\n}\n",
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let text = std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap();
        let written: serde_json::Value = serde_json::from_str(&text).unwrap();
        let entries = written["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0]["matcher"], "Agent");
        assert_eq!(
            entries[1]["hooks"][0]["command"],
            "superdev aokf hook validate"
        );
        assert!(written["permissions"].is_object());
        assert!(
            lock.files.contains_key(
                ".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]"
            ),
            "lock: {:?}",
            lock.files
        );
    }

    #[test]
    fn ensure_array_element_replaces_a_stale_superdev_entry() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        // A prior release's entry: same marker, different matcher.
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#,
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let written: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
        )
        .unwrap();
        let entries = written["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(entries.len(), 1, "replaced, not duplicated");
        assert_eq!(entries[0]["matcher"], "Edit|Write");
    }

    #[test]
    fn ensure_array_element_creates_the_file_and_path_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let written: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            written["hooks"]["PostToolUse"][0]["hooks"][0]["command"],
            "superdev aokf hook validate"
        );
    }

    #[test]
    fn ensure_array_element_rejects_a_non_array_pointer() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(".claude/settings.json"),
            r#"{"hooks":{"PostToolUse":{"matcher":"Edit"}}}"#,
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        let (_, ActionOutcome::Failed(message)) = &result.reports[0].outcomes[0] else {
            panic!("expected a failure");
        };
        assert_eq!(
            message,
            ".claude/settings.json: `PostToolUse` is not a JSON array"
        );
        // The user's file is left exactly as they wrote it.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
            r#"{"hooks":{"PostToolUse":{"matcher":"Edit"}}}"#
        );
        assert!(lock.files.is_empty());
    }

    #[test]
    fn ensure_array_element_on_a_malformed_file_fails_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(dir.path().join(".claude/settings.json"), "not json\n").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![ensure_hook()],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
            "not json\n"
        );
    }

    #[test]
    fn pointers_parse_navigate_and_remove() {
        assert_eq!(parse_pointer("a.b"), (vec!["a", "b"], None));
        assert_eq!(
            parse_pointer("hooks.PostToolUse[superdev aokf hook validate]"),
            (
                vec!["hooks", "PostToolUse"],
                Some("superdev aokf hook validate")
            )
        );

        let json = r#"{"mcpServers":{"superdev-aokf":{"command":"superdev"},"mine":{}}}"#;
        let value = json_value_at("f", json, "mcpServers.superdev-aokf")
            .unwrap()
            .unwrap();
        assert!(value.contains("superdev"));
        assert_eq!(json_value_at("f", json, "mcpServers.gone").unwrap(), None);
        assert!(json_value_at("f", "not json", "a").is_err());

        let (content, removed) = remove_json_pointer("f", json, "mcpServers.superdev-aokf")
            .unwrap()
            .unwrap();
        assert!(removed.contains("superdev"));
        let root: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(root["mcpServers"].get("superdev-aokf").is_none());
        // The user's key and the (possibly emptied) parent survive.
        assert!(root["mcpServers"].get("mine").is_some());
        assert_eq!(
            remove_json_pointer("f", json, "mcpServers.gone").unwrap(),
            None
        );

        let hooks = r#"{"hooks":{"PostToolUse":[{"matcher":"Agent","hooks":[]},{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#;
        let pointer = "hooks.PostToolUse[superdev aokf hook validate]";
        assert!(
            json_value_at("f", hooks, pointer)
                .unwrap()
                .unwrap()
                .contains("Edit|Write")
        );
        let (content, _) = remove_json_pointer("f", hooks, pointer).unwrap().unwrap();
        let root: serde_json::Value = serde_json::from_str(&content).unwrap();
        let items = root["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(items.len(), 1, "only superdev's element goes");
        assert_eq!(items[0]["matcher"], "Agent");
    }

    #[test]
    fn plan_runs_every_component() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = crate::component::Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let components = crate::components::enabled(&manifest).unwrap();
        let planned = plan(&components, &ctx).unwrap();
        assert_eq!(planned.len(), components.len());
        assert_eq!(planned[0].provider, "mattpocock-skills");
        assert!(planned.iter().all(|p| p.capability.is_some()));
        assert!(planned.iter().any(|p| !p.actions.is_empty()));

        // A component that fails to plan aborts the whole plan.
        let mut broken = Manifest::default_for("0.1.0", &[]);
        broken.capabilities.get_mut("workflows").unwrap().version = Some("9.9.9".into());
        let ctx = crate::component::Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &broken,
            lock: &lock,
        };
        assert!(plan(&crate::components::enabled(&broken).unwrap(), &ctx).is_err());
    }

    #[test]
    fn satisfied_scaffold_and_line_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "mine").unwrap();
        std::fs::write(dir.path().join(".gitignore"), "target\n.superdev/cache/\n").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![
                Action::WriteFile {
                    path: "AGENTS.md".into(),
                    content: "blueprint".into(),
                    ownership: Ownership::Scaffold,
                    reason: "entry point".into(),
                },
                Action::EnsureLine {
                    path: ".gitignore".into(),
                    line: ".superdev/cache/".into(),
                    reason: "ignore machine state".into(),
                },
            ],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Skipped("exists".into())
        );
        assert_eq!(
            result.reports[0].outcomes[1].1,
            ActionOutcome::Skipped("present".into())
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
            "mine"
        );
        // A repo-level entry owns no capability, so the lock records none.
        assert!(lock.components.is_empty());
    }

    #[test]
    fn ensure_line_appends_and_creates() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "first").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let line = |path: &str| Action::EnsureLine {
            path: path.into(),
            line: "added".into(),
            reason: "test".into(),
        };
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![line("a.txt"), line("b.txt")],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "first\nadded\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("b.txt")).unwrap(),
            "added\n"
        );
    }

    #[test]
    fn owned_overwrite_backs_up_and_notes_user_edits() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("owned.txt"), "edited by hand").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Knowledge),
            provider: "aokf".into(),
            actions: vec![write_owned("owned.txt")],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Applied {
                note: Some("overwrote a user-edited file (backed up)".into())
            }
        );
        let backups = std::fs::read_dir(dir.path().join(".superdev/cache/backup"))
            .unwrap()
            .map(|e| e.unwrap().path().join("owned.txt"))
            .collect::<Vec<_>>();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            std::fs::read_to_string(&backups[0]).unwrap(),
            "edited by hand"
        );

        // Re-applying over superdev's own content is not a user edit.
        std::fs::write(dir.path().join("owned.txt"), "stale").unwrap();
        lock.files
            .insert("owned.txt".into(), crate::lock::sha256_hex(b"stale"));
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Applied { note: None }
        );
    }

    #[test]
    fn unwind_restores_content_runs_undo_and_reports_leftovers() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("owned.txt"), "before").unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "codegraph init",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "no node".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![
            Planned {
                capability: Some(crate::capability::Capability::Workflows),
                provider: "superpowers".into(),
                actions: vec![
                    Action::SetMisePin {
                        tool: "http:superpowers".into(),
                        value_toml: "\"6.2.0\"".into(),
                    },
                    write_owned("owned.txt"),
                    Action::Run {
                        program: "claude".into(),
                        args: vec!["plugin".into(), "marketplace".into(), "add".into()],
                        purpose: "register".into(),
                        undo: None,
                        optional: true,
                    },
                    Action::Run {
                        program: "claude".into(),
                        args: vec!["plugin".into(), "install".into(), "superpowers".into()],
                        purpose: "install".into(),
                        undo: Some((
                            "claude".into(),
                            vec!["plugin".into(), "uninstall".into(), "superpowers".into()],
                        )),
                        optional: true,
                    },
                ],
            },
            Planned {
                capability: Some(crate::capability::Capability::CodeIndex),
                provider: "codegraph".into(),
                actions: vec![Action::Run {
                    program: "codegraph".into(),
                    args: vec!["init".into()],
                    purpose: "index".into(),
                    undo: None,
                    optional: false,
                }],
            },
        ];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("owned.txt")).unwrap(),
            "before"
        );
        assert!(!dir.path().join(".mise.toml").exists(), "pin file removed");
        assert!(
            fake.calls()
                .contains(&"claude plugin uninstall superpowers".to_string())
        );
        assert!(result.reverted.iter().any(|r| r.contains("owned.txt")));
        assert!(result.reverted.iter().any(|r| r.contains("uninstall")));
        // The install and the marketplace registration cannot be undone.
        assert!(
            result
                .not_reverted
                .iter()
                .any(|r| r.contains("mise install"))
        );
        assert!(
            result
                .not_reverted
                .iter()
                .any(|r| r.contains("marketplace"))
        );
        // The first entry completed, so its hashes are staged in the lock —
        // which the caller discards, because the run is not ok.
        assert!(lock.files.contains_key("owned.txt"));
    }

    #[test]
    fn mise_install_failure_fails_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "mise install",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "no network".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::SetMisePin {
                tool: "npm:codegraph".into(),
                value_toml: "\"1.0.0\"".into(),
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(!dir.path().join(".mise.toml").exists());
        assert!(matches!(
            result.reports[0].outcomes.last().unwrap(),
            (d, ActionOutcome::Failed(_)) if d.contains("mise install")
        ));
    }

    #[test]
    fn missing_mise_stops_placeholder_resolution() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.missing("mise");
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let run = |optional| Action::Run {
            program: "claude".into(),
            args: vec!["add".into(), "{mise-where:http:superpowers}".into()],
            purpose: "register".into(),
            undo: None,
            optional,
        };
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "superpowers".into(),
            actions: vec![run(true)],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        let ActionOutcome::Skipped(reason) = &result.reports[0].outcomes[0].1 else {
            panic!("expected a skip");
        };
        assert!(reason.starts_with("mise not installed"), "{reason}");

        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "superpowers".into(),
            actions: vec![run(false)],
        }];
        assert!(!apply(dir.path(), &fake, &manifest, &planned, &mut lock).ok);
    }

    #[test]
    fn failing_mise_where_is_a_failure() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "mise where",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "not installed".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "superpowers".into(),
            actions: vec![Action::Run {
                program: "claude".into(),
                args: vec!["add".into(), "{mise-where:http:superpowers}".into()],
                purpose: "register".into(),
                undo: None,
                optional: true,
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
    }

    #[test]
    fn unwritable_target_fails_the_action() {
        let dir = tempfile::tempdir().unwrap();
        // A file where a parent directory must go: the write cannot succeed.
        std::fs::write(dir.path().join("blocked"), "").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![write_owned("blocked/child.txt")],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        // Repo-level entries label as `repo (provider)`, matching plan output.
        assert_eq!(result.reports[0].label, "repo (superdev)");
        assert!(matches!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Failed(_)
        ));
    }

    #[test]
    fn non_utf8_target_is_not_clobbered() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("bin.dat"), [0xff, 0xfe]).unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![write_owned("bin.dat")],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert_eq!(
            std::fs::read(dir.path().join("bin.dat")).unwrap(),
            [0xff, 0xfe]
        );
    }

    #[test]
    fn remove_file_backs_up_journals_and_releases_the_lock_key() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.txt"), "superdev content").unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        lock.files
            .insert("old.txt".into(), sha256_hex(b"superdev content"));
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![Action::RemoveFile {
                path: "old.txt".into(),
                reason: "no longer in the blueprint".into(),
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert!(!dir.path().join("old.txt").exists());
        assert!(!lock.files.contains_key("old.txt"));
        let backups: Vec<_> = std::fs::read_dir(dir.path().join(BACKUP_DIR))
            .unwrap()
            .map(|e| e.unwrap().path().join("old.txt"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(
            std::fs::read_to_string(&backups[0]).unwrap(),
            "superdev content"
        );
    }

    #[test]
    fn remove_file_skips_the_gone_and_the_user_changed() {
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let remove = |path: &str| Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![Action::RemoveFile {
                path: path.into(),
                reason: "no longer in the blueprint".into(),
            }],
        };
        // Already gone: skipped, key released.
        let mut lock = Lock::default();
        lock.files.insert("gone.txt".into(), sha256_hex(b"x"));
        let result = apply(
            dir.path(),
            &fake,
            &manifest,
            &[remove("gone.txt")],
            &mut lock,
        );
        assert!(result.ok);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Skipped("already gone".into())
        );
        assert!(!lock.files.contains_key("gone.txt"));
        // Changed since planning: left in place, key released.
        std::fs::write(dir.path().join("mine.txt"), "edited by hand").unwrap();
        let mut lock = Lock::default();
        lock.files
            .insert("mine.txt".into(), sha256_hex(b"superdev content"));
        let result = apply(
            dir.path(),
            &fake,
            &manifest,
            &[remove("mine.txt")],
            &mut lock,
        );
        assert!(result.ok);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Skipped("changed since superdev wrote it — left in place".into())
        );
        assert!(dir.path().join("mine.txt").exists());
        assert!(!lock.files.contains_key("mine.txt"));
    }

    #[test]
    fn remove_mise_pin_and_json_key_rewrite_only_their_entry() {
        let dir = tempfile::tempdir().unwrap();
        let mise_toml =
            mise::set_pin("[tools]\nnode = \"24\"\n", "http:codegraph", "\"1.5.0\"").unwrap();
        std::fs::write(dir.path().join(".mise.toml"), &mise_toml).unwrap();
        std::fs::write(
            dir.path().join(".mcp.json"),
            r#"{"mcpServers":{"superdev-aokf":{"command":"superdev","args":["mcp","aokf"]},"mine":{"command":"me"}}}"#,
        )
        .unwrap();
        let fake = FakeRunner::new();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let pin_value = mise::current_pin(&mise_toml, "http:codegraph")
            .unwrap()
            .unwrap();
        lock.files.insert(
            mise::pin_lock_key("http:codegraph"),
            sha256_hex(pin_value.as_bytes()),
        );
        let mcp_value: serde_json::Value =
            serde_json::from_str(r#"{"command":"superdev","args":["mcp","aokf"]}"#).unwrap();
        lock.files.insert(
            ".mcp.json:mcpServers.superdev-aokf".into(),
            sha256_hex(mcp_value.to_string().as_bytes()),
        );
        let planned = vec![Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![
                Action::RemoveMisePin {
                    tool: "http:codegraph".into(),
                },
                Action::RemoveJsonKey {
                    path: ".mcp.json".into(),
                    pointer: "mcpServers.superdev-aokf".into(),
                },
            ],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        let mise_after = std::fs::read_to_string(dir.path().join(".mise.toml")).unwrap();
        assert_eq!(
            mise::current_pin(&mise_after, "http:codegraph").unwrap(),
            None
        );
        assert!(mise_after.contains("node = \"24\""));
        let mcp: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert!(mcp["mcpServers"].get("superdev-aokf").is_none());
        assert_eq!(mcp["mcpServers"]["mine"]["command"], "me");
        assert!(lock.files.is_empty());
        // No installs follow a removal.
        assert!(fake.calls().is_empty());
    }

    #[test]
    fn a_later_failure_restores_a_removed_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.txt"), "superdev content").unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "codegraph init",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "boom".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        lock.files
            .insert("old.txt".into(), sha256_hex(b"superdev content"));
        let planned = vec![
            Planned {
                capability: None,
                provider: "superdev".into(),
                actions: vec![Action::RemoveFile {
                    path: "old.txt".into(),
                    reason: "no longer in the blueprint".into(),
                }],
            },
            Planned {
                capability: Some(crate::capability::Capability::CodeIndex),
                provider: "codegraph".into(),
                actions: vec![Action::Run {
                    program: "codegraph".into(),
                    args: vec!["init".into()],
                    purpose: "index".into(),
                    undo: None,
                    optional: false,
                }],
            },
        ];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("old.txt")).unwrap(),
            "superdev content"
        );
        assert!(result.reverted.iter().any(|r| r.contains("old.txt")));
    }

    /// A fake checkout: skills/engineering/{alpha,beta}, skills/productivity/gamma.
    /// alpha has a nested reference file; a stray README sits beside the dirs.
    fn fake_checkout() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let write = |rel: &str, content: &str| {
            let p = dir.path().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        };
        write("skills/engineering/alpha/SKILL.md", "alpha v1");
        write("skills/engineering/alpha/refs/DEEP.md", "alpha deep");
        write("skills/engineering/beta/SKILL.md", "beta v1");
        write("skills/engineering/README.md", "not a skill");
        write("skills/productivity/gamma/SKILL.md", "gamma v1");
        dir
    }

    fn materialise_action(custom: &[&str]) -> Action {
        Action::MaterialiseSkills {
            tool: "http:mattpocock-skills".into(),
            source_dirs: vec!["skills/engineering".into(), "skills/productivity".into()],
            custom: custom.iter().map(|c| (*c).to_string()).collect(),
        }
    }

    fn where_scripted(checkout: &std::path::Path) -> FakeRunner {
        let fake = FakeRunner::new();
        fake.script(
            "mise where http:mattpocock-skills",
            Output {
                status: 0,
                stdout: format!("{}\n", checkout.display()),
                stderr: String::new(),
            },
        );
        fake
    }

    #[test]
    fn materialise_writes_locks_and_attributes() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/alpha/refs/DEEP.md")).unwrap(),
            "alpha deep"
        );
        assert!(dir.path().join(".claude/skills/gamma/SKILL.md").exists());
        // The stray README is not a skill directory and is not copied.
        assert!(!dir.path().join(".claude/skills/README.md").exists());
        let key = ".claude/skills/alpha/SKILL.md";
        assert_eq!(lock.files[key], sha256_hex(b"alpha v1"));
        assert_eq!(lock.owners[key], "workflows");
        assert_eq!(lock.owners.len(), 4);
    }

    #[test]
    fn materialise_reconciles_dropped_skills_and_skips_custom() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        // A managed pin already committed, so the run has tools to install.
        std::fs::write(
            dir.path().join(".mise.toml"),
            mise::set_pin("", "http:superpowers", "\"6.2.0\"").unwrap(),
        )
        .unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        // A previously materialised skill the checkout no longer ships…
        std::fs::create_dir_all(dir.path().join(".claude/skills/old")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/old/SKILL.md"), "old v1").unwrap();
        // …and one the user edited since.
        std::fs::create_dir_all(dir.path().join(".claude/skills/mine")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/mine/SKILL.md"), "edited").unwrap();
        let mut lock = Lock::default();
        for (key, content) in [
            (".claude/skills/old/SKILL.md", "old v1"),
            (".claude/skills/mine/SKILL.md", "mine v1"),
        ] {
            lock.files
                .insert(key.into(), sha256_hex(content.as_bytes()));
            lock.owners.insert(key.into(), "workflows".into());
        }
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&["beta"])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        // alpha's two files and gamma's one; old swept, mine released.
        let materialised = result.reports[0]
            .outcomes
            .iter()
            .find(|(d, _)| d.starts_with("materialise"))
            .map(|(_, o)| o.clone())
            .unwrap_or_else(|| panic!("outcomes: {:?}", result.reports[0].outcomes));
        assert_eq!(
            materialised,
            ActionOutcome::Applied {
                note: Some("wrote 3, kept 0, removed 1, released 1".into())
            }
        );
        // Dropped and unmodified: deleted, with a backup.
        assert!(!dir.path().join(".claude/skills/old/SKILL.md").exists());
        // User-edited: left in place, released from the lock.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/mine/SKILL.md")).unwrap(),
            "edited"
        );
        assert!(!lock.files.contains_key(".claude/skills/old/SKILL.md"));
        assert!(!lock.files.contains_key(".claude/skills/mine/SKILL.md"));
        assert!(
            lock.owners
                .keys()
                .all(|k| !k.contains("/old/") && !k.contains("/mine/"))
        );
        // Custom skill: never written, never attributed.
        assert!(!dir.path().join(".claude/skills/beta").exists());
        assert!(!lock.files.keys().any(|k| k.contains("/beta/")));
        // The pinned tool is installed before the checkout is read.
        let calls = fake.calls();
        let install = calls
            .iter()
            .position(|c| c.starts_with("mise install"))
            .unwrap();
        let where_ = calls
            .iter()
            .position(|c| c.starts_with("mise where"))
            .unwrap();
        assert!(install < where_, "calls: {calls:?}");
    }

    #[test]
    fn materialise_failures_unwind() {
        // No checkout: `mise where` fails.
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "mise where http:mattpocock-skills",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "not installed".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(lock.files.is_empty());

        // Non-UTF-8 checkout content: fails and unwinds the files written first.
        let checkout = fake_checkout();
        std::fs::write(
            checkout.path().join("skills/productivity/gamma/SKILL.md"),
            [0xff, 0xfe],
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let fake = where_scripted(checkout.path());
        let mut lock = Lock::default();
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(
            !dir.path().join(".claude/skills/alpha/SKILL.md").exists(),
            "earlier writes must unwind"
        );
    }

    #[test]
    fn a_converged_materialise_rewrites_nothing() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let mut lock = Lock::default();
        let fake = where_scripted(checkout.path());
        assert!(apply(dir.path(), &fake, &manifest, &planned, &mut lock).ok);
        let fake = where_scripted(checkout.path());
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        // Every file was new on the first run and unchanged on the second, so
        // nothing was ever overwritten and no backup exists.
        let backups = std::fs::read_dir(dir.path().join(BACKUP_DIR))
            .map(|d| d.count())
            .unwrap_or(0);
        assert_eq!(backups, 0, "an unchanged file is never backed up");
        assert_eq!(lock.owners.len(), 4);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Applied {
                note: Some("wrote 0, kept 4, removed 0, released 0".into())
            }
        );
    }

    #[test]
    fn materialise_notes_the_user_edits_it_overwrote() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        // A skill file the user rewrote: the lock never saw this content.
        std::fs::create_dir_all(dir.path().join(".claude/skills/beta")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/beta/SKILL.md"), "mine").unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Applied {
                note: Some(
                    "wrote 4, kept 0, removed 0, released 0; overwrote 1 user-edited (backed up)"
                        .into()
                )
            }
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/beta/SKILL.md")).unwrap(),
            "beta v1"
        );
    }

    /// A provider switch in one run: the old skill pack's file is orphaned
    /// (the orphan entry is planned last, from pre-run claims) while the new
    /// provider materialises the very same path. The materialise must win.
    #[test]
    fn a_materialised_key_survives_the_same_runs_orphan_removal() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let key = ".claude/skills/alpha/SKILL.md";
        // The old pack's file, on disk and in the lock, unowned.
        std::fs::create_dir_all(dir.path().join(".claude/skills/alpha")).unwrap();
        std::fs::write(dir.path().join(key), "pack alpha").unwrap();
        let mut lock = Lock::default();
        lock.files.insert(key.into(), sha256_hex(b"pack alpha"));
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        let planned = vec![
            Planned {
                capability: Some(crate::capability::Capability::Workflows),
                provider: "mattpocock-skills".into(),
                actions: vec![materialise_action(&[])],
            },
            Planned {
                capability: None,
                provider: "orphan".into(),
                actions: vec![Action::RemoveFile {
                    path: key.into(),
                    reason: "no longer claimed".into(),
                }],
            },
        ];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        // The orphan still reports the skip: the file it saw is not the one
        // it planned against.
        assert_eq!(
            result.reports[1].outcomes[0].1,
            ActionOutcome::Skipped("changed since superdev wrote it — left in place".into())
        );
        // …but the fresh file stays managed.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(key)).unwrap(),
            "alpha v1"
        );
        assert_eq!(lock.files[key], sha256_hex(b"alpha v1"));
        assert_eq!(lock.owners[key], "workflows");
    }
}
