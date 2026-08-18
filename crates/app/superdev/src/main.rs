//! superdev CLI entry point.
//!
//! Argument parsing and exit codes only: the verbs live in [`manage`] and
//! [`aokf_cli`], the domain logic in `superdev-core`. `completions` and the
//! hidden `man` are the plumbing the release pipeline needs.
// Under the nightly coverage job (cargo-llvm-cov sets `coverage_nightly`), enable
// the attribute used to exclude genuinely untestable glue from coverage. Inert on
// the stable toolchain used for normal builds and tests.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(missing_docs)]

mod aokf_cli;
mod manage;
mod template_select;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use superdev_core::error::{Error, Result};

#[derive(Parser)]
#[command(name = "superdev", version = superdev_core::version(), about = "superdev — project scaffold")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Set this repo up for agent-driven development
    Init(manage::InitArgs),
    /// Report drift between the repo and its blueprint
    Status,
    /// Re-apply the blueprint so the repo matches the manifest
    Sync {
        /// Print the plan without applying it
        #[arg(long)]
        dry_run: bool,
    },
    /// Move version pins to this binary's defaults, then sync
    Update {
        /// Capability to update, optionally `<capability>@<version>`
        target: Option<String>,
        /// Provider to switch the target capability to
        #[arg(long, value_name = "ID")]
        provider: Option<String>,
    },
    /// Serve project subsystems over MCP
    #[command(subcommand)]
    Mcp(aokf_cli::McpCommand),
    /// AOKF knowledgebase commands
    #[command(subcommand)]
    Aokf(aokf_cli::AokfCommand),
    /// Write a completion script for the given shell to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Write the man page (roff) to stdout
    #[command(hide = true)]
    Man,
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> ExitCode {
    match run(&Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        // A closed stdout (`| head`, a quit pager) is a reader stopping early,
        // not a failure.
        Err(Error::Io { source, .. }) if source.kind() == io::ErrorKind::BrokenPipe => {
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli) -> Result<u8> {
    match &cli.command {
        Some(Command::Init(args)) => manage::init(&root()?, args),
        Some(Command::Status) => manage::status(&root()?),
        Some(Command::Sync { dry_run }) => manage::sync(&root()?, *dry_run),
        Some(Command::Update { target, provider }) => {
            manage::update(&root()?, target.as_deref(), provider.as_deref())
        }
        Some(Command::Mcp(cmd)) => aokf_cli::run_mcp(cmd, &root()?),
        Some(Command::Aokf(cmd)) => aokf_cli::run_aokf(cmd, &root()?),
        Some(Command::Completions { shell }) => {
            // Render into a buffer first: clap_complete panics rather than
            // returning an error when a write fails.
            let mut buf = Vec::new();
            clap_complete::generate(*shell, &mut Cli::command(), "superdev", &mut buf);
            write_stdout(&buf)
        }
        Some(Command::Man) => {
            let mut buf = Vec::new();
            clap_mangen::Man::new(Cli::command())
                .render(&mut buf)
                .map_err(stdout_error)?;
            write_stdout(&buf)
        }
        None => {
            Cli::command().print_help().map_err(stdout_error)?;
            Ok(0)
        }
    }
}

/// The repo superdev manages: wherever it was run.
fn root() -> Result<PathBuf> {
    std::env::current_dir().map_err(|source| Error::Io {
        path: ".".into(),
        source,
    })
}

fn write_stdout(buf: &[u8]) -> Result<u8> {
    io::stdout().write_all(buf).map_err(stdout_error)?;
    Ok(0)
}

/// Stdout failures carry `-` as their path, so `main` can spot a broken pipe.
fn stdout_error(source: io::Error) -> Error {
    Error::Io {
        path: "-".into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn cli_definition_is_consistent() {
        Cli::command().debug_assert();
    }
}
