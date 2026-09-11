use super::*;
use crate::engine::tx;

#[test]
fn plan_runs_every_component() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let lock = Lock::default();
    let fake = FakeRunner::new();
    let ctx = crate::component::Ctx {
        root: dir.path(),
        runner: &fake,
        manifest: &manifest,
        lock: &lock,
        content: crate::content::test_snapshot(),
    };
    let components = crate::components::enabled(&manifest).unwrap();
    let planned = plan(&components, &ctx).unwrap();
    assert_eq!(planned.len(), components.len());
    assert_eq!(planned[0].provider, "frontend-design");
    // Every capability provider names its slot; SOKF, planned last,
    // names none — that is what makes it core.
    let (core, slotted): (Vec<_>, Vec<_>) = planned.iter().partition(|p| p.capability.is_none());
    assert_eq!(core.len(), 1);
    assert_eq!(core[0].provider, crate::components::sokf::NAME);
    assert!(slotted.iter().all(|p| p.capability.is_some()));
    assert!(planned.iter().any(|p| !p.actions.is_empty()));

    // A component that fails to plan aborts the whole plan.
    let mut broken = Manifest::default_for("0.1.0", &[]);
    broken.capabilities.get_mut("code-index").unwrap()[0].version = Some("9.9.9".into());
    let ctx = crate::component::Ctx {
        root: dir.path(),
        runner: &fake,
        manifest: &broken,
        lock: &lock,
        content: crate::content::test_snapshot(),
    };
    assert!(plan(&crate::components::enabled(&broken).unwrap(), &ctx).is_err());
}

#[test]
fn satisfied_scaffold_and_line_are_skipped() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "mine").unwrap();
    std::fs::write(dir.path().join(".gitignore"), "target\n.superdev/cache/\n").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![
            Action::WriteFile {
                path: "AGENTS.md".into(),
                content: "blueprint".into(),
                ownership: Ownership::Scaffold,
                reason: "entry point".into(),
            },
            Action::EnsureLine {
                path: ".gitignore".into(),
                line: ".superdev/cache/".into(),
                reason: "ignore machine state".into(),
                append_note: None,
            },
        ],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    assert_eq!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Skipped("exists".into())
    );
    assert_eq!(
        result.reports[0].outcomes[1].1,
        ActionOutcome::Skipped("present".into())
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        "mine"
    );
    // A repo-level entry owns no capability, so the lock records none.
    assert!(lock.components.is_empty());
}

#[test]
fn ensure_line_appends_and_creates() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "first").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let line = |path: &str| Action::EnsureLine {
        path: path.into(),
        line: "added".into(),
        reason: "test".into(),
        append_note: None,
    };
    let planned = vec![Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![line("a.txt"), line("b.txt")],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
        "first\nadded\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("b.txt")).unwrap(),
        "added\n"
    );
}

#[test]
fn ensure_line_notes_appends_to_existing_files_only() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "first\n").unwrap();
    std::fs::write(dir.path().join("c.txt"), "added\n").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let line = |path: &str| Action::EnsureLine {
        path: path.into(),
        line: "added".into(),
        reason: "test".into(),
        append_note: Some("the rest is yours".into()),
    };
    let planned = vec![Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![line("a.txt"), line("b.txt"), line("c.txt")],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    // Appended to an existing file: the note fires.
    assert_eq!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Applied {
            note: Some("the rest is yours".into())
        }
    );
    // Created fresh: nothing of the user's to talk about.
    assert_eq!(
        result.reports[0].outcomes[1].1,
        ActionOutcome::Applied { note: None }
    );
    // Already present: skipped, no note.
    assert_eq!(
        result.reports[0].outcomes[2].1,
        ActionOutcome::Skipped("present".into())
    );
}

#[test]
fn owned_overwrite_backs_up_and_notes_user_edits() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("owned.txt"), "edited by hand").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "knowledge".into(),
        actions: vec![write_owned("owned.txt")],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    assert_eq!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Applied {
            note: Some("overwrote a user-edited file (backed up)".into())
        }
    );
    let backups = std::fs::read_dir(dir.path().join(tx::BACKUP_DIR))
        .unwrap()
        .map(|e| e.unwrap().path().join("owned.txt"))
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        std::fs::read_to_string(&backups[0]).unwrap(),
        "edited by hand"
    );

    // Re-applying over superdev's own content is not a user edit.
    std::fs::write(dir.path().join("owned.txt"), "stale").unwrap();
    lock.files
        .insert("owned.txt".into(), crate::lock::sha256_hex(b"stale"));
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert_eq!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Applied { note: None }
    );
}

