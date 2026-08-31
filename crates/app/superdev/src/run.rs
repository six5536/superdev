//! run.rs — the run-state verbs for an unattended workflow run.
//!
//! The state file is the seam between the driver skill and the Stop hook
//! (contract-009): the verbs here are its only writers, and `begin`'s
//! exclusive create is what makes a second run a refusal instead of a race.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use superdev_core::error::{Error, Result};

use crate::cli::out;

/// Where the state lives, relative to the repo root. It is machine state:
/// `.superdev/cache/` is gitignored by `init`.
const RUN_STATE_PATH: &str = ".superdev/cache/run.toml";

/// The run state, present exactly while a run is active. An absent file
/// means no run.
#[derive(Debug, Serialize, Deserialize)]
pub struct RunState {
    /// The session that owns the run; empty until a session claims it.
    pub session_id: String,
    /// What the driver does next; the Stop hook's exit-2 message names it.
    pub next: String,
    /// Turn boundaries crossed since the last `advance`. Hook-owned: only
    /// `superdev hook run` increments it; only `advance` resets it.
    pub continues: u32,
    /// When the run began (ISO 8601). Informational.
    pub started: String,
    /// Pid of the `begin` process. Informational, for diagnosing a stale
    /// state by hand.
    pub pid: u32,
}

/// Drive the state of an unattended workflow run.
#[derive(clap::Subcommand)]
pub enum RunCommand {
    /// Arm an unattended run: create the run state exclusively
    Begin {
        /// Session that owns the run (default: $CLAUDE_SESSION_ID)
        #[arg(long, value_name = "ID")]
        session: Option<String>,
        /// The first next step, named by the Stop hook when it continues
        #[arg(long, value_name = "TEXT")]
        next: Option<String>,
    },
    /// Record a step forward: rewrite next, reset the watchdog, refresh the
    /// owner
    Advance {
        /// The next step, named by the Stop hook when it continues
        #[arg(long, value_name = "TEXT")]
        next: String,
        /// Session that owns the run (default: $CLAUDE_SESSION_ID)
        #[arg(long, value_name = "ID")]
        session: Option<String>,
    },
    /// End the run: remove the state; harmless when none exists
    End,
}

/// Run one verb against the repo at `root`.
pub fn run(cmd: &RunCommand, root: &Path) -> Result<u8> {
    let env = std::env::var("CLAUDE_SESSION_ID").ok();
    match cmd {
        RunCommand::Begin { session, next } => begin(
            root,
            &owner(session.as_deref(), env.as_deref()),
            next.as_deref().unwrap_or_default(),
        ),
        RunCommand::Advance { next, session } => {
            advance(root, &owner(session.as_deref(), env.as_deref()), next)
        }
        RunCommand::End => end(root),
    }
}

/// The owning session: the flag wins, then the environment, else unclaimed
/// (empty) — the Stop hook adopts the first Stop payload's session for an
/// unclaimed run (contract-009).
fn owner(flag: Option<&str>, env: Option<&str>) -> String {
    flag.or(env).unwrap_or_default().to_string()
}

fn state_path(root: &Path) -> PathBuf {
    root.join(RUN_STATE_PATH)
}

/// Create the state exclusively; refuse when one exists, naming the owner
/// and the verb that clears it.
fn begin(root: &Path, owner: &str, next: &str) -> Result<u8> {
    let path = state_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let state = RunState {
        session_id: owner.to_string(),
        next: next.to_string(),
        continues: 0,
        started: iso8601_utc(unix_now()),
        pid: std::process::id(),
    };
    let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            // The refusal names the standing run so a stale one is
            // diagnosable without opening the file.
            let standing = load(&path)
                .map(|s| format!("owner: {}, started {}", claimed(&s.session_id), s.started))
                .unwrap_or_else(|_| "unreadable state".to_string());
            return Err(run_error(
                &path,
                format!("a run is already active ({standing}); `superdev run end` clears it"),
            ));
        }
        Err(source) => return Err(Error::Io { path, source }),
    };
    file.write_all(render(&state, &path)?.as_bytes())
        .map_err(|source| Error::Io { path, source })?;
    out(&format!(
        "run begun — owner: {}, next: {}",
        claimed(&state.session_id),
        named(&state.next)
    ))?;
    Ok(0)
}

/// Rewrite `next`, reset the watchdog counter, and refresh the owner so a
/// resumed session does not orphan its own run.
fn advance(root: &Path, owner: &str, next: &str) -> Result<u8> {
    let path = state_path(root);
    let mut state = load(&path)?;
    state.next = next.to_string();
    state.continues = 0;
    if !owner.is_empty() {
        state.session_id = owner.to_string();
    }
    fs::write(&path, render(&state, &path)?).map_err(|source| Error::Io {
        path: path.clone(),
        source,
    })?;
    out(&format!("run advanced — next: {}", named(&state.next)))?;
    Ok(0)
}

