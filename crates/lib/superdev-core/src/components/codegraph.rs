//! components/codegraph.rs — the code-index capability via codegraph.

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;
use crate::registry::CODEGRAPH_PLATFORMS;

use super::item::{ManagedItem, claims, plan_items};

/// mise `[tools]` key for codegraph.
pub const CODEGRAPH_MISE_TOOL: &str = "http:codegraph";
/// Directory `codegraph init` creates.
pub const CODEGRAPH_INDEX_DIR: &str = ".codegraph";

/// The instruction file telling agents the index exists and how to query it,
/// imported by the `.agents/superdev.md` aggregator.
const INSTRUCTIONS_PATH: &str = ".agents/codegraph.md";
const INSTRUCTIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/codegraph/codegraph.md"
));

/// codegraph's MCP server in the shared `.mcp.json`, launched through mise
/// because the pinned binary is on no PATH the client can see.
const MCP_POINTER: &str = "mcpServers.codegraph";
const MCP_VALUE: &str =
    r#"{"command":"mise","args":["exec","http:codegraph","--","codegraph","serve","--mcp"]}"#;

/// The declarative half: the instruction file and the MCP registration.
/// The pin and the init run stay hand-written below.
fn items() -> Vec<ManagedItem> {
    vec![
        ManagedItem::OwnedFile {
            path: INSTRUCTIONS_PATH.into(),
            content: INSTRUCTIONS.into(),
            reason: "code-index instructions".into(),
        },
        ManagedItem::JsonEntry {
            path: ".mcp.json".into(),
            pointer: MCP_POINTER.into(),
            marker: None,
            value_json: MCP_VALUE.into(),
        },
    ]
}

/// The codegraph provider.
pub struct Codegraph;

/// The `.mise.toml` value for the pinned release: one checksummed bundle per
/// platform, in registry order, so the same version always renders the same
/// fragment.
fn pin_value(version: &str) -> String {
    let platforms: Vec<String> = CODEGRAPH_PLATFORMS
        .iter()
        .map(|(platform, url, checksum)| {
            format!("\"{platform}\" = {{ url = \"{url}\", checksum = \"{checksum}\" }}")
        })
        .collect();
    format!(
        "{{ version = \"{version}\", platforms = {{ {} }}, strip_components = 1 }}",
        platforms.join(", ")
    )
}

impl Component for Codegraph {
    fn capability(&self) -> Option<Capability> {
        Some(Capability::CodeIndex)
    }

    fn provider(&self) -> &'static str {
        "codegraph"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        // The registry entry only feeds the pin value; `planned_pin` owns the
        // version validation, like the other pinned components.
        let version = crate::registry::entry_for(Capability::CodeIndex, "codegraph")
            .and_then(|e| e.version)
            .expect("registry pins codegraph")
            .version;
        let mut actions = Vec::new();
        if let Some(pin) = super::pin::planned_pin(
            ctx,
            Capability::CodeIndex,
            "codegraph",
            CODEGRAPH_MISE_TOOL,
            &pin_value(version),
        )? {
            actions.push(pin);
        }
        actions.extend(plan_items(ctx.root, &items()));
        if !ctx.root.join(CODEGRAPH_INDEX_DIR).is_dir() {
            // Through mise: the tool is pinned in this repo's `.mise.toml`,
            // and mise install puts it on no PATH the running process can see.
            // Naming the tool keeps mise from installing the repo's whole
            // toolchain first — one unbuildable pin of the user's would
            // otherwise fail this run.
            actions.push(Action::Run {
                program: "mise".into(),
                args: vec![
                    "exec".into(),
                    CODEGRAPH_MISE_TOOL.into(),
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

    fn owned(&self, _ctx: &Ctx<'_>) -> Vec<Claim> {
        let mut owned = vec![Claim::MisePin(CODEGRAPH_MISE_TOOL.to_string())];
        owned.extend(claims(&items()));
        owned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Claim, Component, Ctx};
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
            content: crate::content::test_snapshot(),
        };
        let descs: Vec<String> = Codegraph
            .plan(&ctx)
            .unwrap()
            .iter()
            .map(|a| a.describe())
            .collect();
        assert!(descs.iter().any(|d| d.contains(CODEGRAPH_MISE_TOOL)));
        // The pin carries every platform's bundle and checksum, so a repo
        // synced on one machine installs the same bytes on the others.
        let pin = pin_value("1.5.0");
        for (platform, url, checksum) in CODEGRAPH_PLATFORMS {
            assert!(pin.contains(platform), "{pin}");
            assert!(pin.contains(url), "{pin}");
            assert!(pin.contains(checksum), "{pin}");
        }
        assert!(pin.contains("strip_components = 1"), "{pin}");
        // Never bare `codegraph`: it exists only inside the repo's mise env.
        assert!(
            descs
                .iter()
                .any(|d| d.contains("run `mise exec http:codegraph -- codegraph init`")),
            "descs: {descs:?}"
        );
        // The agent wiring: the instruction file and the MCP registration.
        assert!(
            descs
                .iter()
                .any(|d| d.contains("write .agents/codegraph.md")),
            "descs: {descs:?}"
        );
        assert!(
            descs
                .iter()
                .any(|d| d.contains("set mcpServers.codegraph in .mcp.json")),
            "descs: {descs:?}"
        );
        let claimed: Vec<String> = Codegraph.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(claimed.contains(&".agents/codegraph.md".to_string()));
        assert!(claimed.contains(&".mcp.json:mcpServers.codegraph".to_string()));
    }

    #[test]
    fn indexed_repo_with_pin_plans_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let version = manifest.capabilities["code-index"][0]
            .version
            .clone()
            .unwrap();
        std::fs::write(
            dir.path().join(".mise.toml"),
            crate::components::mise::set_pin("", CODEGRAPH_MISE_TOOL, &pin_value(&version))
                .unwrap(),
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join(CODEGRAPH_INDEX_DIR)).unwrap();
        std::fs::create_dir_all(dir.path().join(".agents")).unwrap();
        std::fs::write(dir.path().join(INSTRUCTIONS_PATH), INSTRUCTIONS).unwrap();
        std::fs::write(
            dir.path().join(".mcp.json"),
            format!(r#"{{"mcpServers":{{"codegraph":{MCP_VALUE}}}}}"#),
        )
        .unwrap();
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
            content: crate::content::test_snapshot(),
        };
        assert!(Codegraph.plan(&ctx).unwrap().is_empty());
    }

    #[test]
    fn a_version_off_the_registry_default_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("code-index").unwrap()[0].version = Some("9.9.9".into());
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
            content: crate::content::test_snapshot(),
        };
        // superdev ships checksums for one release only, so it can install
        // that one only.
        let err = Codegraph.plan(&ctx).unwrap_err().to_string();
        assert!(err.contains("must match the registry default"), "{err}");
    }

    #[test]
    fn reports_its_slot_and_provider() {
        assert_eq!(Codegraph.capability(), Some(Capability::CodeIndex));
        assert_eq!(Codegraph.provider(), "codegraph");
    }
}
