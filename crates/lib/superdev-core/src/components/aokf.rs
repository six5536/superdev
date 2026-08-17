//! components/aokf.rs — the knowledge capability: AOKF is native to superdev.
//! The blueprint's files ship inside the binary.

use crate::action::{Action, Ownership};
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;

macro_rules! asset {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $rel))
    };
}

/// The one asset carrying a `{name}` token, replaced with the repo's name.
const NAMED_ASSET: &str = "knowledge/manifest.aokf.yaml";

/// The workflow-framework override each provider gets, as
/// (provider id, target path, embedded asset).
const WORKFLOW_OVERRIDES: [(&str, &str, &str); 2] = [
    (
        "superpowers",
        ".agents/SUPERPOWERS.md",
        asset!("agents/SUPERPOWERS.md"),
    ),
    (
        "mattpocock-skills",
        ".agents/MATT-POCOCK-SKILLS.md",
        asset!("agents/MATT-POCOCK-SKILLS.md"),
    ),
];

/// The enabled workflows provider's override, when there is one.
fn workflow_override(ctx: &Ctx<'_>) -> Option<(&'static str, &'static str)> {
    let config = ctx.config(Capability::Workflows)?;
    WORKFLOW_OVERRIDES
        .iter()
        .find(|(provider, ..)| *provider == config.provider)
        .map(|(_, path, content)| (*path, *content))
}

/// Where agent tools find MCP servers, and superdev's key inside it. The file
/// is shared with the user's own servers, so only this key is managed.
const MCP_PATH: &str = ".mcp.json";
const MCP_POINTER: &str = "mcpServers.superdev-aokf";
/// The registration itself: the installed binary serving this repo's bundle.
const MCP_VALUE: &str = r#"{"command":"superdev","args":["mcp","aokf"]}"#;

/// Claude Code reads CLAUDE.md, not AGENTS.md: without this import, every
/// rule superdev writes into AGENTS.md is invisible to it. Behaves like the
/// .gitignore lines — added when missing, never rewritten, never locked.
const CLAUDE_ENTRY_PATH: &str = "CLAUDE.md";
const CLAUDE_ENTRY_LINE: &str = "@AGENTS.md";

/// (target path, asset content, ownership, reason)
const FILES: &[(&str, &str, Ownership, &str)] = &[
    (
        ".agents/aokf/SPEC.md",
        asset!("agents/aokf/SPEC.md"),
        Ownership::Owned,
        "AOKF specification",
    ),
    (
        ".agents/VALIDATION.md",
        asset!("agents/VALIDATION.md"),
        Ownership::Owned,
        "validation rules",
    ),
    (
        ".agents/CODING.md",
        asset!("agents/CODING.md"),
        Ownership::Scaffold,
        "coding rules",
    ),
    (
        ".agents/PROSE.md",
        asset!("agents/PROSE.md"),
        Ownership::Scaffold,
        "prose rules",
    ),
    (
        "AGENTS.md",
        asset!("AGENTS.md"),
        Ownership::Scaffold,
        "agent entry point",
    ),
    (
        "knowledge/index.md",
        asset!("knowledge/index.md"),
        Ownership::Scaffold,
        "bundle index",
    ),
    (
        "knowledge/manifest.aokf.yaml",
        asset!("knowledge/manifest.aokf.yaml"),
        Ownership::Scaffold,
        "bundle manifest",
    ),
    (
        "knowledge/specs/index.md",
        asset!("knowledge/specs/index.md"),
        Ownership::Scaffold,
        "specs index",
    ),
];

/// The native AOKF provider.
pub struct Aokf;

impl Component for Aokf {
    fn capability(&self) -> Capability {
        Capability::Knowledge
    }