/// Remove the state. No state is no failure: the run is as ended as it can
/// be.
fn end(root: &Path) -> Result<u8> {
    let path = state_path(root);
    match fs::remove_file(&path) {
        Ok(()) => out("run ended")?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => out("no run to end")?,
        Err(source) => return Err(Error::Io { path, source }),
    }
    Ok(0)
}

/// Read and parse the state. Absent is a guided error naming `begin`;
/// malformed names the parser's complaint.
fn load(path: &Path) -> Result<RunState> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(run_error(
                path,
                "no run is active; `superdev run begin` starts one".to_string(),
            ));
        }
        Err(source) => {
            return Err(Error::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    toml_edit::de::from_str(&text).map_err(|e| Error::Toml {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn render(state: &RunState, path: &Path) -> Result<String> {
    toml_edit::ser::to_string(state).map_err(|e| Error::Toml {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

/// A run-state failure, worded for the caller: the path locates it, the
/// message says what to run.
fn run_error(path: &Path, message: String) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source: io::Error::other(message),
    }
}

/// An empty owner or next, printed honestly.
fn claimed(session: &str) -> &str {
    if session.is_empty() {
        "unclaimed"
    } else {
        session
    }
}

fn named(next: &str) -> &str {
    if next.is_empty() { "(none)" } else { next }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Seconds since the epoch as `YYYY-MM-DDThh:mm:ssZ`.
fn iso8601_utc(secs: u64) -> String {
    let (h, m, s) = (secs % 86_400 / 3_600, secs % 3_600 / 60, secs % 60);
    let (year, month, day) = civil_from_days(secs / 86_400);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm,
/// restricted to dates at or after the epoch).
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z % 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_writes_the_state_and_a_second_begin_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(begin(dir.path(), "session-1", "slice 1").unwrap(), 0);

        let state = load(&state_path(dir.path())).unwrap();
        assert_eq!(state.session_id, "session-1");
        assert_eq!(state.next, "slice 1");
        assert_eq!(state.continues, 0);
        assert_eq!(state.pid, std::process::id());
        assert!(state.started.ends_with('Z'), "{}", state.started);

        let refusal = begin(dir.path(), "session-2", "other")
            .unwrap_err()
            .to_string();
        assert!(refusal.contains("already active"), "{refusal}");
        assert!(refusal.contains("session-1"), "{refusal}");
        assert!(refusal.contains("superdev run end"), "{refusal}");
        // The standing run is untouched by the refused begin.
        assert_eq!(load(&state_path(dir.path())).unwrap().next, "slice 1");
    }

    #[test]
    fn advance_rewrites_next_resets_the_counter_and_refreshes_the_owner() {
        let dir = tempfile::tempdir().unwrap();
        begin(dir.path(), "", "slice 1").unwrap();
        // A hook has spent continues in the meantime.
        let path = state_path(dir.path());
        let mut state = load(&path).unwrap();
        state.continues = 7;
        std::fs::write(&path, render(&state, &path).unwrap()).unwrap();

        assert_eq!(advance(dir.path(), "session-9", "slice 2").unwrap(), 0);
        let state = load(&path).unwrap();
        assert_eq!(state.next, "slice 2");
        assert_eq!(state.continues, 0);
        assert_eq!(state.session_id, "session-9", "owner refreshed");

        // No resolvable session keeps the recorded owner.
        advance(dir.path(), "", "slice 3").unwrap();
        assert_eq!(load(&path).unwrap().session_id, "session-9");
    }

    #[test]
    fn advance_without_a_run_names_begin() {
        let dir = tempfile::tempdir().unwrap();
        let e = advance(dir.path(), "s", "step").unwrap_err().to_string();
        assert!(e.contains("no run is active"), "{e}");
        assert!(e.contains("superdev run begin"), "{e}");
    }

    #[test]
    fn end_removes_the_state_and_is_harmless_without_one() {
        let dir = tempfile::tempdir().unwrap();
        begin(dir.path(), "s", "step").unwrap();
        assert_eq!(end(dir.path()).unwrap(), 0);
        assert!(!state_path(dir.path()).exists());
        // A second end is a report, not a failure.
        assert_eq!(end(dir.path()).unwrap(), 0);
    }

    #[test]
    fn a_malformed_state_is_a_parse_error_naming_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = state_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not = [toml").unwrap();
        let e = advance(dir.path(), "s", "step").unwrap_err().to_string();
        assert!(e.contains("run.toml"), "{e}");
    }

    #[test]
    fn the_owner_is_flag_then_environment_then_unclaimed() {
        assert_eq!(owner(Some("flag"), Some("env")), "flag");
        assert_eq!(owner(None, Some("env")), "env");
        assert_eq!(owner(None, None), "");
    }

    #[test]
    fn iso8601_utc_matches_known_instants() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(86_399), "1970-01-01T23:59:59Z");
        assert_eq!(iso8601_utc(86_400), "1970-01-02T00:00:00Z");
        // A leap day, and a date this code was written after.
        assert_eq!(iso8601_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso8601_utc(1_788_220_799), "2026-08-31T23:59:59Z");
    }
}
