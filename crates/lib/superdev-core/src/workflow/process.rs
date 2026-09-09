//! Liveness of the processes recorded in transient workflow ownership.
//!
//! Ownership is transient, so a recorded claim can outlive the process that
//! made it. Deciding whether a claim is still executing belongs to the service
//! rather than to any adapter, so every caller observes one answer.
//!
//! A process ID alone is not an identity, because the operating system reuses
//! it. Every recorded process therefore carries a start identity that a later
//! process holding the same ID cannot reproduce.

/// What is known about one recorded process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    /// The recorded process is still running under its recorded identity.
    Live,
    /// The recorded process has exited, or a different process now holds its ID.
    Dead,
    /// Nothing was recorded, so liveness is unknowable and never assumed.
    Unknown,
}

/// Read the operating-system start identity that separates one process from a
/// later reuse of its ID. Absence means the process does not exist now.
///
/// The identity is opaque: callers compare it, never interpret it.
#[cfg(target_os = "linux")]
pub fn start_identity(pid: u32) -> Option<String> {
    // `/proc/<pid>/stat` holds the executable name, unescaped, inside
    // parentheses. Only the fields after the final `)` can be split on
    // whitespace; `starttime` is the twentieth of them.
    let status = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let fields = status.rsplit_once(')')?.1;
    fields.split_whitespace().nth(19).map(str::to_owned)
}

/// Read the process start identity where no `/proc` filesystem exists.
///
/// Signal zero performs the kernel's own permission and existence checks
/// without delivering anything, so it reports existence without side effects.
/// It cannot distinguish a reused ID, so the ID itself is the only identity
/// available here, and a reused ID is indistinguishable from its predecessor.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn start_identity(pid: u32) -> Option<String> {
    let pid = rustix::process::Pid::from_raw(pid as i32)?;
    rustix::process::test_kill_process(pid)
        .ok()
        .map(|()| pid.as_raw_nonzero().to_string())
}

/// Report no start identity where the platform exposes no process inspection.
///
/// An unknowable process is never treated as dead, so ownership on such a
/// platform behaves exactly as it did before liveness existed.
#[cfg(not(unix))]
pub fn start_identity(_pid: u32) -> Option<String> {
    None
}

/// Classify a recorded process.
///
/// A claim is dead only when its recorded identity is contradicted by the
/// running system. Anything unrecorded stays `Unknown`, because reclaiming a
/// claim that might still be executing would break the single-owner rule.
pub fn liveness(pid: Option<u32>, started: Option<&str>) -> Liveness {
    let (Some(pid), Some(started)) = (pid, started) else {
        return Liveness::Unknown;
    };
    match start_identity(pid) {
        Some(current) if current == started => Liveness::Live,
        _ => Liveness::Dead,
    }
}

/// Record one process for later liveness comparison.
///
/// The identity is derived here rather than supplied, so one component owns
/// its format. The two halves are independent: the process ID is kept whenever
/// one is given, because callers track and signal processes by ID, while the
/// start identity is best effort. A platform that exposes no process
/// inspection therefore records an ID with no identity, which reads back as
/// `Unknown` and is never reclaimed.
pub fn record(pid: Option<u32>) -> (Option<u32>, Option<String>) {
    match pid {
        // Zero is never a real process, so it identifies nothing.
        Some(pid) if pid != 0 => (Some(pid), start_identity(pid)),
        _ => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unrecorded_process_is_never_assumed_dead() {
        assert_eq!(liveness(None, None), Liveness::Unknown);
        assert_eq!(liveness(Some(1), None), Liveness::Unknown);
        assert_eq!(liveness(None, Some("x")), Liveness::Unknown);
    }

    #[test]
    fn a_process_id_is_recorded_even_where_no_identity_can_be_derived() {
        // The two halves are independent, so a platform without process
        // inspection still tracks the ID and simply reads back as `Unknown`.
        let pid = std::process::id();
        let (recorded, started) = record(Some(pid));
        assert_eq!(recorded, Some(pid));
        assert_eq!(
            liveness(recorded, started.as_deref()),
            if cfg!(unix) {
                Liveness::Live
            } else {
                Liveness::Unknown
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_reused_process_id_does_not_inherit_the_recorded_claim() {
        let pid = std::process::id();
        assert_eq!(
            liveness(Some(pid), Some("not-this-process")),
            Liveness::Dead
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_exited_process_is_dead_rather_than_unknown() {
        let mut child = std::process::Command::new("true").spawn().unwrap();
        let pid = child.id();
        let (recorded, started) = record(Some(pid));
        child.wait().unwrap();
        // A zombie keeps its entry until reaped, so only the post-wait
        // observation is meaningful.
        assert_eq!(liveness(recorded, started.as_deref()), Liveness::Dead);
    }

    #[test]
    fn an_absent_process_id_records_nothing() {
        // Process ID zero is never a real process on any supported platform.
        assert_eq!(record(Some(0)), (None, None));
        assert_eq!(record(None), (None, None));
    }
}
