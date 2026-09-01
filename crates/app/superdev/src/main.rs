//! superdev CLI entry point.
//!
//! Argument parsing and exit codes only: the verbs live in [`manage`],
//! [`validate_cli`] and [`sokf_cli`], the domain logic in `superdev-core`.
//! `completions` and the hidden `man` are the plumbing the release pipeline
//! needs.
// Under the nightly coverage job (cargo-llvm-cov sets `coverage_nightly`), enable
// the attribute used to exclude genuinely untestable glue from coverage. Inert on
// the stable toolchain used for normal builds and tests.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![warn(missing_docs)]

mod cli;
#[cfg(test)]
mod contract;
mod manage;
mod run;
mod sokf_cli;
mod template_select;
mod validate_cli;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use superdev_core::error::{Error, Result};

#[derive(Parser)]
#[command(
    name = "superdev",
    version = superdev_core::version(),
    about = "superdev — project scaffold",
    // Hand-wrapped: clap's `wrap_help` is not enabled, so a paragraph here
    // renders as one long line and runs off the terminal. roff reflows it
    // again for the man page, so the breaks cost nothing there.
    long_about = "superdev — project scaffold.\n\n\
        Sets a repository up for agent-driven development and keeps that\n\
        setup current. The skills, templates and scaffolds it writes come\n\
        from a content pack: one ships inside this binary, and `[[packs]]`\n\
        in .superdev/config.toml points at another — a git source or a\n\
        directory — to add your own or supersede superdev's. Content\n\
        releases under its own assets-vX.Y.Z tags, and `superdev update`\n\
        is the verb that goes looking for the newest one this binary can\n\
        read."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Set this repo up for agent-driven development
    Init(manage::InitArgs),
    /// Report drift between the repo and its blueprint
    Status {
        /// Exit on drift alone, ignoring external state a checkout never
        /// carries (an unbuilt code index, an uninstalled tool)
        #[arg(long)]
        drift: bool,
    },
    /// Re-apply the blueprint so the repo matches the manifest
    Sync {
        /// Print the plan without applying it
        #[arg(long)]
        dry_run: bool,
    },
    /// Bring pins current, then sync
    // Explicit, and hand-wrapped: clap joins a doc comment's lines into one
    // paragraph, and without `wrap_help` that paragraph never breaks.
    #[command(long_about = "Bring pins current, then sync.\n\n\
            A capability's pin moves to this binary's default. The pack's\n\
            moves to the newest release its source carries that this binary\n\
            can read, which may be past what it embeds — the one place\n\
            superdev reaches the network unasked. A release it cannot read\n\
            is reported and the pin stays where it was.")]
    Update {
        /// Capability to update, optionally `<capability>@<version>`
        target: Option<String>,
        /// Provider to switch the target capability to
        #[arg(long, value_name = "ID")]
        provider: Option<String>,
    },
    /// Check the SOKF knowledge and the files the grammar governs
    Validate(validate_cli::ValidateArgs),
    /// Inspect and render the shipped project templates
    #[command(subcommand)]
    Template(manage::TemplateCommand),
    /// Serve project subsystems over MCP
    #[command(subcommand)]
    Mcp(sokf_cli::McpCommand),
    /// SOKF knowledge commands
    #[command(subcommand)]
    Sokf(sokf_cli::SokfCommand),
    /// Drive the state of an unattended workflow run
    #[command(subcommand)]
    Run(run::RunCommand),
    /// Agent hook plumbing (reads the hook payload from stdin)
    #[command(subcommand)]
    Hook(validate_cli::HookCommand),
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
        Some(Command::Status { drift }) => manage::status(&root()?, *drift),
        Some(Command::Sync { dry_run }) => manage::sync(&root()?, *dry_run),
        Some(Command::Update { target, provider }) => {
            manage::update(&root()?, target.as_deref(), provider.as_deref())
        }
        Some(Command::Validate(args)) => validate_cli::run_validate(args, &root()?),
        Some(Command::Template(cmd)) => manage::template(cmd),
        Some(Command::Mcp(cmd)) => sokf_cli::run_mcp(cmd, &root()?),
        Some(Command::Sokf(cmd)) => sokf_cli::run_sokf(cmd, &root()?),
        Some(Command::Run(cmd)) => run::run(cmd, &root()?),
        Some(Command::Hook(cmd)) => validate_cli::run_hook(cmd, &root()?),
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
