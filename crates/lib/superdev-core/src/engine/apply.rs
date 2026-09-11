//! engine/apply.rs — the plan runner: the appliers compute content and make
//! policy calls (skip, drift guard, backup); every actual side effect goes
//! through `tx::Tx`, which journals it so the first failure can unwind the
//! run instead of leaving the repo half-changed.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::action::{Action, Ownership};
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::{Error, Result};
use crate::fsutil::{has_line, read_text};
use crate::json_edit::{edit_json_array_element, edit_json_key};
use crate::lock::{Lock, LockedComponent, sha256_hex};
use crate::manifest::Manifest;
use crate::runner::CommandRunner;

use super::pins::PinEffects;
use super::tx::Tx;

/// Skip reason for a removal target that is already absent. Named, because
/// reconciliation reads it back to tell a swept file from a released one.
pub(super) const ALREADY_GONE: &str = "already gone";

/// One component's planned changes.
#[derive(Debug, Clone)]
pub struct Planned {
    /// None for actions no capability owns: the repo-level entry, and the
    /// core components — SOKF today.
    pub capability: Option<Capability>,
    /// Provider that produced these actions, or the core component's name.
    pub provider: String,
    /// Actions, in apply order.
    pub actions: Vec<Action>,
}

/// What the repo-level entry calls itself. Named here so the label can tell
/// it from a core component, which has no slot either.
pub const REPO_PROVIDER: &str = "superdev";

impl Planned {
    /// How this entry heads its section of a plan or an apply report.
    ///
    /// A capability names its slot and its provider, because a slot can be
    /// filled more than one way. The repo entry keeps its `repo (…)` form.
    /// A core component names itself alone: nothing competes for it, so
    /// there is no second name to disambiguate against.
    pub fn label(&self) -> String {
        match self.capability {
            Some(c) => format!("{} ({})", c.as_str(), self.provider),
            None if self.provider == REPO_PROVIDER => format!("repo ({})", self.provider),
            None => self.provider.clone(),
        }
    }
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
                capability: c.capability(),
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
    let (mut ok, mut pin_effects) = session.apply_pins(planned);
    if ok {
        for (index, entry) in planned.iter().enumerate() {
            // Each entry takes ownership of the pin effects it earned in the
            // pin phase; the lock learns about them only when the entry
            // completes.
            let effects = std::mem::take(&mut pin_effects[index]);
            if !session.apply_entry(index, entry, manifest, lock, effects) {
                ok = false;
                break;
            }
        }
    }
    let (reverted, not_reverted) = if ok {
        (Vec::new(), Vec::new())
    } else {
        session.tx.unwind(session.runner)
    };
    ApplyResult {
        reports: session.reports,
        reverted,
        not_reverted,
        ok,
    }
}

