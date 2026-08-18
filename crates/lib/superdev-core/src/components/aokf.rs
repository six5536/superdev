//! components/aokf.rs — the knowledge capability: AOKF is native to superdev.
//! The blueprint's files ship inside the binary.

use std::path::Path;

use crate::action::{Action, Ownership};
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::error::Result;
use crate::manifest::Manifest;

use super::item::{self, ManagedItem};

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
        asset!("aokf/agents/SUPERPOWERS.md"),
    ),
    (
        "mattpocock-skills",
        ".agents/MATT-POCOCK-SKILLS.md",
        asset!("aokf/agents/MATT-POCOCK-SKILLS.md"),
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
        asset!("aokf/agents/aokf/SPEC.md"),
        Ownership::Owned,
        "AOKF specification",
    ),
    (
        ".agents/VALIDATION.md",
        asset!("aokf/agents/VALIDATION.md"),
        Ownership::Owned,
        "validation rules",
    ),
    (
        ".agents/CODING.md",
        asset!("aokf/agents/CODING.md"),
        Ownership::Scaffold,
        "coding rules",
    ),
    (
        ".agents/PROSE.md",
        asset!("aokf/agents/PROSE.md"),
        Ownership::Scaffold,
        "prose rules",
    ),
    (
        "AGENTS.md",
        asset!("aokf/AGENTS.md"),
        Ownership::Scaffold,
        "agent entry point",
    ),
    (
        "knowledge/index.md",
        asset!("aokf/knowledge/index.md"),
        Ownership::Scaffold,
        "bundle index",
    ),
    (
        "knowledge/manifest.aokf.yaml",
        asset!("aokf/knowledge/manifest.aokf.yaml"),
        Ownership::Scaffold,
        "bundle manifest",
    ),
    (
        "knowledge/specs/index.md",
        asset!("aokf/knowledge/specs/index.md"),
        Ownership::Scaffold,
        "specs index",
    ),
    (
        "knowledge/api-contracts.md",
        asset!("aokf/knowledge/api-contracts.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/architectural-rules.md",
        asset!("aokf/knowledge/architectural-rules.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/architecture.md",
        asset!("aokf/knowledge/architecture.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/backlog.md",
        asset!("aokf/knowledge/backlog.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/coding-standards.md",
        asset!("aokf/knowledge/coding-standards.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/configuration.md",
        asset!("aokf/knowledge/configuration.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/constraints-non-goals.md",
        asset!("aokf/knowledge/constraints-non-goals.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/definition-of-done.md",
        asset!("aokf/knowledge/definition-of-done.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/dependency-policy.md",
        asset!("aokf/knowledge/dependency-policy.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/development-commands.md",
        asset!("aokf/knowledge/development-commands.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/development-procedure.md",
        asset!("aokf/knowledge/development-procedure.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/directory-structure.md",
        asset!("aokf/knowledge/directory-structure.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/error-handling.md",
        asset!("aokf/knowledge/error-handling.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/glossary.md",
        asset!("aokf/knowledge/glossary.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/project-overview.md",
        asset!("aokf/knowledge/project-overview.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/release-procedure.md",
        asset!("aokf/knowledge/release-procedure.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/security-requirements.md",
        asset!("aokf/knowledge/security-requirements.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/software-components.md",
        asset!("aokf/knowledge/software-components.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/technology-stack.md",
        asset!("aokf/knowledge/technology-stack.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/testing-strategy.md",
        asset!("aokf/knowledge/testing-strategy.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
];

/// The knowledge-lifecycle skills, carried by this component so they exist
/// exactly where a bundle exists: (skill name, embedded SKILL.md).
pub(crate) const SKILLS: [(&str, &str); 2] = [
    ("aokf-bootstrap", asset!("aokf/skills/aokf-bootstrap/SKILL.md")),
    (
        "aokf-maintain",
        asset!("aokf/skills/aokf-maintain/SKILL.md"),
    ),
];

/// Where Claude Code reads hook registrations. Shared with the user's own
/// hooks, so only superdev's array element is managed.
const SETTINGS_PATH: &str = ".claude/settings.json";
/// The array the hook entry lives in.
const HOOK_POINTER: &str = "hooks.PostToolUse";
/// What identifies superdev's element among the user's.
const HOOK_MARKER: &str = "superdev aokf hook validate";
/// The registration itself: validate the bundle after an Edit/Write. It
/// ships with this capability, so a `--no-knowledge` repo never gets a hook
/// blocking edits to a `knowledge/` directory superdev does not manage.
const HOOK_ELEMENT: &str = r#"{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}"#;

/// Release, at adoption time, every aokf skill the repo already has under
/// its own name and with its own content. Returns the lines to print.
pub(crate) fn adopt_existing(root: &Path, manifest: &mut Manifest) -> Vec<String> {
    super::skills::adopt_existing(root, Capability::Knowledge, &SKILLS, manifest)
}

/// The native AOKF provider.
pub struct Aokf;

/// Everything the knowledge capability keeps in the repo, as one list the
/// driver derives both `plan` and `owned` from.
fn items(ctx: &Ctx<'_>) -> Vec<ManagedItem> {
    let repo_name = ctx
        .root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();
    let mut items = Vec::new();
    for (path, content, ownership, reason) in FILES {
        // Every other asset is shipped verbatim: a stray `{name}` in prose
        // is not a placeholder.
        let content = match *path {
            NAMED_ASSET => content.replace("{name}", &repo_name),
            "AGENTS.md" => match workflow_override(ctx) {
                Some((path, _)) => content.replace("{workflows_overrides}", &format!("@{path}")),
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
        items.push(match ownership {
            Ownership::Owned => ManagedItem::OwnedFile {
                path: (*path).to_string(),
                content,
                reason: (*reason).to_string(),
            },
            Ownership::Scaffold => ManagedItem::Scaffold {
                path: (*path).to_string(),
                content,
                reason: (*reason).to_string(),
            },
        });
    }
    if let Some((path, content)) = workflow_override(ctx) {
        items.push(ManagedItem::OwnedFile {
            path: path.to_string(),
            content: content.to_string(),
            reason: "workflows overrides".to_string(),
        });
    }
    items.push(ManagedItem::EnsureLine {
        path: CLAUDE_ENTRY_PATH.into(),
        line: CLAUDE_ENTRY_LINE.into(),
        reason: "make Claude Code read AGENTS.md".into(),
    });
    items.push(ManagedItem::JsonEntry {
        path: MCP_PATH.into(),
        pointer: MCP_POINTER.into(),
        marker: None,
        value_json: MCP_VALUE.into(),
    });
    let custom = ctx
        .config(Capability::Knowledge)
        .map(|c| c.custom.as_slice())
        .unwrap_or_default();
    items.extend(super::skills::skill_items(&SKILLS, custom));
    items.push(ManagedItem::JsonEntry {
        path: SETTINGS_PATH.into(),
        pointer: HOOK_POINTER.into(),
        marker: Some(HOOK_MARKER.into()),
        value_json: HOOK_ELEMENT.into(),
    });
    items
}

impl Component for Aokf {
    fn capability(&self) -> Capability {
        Capability::Knowledge
    }

    fn provider(&self) -> &'static str {
        "aokf"
    }

    fn plan(&self, ctx: &Ctx<'_>) -> Result<Vec<Action>> {
        Ok(item::plan_items(ctx.root, &items(ctx)))
    }

    fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim> {
        item::claims(&items(ctx))
    }
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
    fn ships_the_lifecycle_skills_and_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        let actions = plan_in(dir.path());
        let descs: Vec<String> = actions.iter().map(Action::describe).collect();
        for (name, _) in SKILLS {
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!(".claude/skills/{name}/SKILL.md"))),
                "{descs:?}"
            );
        }
        assert!(
            descs
                .iter()
                .any(|d| d.contains("superdev aokf hook validate")),
            "{descs:?}"
        );
    }

    #[test]
    fn a_custom_skill_is_released_and_the_hook_stays() {
        use crate::component::Claim;
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("knowledge").unwrap().custom = vec!["aokf-maintain".into()];
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let keys: Vec<String> = Aokf.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(
            !keys.iter().any(|k| k.contains("aokf-maintain")),
            "{keys:?}"
        );
        assert!(keys.contains(&".claude/skills/aokf-bootstrap/SKILL.md".to_string()));
        assert!(keys.contains(
            &".claude/settings.json:hooks.PostToolUse[superdev aokf hook validate]".to_string()
        ));
    }

    #[test]
    fn a_stale_hook_entry_replans_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        // Same marker, older shape: must be replaced, so it must be planned.
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(SETTINGS_PATH),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"superdev aokf hook validate"}]}]}}"#,
        )
        .unwrap();
        let actions = plan_in(dir.path());
        assert!(
            actions
                .iter()
                .any(|a| a.describe().contains("hooks.PostToolUse")),
            "{actions:?}"
        );
    }

    #[test]
    fn adoption_keeps_the_repos_own_lifecycle_skills() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".claude/skills/aokf-maintain/SKILL.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "# Ours, thanks\n").unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let lines = adopt_existing(dir.path(), &mut manifest);
        assert_eq!(manifest.capabilities["knowledge"].custom, ["aokf-maintain"]);
        assert_eq!(
            lines,
            vec![format!(
                "knowledge: kept your aokf-maintain — marked custom in {}",
                crate::manifest::CONFIG_PATH
            )]
        );
    }

    /// The starter bundle must itself conform: write every knowledge file
    /// the component plans into an empty repo, then run the embedded
    /// validator over it at level 2. A skeleton that ships broken would fail
    /// the very hook superdev installs beside it.
    #[test]
    fn the_seeded_bundle_validates_clean_at_level_2() {
        let dir = tempfile::tempdir().unwrap();
        for action in plan_in(dir.path()) {
            if let Action::WriteFile { path, content, .. } = action {
                let target = dir.path().join(&path);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::write(target, content).unwrap();
            }
        }
        let bundle = crate::aokf::bundle::load_bundle(&dir.path().join("knowledge")).unwrap();
        let report = crate::aokf::validate::validate(&bundle, dir.path(), 2);
        assert!(report.findings.is_empty(), "{:#?}", report.findings);
        assert_eq!(report.achieved_level, 2);
        assert!(report.concept_count >= 20, "{}", report.concept_count);
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
                Action::SetJsonKey { .. }
                | Action::EnsureLine { .. }
                | Action::EnsureJsonArrayElement { .. } => None,
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
                Action::EnsureJsonArrayElement {
                    path, value_json, ..
                } => {
                    let json = format!("{{ \"hooks\": {{ \"PostToolUse\": [{value_json}] }} }}");
                    std::fs::write(dir.path().join(path), json).unwrap();
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
