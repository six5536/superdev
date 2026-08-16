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
    /// Set one top-level key path in a JSON file, preserving all other content.
    /// Creates the file (`{}`-rooted) when absent. Used for .mcp.json.
    SetJsonKey {
        /// Target path (repo-relative).
        path: String,
        /// Dotted key path, e.g. `mcpServers.superdev-aokf`.
        pointer: String,
        /// The value to set, as a JSON string.
        value_json: String,
    },
    /// Ensure a JSON array carries one superdev-owned element, found by
    /// marker; every other element is the user's. Creates the file and the
    /// path to the array when absent. Used for .claude/settings.json hooks.
    EnsureJsonArrayElement {
        /// Target path (repo-relative).
        path: String,
        /// Dotted key path to the array, e.g. `hooks.PostToolUse`.
        pointer: String,
        /// Substring identifying superdev's element among the array's entries.
        marker: String,
        /// The desired element, as a JSON string.
        value_json: String,
    },
    /// Delete a file superdev previously wrote.
    RemoveFile {
        /// Target path.
        path: String,
        /// Short human reason, shown in plans.
        reason: String,
    },
    /// Drop one `[tools]` key from `.mise.toml`, preserving the rest.
    RemoveMisePin {
        /// Tool key, e.g. `http:superpowers`.
        tool: String,
    },
    /// Drop one key path from a JSON file, preserving all other content. The
    /// pointer may end in `[marker]` to name a superdev-owned array element.
    RemoveJsonKey {
        /// Target path (repo-relative).
        path: String,
        /// Dotted key path, e.g. `mcpServers.superdev-aokf`.
        pointer: String,
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
            Action::SetJsonKey { path, pointer, .. } => format!("set {pointer} in {path}"),
            Action::EnsureJsonArrayElement {
                path,
                pointer,
                marker,
                ..
            } => format!("ensure {path} {pointer} has the `{marker}` entry"),
            Action::RemoveFile { path, reason } => format!("remove {path} ({reason})"),
            Action::RemoveMisePin { tool } => format!("unpin {tool} in .mise.toml"),
            Action::RemoveJsonKey { path, pointer } => format!("remove {pointer} from {path}"),
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

        let a = Action::SetJsonKey {
            path: ".mcp.json".into(),
            pointer: "mcpServers.superdev-aokf".into(),
            value_json: "{}".into(),
        };
        assert_eq!(a.describe(), "set mcpServers.superdev-aokf in .mcp.json");

        let a = Action::EnsureJsonArrayElement {
            path: ".claude/settings.json".into(),
            pointer: "hooks.PostToolUse".into(),
            marker: "superdev aokf hook validate".into(),
            value_json: "{}".into(),
        };
        assert_eq!(
            a.describe(),
            "ensure .claude/settings.json hooks.PostToolUse has the `superdev aokf hook validate` entry"
        );

        let a = Action::RemoveFile {
            path: ".claude/skills/humanise/SKILL.md".into(),
            reason: "no longer in the blueprint".into(),
        };
        assert_eq!(
            a.describe(),
            "remove .claude/skills/humanise/SKILL.md (no longer in the blueprint)"
        );
        let a = Action::RemoveMisePin {
            tool: "http:codegraph".into(),
        };
        assert_eq!(a.describe(), "unpin http:codegraph in .mise.toml");
        let a = Action::RemoveJsonKey {
            path: ".mcp.json".into(),
            pointer: "mcpServers.superdev-aokf".into(),
        };
        assert_eq!(
            a.describe(),
            "remove mcpServers.superdev-aokf from .mcp.json"
        );
    }
}