/// State carried through a single apply run.
pub(super) struct Session<'a> {
    pub(super) root: &'a Path,
    pub(super) runner: &'a dyn CommandRunner,
    /// The journal every side effect goes through.
    pub(super) tx: Tx<'a>,
    /// Locked hashes as of the start of the run, for user-edit detection.
    pub(super) prior_hashes: BTreeMap<String, String>,
    /// Lock attribution as of the start of the run, for reconciliation.
    /// Lock keys earlier entries wrote in this run. Removals were planned
    /// against the pre-run state, so a later one must not drop them.
    pub(super) written_keys: BTreeSet<String>,
    pub(super) reports: Vec<ComponentReport>,
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
            tx: Tx::new(root),
            prior_hashes: lock.files.clone(),
            written_keys: BTreeSet::new(),
            reports: planned
                .iter()
                .map(|p| ComponentReport {
                    label: p.label(),
                    outcomes: Vec::new(),
                })
                .collect(),
        }
    }

    pub(super) fn record(&mut self, entry: usize, action: &Action, outcome: ActionOutcome) {
        self.reports[entry]
            .outcomes
            .push((action.describe(), outcome));
    }

    /// Apply one entry's non-pin actions, then record what it applied.
    fn apply_entry(
        &mut self,
        index: usize,
        entry: &Planned,
        manifest: &Manifest,
        lock: &mut Lock,
        pin_effects: PinEffects,
    ) -> bool {
        let mut written = Vec::new();
        let mut removed: Vec<String> = Vec::new();
        for action in &entry.actions {
            let before = written.len();
            let outcome = match action {
                // Pins were applied as one grouped edit before any entry ran.
                Action::SetMisePin { .. } => continue,
                Action::WriteFile {
                    path,
                    content,
                    ownership,
                    ..
                } => self.write_action(path, content, *ownership, &mut written),
                Action::EnsureAgentInstructions { content } => {
                    self.ensure_agent_instructions(content)
                }
                Action::EnsureLine {
                    path,
                    line,
                    append_note,
                    ..
                } => self.ensure_line(path, line, append_note),
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
                Action::Run {
                    program,
                    args,
                    undo,
                    optional,
                    ..
                } => self.run_action(program, args, undo, *optional),
                Action::Remove { claim, .. } => self.remove_claim(claim, &mut removed),
            };
            // The backstop for collisions the planner cannot enumerate (a
            // checkout-derived path on a first sync): a key another entry
            // already wrote this run means two capabilities ship one path.
            // A real failure keeps its own message.
            let outcome = if matches!(outcome, ActionOutcome::Failed(_)) {
                outcome
            } else {
                match written[before..]
                    .iter()
                    .find(|(key, _)| self.written_keys.contains(key))
                {
                    Some((key, _)) => ActionOutcome::Failed(format!(
                        "collision: {key} was already written by an earlier capability this run — add its skill to one side's custom list, or upgrade superdev"
                    )),
                    None => outcome,
                }
            };
            let failed = matches!(outcome, ActionOutcome::Failed(_));
            self.record(index, action, outcome);
            if failed {
                return false;
            }
        }
        let keys: Vec<String> = pin_effects
            .hashes
            .into_iter()
            .chain(written)
            .map(|(key, hash)| {
                lock.files.insert(key.clone(), hash);
                key
            })
            .collect();
        for key in removed {
            // An earlier entry rewrote this file in this same run: the removal
            // was planned against the old state and would strand the fresh
            // file, unlocked and unowned, until some later run noticed.
            if self.written_keys.contains(&key) {
                continue;
            }
            lock.files.remove(&key);
        }
        self.written_keys.extend(keys);
        if let Some(capability) = entry.capability {
            let record = LockedComponent {
                provider: entry.provider.clone(),
                version: manifest
                    .config_of(capability, &entry.provider)
                    .and_then(|c| c.version.clone()),
            };
            let records = lock
                .components
                .entry(capability.as_str().to_string())
                .or_default();
            match records.iter_mut().find(|r| r.provider == entry.provider) {
                Some(existing) => *existing = record,
                None => records.push(record),
            }
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
        let existing = match read_text(&self.root.join(path)) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        if ownership == Ownership::Scaffold && existing.is_some() {
            return ActionOutcome::Skipped("exists".into());
        }
        let mut note = None;
        if let Some(old) = &existing {
            if let Err(e) = self.tx.backup(path, old) {
                return ActionOutcome::Failed(e.to_string());
            }
            if ownership == Ownership::Owned
                && self.prior_hashes.get(path) != Some(&sha256_hex(old.as_bytes()))
            {
                note = Some("overwrote a user-edited file (backed up)".to_string());
            }
        }
        if let Err(e) = self.tx.write(path, existing, content) {
            return ActionOutcome::Failed(e.to_string());
        }
        if ownership == Ownership::Owned {
            written.push((path.to_string(), sha256_hex(content.as_bytes())));
        }
        ActionOutcome::Applied { note }
    }

    fn ensure_agent_instructions(&mut self, instructions: &str) -> ActionOutcome {
        let path = crate::agent_file::PATH;
        let existing = match read_text(&self.root.join(path)) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        // Compose again at apply time so edits made after planning survive.
        let next = match crate::agent_file::render(
            existing.as_deref().unwrap_or_default(),
            instructions,
        ) {
            Ok(next) => next,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        if existing.as_deref() == Some(&next) {
            return ActionOutcome::Skipped("present".into());
        }
        // AGENTS.md remains a shared file: journal it, but do not claim or hash it.
        match self.tx.write(path, existing, &next) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    fn ensure_line(
        &mut self,
        path: &str,
        line: &str,
        append_note: &Option<String>,
    ) -> ActionOutcome {
        let existing = match read_text(&self.root.join(path)) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let mut next = existing.clone().unwrap_or_default();
        if has_line(&next, line) {
            return ActionOutcome::Skipped("present".into());
        }
        if !next.is_empty() && !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(line);
        next.push('\n');
        // The note fires only for appends to a pre-existing file: a fresh
        // create has nothing of the user's to talk about.
        let note = existing.is_some().then(|| append_note.clone()).flatten();
        match self.tx.write(path, existing, &next) {
            Ok(()) => ActionOutcome::Applied { note },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Take back a claimed entry the blueprint dropped — the drift guard's
    /// only home, for all three shapes. The lock key is released even when
    /// the removal is skipped: gone or user-changed, the entry is no longer
    /// superdev's.
    pub(super) fn remove_claim(
        &mut self,
        claim: &Claim,
        removed: &mut Vec<String>,
    ) -> ActionOutcome {
        let key = claim.lock_key();
        removed.push(key.clone());
        let file = claim.file_path();
        let content = match read_text(&self.root.join(file)) {
            Ok(Some(content)) => content,
            Ok(None) => return ActionOutcome::Skipped(ALREADY_GONE.into()),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let value = match claim.value_in(&content) {
            Ok(Some(value)) => value,
            Ok(None) => return ActionOutcome::Skipped(ALREADY_GONE.into()),
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        // Re-check at apply time: an edit between plan and apply is the
        // user's, and superdev takes back only what it wrote.
        if self.prior_hashes.get(&key) != Some(&sha256_hex(value.as_bytes())) {
            return ActionOutcome::Skipped(
                "changed since superdev wrote it — left in place".into(),
            );
        }
        match claim.removed_from(&content) {
            Err(e) => ActionOutcome::Failed(e.to_string()),
            // The claim is the whole file: back it up, then delete.
            Ok(None) => {
                if let Err(e) = self.tx.backup(file, &content) {
                    return ActionOutcome::Failed(e.to_string());
                }
                match self.tx.remove(file, content) {
                    Ok(()) => ActionOutcome::Applied { note: None },
                    Err(e) => ActionOutcome::Failed(e.to_string()),
                }
            }
            // A shared file: rewrite it without the entry.
            Ok(Some(next)) => match self.tx.write(file, Some(content), &next) {
                Ok(()) => ActionOutcome::Applied { note: None },
                Err(e) => ActionOutcome::Failed(e.to_string()),
            },
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
        let existing = match read_text(&self.root.join(path)) {
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
        if let Err(e) = self.tx.write(path, existing, &content) {
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
        let existing = match read_text(&self.root.join(path)) {
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
        if let Err(e) = self.tx.write(path, existing, &content) {
            return ActionOutcome::Failed(e.to_string());
        }
        written.push((
            format!("{path}:{pointer}[{marker}]"),
            sha256_hex(value.as_bytes()),
        ));
        ActionOutcome::Applied { note: None }
    }

    pub(super) fn run_action(
        &mut self,
        program: &str,
        args: &[String],
        undo: &Option<(String, Vec<String>)>,
        optional: bool,
    ) -> ActionOutcome {
        match self.runner.run(program, args, self.root) {
            Err(e) => missing_or_failed(program, e, optional),
            Ok(out) if out.status != 0 => ActionOutcome::Failed(
                Error::Command {
                    command: command_line(program, args),
                    status: Some(out.status),
                    stderr: out.stderr,
                }
                .to_string(),
            ),
            Ok(_) => {
                match undo {
                    Some((program, args)) => {
                        self.tx.record_command_undo(program.clone(), args.clone());
                    }
                    None => self.tx.mark_irreversible(format!(
                        "`{}` has no undo",
                        command_line(program, args)
                    )),
                }
                ActionOutcome::Applied { note: None }
            }
        }
    }
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

pub(super) fn command_line(program: &str, args: &[String]) -> String {
    format!("{program} {}", args.join(" "))
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests;
