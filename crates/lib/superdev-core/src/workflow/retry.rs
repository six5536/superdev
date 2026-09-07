//! Stable BUILD failure fingerprints and project-owned retry accounting.

use sha2::{Digest, Sha256};

/// Durable retry fields materialized in a plan's Build state line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryState {
    /// Dependency-ready block being attempted.
    pub current_block: u32,
    /// Consecutive equivalent failures for this block.
    pub attempts: u32,
    /// Full verification/review correction cycles.
    pub final_corrections: u32,
    /// Last normalized failure fingerprint.
    pub fingerprint: Option<String>,
    /// Bounded human-readable blocker state.
    pub blocker: String,
}

/// Normalize bounded diagnostics before hashing or recording them.
pub fn normalize_diagnostics(value: &str) -> String {
    value
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .take(2_000)
        .collect()
}

/// Bind a failure to its command, exit status, and normalized diagnostics.
pub fn failure_fingerprint(command: &str, exit_status: i32, diagnostics: &str) -> String {
    let normalized = normalize_diagnostics(diagnostics);
    let mut digest = Sha256::new();
    digest.update(command.trim().as_bytes());
    digest.update([0]);
    digest.update(exit_status.to_be_bytes());
    digest.update([0]);
    digest.update(normalized.as_bytes());
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Count an equivalent failure or reset the consecutive count on new evidence.
pub fn record_failure(
    state: &RetryState,
    command: &str,
    exit_status: i32,
    diagnostics: &str,
    max_attempts: u32,
) -> RetryState {
    let fingerprint = failure_fingerprint(command, exit_status, diagnostics);
    let attempts = if state.fingerprint.as_deref() == Some(&fingerprint) {
        state.attempts.saturating_add(1)
    } else {
        1
    };
    let normalized = normalize_diagnostics(diagnostics).replace('\n', " | ");
    let blocker = if attempts >= max_attempts {
        format!("stalled after {attempts} equivalent failures: {normalized}")
    } else {
        format!("retrying after failure {}", &fingerprint[..12])
    };
    RetryState {
        current_block: state.current_block,
        attempts,
        final_corrections: state.final_corrections,
        fingerprint: Some(fingerprint),
        blocker,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> RetryState {
        RetryState {
            current_block: 2,
            attempts: 0,
            final_corrections: 1,
            fingerprint: None,
            blocker: "none".into(),
        }
    }

    #[test]
    fn whitespace_only_diagnostic_changes_have_one_fingerprint() {
        assert_eq!(
            failure_fingerprint("cargo test", 1, "failed   at x\n"),
            failure_fingerprint("cargo test", 1, " failed at x ")
        );
    }

    #[test]
    fn equivalent_failures_stall_at_project_limit_and_new_evidence_resets() {
        let first = record_failure(&state(), "cargo test", 1, "same", 2);
        assert_eq!(first.attempts, 1);
        let second = record_failure(&first, "cargo test", 1, "same", 2);
        assert_eq!(second.attempts, 2);
        assert!(second.blocker.starts_with("stalled"));
        let changed = record_failure(&second, "cargo test", 1, "different", 2);
        assert_eq!(changed.attempts, 1);
        assert!(changed.blocker.starts_with("retrying"));
    }
}
