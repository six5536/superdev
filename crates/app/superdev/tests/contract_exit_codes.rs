//! contract_exit_codes.rs — the exit codes the CLI contract declares, proved
//! by running the binary.
//!
//! No command-line framework knows what a command returns, so the drift test
//! in `src/contract.rs` cannot reach the `exit` maps. [ADR-036] says a facet
//! no introspection reports is bound by exercising the interface: each case
//! below runs the real binary and asserts the code, and asserts that the code
//! it expects is one the contract declares for that command — so the probe
//! and the contract cannot drift apart either.

use std::collections::BTreeMap;

use assert_cmd::Command;

/// The repository root, where `validate` finds the live knowledge.
const REPO_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");

/// The contract's declared exit codes, keyed by command path.
fn declared() -> BTreeMap<String, Vec<i64>> {
    let path: std::path::PathBuf = [
        REPO_ROOT,
        "knowledge/contracts/public/active/contract-002-cli-superdev.md",
    ]
    .iter()
    .collect();
    let text = std::fs::read_to_string(path).expect("the CLI contract is on file");
    let block = text
        .split("```yaml\n")
        .nth(1)
        .and_then(|rest| rest.split("\n```").next())
        .expect("the Commands section carries a yaml block");
    let raw: BTreeMap<String, serde_yaml_ng::Value> =
        serde_yaml_ng::from_str(block).expect("the definition block parses as yaml");
    raw.into_iter()
        .map(|(path, entry)| {
            let codes = entry
                .get("exit")
                .and_then(|v| v.as_mapping())
                .map(|m| m.keys().filter_map(serde_yaml_ng::Value::as_i64).collect())
                .unwrap_or_default();
            (path, codes)
        })
        .collect()
}

/// Run `args` and assert the binary exits `code`, which the contract must
/// declare for `command` — or be `2`, which every command returns on a usage
/// error, as the contract's Exit codes section states once.
fn probe(command: &str, args: &[&str], code: i64) {
    let declared = declared();
    let codes = declared
        .get(command)
        .unwrap_or_else(|| panic!("the contract declares no `{command}`"));
    assert!(
        code == 2 || codes.contains(&code),
        "`{command}` returns {code}, and its contract declares {codes:?}"
    );
    let out = Command::cargo_bin("superdev")
        .unwrap()
        .args(args)
        .current_dir(REPO_ROOT)
        .output()
        .expect("the binary runs");
    assert_eq!(
        out.status.code(),
        Some(i32::try_from(code).expect("an exit code fits")),
        "`superdev {}` exited {:?}, and its contract declares {code}\nstderr: {}",
        args.join(" "),
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Covers I035 criterion 5: the success codes the contract declares are the
/// codes the binary returns.
#[test]
fn the_declared_success_codes_are_what_the_binary_returns() {
    probe("superdev", &[], 0);
    probe("superdev", &["--version"], 0);
    probe("superdev completions", &["completions", "bash"], 0);
    probe("superdev man", &["man"], 0);
    probe("superdev template list", &["template", "list"], 0);
    probe("superdev run end", &["run", "end"], 0);
}

/// Covers I035 criterion 5: a usage error is `2` from every command, which is
/// the rule the contract states once rather than in each entry.
#[test]
fn a_usage_error_is_two_from_every_command() {
    probe("superdev", &["nonsense"], 2);
    probe("superdev completions", &["completions", "klingon"], 2);
    probe("superdev status", &["status", "--nonsense"], 2);
    probe("superdev template", &["template"], 2);
    probe("superdev run", &["run"], 2);
    probe("superdev sokf", &["sokf"], 2);
    probe("superdev hook", &["hook"], 2);
    probe("superdev mcp", &["mcp"], 2);
}

/// Covers I035 criterion 5: `validate` returns `0` when the knowledge it
/// reads has no errors, which is the code CI gates on.
#[test]
fn validate_returns_the_code_it_declares_for_a_clean_run() {
    probe("superdev validate", &["validate"], 0);
}
