//! runner.rs — the process boundary. Everything that spawns goes through
//! `CommandRunner`, so tests can fake the outside world.

use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

/// How often a deadline is checked while a child runs.
///
/// Short enough that the reported wait is the deadline rather than the poll,
/// long enough that waiting costs nothing measurable.
const POLL: Duration = Duration::from_millis(10);

// The process seam is the pack resolution contract's Definition
// (contract-007): the `pack-resolution` region below is the output, the
// options and the one trait everything that spawns goes through.
// sokf:begin pack-resolution
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

/// How a command is run: everything beyond the program and its arguments.
///
/// One struct rather than a method per concern, so the next thing that needs
/// the process boundary has somewhere to go. ADR-015.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Kill the child and fail after this long. `None` waits as long as it
    /// takes, which is what a toolchain install needs.
    pub timeout: Option<Duration>,
    /// Extra environment for the child, over the inherited one.
    pub env: Vec<(String, String)>,
}

/// The single seam to the outside world for process execution.
///
/// Two calling forms, one implementation: [`CommandRunner::run_with`] is the
/// required method and [`CommandRunner::run`] defaults onto it with no
/// options. A caller that needs neither a deadline nor an environment writes
/// `run` and is unaffected by either existing.
pub trait CommandRunner {
    /// Run `program args…` in `cwd`, capturing output. A missing program is
    /// [`Error::Command`] with `status: None` and stderr `"not found"`.
    fn run(&self, program: &str, args: &[String], cwd: &Path) -> Result<Output> {
        self.run_with(program, args, cwd, &RunOptions::default())
    }

    /// The same, with a deadline and an environment.
    ///
    /// A deadline that expires is an [`Error::Command`] like any other failed
    /// spawn, so a caller that only wants to know it did not work — the pin
    /// query reporting "could not reach it" — needs no new arm.
    fn run_with(
        &self,
        program: &str,
        args: &[String],
        cwd: &Path,
        opts: &RunOptions,
    ) -> Result<Output>;
}
// sokf:end pack-resolution

/// Real implementation via `std::process::Command`.
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run_with(
        &self,
        program: &str,
        args: &[String],
        cwd: &Path,
        opts: &RunOptions,
    ) -> Result<Output> {
        let command_line = format!("{program} {}", args.join(" "))
            .trim_end()
            .to_string();
        let failed = |e: std::io::Error| {
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
        };

        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(cwd)
            // What `output()` gives a child, kept for the deadline path too:
            // a credential prompt reading stdin gets EOF rather than a
            // terminal, so it fails instead of waiting for a person.
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (name, value) in &opts.env {
            command.env(name, value);
        }
        let mut child = command.spawn().map_err(failed)?;

        // A thread per pipe, because a child that fills one while superdev
        // reads the other deadlocks — which is the whole reason `output()`
        // exists, and it cannot be used here: it waits without a deadline.
        let out_pipe = drain(child.stdout.take());
        let err_pipe = drain(child.stderr.take());

        let status = match opts.timeout {
            None => child.wait().map_err(failed)?,
            Some(limit) => match wait_until(&mut child, limit) {
                Ok(Some(status)) => status,
                Ok(None) => {
                    return Err(Error::Command {
                        command: command_line,
                        status: None,
                        stderr: format!(
                            "no answer within {}s, so it was stopped",
                            limit.as_secs_f32()
                        ),
                    });
                }
                Err(e) => return Err(failed(e)),
            },
        };
        Ok(Output {
            status: status.code().unwrap_or(-1),
            stdout: out_pipe.join().unwrap_or_default(),
            stderr: err_pipe.join().unwrap_or_default(),
        })
    }
}

/// Read one pipe to the end on a thread of its own.
fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut buffer);
        }
        String::from_utf8_lossy(&buffer).into_owned()
    })
}

/// Wait for `child` for at most `limit`, killing it if it outlasts that.
///
/// `Ok(None)` is the deadline expiring. The child is killed and reaped before
/// returning, so it is not left running behind the error — best effort, as a
/// kill always is: a process that ignores the signal outlives this, and the
/// timeout is reported rather than pretended away.
///
/// The reader threads are not joined on that path. Killing the child closes
/// its pipes, but a grandchild holding one — `git` hands a fetch to
/// `git-remote-https` — would not, and joining would then wait exactly as
/// long as the deadline exists to prevent.
fn wait_until(
    child: &mut Child,
    limit: Duration,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if start.elapsed() >= limit {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        thread::sleep(POLL);
    }
}

#[cfg(test)]
pub(crate) use fake::FakeRunner;

#[cfg(test)]
mod fake {
    use std::cell::RefCell;
    use std::path::Path;

