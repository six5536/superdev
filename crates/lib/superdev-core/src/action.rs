//! action.rs — the planned changes a component wants applied. Pure data;
//! the engine is the only thing that executes them.

use crate::component::Claim;

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
        /// Report note when the line is appended to a file that already
        /// existed — the migrating population, not fresh creates.
        append_note: Option<String>,
    },
    /// Set one `[tools]` key in `.mise.toml`, preserving the rest.
    SetMisePin {
        /// Tool key, e.g. `http:codegraph`.
        tool: String,
        /// Value as a TOML fragment, e.g. `"1.0.0"` or an inline table.
        value_toml: String,
    },
    /// Set one top-level key path in a JSON file, preserving all other content.
    /// Creates the file (`{}`-rooted) when absent. Used for .mcp.json.
    SetJsonKey {
        /// Target path (repo-relative).
        path: String,
        /// Dotted key path, e.g. `mcpServers.superdev-sokf`.
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
    /// Take back a claimed entry superdev previously wrote: delete the
    /// whole-file shape, rewrite the shared-file shapes without their entry.
    Remove {
        /// The claim being taken back.
        claim: Claim,
        /// Short human reason, shown in plans.
        reason: String,
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
            Action::Remove { claim, reason } => match claim {
                Claim::File(path) => format!("remove {path} ({reason})"),
                Claim::MisePin(tool) => format!("unpin {tool} in .mise.toml"),
                Claim::JsonKey { path, pointer } => format!("remove {pointer} from {path}"),
            },
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
            path: ".agents/sokf/SPEC.md".into(),
            content: "x".into(),
            ownership: Ownership::Owned,
            reason: "SOKF spec".into(),
        };
        assert_eq!(a.describe(), "write .agents/sokf/SPEC.md (SOKF spec)");

        let a = Action::SetMisePin {
            tool: "http:codegraph".into(),
            value_toml: "\"1.0.0\"".into(),
        };
        assert_eq!(a.describe(), "pin http:codegraph in .mise.toml");

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
            append_note: None,
        };
        assert_eq!(
            a.describe(),
            "ensure .gitignore contains `.superdev/cache/`"
        );

        let a = Action::SetJsonKey {
            path: ".mcp.json".into(),
            pointer: "mcpServers.superdev-sokf".into(),
            value_json: "{}".into(),
        };
        assert_eq!(a.describe(), "set mcpServers.superdev-sokf in .mcp.json");

        let a = Action::EnsureJsonArrayElement {
            path: ".claude/settings.json".into(),
            pointer: "hooks.PostToolUse".into(),
            marker: "superdev hook validate".into(),
            value_json: "{}".into(),
        };
        assert_eq!(
            a.describe(),
            "ensure .claude/settings.json hooks.PostToolUse has the `superdev hook validate` entry"
        );

        let a = Action::Remove {
            claim: Claim::File(".claude/skills/humanise/SKILL.md".into()),
            reason: "no longer in the blueprint".into(),
        };
        assert_eq!(
            a.describe(),
            "remove .claude/skills/humanise/SKILL.md (no longer in the blueprint)"
        );
        let a = Action::Remove {
            claim: Claim::MisePin("http:codegraph".into()),
            reason: "no longer in the blueprint".into(),
        };
        assert_eq!(a.describe(), "unpin http:codegraph in .mise.toml");
        let a = Action::Remove {
            claim: Claim::JsonKey {
                path: ".mcp.json".into(),
                pointer: "mcpServers.superdev-sokf".into(),
            },
            reason: "no longer in the blueprint".into(),
        };
        assert_eq!(
            a.describe(),
            "remove mcpServers.superdev-sokf from .mcp.json"
        );
    }
}
