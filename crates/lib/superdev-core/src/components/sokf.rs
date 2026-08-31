//! components/sokf.rs — the SOKF knowledge. Not a capability: SOKF is part
//! of superdev, so this component is always planned and no manifest entry
//! turns it off. The specification and the instructions ship inside the
//! binary; everything else it writes comes from the content pack.

use std::path::Path;

use crate::action::Action;
use crate::capability::Capability;
use crate::component::{Claim, Component, Ctx};
use crate::content::{ContentSet, ItemKind, Owner};
use crate::error::Result;
use crate::manifest::Manifest;

use super::item::{self, ManagedItem};

macro_rules! asset {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $rel))
    };
}

/// The one asset carrying a `{name}` token, replaced with the repo's name.
const NAMED_ASSET: &str = "knowledge/manifest.sokf.yaml";

/// The workflow-framework override each provider gets, as
/// (provider id, target path, embedded asset).
/// Where agent tools find MCP servers, and superdev's key inside it. The file
/// is shared with the user's own servers, so only this key is managed.
const MCP_PATH: &str = ".mcp.json";
const MCP_POINTER: &str = "mcpServers.superdev-sokf";
/// The registration itself: the installed binary serving this repo's bundle.
const MCP_VALUE: &str = r#"{"command":"superdev","args":["mcp","sokf"]}"#;

/// Claude Code reads CLAUDE.md, not AGENTS.md: without this import, every
/// rule superdev writes into AGENTS.md is invisible to it. Behaves like the
/// .gitignore lines — added when missing, never rewritten, never locked.
const CLAUDE_ENTRY_PATH: &str = "CLAUDE.md";
const CLAUDE_ENTRY_LINE: &str = "@AGENTS.md";

/// The files the binary owns rather than the pack: (target path, content,
/// reason). Each describes a format this binary's compiled validator
/// enforces, so all move with the binary and no pack may supply them. The
/// grammar ships from the validator's own embedded copy, so the file a
/// repository carries is byte-identical to the one the binary checks with.
const BINARY_OWNED: &[(&str, &str, &str)] = &[
    (
        ".agents/sokf/SPEC.md",
        asset!("sokf/agents/sokf/SPEC.md"),
        "SOKF specification",
    ),
    (
        ".agents/sokf/changelog.md",
        asset!("sokf/agents/sokf/changelog.md"),
        "SOKF changelog",
    ),
    (
        ".agents/sokf/grammar.yaml",
        crate::validate::schema::EMBEDDED_GRAMMAR,
        "SOKF schema grammar",
    ),
];

/// How many files this binary owns rather than the pack.
#[cfg(test)]
pub(crate) fn binary_owned_count() -> usize {
    BINARY_OWNED.len()
}

/// The bundle scaffolds that are not starter concepts, in the order they are
/// written, each with the reason printed for it. Presentation only: a
/// scaffold a pack adds needs no entry here — it takes the generic reason and
/// sorts with the concepts, after the structural files that frame them.
const SKELETON_REASONS: &[(&str, &str)] = &[
    ("index.md", "knowledge index"),
    ("manifest.sokf.yaml", "SOKF manifest"),
    ("adrs", "ADRs index"),
    ("contracts", "contracts indexes"),
    ("ideas", "ideas index"),
    ("issues", "issues index"),
    ("plans", "plans index"),
    ("issue-tracker.md", "issue-tracker convention"),
];

/// SOKF owns the workflow phases and their support skills: whatever the
/// resolved content carries under this owner, so they exist wherever the
/// SOKF knowledge does.
pub(crate) const OWNER: Owner = Owner::Knowledge;

/// Where Claude Code reads hook registrations. Shared with the user's own
/// hooks, so only superdev's array element is managed.
const SETTINGS_PATH: &str = ".claude/settings.json";
/// The array the hook entry lives in.
const HOOK_POINTER: &str = "hooks.PostToolUse";
/// What identifies superdev's element among the user's.
const HOOK_MARKER: &str = "superdev hook validate";
/// The registration itself: validate the repository after an Edit/Write.
const HOOK_ELEMENT: &str =
    r#"{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev hook validate"}]}"#;

/// The array the Stop entry lives in.
const STOP_POINTER: &str = "hooks.Stop";
/// What identifies superdev's element among the user's.
const STOP_MARKER: &str = "superdev hook run";
/// The registration itself: continue an active unattended run, or let the
/// turn end. Without a run state the hook is invisible (contract-009).
const STOP_ELEMENT: &str = r#"{"hooks":[{"type":"command","command":"superdev hook run"}]}"#;

/// Release, at adoption time, every SOKF skill the repo already has under
/// its own name and with its own content. Returns the lines to print.
pub(crate) fn adopt_existing(
    root: &Path,
    content: &ContentSet,
    manifest: &mut Manifest,
) -> Vec<String> {
    let identities = super::skills::skill_identities(content, OWNER);
    super::skills::adopt_existing(root, NAME, &mut manifest.knowledge.custom, &identities)
}

