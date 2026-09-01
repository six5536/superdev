//! run.rs — the run-state verbs for an unattended workflow run.
//!
//! The state file is the seam between the driver skill and the Stop hook
//! (contract-009): the verbs here are its only writers, and `begin`'s
//! exclusive create is what makes a second run a refusal instead of a race.

use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use superdev_core::error::{Error, Result};

use crate::cli::out;

/// Where the state lives, relative to the repo root. It is machine state:
/// `.superdev/cache/` is gitignored by `init`.
const RUN_STATE_PATH: &str = ".superdev/cache/run.toml";

/// The watchdog cap (ADR-019): at this many continues without an `advance`,
/// the hook stops continuing the run.
pub const CONTINUE_CAP: u32 = 10;

/// The hold cap (ADR-039): at this many turns held for the same unresolved
/// knowledge, the hook reports and lets the turn end, so a finding the agent
/// cannot settle stalls nothing.
pub const HOLD_CAP: u32 = 3;

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

/// Where the hold count lives, relative to the repo root. Separate from the
/// run state because a hold happens whether or not a run is armed, and
/// `run.toml`'s presence means a run is active: a hook that created one to
/// count holds would make the next `superdev run begin` refuse (ADR-039).
const HOLD_STATE_PATH: &str = ".superdev/cache/hold.toml";

/// What the Stop hook has held open for one session. An absent file means
/// nothing is being held.
#[derive(Debug, Serialize, Deserialize)]
pub struct HoldState {
    /// The session the count belongs to. A payload from another session
    /// starts the count again.
    pub session_id: String,
    /// Turns held open because the knowledge carried an error. Hook-owned.
    pub holds: u32,
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
/// unclaimed run (contract-009). An empty flag is no flag.
fn owner(flag: Option<&str>, env: Option<&str>) -> String {
    flag.filter(|f| !f.is_empty())
        .or(env)
        .unwrap_or_default()
        .to_string()
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
    let mut retried = false;
    let mut file = loop {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => break file,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // An empty file is an interrupted begin, not a run: clear it
                // and try once more.
                if !retried && fs::metadata(&path).is_ok_and(|m| m.len() == 0) {
                    retried = true;
                    eprintln!("superdev run: clearing an empty state left by an interrupted begin");
                    let _ = fs::remove_file(&path);
                    continue;
                }
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
        }
    };
    if let Err(source) = file.write_all(render(&state).as_bytes()) {
        // A partial file would wedge every later begin; remove what this
        // begin created before reporting.
        if let Err(e) = fs::remove_file(&path) {
            eprintln!(
                "superdev run: could not remove the partial state at {}: {e}",
                path.display()
            );
        }
        return Err(Error::Io { path, source });
    }
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
    write_state(&path, &state)?;
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

/// The Stop hook body: payload on stdin. Exit 0 lets the turn end; exit 2
/// blocks the stop and hands stderr back as the instruction to continue.
/// An unreadable payload is a loud exit 2, matching `hook validate`.
pub fn hook_run(root: &Path) -> Result<u8> {
    // Hooks run with the project as the working directory, but Claude Code
    // also names it explicitly; prefer the explicit form.
    let root =
        std::env::var_os("CLAUDE_PROJECT_DIR").map_or_else(|| root.to_path_buf(), PathBuf::from);
    let mut payload = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut payload) {
        eprintln!("superdev hook: could not read the stop payload from stdin: {e}");
        return Ok(2);
    }
    hook_run_on(&payload, &root)
}

/// The decision for one Stop payload. An unreadable run state is a report
/// and exit 0 — a Stop hook that fails closed holds every session in the
/// repo open.
fn hook_run_on(payload: &str, root: &Path) -> Result<u8> {
    let parsed: serde_json::Value = match serde_json::from_str(payload) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("superdev hook: malformed stop payload on stdin: {e}");
            return Ok(2);
        }
    };
    let session = parsed["session_id"].as_str().unwrap_or_default();
    if session.is_empty() {
        // A payload without a session matches nothing: an unclaimed run is
        // driven only after adoption (contract-009).
        return Ok(0);
    }
    // The knowledge gate comes first: a turn that leaves a decidable finding
    // behind does not end, whether or not a run is armed (ADR-039). The
    // edit-time hook does not judge the findings only the whole tree settles,
    // so this is where they are caught.
    if let Some(held) = knowledge_hold(root, session) {
        return Ok(held);
    }

    let path = state_path(root);
    let mut state = match read_state(&path) {
        Ok(None) => return Ok(0),
        Ok(Some(state)) => state,
        Err(e) => {
            eprintln!("superdev hook: run state unreadable, letting the turn end: {e}");
            return Ok(0);
        }
    };
    if state.session_id.is_empty() {
        // An unclaimed run: the first Stop payload's session becomes the
        // owner (contract-009).
        state.session_id = session.to_string();
    }
    if state.session_id != session || state.next.is_empty() || state.continues >= CONTINUE_CAP {
        return Ok(0);
    }
    state.continues += 1;
    if let Err(e) = write_state(&path, &state) {
        // A hook that fails closed holds every session open; report and let
        // the turn end.
        eprintln!("superdev hook: could not record the continue, letting the turn end: {e}");
        return Ok(0);
    }
    eprintln!(
        "An unattended superdev run is active ({} of {CONTINUE_CAP} continues since \
         the last advance). Do not stop. Continue with: {}. Record each step \
         forward with `superdev run advance --next <TEXT>`; end the run with \
         `superdev run end`.",
        state.continues, state.next
    );
    Ok(2)
}

