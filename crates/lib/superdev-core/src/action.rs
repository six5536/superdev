//! action.rs — the planned changes a component wants applied. Pure data;
//! the engine is the only thing that executes them.

/// Who may rewrite a file superdev writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// superdev-owned: hashed into the lock, rewritten freely on sync.
    Owned,
    /// User-owned scaffold: created if missing, never touched again.
    Scaffold,
}

/// One planned change. Paths are repo-relative with forward slashes.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Write a file superdev provides.
    WriteFile {
        /// Target path.
        path: String,
        /// Full file content.
        content: String,
        /// Overwrite rule.
        ownership: Ownership,
        /// Short human reason, shown in plans.
        reason: String,
    },
    /// Append a line unless the file already contains it.
    EnsureLine {
        /// Target path.
        path: String,
        /// Exact line to guarantee.
        line: String,
        /// Short human reason.
        reason: String,
    },
    /// Set one `[tools]` key in `.mise.toml`, preserving the rest.
    SetMisePin {
        /// Tool key, e.g. `http:superpowers`.
        tool: String,
        /// Value as a TOML fragment, e.g. `"6.2.0"` or an inline table.
        value_toml: String,
    },
    /// Run an external command in the repo root.
    Run {
        /// Program name.
        program: String,
        /// Arguments.
        args: Vec<String>,
        /// Short human reason.
        purpose: String,
        /// Inverse command for rollback, when one exists.
        undo: Option<(String, Vec<String>)>,
        /// Skip with a reason (instead of failing) when the program is missing.
        optional: bool,
    },
}

impl Action {
    /// One-line human description, used by `status` and plan output.
    pub fn describe(&self) -> String {
        match self {
            Action::WriteFile { path, reason, .. } => format!("write {path} ({reason})"),
            Action::EnsureLine { path, line, .. } => {
                format!("ensure {path} contains `{line}`")
            }
            Action::SetMisePin { tool, .. } => format!("pin {tool} in .mise.toml"),
            Action::Run {
                program,
                args,
                purpose,
                ..
            } => {
                format!("run `{program} {}` ({purpose})", args.join(" "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_names_the_target() {
        let a = Action::WriteFile {
            path: ".agents/aokf/SPEC.md".into(),
            content: "x".into(),
            ownership: Ownership::Owned,
            reason: "AOKF spec".into(),
        };
        assert_eq!(a.describe(), "write .agents/aokf/SPEC.md (AOKF spec)");

        let a = Action::SetMisePin {
            tool: "http:superpowers".into(),
            value_toml: "\"6.2.0\"".into(),
        };
        assert_eq!(a.describe(), "pin http:superpowers in .mise.toml");

        let a = Action::Run {
            program: "codegraph".into(),
            args: vec!["init".into()],
            purpose: "build the code index".into(),
            undo: None,
            optional: false,
        };
        assert_eq!(a.describe(), "run `codegraph init` (build the code index)");

        let a = Action::EnsureLine {
            path: ".gitignore".into(),
            line: ".superdev/cache/".into(),
            reason: "ignore machine state".into(),
        };
        assert_eq!(
            a.describe(),
            "ensure .gitignore contains `.superdev/cache/`"
        );
    }
}
