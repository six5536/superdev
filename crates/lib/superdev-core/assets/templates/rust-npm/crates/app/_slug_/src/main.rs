//! The {{superdev:project-name}} binary: parse arguments, call the core
//! library, exit. Keep logic in the core crate, keep this thin.

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") => {
            println!("{{superdev:project-slug}} {}", {{superdev:project-ident}}_core::version());
            ExitCode::SUCCESS
        }
        // Usage errors exit 2 — the launcher and the smoke tests rely on the
        // code, so keep it when replacing this stub with a real CLI parser.
        Some(flag) if flag.starts_with('-') => {
            eprintln!("error: unknown option {flag}");
            eprintln!("usage: {{superdev:project-slug}} [--version]");
            ExitCode::from(2)
        }
        _ => {
            println!("{}", {{superdev:project-ident}}_core::greeting());
            ExitCode::SUCCESS
        }
    }
}