/// Hold the turn open while the knowledge carries an error, or `None` to
/// leave the decision to the run state below.
///
/// Fails open twice over: knowledge that cannot be read or checked lets the
/// turn end, because a Stop hook that fails closed holds every session in the
/// repository open; and after `HOLD_CAP` holds for one session it reports and
/// lets the turn end, so a finding the agent cannot settle stalls nothing.
fn knowledge_hold(root: &Path, session: &str) -> Option<u8> {
    let grammar = match superdev_core::validate::schema::load_grammar(root) {
        Ok(grammar) => grammar,
        Err(e) => {
            eprintln!("superdev hook: grammar unreadable, letting the turn end: {e}");
            return Some(0);
        }
    };
    let knowledge = root.join(crate::cli::KNOWLEDGE_DIR);
    let run = match superdev_core::validate::validate_repo(root, &knowledge, &[], &grammar) {
        Ok(run) => run,
        Err(e) => {
            eprintln!("superdev hook: knowledge unreadable, letting the turn end: {e}");
            return Some(0);
        }
    };
    if run.report.passed() {
        clear_holds(root);
        return None;
    }
    let held = read_holds(root, session) + 1;
    if held > HOLD_CAP {
        eprintln!(
            "superdev: the knowledge still has findings after {HOLD_CAP} turns held; \
             letting the turn end. Run `superdev validate` to see them."
        );
        clear_holds(root);
        return None;
    }
    write_holds(root, session, held);
    eprintln!(
        "superdev: the knowledge has findings, so this turn does not end \
         ({held} of {HOLD_CAP}). Fix them, or say why they stand:\n{}",
        run.report.render_human().trim_end_matches('\n')
    );
    Some(2)
}

/// The hold count for `session`, or zero when nothing is held or the file
/// belongs to another session. Unreadable is zero: a hook that fails closed
/// holds every session in the repository open.
fn read_holds(root: &Path, session: &str) -> u32 {
    let Ok(text) = fs::read_to_string(root.join(HOLD_STATE_PATH)) else {
        return 0;
    };
    toml_edit::de::from_str::<HoldState>(&text)
        .ok()
        .filter(|held| held.session_id == session)
        .map_or(0, |held| held.holds)
}

/// Record `holds` for `session`. A failure to record is reported and not
/// fatal: the turn is held on what the validator said, not on the bookkeeping.
fn write_holds(root: &Path, session: &str, holds: u32) {
    let path = root.join(HOLD_STATE_PATH);
    let state = HoldState {
        session_id: session.to_string(),
        holds,
    };
    let rendered = match toml_edit::ser::to_string(&state) {
        Ok(rendered) => rendered,
        Err(e) => {
            eprintln!("superdev hook: could not render the hold count: {e}");
            return;
        }
    };
    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        eprintln!("superdev hook: could not record the hold count: {e}");
        return;
    }
    if let Err(e) = fs::write(&path, rendered) {
        eprintln!("superdev hook: could not record the hold count: {e}");
    }
}

/// Forget what was held: the knowledge is clean, so the next finding starts
/// its own count.
fn clear_holds(root: &Path) {
    let path = root.join(HOLD_STATE_PATH);
    if let Err(e) = fs::remove_file(&path)
        && e.kind() != io::ErrorKind::NotFound
    {
        eprintln!("superdev hook: could not clear the hold count: {e}");
    }
}