    fn provider(&self) -> &'static str {
        "aokf"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        let repo_name = ctx
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();
        let mut actions = Vec::new();
        for (path, content, ownership, reason) in FILES {
            // Every other asset is shipped verbatim: a stray `{name}` in prose
            // is not a placeholder.
            let content = match *path {
                NAMED_ASSET => content.replace("{name}", &repo_name),
                "AGENTS.md" => match workflow_override(ctx) {
                    Some((path, _)) => {
                        content.replace("{workflows_overrides}", &format!("@{path}"))
                    }
                    None => {
                        content
                            .lines()
                            .filter(|l| *l != "{workflows_overrides}")
                            .collect::<Vec<_>>()
                            .join("\n")
                            + "\n"
                    }
                },
                _ => (*content).to_string(),
            };
            let existing = std::fs::read_to_string(ctx.root.join(path)).ok();
            let wanted = match ownership {
                Ownership::Owned => existing.as_deref() != Some(content.as_str()),
                Ownership::Scaffold => existing.is_none(),
            };
            if wanted {
                actions.push(Action::WriteFile {
                    path: (*path).to_string(),
                    content,
                    ownership: *ownership,
                    reason: (*reason).to_string(),
                });
            }
        }
        if let Some((path, content)) = workflow_override(ctx) {
            let existing = std::fs::read_to_string(ctx.root.join(path)).ok();
            if existing.as_deref() != Some(content) {
                actions.push(Action::WriteFile {
                    path: path.to_string(),
                    content: content.to_string(),
                    ownership: Ownership::Owned,
                    reason: "workflows overrides".to_string(),
                });
            }
        }
        let claude = std::fs::read_to_string(ctx.root.join(CLAUDE_ENTRY_PATH)).unwrap_or_default();
        // Exact whole-line match, the same rule the engine applies.
        if !claude.lines().any(|l| l == CLAUDE_ENTRY_LINE) {
            actions.push(Action::EnsureLine {
                path: CLAUDE_ENTRY_PATH.into(),
                line: CLAUDE_ENTRY_LINE.into(),
                reason: "make Claude Code read AGENTS.md".into(),
            });
        }
        if mcp_registration_missing(ctx.root) {
            actions.push(Action::SetJsonKey {
                path: MCP_PATH.to_string(),
                pointer: MCP_POINTER.to_string(),
                value_json: MCP_VALUE.to_string(),
            });
        }
        Ok(actions)
    }

    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim> {
        let mut claims: Vec<Claim> = FILES
            .iter()
            .filter(|(_, _, ownership, _)| *ownership == Ownership::Owned)
            .map(|(path, ..)| Claim::File((*path).to_string()))
            .collect();
        if let Some((path, _)) = workflow_override(ctx) {
            claims.push(Claim::File(path.to_string()));
        }
        claims.push(Claim::JsonKey {
            path: MCP_PATH.into(),
            pointer: MCP_POINTER.into(),
        });
        claims
    }
}

