//! Transient controller/worker ownership, separate from durable approval.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    cache, process,
    store::{Files, refusal},
};
use crate::error::Result;

const CLAIM_FILE: &str = "workflow-claim.json";

/// Live controller credentials. Never serialised into durable workflow records.
pub struct Controller<'a> {
    /// Owning Pi session.
    pub session: &'a str,
    /// Long-lived controller process, not the short-lived service process.
    pub pid: u32,
    /// Secret held by the interactive adapter; absent from model/worker environments.
    pub capability: &'a str,
}

/// A live claim is not progress or approval evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    /// Claim schema version.
    pub version: u32,
    /// Local workflow identity.
    pub workflow: String,
    /// Owning Pi session.
    pub session: String,
    /// Digest of the private controller capability.
    pub authority_digest: String,
    /// Controller process ID.
    pub owner_pid: Option<u32>,
    /// Process start identity, preventing PID-reuse recovery.
    pub owner_started: Option<String>,
    /// Worker process, retained until its exit is established.
    pub child_pid: Option<u32>,
    /// Worker start identity.
    pub child_started: Option<String>,
}

impl Claim {
    /// Whether both recorded writers are provably stopped.
    pub fn stopped(&self) -> bool {
        process::liveness(self.owner_pid, self.owner_started.as_deref()) == process::Liveness::Dead
            && self.child_stopped()
    }

    fn child_stopped(&self) -> bool {
        self.child_pid.is_none()
            || process::liveness(self.child_pid, self.child_started.as_deref())
                == process::Liveness::Dead
    }
}

pub(super) fn load(root: &Path) -> Result<Option<Claim>> {
    let claim: Option<Claim> = Files::open(root, ".superdev/cache")?.read(CLAIM_FILE)?;
    if claim.as_ref().is_some_and(|claim| claim.version != 1) {
        return Err(refusal(
            "unsupported workflow claim version; stop its writers before recovery",
        ));
    }
    Ok(claim)
}

pub(super) fn acquire(root: &Path, id: &str, controller: &Controller<'_>) -> Result<()> {
    let claim = available(root, id, controller)?;
    Files::open(root, ".superdev/cache")?.write(CLAIM_FILE, &claim)
}

pub(super) fn available(root: &Path, id: &str, controller: &Controller<'_>) -> Result<Claim> {
    let digest = cache::authority_digest(controller.capability)?;
    if controller.session.trim().is_empty() || controller.pid == 0 {
        return Err(refusal(
            "workflow ownership requires a session and controller process",
        ));
    }
    if let Some(claim) = load(root)? {
        if claim.workflow == id
            && claim.session == controller.session
            && claim.authority_digest == digest
        {
            if !claim.child_stopped() {
                return Err(refusal(
                    "worker must stop before controller ownership resumes",
                ));
            }
        } else if !claim.stopped() {
            return Err(refusal(
                "another controller or worker still owns this checkout; stop it before resuming",
            ));
        }
    }
    let (owner_pid, owner_started) = process::record(Some(controller.pid));
    if process::liveness(owner_pid, owner_started.as_deref()) == process::Liveness::Dead {
        return Err(refusal(
            "cannot acquire a workflow for a stopped controller",
        ));
    }
    Ok(Claim {
        version: 1,
        workflow: id.into(),
        session: controller.session.into(),
        authority_digest: digest,
        owner_pid,
        owner_started,
        child_pid: None,
        child_started: None,
    })
}

pub(super) fn verify(root: &Path, id: &str, controller: &Controller<'_>) -> Result<()> {
    let claim = owned(root, id, controller)?;
    if !claim.child_stopped() {
        return Err(refusal(
            "pause the worker before writing from the controller",
        ));
    }
    Ok(())
}

/// Check controller authority while a worker may still be executing.
///
/// Only durable execution facts use this path. The worker remains the sole
/// writer of the checkout; recording its progress is not a checkout mutation.
pub(super) fn verify_active(root: &Path, id: &str, controller: &Controller<'_>) -> Result<()> {
    owned(root, id, controller).map(|_| ())
}

fn owned(root: &Path, id: &str, controller: &Controller<'_>) -> Result<Claim> {
    let claim =
        load(root)?.ok_or_else(|| refusal("workflow has no live claim; resume it first"))?;
    let digest = cache::authority_digest(controller.capability)?;
    if claim.workflow != id
        || claim.session != controller.session
        || claim.authority_digest != digest
    {
        return Err(refusal(
            "workflow controller authority is absent or changed",
        ));
    }
    Ok(claim)
}

/// Record one executing worker. A second worker is refused while the first runs.
pub(super) fn attach_child(
    root: &Path,
    id: &str,
    controller: &Controller<'_>,
    pid: u32,
) -> Result<()> {
    let mut claim = owned(root, id, controller)?;
    if !claim.child_stopped() {
        return Err(refusal(
            "a worker already owns this checkout; stop it before starting another",
        ));
    }
    let (child_pid, child_started) = process::record(Some(pid));
    // Attachment asserts a running writer, so only observed liveness will do.
    // An unknowable process is refused here, rather than recorded as an owner
    // that nothing can later prove stopped.
    if process::liveness(child_pid, child_started.as_deref()) != process::Liveness::Live {
        return Err(refusal(
            "cannot attach a worker whose process is not observably running",
        ));
    }
    claim.child_pid = child_pid;
    claim.child_started = child_started;
    Files::open(root, ".superdev/cache")?.write(CLAIM_FILE, &claim)
}

/// Forget a worker only once the running system contradicts it.
pub(super) fn detach_child(root: &Path, id: &str, controller: &Controller<'_>) -> Result<()> {
    let mut claim = owned(root, id, controller)?;
    if claim.child_pid.is_none() {
        return Err(refusal("no worker is attached to this workflow"));
    }
    if !claim.child_stopped() {
        return Err(refusal(
            "worker is still running; stop it before releasing its ownership",
        ));
    }
    claim.child_pid = None;
    claim.child_started = None;
    Files::open(root, ".superdev/cache")?.write(CLAIM_FILE, &claim)
}

pub(super) fn release(root: &Path, id: &str, controller: &Controller<'_>) -> Result<()> {
    verify(root, id, controller)?;
    Files::open(root, ".superdev/cache")?.remove(CLAIM_FILE)
}
