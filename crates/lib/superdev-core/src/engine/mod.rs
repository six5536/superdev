//! engine/mod.rs — plan every component, apply the result, roll back on
//! failure. The appliers here compute content and make policy calls (skip,
//! drift guard, backup); every actual side effect goes through [`tx::Tx`],
//! which journals it so the first failure can unwind the run instead of
//! leaving the repo half-changed. The mise pin phase lives in [`pins`], the
//! skill materialiser in [`materialise`].

mod materialise;
mod pins;
mod tx;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::action::{Action, Ownership};
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::{Error, Result};
use crate::fsutil::read_text;
use crate::json_edit::{edit_json_array_element, edit_json_key};
use crate::lock::{Lock, LockedComponent, sha256_hex};
use crate::manifest::Manifest;
use crate::runner::CommandRunner;

use pins::PinEffects;
use tx::Tx;

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
    /// The journal every side effect goes through.
    tx: Tx<'a>,
    /// Locked hashes as of the start of the run, for user-edit detection.
    prior_hashes: BTreeMap<String, String>,
    /// Lock attribution as of the start of the run, for reconciliation.
    prior_owners: BTreeMap<String, String>,
    /// Lock keys earlier entries wrote in this run. Removals were planned
    /// against the pre-run state, so a later one must not drop them.
    written_keys: BTreeSet<String>,
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
            tx: Tx::new(root),
            prior_hashes: lock.files.clone(),
            prior_owners: lock.owners.clone(),
            written_keys: BTreeSet::new(),
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
                Action::Remove { claim, .. } => self.remove_claim(claim, &mut removed),
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

    fn ensure_line(&mut self, path: &str, line: &str) -> ActionOutcome {
        let existing = match read_text(&self.root.join(path)) {
            Ok(existing) => existing,
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let mut next = existing.clone().unwrap_or_default();
        if next.lines().any(|l| l == line) {
            return ActionOutcome::Skipped("present".into());
        }
        if !next.is_empty() && !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str(line);
        next.push('\n');
        match self.tx.write(path, existing, &next) {
            Ok(()) => ActionOutcome::Applied { note: None },
            Err(e) => ActionOutcome::Failed(e.to_string()),
        }
    }

    /// Take back a claimed entry the blueprint dropped — the drift guard's
    /// only home, for all three shapes. The lock key is released even when
    /// the removal is skipped: gone or user-changed, the entry is no longer
    /// superdev's.
    fn remove_claim(&mut self, claim: &Claim, removed: &mut Vec<String>) -> ActionOutcome {
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
                    Some((program, args)) => {
                        self.tx.record_command_undo(program.clone(), args.clone());
                    }
                    None => self.tx.mark_irreversible(format!(
                        "`{}` has no undo",
                        command_line(program, &args)
                    )),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, Ownership};
    use crate::components::mise;
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
        let backups = std::fs::read_dir(dir.path().join(tx::BACKUP_DIR))
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
            actions: vec![Action::Remove {
                claim: Claim::File("old.txt".into()),
                reason: "no longer in the blueprint".into(),
            }],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        assert!(!dir.path().join("old.txt").exists());
        assert!(!lock.files.contains_key("old.txt"));
        let backups: Vec<_> = std::fs::read_dir(dir.path().join(tx::BACKUP_DIR))
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
            actions: vec![Action::Remove {
                claim: Claim::File(path.into()),
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
                Action::Remove {
                    claim: Claim::MisePin("http:codegraph".into()),
                    reason: "no longer in the blueprint".into(),
                },
                Action::Remove {
                    claim: Claim::JsonKey {
                        path: ".mcp.json".into(),
                        pointer: "mcpServers.superdev-aokf".into(),
                    },
                    reason: "no longer in the blueprint".into(),
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
                actions: vec![Action::Remove {
                    claim: Claim::File("old.txt".into()),
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
}
