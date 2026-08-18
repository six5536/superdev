//! report.rs — human-readable rendering of a plan and of an apply run.

use crate::engine::{ActionOutcome, ApplyResult, Planned};

/// Render a plan: one line per component, its actions indented beneath.
pub fn render_plan(planned: &[Planned]) -> String {
    let mut out = String::new();
    for entry in planned {
        let label = match entry.capability {
            Some(c) => format!("{} ({})", c.as_str(), entry.provider),
            None => format!("repo ({})", entry.provider),
        };
        if entry.actions.is_empty() {
            out.push_str(&format!("{label}: ok\n"));
            continue;
        }
        out.push_str(&format!("{label}: {} change(s)\n", entry.actions.len()));
        for action in &entry.actions {
            out.push_str(&format!("  - {}\n", action.describe()));
        }
    }
    out
}

/// Render an apply run: one line per action outcome, then anything the
/// rollback undid or could not undo.
pub fn render_apply(result: &ApplyResult) -> String {
    let mut out = String::new();
    for report in &result.reports {
        out.push_str(&format!("{}\n", report.label));
        for (description, outcome) in &report.outcomes {
            // Pad to a fixed width so the descriptions line up in a column.
            let (status, detail) = match outcome {
                ActionOutcome::Applied { note } => ("applied", note.clone()),
                ActionOutcome::Skipped(reason) => ("skipped", Some(reason.clone())),
                ActionOutcome::Failed(error) => ("FAILED", Some(error.clone())),
            };
            out.push_str(&format!("  {status:<8} {description}"));
            if let Some(detail) = detail {
                out.push_str(&format!(": {detail}"));
            }
            out.push('\n');
        }
    }
    if !result.reverted.is_empty() || !result.not_reverted.is_empty() {
        out.push('\n');
        for description in &result.reverted {
            out.push_str(&format!("reverted: {description}\n"));
        }
        for description in &result.not_reverted {
            out.push_str(&format!("NOT reverted: {description}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::capability::Capability;
    use crate::engine::{ActionOutcome, ApplyResult, ComponentReport, Planned};

    #[test]
    fn plan_lists_changes_and_marks_ok() {
        let planned = vec![
            Planned {
                capability: Some(Capability::Frontend),
                provider: "frontend-design".into(),
                actions: vec![],
            },
            Planned {
                capability: Some(Capability::Knowledge),
                provider: "aokf".into(),
                actions: vec![Action::EnsureLine {
                    path: ".gitignore".into(),
                    line: "x".into(),
                    reason: "r".into(),
                    append_note: None,
                }],
            },
        ];
        let s = render_plan(&planned);
        assert!(s.contains("frontend (frontend-design): ok"));
        assert!(s.contains("knowledge (aokf): 1 change(s)"));
        assert!(s.contains("  - ensure .gitignore contains `x`"));
    }

    #[test]
    fn apply_report_shows_outcomes_and_reverts() {
        let result = ApplyResult {
            reports: vec![ComponentReport {
                label: "knowledge (aokf)".into(),
                outcomes: vec![
                    (
                        "write AGENTS.md (agent entry point)".into(),
                        ActionOutcome::Applied { note: None },
                    ),
                    (
                        "run `codegraph init` (index)".into(),
                        ActionOutcome::Failed("exit 1".into()),
                    ),
                ],
            }],
            reverted: vec!["write AGENTS.md (agent entry point)".into()],
            not_reverted: vec!["mise install".into()],
            ok: false,
        };
        let s = render_apply(&result);
        assert!(s.contains("applied  write AGENTS.md"));
        assert!(s.contains("FAILED   run `codegraph init` (index): exit 1"));
        assert!(s.contains("reverted: write AGENTS.md"));
        assert!(s.contains("NOT reverted: mise install"));
    }

    #[test]
    fn plan_labels_repo_level_entries() {
        let planned = vec![Planned {
            capability: None,
            provider: "gitignore".into(),
            actions: vec![],
        }];
        assert_eq!(render_plan(&planned), "repo (gitignore): ok\n");
    }

    #[test]
    fn apply_renders_notes_and_skips_without_a_revert_section() {
        let result = ApplyResult {
            reports: vec![ComponentReport {
                label: "frontend (frontend-design)".into(),
                outcomes: vec![
                    (
                        "write AGENTS.md (entry point)".into(),
                        ActionOutcome::Applied {
                            note: Some("replaced a user edit".into()),
                        },
                    ),
                    (
                        "run `mise install` (tools)".into(),
                        ActionOutcome::Skipped("already installed".into()),
                    ),
                ],
            }],
            reverted: vec![],
            not_reverted: vec![],
            ok: true,
        };
        assert_eq!(
            render_apply(&result),
            "frontend (frontend-design)\n\
             \x20 applied  write AGENTS.md (entry point): replaced a user edit\n\
             \x20 skipped  run `mise install` (tools): already installed\n"
        );
    }
}
