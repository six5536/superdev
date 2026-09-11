use super::*;
use crate::runner::FakeRunner;

const START: &str = "<!-- superdev:instructions -->";
const END: &str = "<!-- /superdev:instructions -->";

fn block(content: &str) -> String {
    format!("{START}\n{content}{END}\n")
}

fn sync(root: &Path, lock: &mut Lock) {
    let manifest = Manifest::default_for(crate::version(), &Capability::ALL);
    if let Some(entry) = repo_entry(root, &manifest, content::test_snapshot()).unwrap() {
        let result = engine::apply(root, &FakeRunner::new(), &manifest, &[entry], lock);
        assert!(result.ok, "{result:?}");
    }
}

// contract-002 P_init-agents-chain and P_sync-agent-instructions.
#[test]
fn agent_instructions_precede_user_content_and_sync_is_idempotent() {
    for user in [
        None,
        Some(""),
        Some("# Mine\n"),
        Some("\r\n# Mine\r\n\r\n尾部"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        if let Some(user) = user {
            std::fs::write(&path, user).unwrap();
        }
        let mut lock = Lock::default();
        sync(dir.path(), &mut lock);
        let expected = format!("{}{}", block(AGGREGATOR_TEMPLATE), user.unwrap_or_default());
        assert_eq!(std::fs::read(&path).unwrap(), expected.as_bytes());
        assert!(
            !lock.files.contains_key("AGENTS.md"),
            "user file is not owned"
        );
        assert_eq!(
            std::fs::read(dir.path().join(AGGREGATOR_PATH)).unwrap(),
            AGGREGATOR_TEMPLATE.as_bytes()
        );
        sync(dir.path(), &mut lock);
        assert_eq!(std::fs::read(&path).unwrap(), expected.as_bytes());
        let manifest = Manifest::default_for(crate::version(), &Capability::ALL);
        assert!(
            repo_entry(dir.path(), &manifest, content::test_snapshot())
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn agent_import_migration_removes_only_exact_standalone_lines() {
    for (before, user) in [
        ("@.agents/superdev.md\n", ""),
        ("# Mine\r\n@.agents/superdev.md\r\nTail", "# Mine\r\nTail"),
        ("# Mine\n@.agents/superdev.md", "# Mine\n"),
        ("@.agents/superdev.md\n\n@.agents/superdev.md\n", "\n"),
        (
            "See `@.agents/superdev.md`.\n@other.md\n",
            "See `@.agents/superdev.md`.\n@other.md\n",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        std::fs::write(&path, before).unwrap();
        sync(dir.path(), &mut Lock::default());
        assert_eq!(
            std::fs::read(&path).unwrap(),
            format!("{}{user}", block(AGGREGATOR_TEMPLATE)).as_bytes()
        );
    }
}

#[test]
fn agent_block_is_updated_and_moved_to_the_start_without_changing_user_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("AGENTS.md");
    std::fs::write(&path, format!("# Before\r\n{}\r\n# After", block("old\n"))).unwrap();
    sync(dir.path(), &mut Lock::default());
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        format!("{}# Before\r\n\r\n# After", block(AGGREGATOR_TEMPLATE))
    );
}

#[test]
fn malformed_agent_markers_refuse_without_writes() {
    for before in [
        format!("{START}\nUser text"),
        format!("{END}\nUser text"),
        format!("{END}\n{START}\n"),
        format!("{START}\n{START}\n{END}\n"),
        format!("{}{}", block("old\n"), block("old\n")),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("AGENTS.md");
        std::fs::write(&path, &before).unwrap();
        let manifest = Manifest::default_for(crate::version(), &Capability::ALL);
        let error = repo_entry(dir.path(), &manifest, content::test_snapshot())
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("AGENTS.md") && error.contains("markers"),
            "{error}"
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
        assert!(!dir.path().join(AGGREGATOR_PATH).exists());
    }
}

#[test]
fn agent_apply_preserves_edits_made_after_planning_and_rolls_back_on_failure() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("AGENTS.md");
    std::fs::write(&path, "original").unwrap();
    let manifest = Manifest::default_for(crate::version(), &Capability::ALL);
    let mut entry = repo_entry(dir.path(), &manifest, content::test_snapshot())
        .unwrap()
        .unwrap();
    std::fs::write(&path, "new user content").unwrap();
    let mut lock = Lock::default();
    let result = engine::apply(
        dir.path(),
        &FakeRunner::new(),
        &manifest,
        &[entry.clone()],
        &mut lock,
    );
    assert!(result.ok, "{result:?}");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        format!("{}new user content", block(AGGREGATOR_TEMPLATE))
    );

    // A later action fails after AGENTS.md is changed: its exact prior bytes return.
    std::fs::write(&path, "before failed sync").unwrap();
    std::fs::create_dir(dir.path().join("not-a-file")).unwrap();
    entry.actions.push(Action::EnsureLine {
        path: "not-a-file".into(),
        line: "fail".into(),
        reason: "test".into(),
        append_note: None,
    });
    let result = engine::apply(
        dir.path(),
        &FakeRunner::new(),
        &manifest,
        &[entry],
        &mut lock,
    );
    assert!(!result.ok);
    assert_eq!(std::fs::read_to_string(path).unwrap(), "before failed sync");
}
