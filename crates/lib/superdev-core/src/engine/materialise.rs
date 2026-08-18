//! engine/materialise.rs — copy a pinned checkout's skill directories into
//! the repo as owned, attributed files, then reconcile what the checkout no
//! longer ships. Observation happens at apply time deliberately: planning
//! needs no checkout on disk.

use std::fs;

use crate::capability::Capability;
use crate::error::Error;
use crate::fsutil::{collect_files, read_text};
use crate::lock::sha256_hex;

use super::{ALREADY_GONE, ActionOutcome, LockEffects, Session, command_line};

impl<'a> Session<'a> {
    /// Copy a pinned checkout's skill directories into the repo, then
    /// reconcile: attributed entries the checkout no longer ships leave by
    /// the same rules as RemoveFile. One aggregate outcome carries the counts.
    pub(super) fn materialise_skills(
        &mut self,
        capability: Option<Capability>,
        tool: &str,
        source_dirs: &[String],
        custom: &[String],
        effects: LockEffects<'_>,
    ) -> ActionOutcome {
        // Repo-level entries own nothing, so they have nothing to attribute —
        // and never materialise.
        let Some(capability) = capability else {
            return ActionOutcome::Failed("materialise needs an owning capability".into());
        };
        let args = vec!["where".to_string(), tool.to_string()];
        let checkout = match self.runner.run("mise", &args, self.root) {
            Ok(out) if out.status == 0 => std::path::PathBuf::from(out.stdout.trim()),
            Ok(out) => {
                return ActionOutcome::Failed(
                    Error::Command {
                        command: command_line("mise", &args),
                        status: Some(out.status),
                        stderr: out.stderr,
                    }
                    .to_string(),
                );
            }
            Err(e) => return ActionOutcome::Failed(e.to_string()),
        };
        let mut wrote = 0usize;
        let mut kept = 0usize;
        let mut edited = 0usize;
        let mut fresh: Vec<String> = Vec::new();
        for source_dir in source_dirs {
            let dir = checkout.join(source_dir);
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(e) => {
                    return ActionOutcome::Failed(
                        Error::Io {
                            path: dir,
                            source: e,
                        }
                        .to_string(),
                    );
                }
            };
            for entry in entries {
                let Ok(entry) = entry else { continue };
                let path = entry.path();
                // A skill is a directory; loose files beside them are the
                // checkout's own, e.g. its README.
                if !path.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if custom.contains(&name) {
                    continue;
                }
                let mut files = Vec::new();
                if let Err(e) = collect_files(&path, &mut files) {
                    return ActionOutcome::Failed(e.to_string());
                }
                for file in files {
                    let rel = file
                        .strip_prefix(&path)
                        .expect("collected under this skill directory");
                    let target = format!(
                        ".claude/skills/{name}/{}",
                        rel.display().to_string().replace('\\', "/")
                    );
                    let content = match read_text(&file) {
                        Ok(Some(content)) => content,
                        Ok(None) => continue,
                        Err(e) => return ActionOutcome::Failed(e.to_string()),
                    };
                    fresh.push(target.clone());
                    let existing = match read_text(&self.root.join(&target)) {
                        Ok(existing) => existing,
                        Err(e) => return ActionOutcome::Failed(e.to_string()),
                    };
                    // An unchanged file is still claimed, so a converged run
                    // keeps its lock entry and its attribution.
                    if existing.as_deref() == Some(content.as_str()) {
                        kept += 1;
                        effects
                            .written
                            .push((target.clone(), sha256_hex(content.as_bytes())));
                        effects.attributed.push(target);
                        continue;
                    }
                    // The owned-write ritual, on Tx directly: back up what was
                    // there, note a user edit, journal, write.
                    if let Some(old) = &existing {
                        if let Err(e) = self.tx.backup(&target, old) {
                            return ActionOutcome::Failed(e.to_string());
                        }
                        if self.prior_hashes.get(&target) != Some(&sha256_hex(old.as_bytes())) {
                            edited += 1;
                        }
                    }
                    if let Err(e) = self.tx.write(&target, existing, &content) {
                        return ActionOutcome::Failed(e.to_string());
                    }
                    wrote += 1;
                    effects
                        .written
                        .push((target.clone(), sha256_hex(content.as_bytes())));
                    effects.attributed.push(target);
                }
            }
        }
        // Reconcile: what this capability had materialised and the checkout
        // no longer ships leaves by the RemoveFile rules.
        let mut released = 0usize;
        let mut swept = 0usize;
        let stale: Vec<String> = self
            .prior_owners
            .iter()
            .filter(|(key, owner)| owner.as_str() == capability.as_str() && !fresh.contains(key))
            .map(|(key, _)| key.clone())
            .collect();
        for key in stale {
            match self.remove_file(&key, effects.removed) {
                ActionOutcome::Applied { .. } => swept += 1,
                ActionOutcome::Skipped(reason) if reason == ALREADY_GONE => {}
                ActionOutcome::Skipped(_) => released += 1,
                ActionOutcome::Failed(e) => return ActionOutcome::Failed(e),
            }
        }
        let mut note = format!("wrote {wrote}, kept {kept}, removed {swept}, released {released}");
        if edited > 0 {
            note.push_str(&format!("; overwrote {edited} user-edited (backed up)"));
        }
        ActionOutcome::Applied { note: Some(note) }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::action::Action;
    use crate::lock::Lock;
    use crate::manifest::Manifest;
    use crate::runner::{FakeRunner, Output};

