//! components/plugin.rs — capabilities delivered as Claude Code plugins.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;

/// Where a plugin's marketplace comes from.
pub enum Marketplace {
    /// GitHub owner/repo added directly.
    GitHub {
        /// `owner/repo` passed to `claude plugin marketplace add`.
        repo: &'static str,
        /// Marketplace name used in `<plugin>@<name>`.
        name: &'static str,
    },
}

/// A capability provided by installing a Claude Code plugin.
pub struct ClaudePlugin {
    /// Slot this plugin fills.
    pub capability: Capability,
    /// Provider id.
    pub provider_id: &'static str,
    /// Plugin name inside the marketplace.
    pub plugin: &'static str,
    /// Marketplace source.
    pub marketplace: Marketplace,
}

/// Anthropic's frontend-design plugin from the official marketplace.
pub fn frontend_design() -> ClaudePlugin {
    ClaudePlugin {
        capability: Capability::Frontend,
        provider_id: "frontend-design",
        plugin: "frontend-design",
        marketplace: Marketplace::GitHub {
            // The repo registers itself under the name its marketplace.json
            // declares, which is not the repo name.
            repo: "anthropics/claude-code",
            name: "claude-code-plugins",
        },
    }
}

impl ClaudePlugin {
    fn installed(&self, ctx: &Ctx<'_>) -> Result<bool> {
        match ctx
            .runner
            .run("claude", &["plugin".into(), "list".into()], ctx.root)
        {
            Ok(out) if out.status == 0 => Ok(out.stdout.contains(self.plugin)),
            // claude missing or errored: treat as not installed; actions are optional.
            _ => Ok(false),
        }
    }

    fn install_actions(&self) -> Vec<Action> {
        let Marketplace::GitHub { repo, name } = &self.marketplace;
        let (source, name) = (repo.to_string(), *name);
        vec![
            Action::Run {
                program: "claude".into(),
                args: vec!["plugin".into(), "marketplace".into(), "add".into(), source],
                purpose: format!("register the {name} marketplace"),
                // A marketplace registration is harmless and may be shared.
                undo: None,
                optional: true,
            },
            Action::Run {
                program: "claude".into(),
                args: vec![
                    "plugin".into(),
                    "install".into(),
                    format!("{}@{name}", self.plugin),
                ],
                purpose: format!("install the {} plugin", self.plugin),
                undo: Some((
                    "claude".into(),
                    vec!["plugin".into(), "uninstall".into(), self.plugin.into()],
                )),
                optional: true,
            },
        ]
    }
}

impl Component for ClaudePlugin {
    fn capability(&self) -> Option<Capability> {
        Some(self.capability)
    }

    fn provider(&self) -> &'static str {
        self.provider_id
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        if self.installed(ctx)? {
            return Ok(Vec::new());
        }
        Ok(self.install_actions())
    }

    fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Component, Ctx};
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;
    use crate::runner::Output;

    fn ctx_parts() -> (Manifest, Lock) {
        (Manifest::default_for("0.1.0", &[]), Lock::default())
    }

    #[test]
    fn installed_plugin_plans_nothing() {
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        fake.script(
            "claude plugin list",
            Output {
                status: 0,
                stdout: "frontend-design 1.0.0\n".into(),
                stderr: String::new(),
            },
        );
        let ctx = Ctx {
            root: std::path::Path::new("."),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
            content: crate::content::test_snapshot(),
        };
        assert!(frontend_design().plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn missing_claude_still_plans_optional_actions() {
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        fake.missing("claude");
        let ctx = Ctx {
            root: std::path::Path::new("."),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
            content: crate::content::test_snapshot(),
        };
        let actions = frontend_design().plan(&ctx).unwrap();
        assert!(!actions.is_empty());
        assert!(
            actions
                .iter()
                .all(|a| matches!(a, crate::action::Action::Run { optional: true, .. }))
        );
        let descs: Vec<String> = actions.iter().map(|a| a.describe()).collect();
        assert!(
            descs
                .iter()
                .any(|d| d.contains("marketplace add anthropics/claude-code"))
        );
        assert!(
            descs
                .iter()
                .any(|d| d.contains("plugin install frontend-design@claude-code-plugins"))
        );
    }

    #[test]
    fn components_report_their_slot_and_provider() {
        assert_eq!(frontend_design().capability(), Some(Capability::Frontend));
        assert_eq!(frontend_design().provider(), "frontend-design");
    }
}