/// Read and parse the state: `None` when no run is active, an error when
/// the file exists and cannot be read or parsed. Callers own the policy.
fn read_state(path: &Path) -> Result<Option<RunState>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(Error::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };
    toml_edit::de::from_str(&text)
        .map(Some)
        .map_err(|e| Error::Toml {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
}

/// The state for a verb that needs one: absent is a guided error naming
/// `begin`.
fn load(path: &Path) -> Result<RunState> {
    read_state(path)?.ok_or_else(|| {
        run_error(
            path,
            "no run is active; `superdev run begin` starts one".to_string(),
        )
    })
}

/// Replace the state atomically: a temp file in the same directory, then a
/// rename, so no reader ever sees a torn write.
fn write_state(path: &Path, state: &RunState) -> Result<()> {
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, render(state)).map_err(|source| Error::Io {
        path: tmp.clone(),
        source,
    })?;
    fs::rename(&tmp, path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// The state as TOML. A plain struct of scalars serialises (the
/// `Lock::save` precedent).
fn render(state: &RunState) -> String {
    toml_edit::ser::to_string(state).expect("the run state serialises")
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
        std::fs::write(&path, render(&state)).unwrap();

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
        assert_eq!(
            owner(Some(""), Some("env")),
            "env",
            "an empty flag is no flag"
        );
        assert_eq!(owner(None, None), "");
    }

    fn stop_payload(session: &str) -> String {
        format!(r#"{{"session_id":"{session}","hook_event_name":"Stop"}}"#)
    }

    #[test]
    fn the_hook_lets_the_turn_end_when_nothing_says_otherwise() {
        let dir = tempfile::tempdir().unwrap();
        // No state at all.
        assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 0);

        // A foreign session's turn, an empty next, and a spent counter each
        // exit 0 without touching the counter.
        begin(dir.path(), "mine", "step").unwrap();
        assert_eq!(hook_run_on(&stop_payload("other"), dir.path()).unwrap(), 0);
        let path = state_path(dir.path());
        assert_eq!(load(&path).unwrap().continues, 0);

        advance(dir.path(), "mine", "").unwrap();
        assert_eq!(hook_run_on(&stop_payload("mine"), dir.path()).unwrap(), 0);

        let mut state = load(&path).unwrap();
        state.next = "step".to_string();
        state.continues = CONTINUE_CAP;
        std::fs::write(&path, render(&state)).unwrap();
        assert_eq!(hook_run_on(&stop_payload("mine"), dir.path()).unwrap(), 0);
        assert_eq!(load(&path).unwrap().continues, CONTINUE_CAP);
    }

    #[test]
    fn an_armed_run_continues_counts_and_dies_at_the_cap() {
        let dir = tempfile::tempdir().unwrap();
        begin(dir.path(), "mine", "slice 2").unwrap();
        let path = state_path(dir.path());
        for expected in 1..=CONTINUE_CAP {
            assert_eq!(hook_run_on(&stop_payload("mine"), dir.path()).unwrap(), 2);
            assert_eq!(load(&path).unwrap().continues, expected);
        }
        // The cap is spent; the run dies.
        assert_eq!(hook_run_on(&stop_payload("mine"), dir.path()).unwrap(), 0);
        // An advance revives it.
        advance(dir.path(), "mine", "slice 3").unwrap();
        assert_eq!(hook_run_on(&stop_payload("mine"), dir.path()).unwrap(), 2);
        assert_eq!(load(&path).unwrap().continues, 1);
    }

    #[test]
    fn an_unclaimed_run_is_adopted_by_the_first_stop_payload() {
        let dir = tempfile::tempdir().unwrap();
        begin(dir.path(), "", "step").unwrap();
        assert_eq!(
            hook_run_on(&stop_payload("adopter"), dir.path()).unwrap(),
            2
        );
        assert_eq!(load(&state_path(dir.path())).unwrap().session_id, "adopter");
    }

    #[test]
    fn a_payload_without_a_session_neither_adopts_nor_drives() {
        let dir = tempfile::tempdir().unwrap();
        begin(dir.path(), "", "step").unwrap();
        assert_eq!(hook_run_on(&stop_payload(""), dir.path()).unwrap(), 0);
        let state = load(&state_path(dir.path())).unwrap();
        assert_eq!(state.session_id, "", "not adopted");
        assert_eq!(state.continues, 0, "not driven");
    }

    #[test]
    fn an_interrupted_begin_is_cleared_by_the_next_begin() {
        let dir = tempfile::tempdir().unwrap();
        let path = state_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "").unwrap();
        assert_eq!(begin(dir.path(), "s", "step").unwrap(), 0);
        assert_eq!(load(&path).unwrap().session_id, "s");
    }

    /// A knowledge with one error, and one without.
    fn knowledge(root: &Path, broken: bool) {
        let dir = root.join("knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("manifest.sokf.yaml"), "sokf: \"0.4\"\nname: t\n").unwrap();
        let body = if broken {
            "---\ntype: T\nid: a\n---\n\nSee [b][sokf:nowhere].\n"
        } else {
            "---\ntype: T\nid: a\n---\n\nNothing cited.\n"
        };
        std::fs::write(dir.join("a.md"), body).unwrap();
    }

    /// Covers plan-022 slice 1 and I012 criteria 1 and 2: a turn does not end
    /// while the knowledge carries an error, and the hold names it.
    #[test]
    fn a_turn_does_not_end_while_the_knowledge_has_a_finding() {
        let dir = tempfile::tempdir().unwrap();
        knowledge(dir.path(), true);
        assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 2);
        assert_eq!(read_holds(dir.path(), "s"), 1, "the hold was recorded");
    }

    /// Covers plan-022 slice 4 and I012 criterion 1: a forward reference is
    /// what holds a turn once the five are errors. This is the case the
    /// edit-time hook deliberately lets through, so it is the one the turn
    /// gate has to catch.
    #[test]
    fn a_forward_reference_holds_the_turn() {
        let dir = tempfile::tempdir().unwrap();
        let k = dir.path().join("knowledge");
        std::fs::create_dir_all(&k).unwrap();
        std::fs::write(k.join("manifest.sokf.yaml"), "sokf: \"0.4\"\nname: t\n").unwrap();
        std::fs::write(
            k.join("a.md"),
            "---\ntype: T\nid: a\n---\n\nA [later](notes.txt) file.\n",
        )
        .unwrap();
        assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 2);

        // The file arrives; the turn may end. It is not a concept, so the
        // path link stays a path link (SPEC §10 item 5).
        std::fs::write(k.join("notes.txt"), "here\n").unwrap();
        assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 0);
    }

    /// Covers plan-022 slice 1: a clean knowledge ends the turn and forgets
    /// what was held, so the next finding starts its own count.
    #[test]
    fn a_clean_knowledge_ends_the_turn_and_clears_the_count() {
        let dir = tempfile::tempdir().unwrap();
        knowledge(dir.path(), true);
        assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 2);
        knowledge(dir.path(), false);
        assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 0);
        assert_eq!(read_holds(dir.path(), "s"), 0, "the count was cleared");
        assert!(!dir.path().join(HOLD_STATE_PATH).exists());
    }

    /// Covers plan-022 slice 1: the cap bounds the hold, so a finding the
    /// agent cannot settle stalls nothing.
    #[test]
    fn the_hold_stops_at_the_cap() {
        let dir = tempfile::tempdir().unwrap();
        knowledge(dir.path(), true);
        for expected in 1..=HOLD_CAP {
            assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 2);
            assert_eq!(read_holds(dir.path(), "s"), expected);
        }
        assert_eq!(
            hook_run_on(&stop_payload("s"), dir.path()).unwrap(),
            0,
            "the cap is spent and the turn ends"
        );
    }

    /// Covers plan-022 slice 1: another session's count is not this one's, so
    /// one session cannot spend another's cap.
    #[test]
    fn a_hold_count_belongs_to_its_session() {
        let dir = tempfile::tempdir().unwrap();
        knowledge(dir.path(), true);
        assert_eq!(hook_run_on(&stop_payload("first"), dir.path()).unwrap(), 2);
        assert_eq!(read_holds(dir.path(), "second"), 0);
    }

    /// Covers plan-022 slice 1: knowledge the hook cannot check lets the turn
    /// end. A Stop hook that fails closed holds every session in the
    /// repository open.
    #[test]
    #[cfg(unix)]
    fn unreadable_knowledge_ends_the_turn() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        knowledge(dir.path(), true);
        let concept = dir.path().join("knowledge/a.md");
        std::fs::set_permissions(&concept, std::fs::Permissions::from_mode(0o000)).unwrap();
        // Only meaningful when the reader is not privileged enough to ignore
        // the mode; root reads it anyway and the case cannot be staged.
        if std::fs::read_to_string(&concept).is_ok() {
            return;
        }
        assert_eq!(
            hook_run_on(&stop_payload("s"), dir.path()).unwrap(),
            0,
            "a hook that fails closed holds every session in the repository open"
        );
        std::fs::set_permissions(&concept, std::fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    fn a_malformed_payload_is_loud_and_a_malformed_state_fails_open() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(hook_run_on("not json", dir.path()).unwrap(), 2);

        let path = state_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not = [toml").unwrap();
        assert_eq!(hook_run_on(&stop_payload("s"), dir.path()).unwrap(), 0);
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
