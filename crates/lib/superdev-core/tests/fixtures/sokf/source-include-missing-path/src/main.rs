use clap::Parser;

// sokf:begin cli
/// The command line.
#[derive(Parser)]
struct Cli {
    /// Repair what the check reports.
    #[arg(long)]
    fix: bool,
}
// sokf:end cli

fn main() {}
