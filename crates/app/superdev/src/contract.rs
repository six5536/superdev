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

use std::collections::{BTreeMap, BTreeSet};

use clap::CommandFactory;

/// One command's surface, as the contract declares it and as clap builds it.
#[derive(Debug, Default, PartialEq, Eq)]
struct Surface {
    /// What the command says it does, as the binary prints it.
    about: String,
    /// Every name the command answers to beyond its own.
    aliases: Vec<String>,
    /// Positional arguments, in order.
    args: Vec<Positional>,
    /// Long flag to what it takes.
    flags: BTreeMap<String, Flag>,
}

/// One positional argument: its usage name, whether it must be given, and
/// whether it takes more than one value.
#[derive(Debug, Default, PartialEq, Eq)]
struct Positional {
    name: String,
    required: bool,
    multiple: bool,
    /// The closed set of values it accepts, when it has one.
    values: Vec<String>,
}

/// One flag: its value type — `bool` for a switch, else the value name — and
/// its short form where it has one.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Flag {
    r#type: String,
    short: Option<String>,
}

/// `-h` and `--help` sit on every command and `-V` on the root: the framework
/// adds them, the contract states the rule once rather than per entry, and
/// this test asserts the rule instead of comparing the repetition.
const FRAMEWORK_FLAGS: [&str; 2] = ["help", "version"];

/// Every command this binary offers, keyed by the path a user types.
fn implemented() -> BTreeMap<String, Surface> {
    let mut out = BTreeMap::new();
    walk(
        &crate::Cli::command(),
        "superdev",
        &BTreeMap::new(),
        &mut out,
    );
    out
}

