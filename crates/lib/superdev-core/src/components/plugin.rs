//! components/plugin.rs — capabilities delivered as Claude Code plugins.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;
use crate::registry::{self, SUPERPOWERS_CHECKSUM, SUPERPOWERS_URL};

/// Where a plugin's marketplace comes from.
pub enum Marketplace {
    /// GitHub owner/repo added directly.
    GitHub {
        /// `owner/repo` passed to `claude plugin marketplace add`.
        repo: &'static str,
        /// Marketplace name used in `<plugin>@<name>`.
        name: &'static str,
    },
    /// A mise-pinned checkout registered as a local marketplace.
    MiseTool {
        /// mise tool key, e.g. `http:superpowers`.
        tool: &'static str,
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

/// mise `[tools]` key for the Superpowers checkout.
pub const SUPERPOWERS_MISE_TOOL: &str = "http:superpowers";

/// The Superpowers workflows plugin (mise-pinned tarball checkout).
pub fn superpowers() -> ClaudePlugin {
    ClaudePlugin {
        capability: Capability::Workflows,
        provider_id: "superpowers",
        plugin: "superpowers",
        marketplace: Marketplace::MiseTool {
            tool: SUPERPOWERS_MISE_TOOL,
            name: "superpowers-dev",
        },
    }
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

    fn mise_pin_action(&self, ctx: &Ctx<'_>) -> Result<Option<Action>> {
        let Marketplace::MiseTool { tool, .. } = &self.marketplace else {
            return Ok(None);
        };
        // By provider, not by capability alone: workflows has more than one
        // entry, and only this plugin's pin belongs in this fragment.
        let default = registry::entry_for(self.capability, self.provider_id)
            .and_then(|e| e.version)
            .expect("registry pins superpowers")
            .version;
        let value = format!(
            "{{ version = \"{default}\", url = \"{SUPERPOWERS_URL}\", checksum = \"{SUPERPOWERS_CHECKSUM}\", strip_components = 1 }}"
        );
        super::pin::planned_pin(ctx, self.capability, self.provider_id, tool, &value)
    }

    fn install_actions(&self) -> Vec<Action> {
        let (source, name) = match &self.marketplace {
            Marketplace::GitHub { repo, name } => (repo.to_string(), *name),
            Marketplace::MiseTool { tool, name } => (format!("{{mise-where:{tool}}}"), *name),
        };
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
    fn capability(&self) -> Capability {
        self.capability
    }

    fn provider(&self) -> &'static str {
        self.provider_id
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        if let Some(pin) = self.mise_pin_action(ctx)? {
            actions.push(pin);
        }
        if !self.installed(ctx)? {
            actions.extend(self.install_actions());
        }
        Ok(actions)
    }

    fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
        match &self.marketplace {
            Marketplace::MiseTool { tool, .. } => vec![Claim::MisePin((*tool).to_string())],
            Marketplace::GitHub { .. } => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Component, Ctx};
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::registry::{SUPERPOWERS_CHECKSUM, SUPERPOWERS_URL};
    use crate::runner::FakeRunner;
    use crate::runner::Output;

    /// The default workflows provider is mattpocock-skills, so these tests name
    /// superpowers explicitly rather than inherit whatever the registry defaults to.
    fn ctx_parts() -> (Manifest, Lock) {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let workflows = manifest.capabilities.get_mut("workflows").unwrap();
        workflows.provider = "superpowers".into();
        workflows.version = Some("6.2.0".into());
        (manifest, Lock::default())
    }

    /// A `.mise.toml` pinning Superpowers, written in a deliberately different
    /// layout so the test also covers pin normalisation.
    fn pinned_mise_toml() -> String {
        format!(
            "[tools]\n\"http:superpowers\" = {{ version = \"6.2.0\", # pinned\n  url = \"{SUPERPOWERS_URL}\",\n  checksum = \"{SUPERPOWERS_CHECKSUM}\",\n  strip_components = 1 }}\n"
        )
    }

    #[test]
    fn installed_and_pinned_plans_nothing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".mise.toml"), pinned_mise_toml()).unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        fake.script(
            "claude plugin list",
            Output {
                status: 0,
                stdout: "superpowers 6.2.0\n".into(),
                stderr: String::new(),
            },
        );
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(superpowers().plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn missing_plugin_plans_pin_and_install() {
        let dir = tempfile::tempdir().unwrap();
        let (manifest, lock) = ctx_parts();
        let fake = FakeRunner::new();
        fake.script(
            "claude plugin list",
            Output {
                status: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let actions = superpowers().plan(&ctx).unwrap();
        let descs: Vec<String> = actions.iter().map(|a| a.describe()).collect();
        assert!(descs.iter().any(|d| d.contains("pin http:superpowers")));
        assert!(
            descs
                .iter()
                .any(|d| d.contains("marketplace add {mise-where:http:superpowers}"))
        );
        assert!(
            descs
                .iter()
                .any(|d| d.contains("plugin install superpowers@superpowers-dev"))
        );
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
    fn non_registry_superpowers_version_is_rejected() {
        let (mut manifest, lock) = ctx_parts();
        manifest.capabilities.get_mut("workflows").unwrap().version = Some("9.9.9".into());
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: std::path::Path::new("."),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(superpowers().plan(&ctx).is_err());
    }

    #[test]
    fn components_report_their_slot_and_provider() {
        assert_eq!(superpowers().capability(), Capability::Workflows);
        assert_eq!(superpowers().provider(), "superpowers");
        assert_eq!(frontend_design().capability(), Capability::Frontend);
        assert_eq!(frontend_design().provider(), "frontend-design");
    }
}
