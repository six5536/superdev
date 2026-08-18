//! engine/tx.rs — the transaction under one apply run. Every side effect is
//! journalled here as it happens, so the first failure can unwind the run in
//! reverse. Tx is dumb on purpose: drift guards, skip decisions and the
//! choice of what to back up stay with the appliers; this type journals,
//! writes, removes, backs up and unwinds — mechanism, never policy.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::fsutil::write_file;
use crate::runner::CommandRunner;

use super::command_line;

/// Where backups of overwritten files go, under the repo root.
pub(super) const BACKUP_DIR: &str = ".superdev/cache/backup";

/// One journalled side effect, and how to take it back.
enum Undo {
    RestoreFile { path: String, prior: Option<String> },
    RunCommand { program: String, args: Vec<String> },
}

/// The journal for one apply run.
pub(super) struct Tx<'a> {
    root: &'a Path,
    /// One timestamp per run, so a run's backups sit together.
    stamp: u64,
    journal: Vec<Undo>,
    /// Side effects with no undo, reported when a later failure unwinds.
    irreversible: Vec<String>,
}

impl<'a> Tx<'a> {
    pub(super) fn new(root: &'a Path) -> Tx<'a> {
        Tx {
            root,
            stamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            journal: Vec::new(),
            irreversible: Vec::new(),
        }
    }

    /// The restore point for `path` — the one place the journal learns about
    /// a file change. `prior` is what the unwind puts back; `None` removes
    /// the file.
    fn restore_point(&mut self, path: &str, prior: Option<String>) {
        self.journal.push(Undo::RestoreFile {
            path: path.to_string(),
            prior,
        });
    }

    /// Journal `prior`, then write `content` to `path`.
    pub(super) fn write(&mut self, path: &str, prior: Option<String>, content: &str) -> Result<()> {
        self.restore_point(path, prior);
        write_file(&self.root.join(path), content)
    }

    /// Journal `prior`, then remove `path`. The raw error keeps the applier's
    /// reported message exactly what `fs::remove_file` says.
    pub(super) fn remove(&mut self, path: &str, prior: String) -> std::io::Result<()> {
        self.restore_point(path, Some(prior));
        fs::remove_file(self.root.join(path))
    }

    /// Copy `content` into this run's backup directory under `path`.
    pub(super) fn backup(&self, path: &str, content: &str) -> Result<()> {
        write_file(
            &self
                .root
                .join(BACKUP_DIR)
                .join(self.stamp.to_string())
                .join(path),
            content,
        )
    }

    /// Journal a command that takes a successful run back.
    pub(super) fn record_command_undo(&mut self, program: String, args: Vec<String>) {
        self.journal.push(Undo::RunCommand { program, args });
    }

    /// Note a side effect nothing can undo; reported when a failure unwinds.
    pub(super) fn mark_irreversible(&mut self, line: String) {
        self.irreversible.push(line);
    }

    /// Take back this run's side effects, newest first.
    pub(super) fn unwind(&mut self, runner: &dyn CommandRunner) -> (Vec<String>, Vec<String>) {
        let mut reverted = Vec::new();
        let mut not_reverted = Vec::new();
        while let Some(undo) = self.journal.pop() {
            match undo {
                Undo::RestoreFile {
                    path,
                    prior: Some(old),
                } => match write_file(&self.root.join(&path), &old) {
                    Ok(()) => reverted.push(format!("restored {path}")),
                    Err(e) => not_reverted.push(format!("{path} left changed: {e}")),
                },
                Undo::RestoreFile { path, prior: None } => {
                    match fs::remove_file(self.root.join(&path)) {
                        Ok(()) => reverted.push(format!("removed {path}")),
                        // Never created: the write is what failed.
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => not_reverted.push(format!("{path} left in place: {e}")),
                    }
                }
                Undo::RunCommand { program, args } => {
                    let line = command_line(&program, &args);
                    match runner.run(&program, &args, self.root) {
                        Ok(out) if out.status == 0 => reverted.push(format!("ran `{line}`")),
                        _ => not_reverted.push(format!("`{line}` did not undo cleanly")),
                    }
                }
            }
        }
        not_reverted.append(&mut self.irreversible);
        (reverted, not_reverted)
    }
}

#[cfg(test)]
mod tests {
    use super::super::command_line;
    use super::*;
    use crate::runner::{FakeRunner, Output};

    #[test]
    fn unwind_replays_newest_first_and_reports_the_irreversible() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "old a").unwrap();
        let fake = FakeRunner::new();
        let mut tx = Tx::new(dir.path());

        tx.write("a.txt", Some("old a".into()), "new a").unwrap();
        tx.write("b.txt", None, "fresh b").unwrap();
        tx.record_command_undo("undoer".into(), vec!["--revert".into()]);
        tx.mark_irreversible("`launch` has no undo".into());

        let (reverted, not_reverted) = tx.unwind(&fake);
        // Newest first: the command undo runs before the file restores.
        assert_eq!(
            reverted,
            vec![
                "ran `undoer --revert`".to_string(),
                "removed b.txt".to_string(),
                "restored a.txt".to_string(),
            ]
        );
        assert_eq!(not_reverted, vec!["`launch` has no undo".to_string()]);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("a.txt")).unwrap(),
            "old a"
        );
        assert!(!dir.path().join("b.txt").exists());
        assert_eq!(fake.calls(), vec!["undoer --revert".to_string()]);
    }

    #[test]
    fn remove_journals_the_prior_and_a_failed_undo_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("gone.txt"), "kept").unwrap();
        let fake = FakeRunner::new();
        fake.script(
            "undoer",
            Output {
                status: 1,
                stdout: String::new(),
                stderr: "no".into(),
            },
        );
        let mut tx = Tx::new(dir.path());
        tx.remove("gone.txt", "kept".into()).unwrap();
        assert!(!dir.path().join("gone.txt").exists());
        tx.record_command_undo("undoer".into(), Vec::new());

        let (reverted, not_reverted) = tx.unwind(&fake);
        assert_eq!(reverted, vec!["restored gone.txt".to_string()]);
        assert_eq!(
            not_reverted,
            vec![format!(
                "`{}` did not undo cleanly",
                command_line("undoer", &[])
            )]
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("gone.txt")).unwrap(),
            "kept"
        );
    }

    #[test]
    fn backups_of_one_run_share_a_stamp_directory() {
        let dir = tempfile::tempdir().unwrap();
        let tx = Tx::new(dir.path());
        tx.backup("x/a.txt", "one").unwrap();
        tx.backup("y/b.txt", "two").unwrap();
        let backups = dir.path().join(".superdev/cache/backup");
        let stamps: Vec<_> = std::fs::read_dir(&backups).unwrap().collect();
        assert_eq!(stamps.len(), 1, "one stamp directory per run");
        let stamp = stamps[0].as_ref().unwrap().path();
        assert_eq!(
            std::fs::read_to_string(stamp.join("x/a.txt")).unwrap(),
            "one"
        );
        assert_eq!(
            std::fs::read_to_string(stamp.join("y/b.txt")).unwrap(),
            "two"
        );
    }
}
