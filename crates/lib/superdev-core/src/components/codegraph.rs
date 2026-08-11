//! components/codegraph.rs — the code-index capability via codegraph.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Component, Ctx};
use crate::error::Result;

/// mise `[tools]` key for codegraph.
pub const CODEGRAPH_MISE_TOOL: &str = "npm:@colbymchenry/codegraph";
/// Directory `codegraph init` creates.
pub const CODEGRAPH_INDEX_DIR: &str = ".codegraph";

/// The codegraph provider.
pub struct Codegraph;

impl Component for Codegraph {
    fn capability(&self) -> Capability {
        Capability::CodeIndex
    }

    fn provider(&self) -> &'static str {
        "codegraph"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let config = ctx
            .config(Capability::CodeIndex)
            .expect("planned only when enabled");
        let version = config.version.clone().expect("registry pins codegraph");
        let mut actions = Vec::new();
        let value = format!("\"{version}\"");
        let current = match std::fs::read_to_string(ctx.root.join(".mise.toml")) {
            Ok(s) => super::mise::current_pin(&s, CODEGRAPH_MISE_TOOL)?,
            Err(_) => None,
        };
        if current.as_deref() != Some(value.as_str()) {
            actions.push(Action::SetMisePin {
                tool: CODEGRAPH_MISE_TOOL.into(),
                value_toml: value,
            });
        }
        if !ctx.root.join(CODEGRAPH_INDEX_DIR).is_dir() {
            // Through mise: the tool is pinned in this repo's `.mise.toml`,
            // and mise install puts it on no PATH the running process can see.
            actions.push(Action::Run {
                program: "mise".into(),
                args: vec![
                    "exec".into(),
                    "--".into(),
                    "codegraph".into(),
                    "init".into(),
                ],
                purpose: "build the code index".into(),
                undo: None,
                optional: false,
            });
        }
        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Component, Ctx};
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;

    #[test]
    fn fresh_repo_plans_pin_and_init() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let descs: Vec<String> = Codegraph
            .plan(&ctx)
            .unwrap()
            .iter()
            .map(|a| a.describe())
            .collect();
        assert!(descs.iter().any(|d| d.contains(CODEGRAPH_MISE_TOOL)));
        // Never bare `codegraph`: it exists only inside the repo's mise env.
        assert!(
            descs
                .iter()
                .any(|d| d.contains("run `mise exec -- codegraph init`")),
            "descs: {descs:?}"
        );
    }

    #[test]
    fn indexed_repo_with_pin_plans_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let version = manifest.capabilities["code-index"].version.clone().unwrap();
        std::fs::write(
            dir.path().join(".mise.toml"),
            crate::components::mise::set_pin("", CODEGRAPH_MISE_TOOL, &format!("\"{version}\""))
                .unwrap(),
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join(CODEGRAPH_INDEX_DIR)).unwrap();
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        assert!(Codegraph.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn reports_its_slot_and_provider() {
        assert_eq!(Codegraph.capability(), Capability::CodeIndex);
        assert_eq!(Codegraph.provider(), "codegraph");
    }
}
