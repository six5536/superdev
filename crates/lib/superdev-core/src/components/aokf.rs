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
pub(crate) const FILES: &[(&str, &str, Ownership, &str)] = &[
    (
        ".agents/aokf/SPEC.md",
        asset!("aokf/agents/aokf/SPEC.md"),
        Ownership::Owned,
        "AOKF specification",
    ),
    (
        ".agents/aokf.md",
        asset!("aokf/agents/aokf.md"),
        Ownership::Owned,
        "knowledge instructions",
    ),
    (
        "knowledge/index.md",
        asset!("knowledge/concepts/index.md"),
        Ownership::Scaffold,
        "bundle index",
    ),
    (
        "knowledge/manifest.aokf.yaml",
        asset!("knowledge/concepts/manifest.aokf.yaml"),
        Ownership::Scaffold,
        "bundle manifest",
    ),
    (
        "knowledge/specs/index.md",
        asset!("knowledge/concepts/specs/index.md"),
        Ownership::Scaffold,
        "specs index",
    ),
    (
        "knowledge/plans/index.md",
        asset!("knowledge/concepts/plans/index.md"),
        Ownership::Scaffold,
        "plans index",
    ),
    (
        "knowledge/issue-tracker.md",
        asset!("knowledge/concepts/issue-tracker.md"),
        Ownership::Scaffold,
        "issue-tracker convention",
    ),
    (
        "knowledge/api-contracts.md",
        asset!("knowledge/concepts/api-contracts.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/architectural-rules.md",
        asset!("knowledge/concepts/architectural-rules.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/architecture.md",
        asset!("knowledge/concepts/architecture.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/backlog.md",
        asset!("knowledge/concepts/backlog.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/coding-standards.md",
        asset!("knowledge/concepts/coding-standards.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/configuration.md",
        asset!("knowledge/concepts/configuration.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/constraints-non-goals.md",
        asset!("knowledge/concepts/constraints-non-goals.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/definition-of-done.md",
        asset!("knowledge/concepts/definition-of-done.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/dependency-policy.md",
        asset!("knowledge/concepts/dependency-policy.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/development-commands.md",
        asset!("knowledge/concepts/development-commands.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/development-procedure.md",
        asset!("knowledge/concepts/development-procedure.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/directory-structure.md",
        asset!("knowledge/concepts/directory-structure.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/error-handling.md",
        asset!("knowledge/concepts/error-handling.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/glossary.md",
        asset!("knowledge/concepts/glossary.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/project-overview.md",
        asset!("knowledge/concepts/project-overview.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/release-procedure.md",
        asset!("knowledge/concepts/release-procedure.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/security-requirements.md",
        asset!("knowledge/concepts/security-requirements.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/software-components.md",
        asset!("knowledge/concepts/software-components.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/technology-stack.md",
        asset!("knowledge/concepts/technology-stack.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
    (
        "knowledge/testing-strategy.md",
        asset!("knowledge/concepts/testing-strategy.md"),
        Ownership::Scaffold,
        "starter concept",
    ),
];

/// The document templates the workflow skills fill in, owned so they
/// version with the skills that reference them: (file name under
/// `knowledge/templates/`, content).
pub(crate) const TEMPLATE_FILES: &[(&str, &str)] = &[
    ("adhoc-plan.md", asset!("knowledge/templates/adhoc-plan.md")),
    ("adr.md", asset!("knowledge/templates/adr.md")),
    (
        "api-contracts.md",
        asset!("knowledge/templates/api-contracts.md"),
    ),
    (
        "architectural-rules.md",
        asset!("knowledge/templates/architectural-rules.md"),
    ),
    (
        "architecture.md",
        asset!("knowledge/templates/architecture.md"),
    ),
    ("backlog.md", asset!("knowledge/templates/backlog.md")),
    ("bug-report.md", asset!("knowledge/templates/bug-report.md")),
    ("changelog.md", asset!("knowledge/templates/changelog.md")),
    (
        "code-review.md",
        asset!("knowledge/templates/code-review.md"),
    ),
    (
        "coding-standards.md",
        asset!("knowledge/templates/coding-standards.md"),
    ),
    (
        "commit-message.md",
        asset!("knowledge/templates/commit-message.md"),
    ),
    (
        "configuration.md",
        asset!("knowledge/templates/configuration.md"),
    ),
    (
        "constraints-non-goals.md",
        asset!("knowledge/templates/constraints-non-goals.md"),
    ),
    (
        "definition-of-done.md",
        asset!("knowledge/templates/definition-of-done.md"),
    ),
    (
        "dependency-policy.md",
        asset!("knowledge/templates/dependency-policy.md"),
    ),
    (
        "development-commands.md",
        asset!("knowledge/templates/development-commands.md"),
    ),
    (
        "development-procedure.md",
        asset!("knowledge/templates/development-procedure.md"),
    ),
    (
        "directory-structure.md",
        asset!("knowledge/templates/directory-structure.md"),
    ),
    (
        "error-handling.md",
        asset!("knowledge/templates/error-handling.md"),
    ),
    (
        "feature-plan.md",
        asset!("knowledge/templates/feature-plan.md"),
    ),
    ("glossary.md", asset!("knowledge/templates/glossary.md")),
    ("index.md", asset!("knowledge/templates/index.md")),
    (
        "interface-contract.md",
        asset!("knowledge/templates/interface-contract.md"),
    ),
    (
        "investigation.md",
        asset!("knowledge/templates/investigation.md"),
    ),
    (
        "issue-tracker.md",
        asset!("knowledge/templates/issue-tracker.md"),
    ),
    (
        "migration-guide.md",
        asset!("knowledge/templates/migration-guide.md"),
    ),
    ("postmortem.md", asset!("knowledge/templates/postmortem.md")),
    (
        "pr-description.md",
        asset!("knowledge/templates/pr-description.md"),
    ),
    (
        "project-overview.md",
        asset!("knowledge/templates/project-overview.md"),
    ),
    ("readme.md", asset!("knowledge/templates/readme.md")),
    (
        "release-notes.md",
        asset!("knowledge/templates/release-notes.md"),
    ),
    (
        "release-procedure.md",
        asset!("knowledge/templates/release-procedure.md"),
    ),
    (
        "security-requirements.md",
        asset!("knowledge/templates/security-requirements.md"),
    ),
    (
        "security-review.md",
        asset!("knowledge/templates/security-review.md"),
    ),
    (
        "software-components.md",
        asset!("knowledge/templates/software-components.md"),
    ),
    ("spec.md", asset!("knowledge/templates/spec.md")),
    (
        "status-update.md",
        asset!("knowledge/templates/status-update.md"),
    ),
    (
        "technology-stack.md",
        asset!("knowledge/templates/technology-stack.md"),
    ),
    ("test-plan.md", asset!("knowledge/templates/test-plan.md")),
    (
        "testing-strategy.md",
        asset!("knowledge/templates/testing-strategy.md"),
    ),
    (
        "visual-system.md",
        asset!("knowledge/templates/visual-system.md"),
    ),
];

/// The aokf-carried skill set lives in the generated
/// [`super::aokf_skills::SKILLS`] table: the workflow phases and their
/// support skills, carried by this component so they exist exactly where a
/// bundle exists.
pub(crate) fn skill_names() -> impl Iterator<Item = &'static str> {
    super::aokf_skills::SKILLS.iter().map(|(name, _)| *name)
}

/// Each skill's identity file, for init-time adoption: (name, SKILL.md).
fn skill_identities() -> Vec<(&'static str, &'static str)> {
    super::aokf_skills::SKILLS
        .iter()
        .map(|(name, files)| {
            let skill_md = files
                .iter()
                .find(|(rel, _)| *rel == "SKILL.md")
                .expect("every skill directory carries a SKILL.md")
                .1;
            (*name, skill_md)
        })
        .collect()
}

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
    super::skills::adopt_existing(
        root,
        Capability::Knowledge,
        "aokf",
        &skill_identities(),
        manifest,
    )
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
    for (name, content) in TEMPLATE_FILES {
        items.push(ManagedItem::OwnedFile {
            path: format!("knowledge/templates/{name}"),
            content: (*content).to_string(),
            reason: "document template".to_string(),
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
        .config(Capability::Knowledge, "aokf")
        .map(|c| c.custom.as_slice())
        .unwrap_or_default();
    items.extend(super::skills::skill_dir_items(
        super::aokf_skills::SKILLS,
        custom,
    ));
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

    #[test]
    fn ships_the_carried_skill_set_and_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        let actions = plan_in(dir.path());
        let descs: Vec<String> = actions.iter().map(Action::describe).collect();
        // A skill is its directory: every file of every skill materialises.
        for (name, files) in super::super::aokf_skills::SKILLS {
            for (rel, _) in *files {
                assert!(
                    descs
                        .iter()
                        .any(|d| d.contains(&format!(".claude/skills/{name}/{rel}"))),
                    ".claude/skills/{name}/{rel} missing from {descs:?}"
                );
            }
        }
        assert!(
            descs
                .iter()
                .any(|d| d.contains("superdev aokf hook validate")),
            "{descs:?}"
        );
        // The template library ships with the skills that reference it.
        for (name, _) in TEMPLATE_FILES {
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!("knowledge/templates/{name}"))),
                "knowledge/templates/{name} missing from {descs:?}"
            );
        }
    }

    #[test]
    fn a_custom_skill_is_released_whole_and_the_hook_stays() {
        use crate::component::Claim;
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.capabilities.get_mut("knowledge").unwrap()[0].custom =
            vec!["maintain".into(), "prototype".into()];
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
        };
        let keys: Vec<String> = Aokf.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(!keys.iter().any(|k| k.contains("/maintain/")), "{keys:?}");
        // Releasing a skill releases its whole directory, companions included.
        assert!(!keys.iter().any(|k| k.contains("/prototype/")), "{keys:?}");
        assert!(keys.contains(&".claude/skills/bootstrap/SKILL.md".to_string()));
        assert!(keys.contains(&".claude/skills/how-do-i/SESSION-BOUNDARIES.md".to_string()));
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
        let path = dir.path().join(".claude/skills/maintain/SKILL.md");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "# Ours, thanks\n").unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        let lines = adopt_existing(dir.path(), &mut manifest);
        assert_eq!(manifest.capabilities["knowledge"][0].custom, ["maintain"]);
        assert_eq!(
            lines,
            vec![format!(
                "knowledge: kept your maintain — marked custom in {}",
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
    fn agents_md_is_never_planned_and_the_instructions_are() {
        // AGENTS.md is the user's file: the repo-level entry ensures its one
        // import line; this component must not touch it.
        let dir = tempfile::tempdir().unwrap();
        let actions = plan_in(dir.path());
        assert!(
            !actions.iter().any(|a| matches!(
                a,
                Action::WriteFile { path, .. } if path == "AGENTS.md"
            )),
            "AGENTS.md planned by the aokf component"
        );
        let instructions = actions
            .into_iter()
            .find_map(|a| match a {
                Action::WriteFile { path, content, .. } if path == ".agents/aokf.md" => {
                    Some(content)
                }
                _ => None,
            })
            .unwrap();
        // Sibling-relative imports: they resolve from `.agents/`.
        assert!(instructions.contains("@aokf/SPEC.md"), "{instructions}");
        assert!(
            instructions.contains("@../knowledge/index.md"),
            "{instructions}"
        );
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
        assert!(paths.contains(&".agents/aokf.md"));
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
        // Every other asset ships byte-for-byte as embedded.
        for action in &actions {
            let Action::WriteFile { path, content, .. } = action else {
                continue;
            };
            if path == NAMED_ASSET {
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