/// One command and its subcommands, in the shape the contract declares.
///
/// A global flag is declared on the command that carries it and on every
/// command it reaches, because a caller may pass it to either.
fn walk(
    command: &clap::Command,
    path: &str,
    inherited: &BTreeMap<String, Flag>,
    out: &mut BTreeMap<String, Surface>,
) {
    let mut surface = Surface {
        about: command
            .get_about()
            .map(ToString::to_string)
            .unwrap_or_default(),
        aliases: command.get_all_aliases().map(ToString::to_string).collect(),
        ..Surface::default()
    };
    surface.flags.clone_from(inherited);
    let mut globals = inherited.clone();
    for arg in command.get_arguments() {
        if FRAMEWORK_FLAGS.contains(&arg.get_id().as_str()) {
            continue;
        }
        let values: Vec<String> = arg
            .get_possible_values()
            .iter()
            .map(|v| v.get_name().to_string())
            .collect();
        if arg.is_positional() {
            surface.args.push(Positional {
                name: arg
                    .get_value_names()
                    .and_then(|names| names.first().map(ToString::to_string))
                    .unwrap_or_else(|| arg.get_id().to_string()),
                required: arg.is_required_set(),
                multiple: matches!(arg.get_action(), clap::ArgAction::Append)
                    || arg.get_num_args().is_some_and(|n| n.max_values() > 1),
                values,
            });
            continue;
        }
        let Some(long) = arg.get_long() else {
            // A short-only flag is surface too, named by its short form.
            if let Some(short) = arg.get_short() {
                surface.flags.insert(
                    format!("-{short}"),
                    Flag {
                        r#type: "bool".to_string(),
                        short: Some(short.to_string()),
                    },
                );
            }
            continue;
        };
        // A switch takes no value, whatever name the framework gives its id.
        let switch = matches!(
            arg.get_action(),
            clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
        );
        let flag = Flag {
            r#type: if switch {
                "bool".to_string()
            } else {
                arg.get_value_names()
                    .and_then(|names| names.first().map(ToString::to_string))
                    .unwrap_or_else(|| arg.get_id().to_string())
            },
            short: arg.get_short().map(|s| s.to_string()),
        };
        if arg.is_global_set() {
            globals.insert(format!("--{long}"), flag.clone());
        }
        surface.flags.insert(format!("--{long}"), flag);
    }
    out.insert(path.to_string(), surface);
    for sub in command.get_subcommands() {
        // `help` is the framework's own command, like its flags.
        if sub.get_name() == "help" {
            continue;
        }
        walk(sub, &format!("{path} {}", sub.get_name()), &globals, out);
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
            let values_for = |arg: &str| -> Vec<String> {
                entry
                    .get("arg-values")
                    .and_then(|v| v.get(arg))
                    .and_then(|v| v.as_sequence())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|v| v.as_str().map(ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let args = entry
                .get("args")
                .and_then(|v| v.as_sequence())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|v| {
                            let name = v.as_str()?.to_string();
                            let values = values_for(&name);
                            Some(Positional {
                                required: entry
                                    .get("arg-required")
                                    .and_then(|r| r.get(name.as_str()))
                                    .and_then(serde_yaml_ng::Value::as_bool)
                                    .unwrap_or(false),
                                multiple: entry
                                    .get("arg-multiple")
                                    .and_then(|r| r.get(name.as_str()))
                                    .and_then(serde_yaml_ng::Value::as_bool)
                                    .unwrap_or(false),
                                name,
                                values,
                            })
                        })
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
                            Some((
                                name,
                                Flag {
                                    r#type: v
                                        .get("type")
                                        .and_then(|t| t.as_str())
                                        .unwrap_or("bool")
                                        .to_string(),
                                    short: v
                                        .get("short")
                                        .and_then(|s| s.as_str())
                                        .map(ToString::to_string),
                                },
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let surface = Surface {
                about: entry
                    .get("about")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                aliases: entry
                    .get("aliases")
                    .and_then(|v| v.as_sequence())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|v| v.as_str().map(ToString::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
                args,
                flags,
            };
            (path, surface)
        })
        .collect()
}

/// Every command an entry marks `pending`, naming the slice that will build
/// it. A pending element is a promise the contract makes ahead of the code
/// (ADR-038); acceptance refuses a contract still carrying one.
fn pending_commands() -> BTreeSet<String> {
    let text = std::fs::read_to_string(contract_path()).expect("the CLI contract is on file");
    let block = fenced_block(&text, "yaml").expect("the Commands section carries a yaml block");
    let raw: BTreeMap<String, serde_yaml_ng::Value> =
        serde_yaml_ng::from_str(&block).expect("the definition block parses as yaml");
    raw.into_iter()
        .filter(|(_, entry)| entry.get("pending").is_some())
        .map(|(path, _)| path)
        .collect()
}

/// Every `exit` key an entry declares, so a code can be read as a code.
fn declared_exit_keys() -> BTreeMap<String, Vec<serde_yaml_ng::Value>> {
    let text = std::fs::read_to_string(contract_path()).expect("the CLI contract is on file");
    let block = fenced_block(&text, "yaml").expect("the Commands section carries a yaml block");
    let raw: BTreeMap<String, serde_yaml_ng::Value> =
        serde_yaml_ng::from_str(&block).expect("the definition block parses as yaml");
    raw.into_iter()
        .map(|(path, entry)| {
            let keys = entry
                .get("exit")
                .and_then(|v| v.as_mapping())
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            (path, keys)
        })
        .collect()
}

/// Every key each entry's `flags` map gives one flag, so a sentence broken
/// into keys by a stray comma is caught where it happens.
fn declared_flag_keys() -> Vec<(String, String, Vec<String>)> {
    let text = std::fs::read_to_string(contract_path()).expect("the CLI contract is on file");
    let block = fenced_block(&text, "yaml").expect("the Commands section carries a yaml block");
    let raw: BTreeMap<String, serde_yaml_ng::Value> =
        serde_yaml_ng::from_str(&block).expect("the definition block parses as yaml");
    let mut out = Vec::new();
    for (path, entry) in raw {
        let Some(flags) = entry.get("flags").and_then(|v| v.as_mapping()) else {
            continue;
        };
        for (name, spec) in flags {
            let keys = spec.as_mapping().map_or_else(Vec::new, |m| {
                m.keys()
                    .filter_map(|k| k.as_str().map(ToString::to_string))
                    .collect()
            });
            out.push((
                path.clone(),
                name.as_str().unwrap_or("(unnamed)").to_string(),
                keys,
            ));
        }
    }
    out
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
                println!("{path}:");
                println!("  about: {}", surface.about);
                println!("  args: {:?}", surface.args);
                println!("  aliases: {:?}", surface.aliases);
                for (flag, spec) in &surface.flags {
                    println!("    {flag}: {spec:?}");
                }
            }
        }
        let declared = declared();

        // A pending element is bound in reverse (ADR-038): the contract
        // promises it, the binary must not have it yet, and the marker
        // fails the moment the work lands.
        let pending = pending_commands();
        let built_pending: Vec<&String> =
            pending.iter().filter(|p| built.contains_key(*p)).collect();
        assert!(
            built_pending.is_empty(),
            "DONE — these are marked pending and the binary now offers them; \
             drop the marker: {built_pending:?}"
        );
        let declared: BTreeMap<String, Surface> = declared
            .into_iter()
            .filter(|(path, _)| !pending.contains(path))
            .collect();

        // The two directions are different things (ADR-038): what the
        // contract has yet to promise is a defect, what the binary has yet
        // to build is a promise still outstanding.
        let undeclared: Vec<&String> = built
            .keys()
            .filter(|k| !declared.contains_key(*k))
            .collect();
        assert!(
            undeclared.is_empty(),
            "DEFECT — the binary offers commands its contract does not declare: {undeclared:?}"
        );
        let unbuilt: Vec<&String> = declared
            .keys()
            .filter(|k| !built.contains_key(*k))
            .collect();
        assert!(
            unbuilt.is_empty(),
            "PENDING — the contract promises commands the binary does not offer yet: {unbuilt:?}"
        );
        for (path, want) in &declared {
            assert_eq!(
                &built[path], want,
                "DRIFT — `{path}` differs between the binary and its contract"
            );
        }
    }

    /// Covers I035 criterion 2: every `exit` key is an integer, so a code
    /// reads as a code. A flow mapping makes every comma a new entry, which
    /// silently turns half a sentence into an exit code.
    #[test]
    fn every_declared_exit_key_is_a_code() {
        let mut wrong = Vec::new();
        for (path, keys) in declared_exit_keys() {
            assert!(!keys.is_empty(), "`{path}` declares no exit codes");
            for key in keys {
                if key.as_i64().is_none() {
                    wrong.push(format!("{path}: {key:?}"));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "an exit key is not a code — a flow mapping cut a sentence in two:\n{}",
            wrong.join("\n")
        );
    }

    /// Covers I035 criterion 2: a flag is described by `type`, `about` and
    /// `short` and nothing else, so a comma inside an `about` cannot become
    /// a key of its own.
    #[test]
    fn every_declared_flag_carries_only_its_own_keys() {
        let allowed = ["type", "about", "short"];
        let mut wrong = Vec::new();
        for (path, flag, keys) in declared_flag_keys() {
            for key in keys {
                if !allowed.contains(&key.as_str()) {
                    wrong.push(format!("{path} {flag}: `{key}`"));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "a flag carries a key that is not one of {allowed:?} — a flow mapping cut a \
             sentence in two:\n{}",
            wrong.join("\n")
        );
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
