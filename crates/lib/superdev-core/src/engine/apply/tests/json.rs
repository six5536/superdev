use super::*;

fn set_mcp_key() -> Action {
    Action::SetJsonKey {
        path: ".mcp.json".into(),
        pointer: "mcpServers.superdev-sokf".into(),
        value_json: r#"{"command":"superdev","args":["mcp","sokf"]}"#.into(),
    }
}

#[test]
fn set_json_key_merges_and_preserves_other_servers() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".mcp.json"),
        "{\n  \"mcpServers\": { \"other\": { \"command\": \"othersrv\" } },\n  \"extra\": true\n}\n",
    )
    .unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "knowledge".into(),
        actions: vec![set_mcp_key()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    let text = std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap();
    assert!(text.ends_with("}\n"), "{text}");
    let written: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(written["mcpServers"]["other"]["command"], "othersrv");
    assert_eq!(written["extra"], true);
    assert_eq!(
        written["mcpServers"]["superdev-sokf"]["command"],
        "superdev"
    );
    assert_eq!(written["mcpServers"]["superdev-sokf"]["args"][1], "sokf");
    assert!(
        lock.files
            .contains_key(".mcp.json:mcpServers.superdev-sokf"),
        "lock: {:?}",
        lock.files
    );
}

#[test]
fn set_json_key_creates_the_file_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "knowledge".into(),
        actions: vec![set_mcp_key()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
            .unwrap();
    assert_eq!(
        written["mcpServers"]["superdev-sokf"],
        serde_json::json!({ "command": "superdev", "args": ["mcp", "sokf"] })
    );
}

#[test]
fn malformed_mcp_json_fails_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".mcp.json"), "not json\n").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "knowledge".into(),
        actions: vec![write_owned("created.txt"), set_mcp_key()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    let (description, outcome) = &result.reports[0].outcomes[1];
    assert_eq!(description, "set mcpServers.superdev-sokf in .mcp.json");
    let ActionOutcome::Failed(message) = outcome else {
        panic!("expected a failure, got {outcome:?}");
    };
    assert!(message.starts_with(".mcp.json:"), "{message}");
    // The user's file is left exactly as they wrote it, and the earlier
    // write in the same run is taken back.
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
        "not json\n"
    );
    assert!(!dir.path().join("created.txt").exists());
    assert!(result.reverted.iter().any(|r| r.contains("created.txt")));
    assert!(lock.files.is_empty());
}

#[test]
fn a_json_key_path_through_a_non_object_fails() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".mcp.json"), "{ \"mcpServers\": 3 }").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "knowledge".into(),
        actions: vec![set_mcp_key()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    let (_, outcome) = &result.reports[0].outcomes[0];
    let ActionOutcome::Failed(message) = outcome else {
        panic!("expected a failure, got {outcome:?}");
    };
    assert_eq!(message, ".mcp.json: `mcpServers` is not a JSON object");
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap(),
        "{ \"mcpServers\": 3 }"
    );

    // A file whose root is not an object at all.
    std::fs::write(dir.path().join(".mcp.json"), "[]").unwrap();
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    let (_, ActionOutcome::Failed(message)) = &result.reports[0].outcomes[0] else {
        panic!("expected a failure");
    };
    assert_eq!(message, ".mcp.json: the root is not a JSON object");
}

/// The hook registration the skills provider plans, used by the array tests.
fn ensure_hook() -> Action {
    Action::EnsureJsonArrayElement {
        path: ".claude/settings.json".into(),
        pointer: "hooks.PostToolUse".into(),
        marker: "superdev hook validate".into(),
        value_json: r#"{"matcher":"Edit|Write","hooks":[{"type":"command","command":"superdev hook validate"}]}"#.into(),
    }
}

#[test]
fn ensure_array_element_appends_and_preserves_user_entries() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(
        dir.path().join(".claude/settings.json"),
        "{\n  \"hooks\": { \"PostToolUse\": [ { \"matcher\": \"Agent\", \"hooks\": [] } ] },\n  \"permissions\": { \"deny\": [] }\n}\n",
    )
    .unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::Skills),
        provider: "superdev-skills".into(),
        actions: vec![ensure_hook()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    let text = std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap();
    let written: serde_json::Value = serde_json::from_str(&text).unwrap();
    let entries = written["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["matcher"], "Agent");
    assert_eq!(entries[1]["hooks"][0]["command"], "superdev hook validate");
    assert!(written["permissions"].is_object());
    assert!(
        lock.files
            .contains_key(".claude/settings.json:hooks.PostToolUse[superdev hook validate]"),
        "lock: {:?}",
        lock.files
    );
}

#[test]
fn ensure_array_element_replaces_a_stale_superdev_entry() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    // A prior release's entry: same marker, different matcher.
    std::fs::write(
        dir.path().join(".claude/settings.json"),
        r#"{"hooks":{"PostToolUse":[{"matcher":"Write","hooks":[{"type":"command","command":"superdev hook validate"}]}]}}"#,
    )
    .unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::Skills),
        provider: "superdev-skills".into(),
        actions: vec![ensure_hook()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    let written: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let entries = written["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "replaced, not duplicated");
    assert_eq!(entries[0]["matcher"], "Edit|Write");
}

#[test]
fn ensure_array_element_creates_the_file_and_path_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::Skills),
        provider: "superdev-skills".into(),
        actions: vec![ensure_hook()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    let written: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        written["hooks"]["PostToolUse"][0]["hooks"][0]["command"],
        "superdev hook validate"
    );
}

#[test]
fn ensure_array_element_rejects_a_non_array_pointer() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(
        dir.path().join(".claude/settings.json"),
        r#"{"hooks":{"PostToolUse":{"matcher":"Edit"}}}"#,
    )
    .unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::Skills),
        provider: "superdev-skills".into(),
        actions: vec![ensure_hook()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    let (_, ActionOutcome::Failed(message)) = &result.reports[0].outcomes[0] else {
        panic!("expected a failure");
    };
    assert_eq!(
        message,
        ".claude/settings.json: `PostToolUse` is not a JSON array"
    );
    // The user's file is left exactly as they wrote it.
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
        r#"{"hooks":{"PostToolUse":{"matcher":"Edit"}}}"#
    );
    assert!(lock.files.is_empty());
}

#[test]
fn ensure_array_element_on_a_malformed_file_fails_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(dir.path().join(".claude/settings.json"), "not json\n").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::Skills),
        provider: "superdev-skills".into(),
        actions: vec![ensure_hook()],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
        "not json\n"
    );
}
