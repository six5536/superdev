//! JSON controller adapter for checkout-local workflow records.

use std::{io::Read, path::Path};

use clap::{Args, Subcommand};
use superdev_core::{
    error::{Error, Result},
    workflow::{claim::Controller, git, request::LocalRequest, service},
};

// sokf:begin cli
/// Local workflow operations. Legacy phase-in-plan mutation commands are removed.
#[derive(Subcommand)]
pub enum WorkflowCommand {
    /// Apply one typed JSON request from standard input, on controller authority
    Apply(ControllerArgs),
    /// Report local records, current approval validity, and transient ownership
    Status {
        /// Emit the versioned JSON response (also the default)
        #[arg(long)]
        json: bool,
    },
}

/// Private controller invocation; neither credentials nor approval flags are model arguments.
#[derive(Args)]
pub struct ControllerArgs {
    /// Long-lived controlling Pi session ID
    #[arg(long)]
    session: String,
    /// Long-lived controller process, not this service process
    #[arg(long)]
    owner_pid: u32,
}
// sokf:end cli

/// Dispatch without inferring phase or approval from canonical document prose.
pub fn run(command: &WorkflowCommand, root: &Path) -> Result<u8> {
    let root = git::repository_root(root)?;
    let (operation, result) = match command {
        WorkflowCommand::Status { .. } => {
            let (workflows, claim) = service::status(&root)?;
            let executable = std::env::current_exe().map_err(|source| Error::Io {
                path: root.join("superdev"),
                source,
            })?;
            let (default_branch, default_branch_diagnostic) = match git::default_branch(&root, None)
            {
                Ok(branch) => (Some(branch), None),
                Err(error) => (None, Some(error.to_string())),
            };
            let (current_branch, current_branch_diagnostic) = match git::current_branch(&root) {
                Ok(branch) => (Some(branch), None),
                Err(error) => (None, Some(error.to_string())),
            };
            // The complete Git state, so a caller can compare before and after
            // rather than guess from the commands a stage happened to run.
            let worktree_state = git::worktree_state(&root).ok();
            // An unreadable manifest reports no policy. The adapter refuses
            // that rather than assuming automatic acceptance.
            let human_acceptance_required = superdev_core::manifest::Manifest::load(&root)
                .ok()
                .map(|manifest| manifest.workflow.human_acceptance_required);
            (
                "status",
                serde_json::json!({
                    "workflows": workflows, "claim": claim, "executable": executable,
                    "defaultBranch": default_branch, "defaultBranchDiagnostic": default_branch_diagnostic,
                    "currentBranch": current_branch, "currentBranchDiagnostic": current_branch_diagnostic,
                    "worktreeState": worktree_state,
                    "humanAcceptanceRequired": human_acceptance_required,
                }),
            )
        }
        WorkflowCommand::Apply(args) => {
            let capability =
                std::env::var("SUPERDEV_UI_AUTHORITY").map_err(|_| Error::Manifest {
                    message: "workflow changes require private interactive-controller authority"
                        .into(),
                })?;
            let mut bytes = Vec::new();
            std::io::stdin()
                .take(1024 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|source| Error::Io {
                    path: "<workflow stdin>".into(),
                    source,
                })?;
            if bytes.len() > 1024 * 1024 {
                return Err(Error::Manifest {
                    message: "workflow request exceeds the size limit".into(),
                });
            }
            let request: LocalRequest =
                serde_json::from_slice(&bytes).map_err(|error| Error::Manifest {
                    message: format!("invalid workflow request: {error}"),
                })?;
            let record = service::apply(
                &root,
                &request,
                &Controller {
                    session: &args.session,
                    pid: args.owner_pid,
                    capability: &capability,
                },
            )?;
            ("apply", serde_json::json!({ "record": record }))
        }
    };
    println!(
        "{}",
        serde_json::json!({
            "protocol": service::LOCAL_WORKFLOW_PROTOCOL,
            "operation": operation,
            "result": result,
        })
    );
    Ok(0)
}
