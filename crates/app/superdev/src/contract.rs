//! contract.rs — the CLI contract bound to the command line it defines.
//!
//! [ADR-036] obliges a project to prove that its implemented interface equals
//! what its contract declares, and leaves the mechanism to the project. This
//! is superdev's: it walks the command tree this binary builds and compares it,
//! element for element in both directions, to the definition block in
//! `contract-002-cli-superdev`. An element on one side and not the other fails,
//! so a flag cannot be added to the binary without being added to the contract.
//!
//! Print the tree in the contract's own shape — the starting point when a
//! command is added — with:
//!
//! ```sh
//! SUPERDEV_PRINT_CLI=1 cargo test -p superdev --bin superdev cli_surface
//! ```

use std::collections::BTreeMap;

use clap::CommandFactory;

/// One command's surface, as the contract declares it and as clap builds it.
#[derive(Debug, Default, PartialEq, Eq)]
struct Surface {
    /// Positional arguments, in order, each as it is written in usage.
    args: Vec<String>,
    /// Long flag to its value type: `bool` for a switch, else the value name.
    flags: BTreeMap<String, String>,
}

/// `-h` and `--help` sit on every command and `-V` on the root: the framework
/// adds them, the contract states the rule once rather than per entry, and
/// this test asserts the rule instead of comparing the repetition.
const FRAMEWORK_FLAGS: [&str; 2] = ["help", "version"];

/// Every command this binary offers, keyed by the path a user types.
fn implemented() -> BTreeMap<String, Surface> {
    let mut out = BTreeMap::new();
    walk(&crate::Cli::command(), "superdev", &mut out);
    out
}

/// One command and its subcommands, in the shape the contract declares.
fn walk(command: &clap::Command, path: &str, out: &mut BTreeMap<String, Surface>) {
    let mut surface = Surface::default();
    for arg in command.get_arguments() {
        if FRAMEWORK_FLAGS.contains(&arg.get_id().as_str()) {
            continue;
        }
        if arg.is_positional() {
            surface.args.push(
                arg.get_value_names()
                    .and_then(|names| names.first().map(ToString::to_string))
                    .unwrap_or_else(|| arg.get_id().to_string()),
            );
            continue;
        }
        let Some(long) = arg.get_long() else { continue };
        // A switch takes no value, whatever name the framework gives its id.
        let switch = matches!(
            arg.get_action(),
            clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
        );
        let value = if switch {
            "bool".to_string()
        } else {
            arg.get_value_names()
                .and_then(|names| names.first().map(ToString::to_string))
                .unwrap_or_else(|| arg.get_id().to_string())
        };
        surface.flags.insert(format!("--{long}"), value);
    }
    out.insert(path.to_string(), surface);
    for sub in command.get_subcommands() {
        // `help` is the framework's own command, like its flags.
        if sub.get_name() == "help" {
            continue;
        }
        walk(sub, &format!("{path} {}", sub.get_name()), out);
    }
}

/// The contract's definition block, as the same shape.
fn declared() -> BTreeMap<String, Surface> {
    let text = std::fs::read_to_string(contract_path()).expect("the CLI contract is on file");
    let block = fenced_block(&text, "yaml").expect("the Commands section carries a yaml block");
    let raw: BTreeMap<String, serde_yaml_ng::Value> =
        serde_yaml_ng::from_str(&block).expect("the definition block parses as yaml");
    raw.into_iter()
        .map(|(path, entry)| {
            let args = entry
                .get("args")
                .and_then(|v| v.as_sequence())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|v| v.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let flags = entry
                .get("flags")
                .and_then(|v| v.as_mapping())
                .map(|map| {
                    map.iter()
                        .filter_map(|(k, v)| {
                            let name = k.as_str()?.to_string();
                            let value = v.get("type").and_then(|t| t.as_str()).unwrap_or("bool");
                            Some((name, value.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default();
            (path, Surface { args, flags })
        })
        .collect()
}

/// Where the contract lives, relative to this crate.
fn contract_path() -> std::path::PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "../../..",
        "knowledge/contracts/public/active/contract-002-cli-superdev.md",
    ]
    .iter()
    .collect()
}

/// The first fenced block carrying `tag`, without its markers.
fn fenced_block(text: &str, tag: &str) -> Option<String> {
    let mut lines = text.lines();
    let marker = loop {
        let line = lines.next()?;
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("```")
            && rest.trim() == tag
        {
            break "```";
        }
    };
    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with(marker) {
            return Some(body.join("\n"));
        }
        body.push(line);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Covers I035 criteria 4 and 6: the declared surface and the implemented
    /// command line agree element for element, in both directions. A command
    /// or flag on one side and not the other fails, naming it.
    #[test]
    fn cli_surface_matches_the_contract() {
        let built = implemented();
        if std::env::var_os("SUPERDEV_PRINT_CLI").is_some() {
            for (path, surface) in &built {
                println!("\"{path}\":");
                println!("  args: {:?}", surface.args);
                for (flag, value) in &surface.flags {
                    println!("    {flag}: {{ type: {value} }}");
                }
            }
        }
        let declared = declared();

        let missing: Vec<&String> = built
            .keys()
            .filter(|k| !declared.contains_key(*k))
            .collect();
        assert!(
            missing.is_empty(),
            "the binary offers commands the contract does not declare: {missing:?}"
        );
        let extra: Vec<&String> = declared
            .keys()
            .filter(|k| !built.contains_key(*k))
            .collect();
        assert!(
            extra.is_empty(),
            "the contract declares commands the binary does not offer: {extra:?}"
        );
        for (path, want) in &declared {
            assert_eq!(
                &built[path], want,
                "`{path}` differs between the binary and its contract"
            );
        }
    }

    /// Covers I035 criterion 6: every command carries the framework's help
    /// flag and the root its version flag, which is the rule the contract
    /// states once instead of repeating in every entry. The framework adds
    /// both while building, so what binds is that neither is disabled.
    #[test]
    fn every_command_carries_the_help_flag() {
        fn check(command: &clap::Command, path: &str) {
            assert!(
                !command.is_disable_help_flag_set(),
                "`{path}` disables --help, and the contract states every command carries it"
            );
            for sub in command.get_subcommands() {
                check(sub, &format!("{path} {}", sub.get_name()));
            }
        }
        let root = crate::Cli::command();
        check(&root, "superdev");
        assert!(
            !root.is_disable_version_flag_set(),
            "the root disables --version, and the contract states it carries it"
        );
    }
}
