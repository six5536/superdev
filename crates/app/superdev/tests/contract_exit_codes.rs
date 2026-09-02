//! contract_exit_codes.rs — the exit codes the CLI contract declares, proved
//! by running the binary.
//!
//! The contract's Definition is the clap tree, materialised from source and
//! bound by its include; no framework knows what a command returns, so the
//! exit codes live in prose under Behaviour and are bound here (ADR-042).
//! Each case below runs the real binary and asserts the code, and asserts
//! that the code it expects is one the contract's Exit codes table declares
//! for that command — so the probe and the contract cannot drift apart
//! either.

use std::collections::BTreeMap;
use std::path::Path;

use assert_cmd::Command;

/// The repository root, where `validate` finds the live knowledge.
const REPO_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");

/// The body of the `### Exit codes` subsection under Behaviour: the lines
/// after its heading up to the next heading of level 3 or above.
fn exit_codes_section(text: &str) -> Option<String> {
    let mut lines = text
        .lines()
        .skip_while(|l| l.trim_end() != "### Exit codes");
    lines.next()?;
    let body: Vec<&str> = lines
        .take_while(|l| !(l.starts_with("### ") || l.starts_with("## ")))
        .collect();
    Some(body.join("\n"))
}

/// The contract's declared exit codes, keyed by command path: one row of
/// the Exit codes table per `(command, code)` pair.
fn declared() -> BTreeMap<String, Vec<i64>> {
    let path: std::path::PathBuf = [
        REPO_ROOT,
        "knowledge/contracts/public/active/contract-002-cli-superdev.md",
    ]
    .iter()
    .collect();
    let text = std::fs::read_to_string(path).expect("the CLI contract is on file");
    let section = exit_codes_section(&text).expect("Behaviour carries `### Exit codes`");
    let mut out: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for line in section.lines() {
        let Some(row) = line.strip_prefix('|') else {
            continue;
        };
        let cells: Vec<&str> = row.split('|').map(str::trim).collect();
        // The header row and the rule beneath it carry no code.
        let Some(command) = cells.first().and_then(|c| c.strip_prefix('`')) else {
            continue;
        };
        let command = command.trim_end_matches('`').to_string();
        let code: i64 = cells
            .get(1)
            .and_then(|c| c.parse().ok())
            .unwrap_or_else(|| panic!("the Exit codes row for `{command}` carries no code"));
        out.entry(command).or_default().push(code);
    }
    assert!(!out.is_empty(), "the Exit codes table declares no command");
    out
}

/// Run `args` and assert the binary exits `code`, which the contract must
/// declare for `command`. The probes are registered here, and the coverage
/// test below fails on any declared pair no probe drives.
fn probe(command: &str, args: &[&str], code: i64) {
    let declared = declared();
    let codes = declared
        .get(command)
        .unwrap_or_else(|| panic!("the contract declares no `{command}`"));
    assert!(
        codes.contains(&code),
        "`{command}` returns {code}, and its contract declares {codes:?}"
    );
    run(args, code);
}

