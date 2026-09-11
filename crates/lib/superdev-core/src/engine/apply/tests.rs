use super::*;
use crate::action::{Action, Ownership};
use crate::components::mise;
use crate::lock::Lock;
use crate::manifest::Manifest;
use crate::runner::{FakeRunner, Output};

fn write_owned(path: &str) -> Action {
    Action::WriteFile {
        path: path.into(),
        content: "content".into(),
        ownership: Ownership::Owned,
        reason: "test".into(),
    }
}

#[test]
fn applies_writes_and_updates_lock() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::Skills),
        provider: "superdev-skills".into(),
        actions: vec![write_owned("a/b.txt")],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    assert_eq!(
        std::fs::read_to_string(dir.path().join("a/b.txt")).unwrap(),
        "content"
    );
    assert!(lock.files.contains_key("a/b.txt"));
    assert_eq!(lock.components["skills"][0].provider, "superdev-skills");
}

/// A core component fills no slot, so there is no provider choice for the
/// lock to record — but its files are locked like any other's.
#[test]
fn a_core_component_locks_its_files_and_records_no_provider() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: None,
        provider: "knowledge".into(),
        actions: vec![write_owned("a/b.txt")],
    }];
    assert!(apply(dir.path(), &fake, &manifest, &planned, &mut lock).ok);
    assert!(lock.files.contains_key("a/b.txt"));
    assert!(lock.components.is_empty(), "{:?}", lock.components);
}

#[test]
fn each_pack_in_a_many_slot_gets_its_own_lock_record() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let entry = |provider: &str, path: &str| Planned {
        capability: Some(crate::capability::Capability::Skills),
        provider: provider.into(),
        actions: vec![write_owned(path)],
    };
    let planned = vec![
        entry("superdev-skills", "a.txt"),
        entry("another-pack", "b.txt"),
    ];
    assert!(apply(dir.path(), &fake, &manifest, &planned, &mut lock).ok);
    let providers: Vec<&str> = lock.components["skills"]
        .iter()
        .map(|r| r.provider.as_str())
        .collect();
    assert_eq!(providers, ["superdev-skills", "another-pack"]);
    // A re-apply updates the existing record instead of duplicating it.
    let planned = vec![entry("superdev-skills", "a.txt")];
    assert!(apply(dir.path(), &fake, &manifest, &planned, &mut lock).ok);
    assert_eq!(lock.components["skills"].len(), 2);
}

#[test]
fn a_cross_entry_duplicate_write_fails_and_unwinds() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![
        Planned {
            capability: None,
            provider: "knowledge".into(),
            actions: vec![write_owned("shared.txt")],
        },
        Planned {
            capability: Some(Capability::Skills),
            provider: "superdev-skills".into(),
            actions: vec![write_owned("shared.txt")],
        },
    ];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    let failed = result
        .reports
        .iter()
        .flat_map(|r| r.outcomes.iter())
        .find_map(|(_, o)| match o {
            ActionOutcome::Failed(e) => Some(e.clone()),
            _ => None,
        })
        .expect("the duplicate write fails");
    assert!(failed.contains("collision: shared.txt"), "{failed}");
    // The unwind took the first write back with it. The in-memory lock
    // keeps the first entry's keys — the caller discards it when not ok.
    assert!(!dir.path().join("shared.txt").exists());
}

#[test]
fn failure_unwinds_earlier_writes() {
    let dir = tempfile::tempdir().unwrap();
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
            capability: None,
            provider: "knowledge".into(),
            actions: vec![write_owned("created.txt")],
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
    assert!(
        !dir.path().join("created.txt").exists(),
        "created file must be deleted on unwind"
    );
    assert!(result.reverted.iter().any(|r| r.contains("created.txt")));
}

#[test]
fn optional_run_with_missing_program_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let fake = FakeRunner::new();
    fake.missing("claude");
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::Frontend),
        provider: "frontend-design".into(),
        actions: vec![Action::Run {
            program: "claude".into(),
            args: vec![
                "plugin".into(),
                "install".into(),
                "frontend-design@claude-code".into(),
            ],
            purpose: "install".into(),
            undo: None,
            optional: true,
        }],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(result.ok);
    assert!(matches!(
        result.reports[0].outcomes[0].1,
        ActionOutcome::Skipped(_)
    ));
}

#[test]
fn malformed_mise_tools_key_fails_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".mise.toml"), "tools = 3\n").unwrap();
    let fake = FakeRunner::new();
    let manifest = Manifest::default_for("0.1.0", &[]);
    let mut lock = Lock::default();
    let planned = vec![Planned {
        capability: Some(crate::capability::Capability::CodeIndex),
        provider: "codegraph".into(),
        actions: vec![Action::SetMisePin {
            tool: "npm:codegraph".into(),
            value_toml: "\"1.0.0\"".into(),
        }],
    }];
    let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
    assert!(!result.ok);
    let (_, outcome) = &result.reports[0].outcomes[0];
    let ActionOutcome::Failed(message) = outcome else {
        panic!("expected a failure, got {outcome:?}");
    };
    // The discriminating text: an Io error would also name the file.
    assert_eq!(message, ".mise.toml: `tools` is not a table");
    // The malformed file is left exactly as the user wrote it.
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".mise.toml")).unwrap(),
        "tools = 3\n"
    );
    assert!(lock.files.is_empty());
}

mod files;
/// The registration the SOKF component plans, used by the JSON tests.
mod json;
