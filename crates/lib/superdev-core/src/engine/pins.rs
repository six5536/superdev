//! engine/pins.rs — the mise pin phase that opens every apply: all
//! `SetMisePin` actions become one grouped `.mise.toml` edit, then the repo's
//! managed tools are trusted and installed before any provider command runs.
//! The lock hashes each pin earns travel back as per-entry [`PinEffects`],
//! handed to the entry when it completes.

use crate::action::Action;
use crate::components::mise;
use crate::fsutil::read_text;
use crate::lock::sha256_hex;

use super::{ActionOutcome, Planned, Session};

/// The lock hashes one entry's pins earned in the pin phase, applied to the
/// lock only when the entry completes.
#[derive(Default)]
pub(super) struct PinEffects {
    /// (lock key, hash) pairs for the entry's pins.
    pub(super) hashes: Vec<(String, String)>,
}

impl<'a> Session<'a> {
    /// Apply every `SetMisePin` as one edit, then trust and install.
    /// Providers run their own commands afterwards, so the tools must exist.
    ///
    /// With no pin edit planned, the tools may still be missing — a fresh
    /// clone commits `.mise.toml` but installs nothing — so
    /// [`Session::install_committed_pins`] covers that case.
    pub(super) fn apply_pins(&mut self, planned: &[Planned]) -> (bool, Vec<PinEffects>) {
        let mut effects: Vec<PinEffects> = planned.iter().map(|_| PinEffects::default()).collect();
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
            return (self.install_committed_pins(planned), effects);
        };
        let prior = match read_text(&self.root.join(".mise.toml")) {
            Ok(prior) => prior,
            Err(e) => {
                self.record(first, first_action, ActionOutcome::Failed(e.to_string()));
                return (false, effects);
            }
        };
        // Edit in memory first: a bad pin then leaves the file untouched.
        let mut content = prior.clone().unwrap_or_default();
        for &(entry, action, tool, value) in &pins {
            match mise::set_pin(&content, tool, value) {
                Ok(next) => content = next,
                Err(e) => {
                    self.record(entry, action, ActionOutcome::Failed(e.to_string()));
                    return (false, effects);
                }
            }
        }
        if let Err(e) = self.tx.write(".mise.toml", prior, &content) {
            self.record(first, first_action, ActionOutcome::Failed(e.to_string()));
            return (false, effects);
        }
        for &(entry, action, tool, _) in &pins {
            // Hash the normalised value, so layout changes never read as drift.
            let value = mise::current_pin(&content, tool)
                .expect("content came from set_pin, so it parses")
                .expect("the pin was just set");
            effects[entry]
                .hashes
                .push((mise::pin_lock_key(tool), sha256_hex(value.as_bytes())));
            self.record(entry, action, ActionOutcome::Applied { note: None });
        }
        // Install what this run pinned plus every managed pin the file already
        // carried: an untouched pin may still be missing on this machine.
        let mut tools: Vec<String> = pins.iter().map(|&(.., tool, _)| tool.to_string()).collect();
        tools.extend(managed_pins_in(&content));
        tools.sort_unstable();
        tools.dedup();
        (self.install_pinned_tools(first, &tools), effects)
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

#[cfg(test)]
mod tests {
    use super::super::*;
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
}