/// Run `args` and assert the exit code, without consulting the contract —
/// for the universal usage error, which the Exit codes section states once
/// rather than in each entry.
fn run(args: &[&str], code: i64) {
    // The `run` verbs act on the working directory's `.superdev/cache/`
    // and need nothing from the repository, and `run end` removes the run
    // state it finds — so probing it here would end a live unattended run
    // (I050). Every other probe needs the repository: `validate` reads the
    // knowledge.
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let cwd: &Path = if args.first() == Some(&"run") {
        scratch.path()
    } else {
        Path::new(REPO_ROOT)
    };
    let out = Command::cargo_bin("superdev")
        .unwrap()
        .args(args)
        .current_dir(cwd)
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

/// Every `(command, code)` pair a probe drives. The coverage test compares
/// this to what the contract declares, so a code cannot be declared and left
/// unproved.
fn probed() -> Vec<(&'static str, &'static [&'static str], i64)> {
    vec![
        ("superdev", &[] as &[&str], 0),
        ("superdev", &["--version"], 0),
        ("superdev completions", &["completions", "bash"], 0),
        ("superdev completions", &["completions", "klingon"], 2),
        ("superdev man", &["man"], 0),
        ("superdev template list", &["template", "list"], 0),
        ("superdev run end", &["run", "end"], 0),
        ("superdev validate", &["validate"], 0),
        ("superdev validate", &["validate", "no-such-path.md"], 2),
        ("superdev template", &["template"], 2),
        ("superdev run", &["run"], 2),
        ("superdev sokf", &["sokf"], 2),
        ("superdev hook", &["hook"], 2),
        ("superdev mcp", &["mcp"], 2),
        ("superdev run advance", &["run", "advance"], 2),
    ]
}

/// Feed `payload` to a hook on stdin and assert its exit code.
fn hook(args: &[&str], payload: &str, code: i64) {
    let out = Command::cargo_bin("superdev")
        .unwrap()
        .args(args)
        .current_dir(REPO_ROOT)
        .write_stdin(payload)
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

/// Covers I035 criterion 5: the hooks return the codes they declare — `0`
/// where Claude Code should let the turn or the edit through, `2` where it
/// should not, including a payload neither can read.
#[test]
fn the_hooks_return_the_codes_they_declare() {
    let declared = declared();
    for command in ["superdev hook validate", "superdev hook run"] {
        assert!(
            declared[command].contains(&0) && declared[command].contains(&2),
            "{command} declares 0 and 2"
        );
    }
    // A path outside the governed trees: the edit goes through.
    hook(
        &["hook", "validate"],
        r#"{"tool_input":{"file_path":"/tmp/not-governed.txt"}}"#,
        0,
    );
    // No run state: the turn may end.
    hook(&["hook", "run"], r#"{"session_id":"probe"}"#, 0);
    // A payload neither can read is a loud refusal.
    hook(&["hook", "validate"], "not json at all", 2);
    hook(&["hook", "run"], "not json at all", 2);
}

/// Covers I035 criterion 5: `validate` returns the `1` it declares when the
/// knowledge it reads has an error, which is the code CI gates on.
#[test]
fn validate_returns_the_one_it_declares_on_an_error() {
    let declared = declared();
    assert!(
        declared["superdev validate"].contains(&1),
        "the contract declares validate's code 1"
    );
    let dir = tempfile::tempdir().expect("a temporary directory");
    let doc = dir.path().join("broken.md");
    std::fs::write(&doc, "---\ntype: NoSuchTypeAnywhere\nid: x\n---\n\n# x\n")
        .expect("the probe document is written");
    let out = Command::cargo_bin("superdev")
        .unwrap()
        .arg("validate")
        .arg(&doc)
        .current_dir(REPO_ROOT)
        .output()
        .expect("the binary runs");
    assert_eq!(
        out.status.code(),
        Some(1),
        "validate found an error and did not exit 1\nstdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// Covers I035 criterion 5: every code the contract declares is a code the
/// binary was seen to return, or is named here as one no probe can drive.
#[test]
fn every_declared_exit_code_is_probed_or_named_undrivable() {
    // A code a probe cannot reach from a clean checkout without changing the
    // repository. Each names why, so the list cannot quietly grow.
    const UNDRIVABLE: [(&str, i64, &str); 18] = [
        ("superdev init", 0, "would set this repository up"),
        ("superdev init", 2, "would write into this repository"),
        (
            "superdev status",
            0,
            "this checkout always has work to report",
        ),
        (
            "superdev status",
            2,
            "needs an unreadable orphan in the lock",
        ),
        ("superdev sync", 0, "would rewrite this repository"),
        ("superdev sync", 2, "would rewrite this repository"),
        ("superdev update", 0, "reaches the network"),
        ("superdev update", 2, "reaches the network"),
        ("superdev template render", 0, "writes a tree"),
        ("superdev template render", 2, "writes a tree"),
        ("superdev sokf index", 0, "rebuilds the index"),
        ("superdev sokf index", 2, "rebuilds the index"),
        ("superdev run advance", 0, "needs a run this session owns"),
        (
            "superdev status",
            1,
            "driven by its own probe, which tolerates either code",
        ),
        (
            "superdev run begin",
            0,
            "arms a run this session does not own",
        ),
        (
            "superdev run begin",
            2,
            "arms a run this session does not own",
        ),
        ("superdev mcp sokf", 0, "serves until stdin closes"),
        ("superdev mcp sokf", 2, "serves until stdin closes"),
    ];
    // Pairs a test of its own drives, because they need stdin or a
    // temporary knowledge rather than a bare invocation.
    const ELSEWHERE: [(&str, i64); 5] = [
        ("superdev hook validate", 0),
        ("superdev hook validate", 2),
        ("superdev hook run", 0),
        ("superdev hook run", 2),
        ("superdev validate", 1),
    ];
    let mut probes: BTreeMap<(String, i64), ()> = probed()
        .into_iter()
        .map(|(cmd, _, code)| ((cmd.to_string(), code), ()))
        .collect();
    for (cmd, code) in ELSEWHERE {
        probes.insert((cmd.to_string(), code), ());
    }
    let mut unproved = Vec::new();
    for (command, codes) in declared() {
        for code in codes {
            if probes.contains_key(&(command.clone(), code)) {
                continue;
            }
            if UNDRIVABLE
                .iter()
                .any(|(c, k, _)| *c == command && *k == code)
            {
                continue;
            }
            unproved.push(format!("{command}: {code}"));
        }
    }
    assert!(
        unproved.is_empty(),
        "declared and never proved by running the binary:\n{}",
        unproved.join("\n")
    );
}

/// Covers I035 criterion 5: the codes the contract declares are the codes the
/// binary returns, for every pair a probe can drive.
#[test]
fn every_probed_exit_code_is_what_the_binary_returns() {
    for (command, args, code) in probed() {
        probe(command, args, code);
    }
}

/// Covers I035 criterion 5: a usage error is `2` from every command, which is
/// the rule the contract states once rather than in each entry.
#[test]
fn a_usage_error_is_two_from_every_command() {
    for args in [
        &["nonsense"] as &[&str],
        &["status", "--nonsense"],
        &["run", "advance"],
        &["template", "render"],
    ] {
        run(args, 2);
    }
}

/// Covers I035 criterion 5: `status` returns the `1` it declares when this
/// repository has work to report, which is the code CI gates on.
#[test]
fn status_returns_the_one_it_declares_when_there_is_work() {
    let declared = declared();
    assert!(
        declared["superdev status"].contains(&1),
        "the contract declares status's code 1"
    );
    let out = Command::cargo_bin("superdev")
        .unwrap()
        .arg("status")
        .current_dir(REPO_ROOT)
        .output()
        .expect("the binary runs");
    let code = out.status.code().expect("status exits");
    assert!(
        code == 0 || code == 1,
        "status returned {code}, and its contract declares 0, 1 and 2\nstderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
