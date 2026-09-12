//! Preservation of stopped v1 ownership before local-record reconstruction.

use super::*;
use crate::workflow::store::Files;

impl Transaction<'_> {
    /// Preserve and retire stopped legacy ownership before local-record migration.
    /// Unknown or live writers are never displaced by migration.
    pub fn archive_legacy(&mut self) -> Result<()> {
        let Some(owner) = self.load()? else {
            return Ok(());
        };
        if owner.version != 1 {
            return Err(Error::Manifest {
                message: "unsupported legacy workflow version; preserve it for manual recovery"
                    .into(),
            });
        }
        if !abandoned(&owner)
            || owner.child_pid.is_some()
                && liveness(owner.child_pid, owner.child_started.as_deref()) != Liveness::Dead
        {
            return Err(Error::Manifest { message: "stop all legacy workflow writers before migration; unknown liveness requires explicit inspection".into() });
        }
        let mut original = String::new();
        self.files
            .directory
            .open("workflow.toml")
            .and_then(|mut file| file.read_to_string(&mut original))
            .map_err(|source| Error::Io {
                path: path(self.root),
                source,
            })?;
        let name = format!("workflow-{}.toml", content_digest(original.as_bytes()));
        // Atomic replacement makes an interrupted backup repeatable. The source
        // remains intact until both the backup file and its directory are synced.
        Files::open(self.root, ".superdev/workflows/legacy")?
            .write_bytes(&name, original.as_bytes())?;
        self.files
            .directory
            .remove_file("workflow.toml")
            .map_err(|source| Error::Io {
                path: path(self.root),
                source,
            })
    }
}