#[test]
fn unwind_restores_content_runs_undo_and_reports_leftovers() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("owned.txt"), "before").unwrap();
    let fake = FakeRunner::new();
    fake.script(
        "codegraph init",
        Output {
            status: 1,
            stdout: String::new(),
            stderr: "no node".into(),
        },
    );
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![
        Planned {
            capability: Some(crate::capability::Capability::Frontend),
            provider: "frontend-design".into(),
            actions: vec![
                Action::SetMisePin {
                    tool: "http:example".into(),
                    value_toml: "\"1.0.0\"".into(),
                },
                write_owned("owned.txt"),
                Action::Run {
                    program: "claude".into(),
                    args: vec!["plugin".into(), "marketplace".into(), "add".into()],
                    purpose: "register".into(),
                    undo: None,
                    optional: true,
                },
                Action::Run {
                    program: "claude".into(),
                    args: vec!["plugin".into(), "install".into(), "frontend-design".into()],
                    purpose: "install".into(),
                    undo: Some((
                        "claude".into(),
                        vec![
                            "plugin".into(),
                            "uninstall".into(),
                            "frontend-design".into(),
                        ],
                    )),
                    optional: true,
                },
            ],
        },
        Planned {
            capability: Some(crate::capability::Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::Run {
                program: "codegraph".into(),
                args: vec!["init".into()],
                purpose: "index".into(),
                undo: None,
                optional: false,
            }],
        },
    ];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("owned.txt")).unwrap(),
        "before"
    );
    assert!(!dir.path().join(".mise.toml").exists(), "pin file removed");
    assert!(
        fake.calls()
            .contains(&"claude plugin uninstall frontend-design".to_string())
    );
    assert!(result.reverted.iter().any(|r| r.contains("owned.txt")));
    assert!(result.reverted.iter().any(|r| r.contains("uninstall")));
    // The install and the marketplace registration cannot be undone.
    assert!(
        result
            .not_reverted
            .iter()
            .any(|r| r.contains("mise install"))
    );
    assert!(
        result
            .not_reverted
            .iter()
            .any(|r| r.contains("marketplace"))
    );
    // The first entry completed, so its hashes are staged in the lock —
    // which the caller discards, because the run is not ok.
    assert!(lock.files.contains_key("owned.txt"));
}

#[test]
fn unwritable_target_fails_the_action() {
    let dir = tempfile::tempdir().unwrap();
    // A file where a parent directory must go: the write cannot succeed.
    std::fs::write(dir.path().join("blocked"), "").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![write_owned("blocked/child.txt")],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    // Repo-level entries label as `repo (provider)`, matching plan output.
    assert_eq!(result.reports[0].label, "repo (superdev)");
    assert!(matches!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Failed(_)
    ));
}

#[test]
fn non_utf8_target_is_not_clobbered() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("bin.dat"), [0xff, 0xfe]).unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![write_owned("bin.dat")],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    assert_eq!(
        std::fs::read(dir.path().join("bin.dat")).unwrap(),
        [0xff, 0xfe]
    );
}

#[test]
fn remove_file_backs_up_journals_and_releases_the_lock_key() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("old.txt"), "superdev content").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    lock.files
        .insert("old.txt".into(), sha256_hex(b"superdev content"));
    let planned = vec![Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![Action::Remove {
            claim: Claim::File("old.txt".into()),
            reason: "no longer in the blueprint".into(),
        }],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    assert!(!dir.path().join("old.txt").exists());
    assert!(!lock.files.contains_key("old.txt"));
    let backups: Vec<_> = std::fs::read_dir(dir.path().join(tx::BACKUP_DIR))
        .unwrap()
        .map(|e| e.unwrap().path().join("old.txt"))
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        std::fs::read_to_string(&backups[0]).unwrap(),
        "superdev content"
    );
}