    use super::{CommandRunner, Output, RunOptions};
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
        options: RefCell<Vec<RunOptions>>,
    }

    impl FakeRunner {
        /// A runner with nothing scripted: every command succeeds emptily.
        pub(crate) fn new() -> FakeRunner {
            FakeRunner {
                scripts: RefCell::new(Vec::new()),
                contains: RefCell::new(Vec::new()),
                missing: RefCell::new(Vec::new()),
                calls: RefCell::new(Vec::new()),
                options: RefCell::new(Vec::new()),
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

        /// The options each of those calls carried, in the same order.
        ///
        /// A caller that sets a deadline or an environment can be checked on
        /// what it asked for rather than on what a real process did with it.
        pub(crate) fn options(&self) -> Vec<RunOptions> {
            self.options.borrow().clone()
        }
    }

    impl CommandRunner for FakeRunner {
        fn run_with(
            &self,
            program: &str,
            args: &[String],
            _cwd: &Path,
            opts: &RunOptions,
        ) -> Result<Output> {
            let line = format!("{program} {}", args.join(" "))
                .trim_end()
                .to_string();
            self.calls.borrow_mut().push(line.clone());
            self.options.borrow_mut().push(opts.clone());
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
    use std::time::Duration;

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

    /// The seam is used as `&dyn CommandRunner` everywhere — `Ctx` holds one
    /// — so a defaulted method that broke object safety would break every
    /// caller. Asserted rather than assumed, because the compiler only
    /// complains where a trait object is formed. ADR-015.
    #[test]
    fn the_seam_is_still_object_safe() {
        let runner: &dyn CommandRunner = &SystemRunner;
        assert!(
            runner
                .run("superdev-definitely-not-a-program", &[], Path::new("."))
                .is_err()
        );
    }

    /// A child that outlives its deadline is stopped and reported, rather
    /// than waited on until the OS gives up. I002.
    #[cfg(unix)]
    #[test]
    fn a_child_that_outlives_its_deadline_is_stopped_and_reported() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("finished");
        let opts = RunOptions {
            timeout: Some(Duration::from_millis(100)),
            ..RunOptions::default()
        };

        let began = Instant::now();
        let err = SystemRunner
            .run_with(
                "sh",
                &["-c".into(), format!("sleep 1; touch {}", marker.display())],
                dir.path(),
                &opts,
            )
            .expect_err("the deadline expired");
        let waited = began.elapsed();

        assert!(
            err.to_string().contains("no answer within"),
            "the failure does not say the deadline expired: {err}"
        );
        assert!(
            waited < Duration::from_millis(800),
            "it waited for the child rather than the deadline: {waited:?}"
        );
        // And it is not still running behind the error: the marker its
        // second second would have written never appears.
        thread::sleep(Duration::from_millis(1400));
        assert!(
            !marker.exists(),
            "the child outlived the call that gave up on it"
        );
    }

    /// `None` is not a deadline of zero. A toolchain install on a slow link
    /// is a legitimately long wait, and the seam has no business ending it.
    #[cfg(unix)]
    #[test]
    fn no_deadline_waits_for_the_child() {
        let out = SystemRunner
            .run_with(
                "sh",
                &["-c".into(), "sleep 0.3; echo done".into()],
                Path::new("."),
                &RunOptions::default(),
            )
            .expect("a slow child with no deadline still answers");
        assert_eq!(out.stdout.trim(), "done");
        assert_eq!(out.status, 0);
    }

    /// The environment is the other half of the seam change: without it
    /// `GIT_TERMINAL_PROMPT=0` has nowhere to go. ADR-015.
    #[cfg(unix)]
    #[test]
    fn an_env_entry_reaches_the_child() {
        let opts = RunOptions {
            env: vec![("SUPERDEV_SEAM".into(), "reached".into())],
            ..RunOptions::default()
        };
        let out = SystemRunner
            .run_with(
                "sh",
                &["-c".into(), "printf %s \"$SUPERDEV_SEAM\"".into()],
                Path::new("."),
                &opts,
            )
            .unwrap();
        assert_eq!(out.stdout, "reached");
    }

    /// Both pipes are drained on threads of their own, so a child that fills
    /// one while superdev reads the other cannot deadlock. More than a pipe
    /// buffer on each, or this passes whatever the implementation does.
    #[cfg(unix)]
    #[test]
    fn a_child_that_fills_both_pipes_does_not_deadlock() {
        let out = SystemRunner
            .run_with(
                "sh",
                &[
                    "-c".into(),
                    "yes abcdefgh | head -c 200000; yes ABCDEFGH | head -c 200000 >&2".into(),
                ],
                Path::new("."),
                &RunOptions {
                    timeout: Some(Duration::from_secs(30)),
                    ..RunOptions::default()
                },
            )
            .expect("both pipes drained");
        assert_eq!(out.stdout.len(), 200_000);
        assert_eq!(out.stderr.len(), 200_000);
    }

    /// The fake records what a caller asked for, so a slice that sets a
    /// deadline can be checked on the asking rather than on a real process.
    #[test]
    fn the_fake_records_the_options_it_was_given() {
        let fake = FakeRunner::new();
        let opts = RunOptions {
            timeout: Some(Duration::from_secs(5)),
            env: vec![("GIT_TERMINAL_PROMPT".into(), "0".into())],
        };
        fake.run("git", &["status".into()], Path::new(".")).unwrap();
        fake.run_with("git", &["ls-remote".into()], Path::new("."), &opts)
            .unwrap();

        let seen = fake.options();
        assert_eq!(seen.len(), 2);
        assert!(seen[0].timeout.is_none(), "`run` sets no deadline");
        assert!(seen[0].env.is_empty(), "`run` sets no environment");
        assert_eq!(seen[1].timeout, Some(Duration::from_secs(5)));
        assert_eq!(seen[1].env, opts.env);
    }

    /// The needle wins over a prefix that also matches. Asserted because the
    /// doc promises it and nothing else would notice the two being swapped
    /// back: a broad prefix answering first leaves every needle dead, and
    /// every test using one passes while exercising nothing.
    #[test]
    fn a_needle_is_answered_before_a_prefix_that_also_matches() {
        let fake = FakeRunner::new();
        let out = |stdout: &str| Output {
            status: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        };
        fake.script("git", out("the broad prefix"));
        fake.script_containing("ls-remote", out("the needle"));

        let answered = fake
            .run(
                "git",
                &["-c".into(), "x=y".into(), "ls-remote".into()],
                Path::new("."),
            )
            .unwrap();

        assert_eq!(answered.stdout, "the needle");
        // And a call the needle does not match still gets the prefix.
        let other = fake.run("git", &["clone".into()], Path::new(".")).unwrap();
        assert_eq!(other.stdout, "the broad prefix");
    }
}
