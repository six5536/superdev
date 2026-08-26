//! runner.rs — the process boundary. Everything that spawns goes through
//! `CommandRunner`, so tests can fake the outside world.

use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

/// Captured result of a finished process.
#[derive(Debug, Clone)]
pub struct Output {
    /// Exit status (`-1` when terminated by a signal).
    pub status: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
}

/// The single seam to the outside world for process execution.
pub trait CommandRunner {
    /// Run `program args…` in `cwd`, capturing output. A missing program is
    /// [`Error::Command`] with `status: None` and stderr `"not found"`.
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<Output>;
}

/// Real implementation via `std::process::Command`.
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<Output> {
        let command_line = format!("{program} {}", args.join(" "))
            .trim_end()
            .to_string();
        let out = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|e| {
                let stderr = if e.kind() == std::io::ErrorKind::NotFound {
                    "not found".to_string()
                } else {
                    e.to_string()
                };
                Error::Command {
                    command: command_line.clone(),
                    status: None,
                    stderr,
                }
            })?;
        Ok(Output {
            status: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
pub(crate) use fake::FakeRunner;

#[cfg(test)]
mod fake {
    use std::cell::RefCell;
    use std::path::Path;

    use super::{CommandRunner, Output};
    use crate::error::{Error, Result};

    /// Scripted runner for tests: records every call, returns the output of
    /// the first matching script, succeeds silently otherwise.
    ///
    /// A needle registered with `script_containing` is tried before any
    /// prefix, because it is the more particular of the two: a broad prefix
    /// registered elsewhere in the same test would otherwise answer first and
    /// leave the needle dead, which is a test that has stopped
    /// discriminating without stopping passing.
    pub(crate) struct FakeRunner {
        scripts: RefCell<Vec<(String, Output)>>,
        contains: RefCell<Vec<(String, Output)>>,
        missing: RefCell<Vec<String>>,
        calls: RefCell<Vec<String>>,
    }

    impl FakeRunner {
        /// A runner with nothing scripted: every command succeeds emptily.
        pub(crate) fn new() -> FakeRunner {
            FakeRunner {
                scripts: RefCell::new(Vec::new()),
                contains: RefCell::new(Vec::new()),
                missing: RefCell::new(Vec::new()),
                calls: RefCell::new(Vec::new()),
            }
        }

        /// Return `output` for the first command line starting with `prefix`.
        pub(crate) fn script(&self, prefix: &str, output: Output) {
            self.scripts.borrow_mut().push((prefix.to_string(), output));
        }

        /// Return `output` for the first command line containing `needle`.
        ///
        /// A prefix cannot pick out a subcommand when the program takes
        /// options before it — every git call carries `-c` overrides — and
        /// widening the prefix to `git` answers every call alike, which is
        /// how a test stops discriminating without stopping passing.
        pub(crate) fn script_containing(&self, needle: &str, output: Output) {
            self.contains
                .borrow_mut()
                .push((needle.to_string(), output));
        }

        /// Simulate `program` not being installed.
        pub(crate) fn missing(&self, program: &str) {
            self.missing.borrow_mut().push(program.to_string());
        }

        /// Every command line run so far, in order.
        pub(crate) fn calls(&self) -> Vec<String> {
            self.calls.borrow().clone()
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, program: &str, args: &[String], _cwd: &Path) -> Result<Output> {
            let line = format!("{program} {}", args.join(" "))
                .trim_end()
                .to_string();
            self.calls.borrow_mut().push(line.clone());
            if self.missing.borrow().iter().any(|m| m == program) {
                return Err(Error::Command {
                    command: line,
                    status: None,
                    stderr: "not found".into(),
                });
            }
            if let Some((_, out)) = self
                .contains
                .borrow()
                .iter()
                .find(|(n, _)| line.contains(n.as_str()))
            {
                return Ok(out.clone());
            }
            if let Some((_, out)) = self
                .scripts
                .borrow()
                .iter()
                .find(|(p, _)| line.starts_with(p))
            {
                return Ok(out.clone());
            }
            Ok(Output {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[cfg(unix)]
    #[test]
    fn system_runner_captures_output_and_status() {
        let out = SystemRunner
            .run(
                "sh",
                &["-c".into(), "echo hi; exit 3".into()],
                Path::new("."),
            )
            .unwrap();
        assert_eq!(out.status, 3);
        assert_eq!(out.stdout.trim(), "hi");
    }

    #[test]
    fn system_runner_missing_program_is_command_error() {
        let err = SystemRunner
            .run("superdev-definitely-not-a-program", &[], Path::new("."))
            .unwrap_err();
        assert!(matches!(
            err,
            crate::error::Error::Command { status: None, .. }
        ));
    }

    #[test]
    fn fake_runner_scripts_and_records() {
        let fake = FakeRunner::new();
        fake.script(
            "claude plugin list",
            Output {
                status: 0,
                stdout: "frontend-design".into(),
                stderr: String::new(),
            },
        );
        fake.missing("codegraph");
        let out = fake
            .run("claude", &["plugin".into(), "list".into()], Path::new("."))
            .unwrap();
        assert_eq!(out.stdout, "frontend-design");
        assert!(
            fake.run("codegraph", &["init".into()], Path::new("."))
                .is_err()
        );
        let ok = fake
            .run("mise", &["install".into()], Path::new("."))
            .unwrap();
        assert_eq!(ok.status, 0);
        assert_eq!(
            fake.calls(),
            vec!["claude plugin list", "codegraph init", "mise install"]
        );
    }
}