#[test]
fn remove_file_skips_the_gone_and_the_user_changed() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let remove = |path: &str| Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![Action::Remove {
            claim: Claim::File(path.into()),
            reason: "no longer in the blueprint".into(),
        }],
    };
    // Already gone: skipped, key released.
    let mut lock = Lock::default();
    lock.files.insert("gone.txt".into(), sha256_hex(b"x"));
    let result = apply(
        dir.path(),
        &fake,
        &manifest,
        &[remove("gone.txt")],
        &mut lock,
    );
    assert!(result.ok);
    assert_eq!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Skipped("already gone".into())
    );
    assert!(!lock.files.contains_key("gone.txt"));
    // Changed since planning: left in place, key released.
    std::fs::write(dir.path().join("mine.txt"), "edited by hand").unwrap();
    let mut lock = Lock::default();
    lock.files
        .insert("mine.txt".into(), sha256_hex(b"superdev content"));
    let result = apply(
        dir.path(),
        &fake,
        &manifest,
        &[remove("mine.txt")],
        &mut lock,
    );
    assert!(result.ok);
    assert_eq!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Skipped("changed since superdev wrote it — left in place".into())
    );
    assert!(dir.path().join("mine.txt").exists());
    assert!(!lock.files.contains_key("mine.txt"));
}

#[test]
fn remove_mise_pin_and_json_key_rewrite_only_their_entry() {
    let dir = tempfile::tempdir().unwrap();
    let mise_toml =
        mise::set_pin("[tools]\nnode = \"24\"\n", "http:codegraph", "\"1.5.0\"").unwrap();
    std::fs::write(dir.path().join(".mise.toml"), &mise_toml).unwrap();
    std::fs::write(
        dir.path().join(".mcp.json"),
        r#"{"mcpServers":{"superdev-sokf":{"command":"superdev","args":["mcp","sokf"]},"mine":{"command":"me"}}}"#,
    )
    .unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let pin_value = mise::current_pin(&mise_toml, "http:codegraph")
        .unwrap()
        .unwrap();
    lock.files.insert(
        mise::pin_lock_key("http:codegraph"),
        sha256_hex(pin_value.as_bytes()),
    );
    let mcp_value: serde_json::Value =
        serde_json::from_str(r#"{"command":"superdev","args":["mcp","sokf"]}"#).unwrap();
    lock.files.insert(
        ".mcp.json:mcpServers.superdev-sokf".into(),
        sha256_hex(mcp_value.to_string().as_bytes()),
    );
    let planned = vec![Planned {
        capability: None,
        provider: "superdev".into(),
        actions: vec![
            Action::Remove {
                claim: Claim::MisePin("http:codegraph".into()),
                reason: "no longer in the blueprint".into(),
            },
            Action::Remove {
                claim: Claim::JsonKey {
                    path: ".mcp.json".into(),
                    pointer: "mcpServers.superdev-sokf".into(),
                },
                reason: "no longer in the blueprint".into(),
            },
        ],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok, "{:?}", result.reports);
    let mise_after = std::fs::read_to_string(dir.path().join(".mise.toml")).unwrap();
    assert_eq!(
        mise::current_pin(&mise_after, "http:codegraph").unwrap(),
        None
    );
    assert!(mise_after.contains("node = \"24\""));
    let mcp: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
            .unwrap();
    assert!(mcp["mcpServers"].get("superdev-sokf").is_none());
    assert_eq!(mcp["mcpServers"]["mine"]["command"], "me");
    assert!(lock.files.is_empty());
    // No installs follow a removal.
    assert!(fake.calls().is_empty());
}

#[test]
fn a_later_failure_restores_a_removed_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("old.txt"), "superdev content").unwrap();
    let fake = FakeRunner::new();
    fake.script(
        "codegraph init",
        Output {
            status: 1,
            stdout: String::new(),
            stderr: "boom".into(),
        },
    );
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    lock.files
        .insert("old.txt".into(), sha256_hex(b"superdev content"));
    let planned = vec![
        Planned {
            capability: None,
            provider: "superdev".into(),
            actions: vec![Action::Remove {
                claim: Claim::File("old.txt".into()),
                reason: "no longer in the blueprint".into(),
            }],
        },
        Planned {
            capability: Some(crate::capability::Capability::CodeIndex),
            provider: "codegraph".into(),
            actions: vec![Action::Run {
                program: "codegraph".into(),
                args: vec!["init".into()],
                purpose: "index".into(),
                undo: None,
                optional: false,
            }],
        },
    ];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("old.txt")).unwrap(),
        "superdev content"
    );
    assert!(result.reverted.iter().any(|r| r.contains("old.txt")));
}