/// What this component is called in reports, lock groups and adoption lines.
/// Not a provider id: nothing competes for the slot, because there is none.
pub(crate) const NAME: &str = "knowledge";

/// The SOKF component.
pub struct Sokf;

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
    for (path, content, reason) in BINARY_OWNED {
        items.push(ManagedItem::OwnedFile {
            path: (*path).to_string(),
            content: (*content).to_string(),
            reason: (*reason).to_string(),
        });
    }
    let mut skeletons: Vec<_> = ctx
        .content
        .items_of(OWNER, ItemKind::KnowledgeSkeleton)
        .map(|item| {
            let rank = SKELETON_REASONS
                .iter()
                .position(|(name, _)| *name == item.name);
            // Listed first, in the table's order; everything else after, by
            // name — so the bundle's frame reads before the concepts in it.
            (rank.unwrap_or(SKELETON_REASONS.len()), item)
        })
        .collect();
    skeletons.sort_by_key(|(rank, item)| (*rank, item.name.clone()));
    for (rank, item) in skeletons {
        let reason = SKELETON_REASONS
            .get(rank)
            .map_or("starter concept", |(_, reason)| *reason);
        for (rel, content) in &item.files {
            let path = join(&format!("knowledge/{}", item.name), rel);
            // Only the bundle manifest carries a `{name}` token: a stray
            // `{name}` in prose is not a placeholder.
            let content = match path.as_str() {
                NAMED_ASSET => content.replace("{name}", &repo_name),
                _ => content.clone(),
            };
            items.push(ManagedItem::Scaffold {
                path,
                content,
                reason: reason.to_string(),
            });
        }
    }
    for item in ctx.content.items_of(OWNER, ItemKind::DocSchema) {
        items.push(ManagedItem::OwnedFile {
            path: format!("knowledge/schemas/{}.md", item.name),
            content: item.files[0].1.clone(),
            reason: "document schema".to_string(),
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
    let custom = ctx.manifest.knowledge.custom.as_slice();
    items.extend(super::skills::skill_dir_items(ctx.content, OWNER, custom));
    items.push(ManagedItem::JsonEntry {
        path: SETTINGS_PATH.into(),
        pointer: HOOK_POINTER.into(),
        marker: Some(HOOK_MARKER.into()),
        value_json: HOOK_ELEMENT.into(),
    });
    items.push(ManagedItem::JsonEntry {
        path: SETTINGS_PATH.into(),
        pointer: STOP_POINTER.into(),
        marker: Some(STOP_MARKER.into()),
        value_json: STOP_ELEMENT.into(),
    });
    items
}

/// An item's base path joined with one file's path under it. A single-file
/// item's path is empty, and the base is the file.
fn join(base: &str, rel: &str) -> String {
    if rel.is_empty() {
        base.to_string()
    } else {
        format!("{base}/{rel}")
    }
}

impl Component for Sokf {
    fn capability(&self) -> Option<Capability> {
        None
    }

    fn provider(&self) -> &'static str {
        NAME
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
            content: crate::content::test_snapshot(),
        };
        Sokf.plan(&ctx).unwrap()
    }

    #[test]
    fn ships_the_carried_skill_set_and_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        let actions = plan_in(dir.path());
        let descs: Vec<String> = actions.iter().map(Action::describe).collect();
        // A skill is its directory: every file of every skill materialises.
        let content = crate::content::snapshot();
        for item in content.items_of(OWNER, ItemKind::Skill) {
            for (rel, _) in &item.files {
                let name = &item.name;
                assert!(
                    descs
                        .iter()
                        .any(|d| d.contains(&format!(".claude/skills/{name}/{rel}"))),
                    ".claude/skills/{name}/{rel} missing from {descs:?}"
                );
            }
        }
        assert!(
            descs.iter().any(|d| d.contains("superdev hook validate")),
            "{descs:?}"
        );
        assert!(descs.iter().any(|d| d.contains("hooks.Stop")), "{descs:?}");
        // The schema library ships with the skills that reference it.
        for item in content.items_of(OWNER, ItemKind::DocSchema) {
            let name = &item.name;
            assert!(
                descs
                    .iter()
                    .any(|d| d.contains(&format!("knowledge/schemas/{name}.md"))),
                "knowledge/schemas/{name}.md missing from {descs:?}"
            );
        }
    }

    #[test]
    fn a_custom_skill_is_released_whole_and_the_hook_stays() {
        use crate::component::Claim;
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = Manifest::default_for("0.1.0", &[]);
        manifest.knowledge.custom = vec!["maintain".into(), "prototype".into()];
        let lock = Lock::default();
        let fake = FakeRunner::new();
        let ctx = Ctx {
            root: dir.path(),
            runner: &fake,
            manifest: &manifest,
            lock: &lock,
            content: crate::content::test_snapshot(),
        };
        let keys: Vec<String> = Sokf.owned(&ctx).iter().map(Claim::lock_key).collect();
        assert!(!keys.iter().any(|k| k.contains("/maintain/")), "{keys:?}");
        // Releasing a skill releases its whole directory, companions included.
        assert!(!keys.iter().any(|k| k.contains("/prototype/")), "{keys:?}");
        assert!(keys.contains(&".claude/skills/bootstrap/SKILL.md".to_string()));
        assert!(keys.contains(&".claude/skills/how-do-i/SESSION-BOUNDARIES.md".to_string()));
        assert!(keys.contains(
            &".claude/settings.json:hooks.PostToolUse[superdev hook validate]".to_string()
        ));
        assert!(keys.contains(&".claude/settings.json:hooks.Stop[superdev hook run]".to_string()));
    }

    #[test]
    fn a_stale_hook_entry_replans_the_hook() {
        let dir = tempfile::tempdir().unwrap();
        // Same marker, older shape: must be replaced, so it must be planned.
        std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
        std::fs::write(
            dir.path().join(SETTINGS_PATH),
            r#"{"hooks":{"PostToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"superdev hook validate"}]}]}}"#,
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
        let lines = adopt_existing(dir.path(), crate::content::test_snapshot(), &mut manifest);
        assert_eq!(manifest.knowledge.custom, ["maintain"]);
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
    /// validator over it. A skeleton that ships broken would fail
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
        let bundle = crate::sokf::bundle::load_bundle(&dir.path().join("knowledge")).unwrap();
        let report = crate::validate::sokf::validate(&bundle, dir.path());
        assert!(report.findings.is_empty(), "{:#?}", report.findings);
        assert!(report.passed());
        assert!(report.concept_count >= 20, "{}", report.concept_count);
    }

    #[test]
    fn agents_md_is_never_planned_and_the_grammar_matches_the_validator() {
        // AGENTS.md is the user's file: the repo-level entry ensures its one
        // import line; this component must not touch it.
        let dir = tempfile::tempdir().unwrap();
        let actions = plan_in(dir.path());
        assert!(
            !actions.iter().any(|a| matches!(
                a,
                Action::WriteFile { path, .. } if path == "AGENTS.md"
            )),
            "AGENTS.md planned by the SOKF component"
        );
        let grammar = actions
            .into_iter()
            .find_map(|a| match a {
                Action::WriteFile { path, content, .. }
                    if path == crate::validate::schema::GRAMMAR_PATH =>
                {
                    Some(content)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(grammar, crate::validate::schema::EMBEDDED_GRAMMAR);
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
        assert!(paths.contains(&".agents/sokf/SPEC.md"));
        assert!(paths.contains(&".agents/sokf/changelog.md"));
        assert!(paths.contains(&".agents/sokf/grammar.yaml"));
        let manifest_action = actions.iter().find_map(|a| match a {
            Action::WriteFile { path, content, .. } if path == "knowledge/manifest.sokf.yaml" => {
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
        let snapshot = crate::content::snapshot();
        for action in &actions {
            let Action::WriteFile { path, content, .. } = action else {
                continue;
            };
            if path == NAMED_ASSET {
                continue;
            }
            let Some(rest) = path.strip_prefix("knowledge/") else {
                continue;
            };
            let (name, rel) = rest.split_once('/').unwrap_or((rest, ""));
            let Some(item) = snapshot.item(OWNER, ItemKind::KnowledgeSkeleton, name) else {
                continue;
            };
            let Some((_, asset)) = item.files.iter().find(|(p, _)| p == rel) else {
                continue;
            };
            assert_eq!(asset, content, "{path} was rewritten");
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
                        format!("{{ \"mcpServers\": {{ \"superdev-sokf\": {value_json} }} }}");
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
                    path,
                    pointer,
                    marker,
                    value_json,
                } => {
                    // The production editor, so the harness cannot drift
                    // from apply semantics; two entries share the file.
                    let p = dir.path().join(&path);
                    let existing = std::fs::read_to_string(&p).unwrap_or_else(|_| "{}".into());
                    let (content, _) = crate::json_edit::edit_json_array_element(
                        &path,
                        &existing,
                        &pointer,
                        &marker,
                        &value_json,
                    )
                    .unwrap();
                    std::fs::write(p, content).unwrap();
                }
                other => panic!("unexpected action {other:?}"),
            }
        }
        assert!(plan_in(dir.path()).is_empty());
        // A user edit to a scaffold stays untouched…
        std::fs::write(dir.path().join("AGENTS.md"), "customised").unwrap();
        // …but an edit to an owned file is drift.
        std::fs::write(dir.path().join(".agents/sokf/SPEC.md"), "tampered").unwrap();
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
        assert_eq!(paths, vec![".agents/sokf/SPEC.md".to_string()]);
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
            "{\n  \"mcpServers\": {\n    \"superdev-sokf\": {\n      \"args\": [\"mcp\", \"sokf\"],\n      \"command\": \"superdev\"\n    }\n  }\n}\n",
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
            "{\"mcpServers\":{\"superdev-sokf\":{\"command\":\"old\"}}}",
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

    /// SOKF fills no slot: that is what makes it core rather than a
    /// provider something else could replace.
    #[test]
    fn fills_no_capability_slot() {
        assert_eq!(Sokf.capability(), None);
        assert_eq!(Sokf.provider(), NAME);
    }
}