    /// A fake checkout: skills/engineering/{alpha,beta}, skills/productivity/gamma.
    /// alpha has a nested reference file; a stray README sits beside the dirs.
    fn fake_checkout() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let write = |rel: &str, content: &str| {
            let p = dir.path().join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        };
        write("skills/engineering/alpha/SKILL.md", "alpha v1");
        write("skills/engineering/alpha/refs/DEEP.md", "alpha deep");
        write("skills/engineering/beta/SKILL.md", "beta v1");
        write("skills/engineering/README.md", "not a skill");
        write("skills/productivity/gamma/SKILL.md", "gamma v1");
        dir
    }

    fn materialise_action(custom: &[&str]) -> Action {
        Action::MaterialiseSkills {
            tool: "http:mattpocock-skills".into(),
            source_dirs: vec!["skills/engineering".into(), "skills/productivity".into()],
            custom: custom.iter().map(|c| (*c).to_string()).collect(),
        }
    }

    fn where_scripted(checkout: &std::path::Path) -> FakeRunner {
        let fake = FakeRunner::new();
        fake.script(
            "mise where http:mattpocock-skills",
            Output {
                status: 0,
                stdout: format!("{}\n", checkout.display()),
                stderr: String::new(),
            },
        );
        fake
    }

    #[test]
    fn materialise_writes_locks_and_attributes() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/alpha/refs/DEEP.md")).unwrap(),
            "alpha deep"
        );
        assert!(dir.path().join(".claude/skills/gamma/SKILL.md").exists());
        // The stray README is not a skill directory and is not copied.
        assert!(!dir.path().join(".claude/skills/README.md").exists());
        let key = ".claude/skills/alpha/SKILL.md";
        assert_eq!(lock.files[key], sha256_hex(b"alpha v1"));
        assert_eq!(lock.owners[key], "workflows");
        assert_eq!(lock.owners.len(), 4);
    }

    #[test]
    fn materialise_reconciles_dropped_skills_and_skips_custom() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        // A managed pin already committed, so the run has tools to install.
        std::fs::write(
            dir.path().join(".mise.toml"),
            mise::set_pin("", "http:superpowers", "\"6.2.0\"").unwrap(),
        )
        .unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        // A previously materialised skill the checkout no longer ships…
        std::fs::create_dir_all(dir.path().join(".claude/skills/old")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/old/SKILL.md"), "old v1").unwrap();
        // …and one the user edited since.
        std::fs::create_dir_all(dir.path().join(".claude/skills/mine")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/mine/SKILL.md"), "edited").unwrap();
        let mut lock = Lock::default();
        for (key, content) in [
            (".claude/skills/old/SKILL.md", "old v1"),
            (".claude/skills/mine/SKILL.md", "mine v1"),
        ] {
            lock.files
                .insert(key.into(), sha256_hex(content.as_bytes()));
            lock.owners.insert(key.into(), "workflows".into());
        }
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&["beta"])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        // alpha's two files and gamma's one; old swept, mine released.
        let materialised = result.reports[0]
            .outcomes
            .iter()
            .find(|(d, _)| d.starts_with("materialise"))
            .map(|(_, o)| o.clone())
            .unwrap_or_else(|| panic!("outcomes: {:?}", result.reports[0].outcomes));
        assert_eq!(
            materialised,
            ActionOutcome::Applied {
                note: Some("wrote 3, kept 0, removed 1, released 1".into())
            }
        );
        // Dropped and unmodified: deleted, with a backup.
        assert!(!dir.path().join(".claude/skills/old/SKILL.md").exists());
        // User-edited: left in place, released from the lock.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/mine/SKILL.md")).unwrap(),
            "edited"
        );
        assert!(!lock.files.contains_key(".claude/skills/old/SKILL.md"));
        assert!(!lock.files.contains_key(".claude/skills/mine/SKILL.md"));
        assert!(
            lock.owners
                .keys()
                .all(|k| !k.contains("/old/") && !k.contains("/mine/"))
        );
        // Custom skill: never written, never attributed.
        assert!(!dir.path().join(".claude/skills/beta").exists());
        assert!(!lock.files.keys().any(|k| k.contains("/beta/")));
        // The pinned tool is installed before the checkout is read.
        let calls = fake.calls();
        let install = calls
            .iter()
            .position(|c| c.starts_with("mise install"))
            .unwrap();
        let where_ = calls
            .iter()
            .position(|c| c.starts_with("mise where"))
            .unwrap();
        assert!(install < where_, "calls: {calls:?}");
    }

    #[test]
    fn materialise_failures_unwind() {
        // No checkout: `mise where` fails.
        let dir = tempfile::tempdir().unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "mise where http:mattpocock-skills",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "not installed".into(),
            },
        );
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(lock.files.is_empty());

        // Non-UTF-8 checkout content: fails and unwinds the files written first.
        let checkout = fake_checkout();
        std::fs::write(
            checkout.path().join("skills/productivity/gamma/SKILL.md"),
            [0xff, 0xfe],
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let fake = where_scripted(checkout.path());
        let mut lock = Lock::default();
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(!result.ok);
        assert!(
            !dir.path().join(".claude/skills/alpha/SKILL.md").exists(),
            "earlier writes must unwind"
        );
    }

    #[test]
    fn a_converged_materialise_rewrites_nothing() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for("0.1.0", &[]);
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let mut lock = Lock::default();
        let fake = where_scripted(checkout.path());
        assert!(apply(dir.path(), &fake, &manifest, &planned, &mut lock).ok);
        let fake = where_scripted(checkout.path());
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok);
        // Every file was new on the first run and unchanged on the second, so
        // nothing was ever overwritten and no backup exists.
        let backups = std::fs::read_dir(dir.path().join(tx::BACKUP_DIR))
            .map(|d| d.count())
            .unwrap_or(0);
        assert_eq!(backups, 0, "an unchanged file is never backed up");
        assert_eq!(lock.owners.len(), 4);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Applied {
                note: Some("wrote 0, kept 4, removed 0, released 0".into())
            }
        );
    }

    #[test]
    fn materialise_notes_the_user_edits_it_overwrote() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        // A skill file the user rewrote: the lock never saw this content.
        std::fs::create_dir_all(dir.path().join(".claude/skills/beta")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/beta/SKILL.md"), "mine").unwrap();
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        let mut lock = Lock::default();
        let planned = vec![Planned {
            capability: Some(crate::capability::Capability::Workflows),
            provider: "mattpocock-skills".into(),
            actions: vec![materialise_action(&[])],
        }];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        assert_eq!(
            result.reports[0].outcomes[0].1,
            ActionOutcome::Applied {
                note: Some(
                    "wrote 4, kept 0, removed 0, released 0; overwrote 1 user-edited (backed up)"
                        .into()
                )
            }
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join(".claude/skills/beta/SKILL.md")).unwrap(),
            "beta v1"
        );
    }

    /// A provider switch in one run: the old skill pack's file is orphaned
    /// (the orphan entry is planned last, from pre-run claims) while the new
    /// provider materialises the very same path. The materialise must win.
    #[test]
    fn a_materialised_key_survives_the_same_runs_orphan_removal() {
        let checkout = fake_checkout();
        let dir = tempfile::tempdir().unwrap();
        let key = ".claude/skills/alpha/SKILL.md";
        // The old pack's file, on disk and in the lock, unowned.
        std::fs::create_dir_all(dir.path().join(".claude/skills/alpha")).unwrap();
        std::fs::write(dir.path().join(key), "pack alpha").unwrap();
        let mut lock = Lock::default();
        lock.files.insert(key.into(), sha256_hex(b"pack alpha"));
        let fake = where_scripted(checkout.path());
        let manifest = Manifest::default_for("0.1.0", &[]);
        let planned = vec![
            Planned {
                capability: Some(crate::capability::Capability::Workflows),
                provider: "mattpocock-skills".into(),
                actions: vec![materialise_action(&[])],
            },
            Planned {
                capability: None,
                provider: "orphan".into(),
                actions: vec![Action::RemoveFile {
                    path: key.into(),
                    reason: "no longer claimed".into(),
                }],
            },
        ];
        let result = apply(dir.path(), &fake, &manifest, &planned, &mut lock);
        assert!(result.ok, "{:?}", result.reports);
        // The orphan still reports the skip: the file it saw is not the one
        // it planned against.
        assert_eq!(
            result.reports[1].outcomes[0].1,
            ActionOutcome::Skipped("changed since superdev wrote it — left in place".into())
        );
        // …but the fresh file stays managed.
        assert_eq!(
            std::fs::read_to_string(dir.path().join(key)).unwrap(),
            "alpha v1"
        );
        assert_eq!(lock.files[key], sha256_hex(b"alpha v1"));
        assert_eq!(lock.owners[key], "workflows");
    }
}