/// True when `.mcp.json` does not already carry superdev's entry. Compares the
/// parsed values, so reformatting or reordering the file is not drift. An
/// unreadable or malformed file counts as missing: the engine reports why.
fn mcp_registration_missing(root: &std::path::Path) -> bool {
    let wanted: serde_json::Value =
        serde_json::from_str(MCP_VALUE).expect("the registration literal is valid JSON");
    let current = std::fs::read_to_string(root.join(MCP_PATH))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|root| {
            MCP_POINTER
                .split('.')
                .try_fold(&root, |value, segment| value.get(segment))
                .cloned()
        });
    current.as_ref() != Some(&wanted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, Ownership};
    use crate::component::{Component, Ctx};
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::FakeRunner;

    fn plan_in(dir: &std::path::Path) -> Vec<Action> {
        let manifest = Manifest::default_for("0.1.0", &[]);
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir,
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        Aokf.plan(&ctx).unwrap()
    }

    fn plan_with_provider(dir: &std::path::Path, provider: Option<&str>) -> Vec<Action> {
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        match provider {
            Some(provider) => {
                let workflows = manifest.capabilities.get_mut("workflows").unwrap();
                workflows.provider = provider.into();
            }
            None => {
                manifest.capabilities.remove("workflows");
            }
        }
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir,
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        Aokf.plan(&ctx).unwrap()
    }

    #[test]
    fn the_override_file_follows_the_workflows_provider() {
        let dir = tempfile::tempdir().unwrap();
        let writes = |actions: &[Action]| -> Vec<String> {
            actions
                .iter()
                .filter_map(|a| match a {
                    Action::WriteFile { path, .. } => Some(path.clone()),
                    _ => None,
                })
                .collect()
        };
        let superpowers = writes(&plan_with_provider(dir.path(), Some("superpowers")));
        assert!(superpowers.contains(&".agents/SUPERPOWERS.md".to_string()));
        assert!(!superpowers.contains(&".agents/MATT-POCOCK-SKILLS.md".to_string()));
        let matt = writes(&plan_with_provider(dir.path(), Some("mattpocock-skills")));
        assert!(matt.contains(&".agents/MATT-POCOCK-SKILLS.md".to_string()));
        assert!(!matt.contains(&".agents/SUPERPOWERS.md".to_string()));
        let none = writes(&plan_with_provider(dir.path(), None));
        assert!(
            !none
                .iter()
                .any(|p| p.starts_with(".agents/SUPERPOWERS")
                    || p.starts_with(".agents/MATT-POCOCK"))
        );
    }

    #[test]
    fn the_scaffold_imports_match_the_provider() {
        let dir = tempfile::tempdir().unwrap();
        let agents_content = |provider: Option<&str>| {
            plan_with_provider(dir.path(), provider)
                .into_iter()
                .find_map(|a| match a {
                    Action::WriteFile { path, content, .. } if path == "AGENTS.md" => Some(content),
                    _ => None,
                })
                .unwrap()
        };
        assert!(agents_content(Some("superpowers")).contains("@.agents/SUPERPOWERS.md"));
        let matt = agents_content(Some("mattpocock-skills"));
        assert!(matt.contains("@.agents/MATT-POCOCK-SKILLS.md"));
        assert!(!matt.contains("SUPERPOWERS"));
        let none = agents_content(None);
        assert!(!none.contains("{workflows_overrides}"));
        assert!(!none.contains("SUPERPOWERS") && !none.contains("MATT-POCOCK"));
    }

    #[test]
    fn owned_follows_the_provider_too() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("workflows").unwrap().provider = "mattpocock-skills".into();
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let keys: Vec<String> = Aokf.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(keys.contains(&".agents/MATT-POCOCK-SKILLS.md".to_string()));
        assert!(!keys.contains(&".agents/SUPERPOWERS.md".to_string()));
    }

    #[test]
    fn fresh_repo_plans_every_file_with_name_substituted() {
        let dir = tempfile::tempdir().unwrap();
        let actions = plan_in(dir.path());
        let paths: Vec<&str> = actions
            .iter()
            .filter_map(|a| match a {
                Action::WriteFile { path, .. } => Some(path.as_str()),
                Action::SetJsonKey { .. } | Action::EnsureLine { .. } => None,
                other => panic!("unexpected action {other:?}"),
            })
            .collect();
        assert!(paths.contains(&".agents/aokf/SPEC.md"));
        assert!(paths.contains(&"AGENTS.md"));
        let manifest_action = actions.iter().find_map(|a| match a {
            Action::WriteFile { path, content, .. } if path == "knowledge/manifest.aokf.yaml" => {
                Some(content)
            }
            _ => None,
        });
        let repo_name = dir.path().file_name().unwrap().to_str().unwrap();
        assert!(
            manifest_action
                .unwrap()
                .contains(&format!("name: {repo_name}-knowledge"))
        );
        // Every other asset ships byte-for-byte as embedded. AGENTS.md is
        // templated (the workflow-override token), and the workflow override
        // itself is planned outside the FILES table — both are exempt.
        for action in &actions {
            let Action::WriteFile { path, content, .. } = action else {
                continue;
            };
            if path == NAMED_ASSET || path == "AGENTS.md" {
                continue;
            }
            let Some((_, asset, ..)) = FILES.iter().find(|(p, ..)| p == path) else {
                continue;
            };
            assert_eq!(asset, &content.as_str(), "{path} was rewritten");
        }
    }

    #[test]
    fn scaffolds_are_not_replanned_but_owned_drift_is() {
        let dir = tempfile::tempdir().unwrap();
        // Simulate a previous apply: write everything as planned.
        for a in plan_in(dir.path()) {
            match a {
                Action::WriteFile { path, content, .. } => {
                    let p = dir.path().join(path);
                    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                    std::fs::write(p, content).unwrap();
                }
                Action::SetJsonKey {
                    path, value_json, ..
                } => {
                    let json =
                        format!("{{ \"mcpServers\": {{ \"superdev-aokf\": {value_json} }} }}");
                    std::fs::write(dir.path().join(path), json).unwrap();
                }
                Action::EnsureLine { path, line, .. } => {
                    let p = dir.path().join(path);
                    let mut content = std::fs::read_to_string(&p).unwrap_or_default();
                    content.push_str(&line);
                    content.push('\n');
                    std::fs::write(p, content).unwrap();
                }
                other => panic!("unexpected action {other:?}"),
            }
        }
        assert!(plan_in(dir.path()).is_empty());
        // A user edit to a scaffold stays untouched…
        std::fs::write(dir.path().join("AGENTS.md"), "customised").unwrap();
        // …but an edit to an owned file is drift.
        std::fs::write(dir.path().join(".agents/aokf/SPEC.md"), "tampered").unwrap();
        let replanned = plan_in(dir.path());
        let paths: Vec<String> = replanned
            .iter()
            .map(|a| match a {
                Action::WriteFile {
                    path, ownership, ..
                } => {
                    assert_eq!(*ownership, Ownership::Owned);
                    path.clone()
                }
                other => panic!("unexpected action {other:?}"),
            })
            .collect();
        assert_eq!(paths, vec![".agents/aokf/SPEC.md".to_string()]);
    }

    #[test]
    fn claude_md_gets_the_agents_import() {
        // No CLAUDE.md at all: plan the line (the engine creates the file).
        let dir = tempfile::tempdir().unwrap();
        let ensure = plan_in(dir.path()).into_iter().find_map(|a| match a {
            Action::EnsureLine { path, line, .. } => Some((path, line)),
            _ => None,
        });
        assert_eq!(
            ensure,
            Some(("CLAUDE.md".to_string(), "@AGENTS.md".to_string()))
        );

        // A CLAUDE.md of the user's own: plan the append, touch nothing else.
        std::fs::write(dir.path().join("CLAUDE.md"), "# My rules\n").unwrap();
        assert!(
            plan_in(dir.path())
                .iter()
                .any(|a| matches!(a, Action::EnsureLine { .. }))
        );

        // The line present (anywhere, exact whole-line): nothing to plan.
        std::fs::write(dir.path().join("CLAUDE.md"), "# My rules\n@AGENTS.md\n").unwrap();
        assert!(
            !plan_in(dir.path())
                .iter()
                .any(|a| matches!(a, Action::EnsureLine { .. }))
        );

        // A substring is not the line: `see @AGENTS.md` does not satisfy it.
        std::fs::write(dir.path().join("CLAUDE.md"), "see @AGENTS.md inline\n").unwrap();
        assert!(
            plan_in(dir.path())
                .iter()
                .any(|a| matches!(a, Action::EnsureLine { .. }))
        );
    }

    #[test]
    fn plans_mcp_registration_when_missing_and_not_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let registration = plan_in(dir.path())
            .into_iter()
            .find(|a| matches!(a, Action::SetJsonKey { .. }))
            .expect("a fresh repo registers the MCP server");
        assert_eq!(
            registration,
            Action::SetJsonKey {
                path: MCP_PATH.into(),
                pointer: MCP_POINTER.into(),
                value_json: MCP_VALUE.into(),
            }
        );

        // The same entry, written differently: key order and whitespace are
        // not drift, because the comparison is semantic.
        std::fs::write(
            dir.path().join(MCP_PATH),
            "{\n  \"mcpServers\": {\n    \"superdev-aokf\": {\n      \"args\": [\"mcp\", \"aokf\"],\n      \"command\": \"superdev\"\n    }\n  }\n}\n",
        )
        .unwrap();
        assert!(
            !plan_in(dir.path())
                .iter()
                .any(|a| matches!(a, Action::SetJsonKey { .. }))
        );

        // A different command is drift; so is an unparseable file.
        std::fs::write(
            dir.path().join(MCP_PATH),
            "{\"mcpServers\":{\"superdev-aokf\":{\"command\":\"old\"}}}",
        )
        .unwrap();
        assert!(
            plan_in(dir.path())
                .iter()
                .any(|a| matches!(a, Action::SetJsonKey { .. }))
        );
        std::fs::write(dir.path().join(MCP_PATH), "not json").unwrap();
        assert!(
            plan_in(dir.path())
                .iter()
                .any(|a| matches!(a, Action::SetJsonKey { .. }))
        );
    }

    #[test]
    fn reports_its_slot_and_provider() {
        assert_eq!(Aokf.capability(), crate::capability::Capability::Knowledge);
        assert_eq!(Aokf.provider(), "aokf");
    }
}
