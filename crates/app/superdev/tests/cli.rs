//! End-to-end tests for the skeleton CLI: the real binary, real exit codes.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc;
use std::time::Duration;

use assert_cmd::Command;

/// The repository root: where `validate` finds the live SOKF knowledge.
const REPO_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../..");

fn superdev() -> Command {
    Command::cargo_bin("superdev").unwrap()
}

#[test]
fn version_reports_name_and_semver() {
    let out = superdev().arg("--version").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.trim().starts_with("superdev "),
        "unexpected --version output: {stdout}"
    );
    assert_eq!(
        stdout.trim(),
        format!("superdev {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn bare_invocation_prints_help_and_exits_zero() {
    let out = superdev().assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("Usage:"), "no usage in: {stdout}");
}

#[test]
fn help_hides_the_man_subcommand() {
    let out = superdev().arg("--help").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("completions"));
    assert!(
        !stdout.contains("\n  man"),
        "man should be hidden: {stdout}"
    );
}

#[test]
fn completions_render_for_every_supported_shell() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let out = superdev().args(["completions", shell]).assert().success();
        assert!(
            !out.get_output().stdout.is_empty(),
            "empty completion script for {shell}"
        );
    }
}

#[test]
fn man_emits_roff() {
    let out = superdev().arg("man").assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains(".TH"),
        "no .TH header in man output: {stdout}"
    );
}

#[test]
fn unknown_flag_is_a_usage_error() {
    superdev().arg("--definitely-not-a-flag").assert().code(2);
}

#[test]
fn unknown_shell_is_a_usage_error() {
    superdev()
        .args(["completions", "notashell"])
        .assert()
        .code(2);
}

#[test]
fn validate_passes_the_live_repository() {
    superdev()
        // workspace root, so a bare run resolves <cwd>/knowledge and the
        // grammar's roots
        .current_dir(REPO_ROOT)
        .args(["validate"])
        .assert()
        .success();
}

/// A knowledge carrying warnings and no error tells the two listings apart:
/// a bare run counts them, `--warnings` lists them, and the summary line and
/// the exit code are the same either way (ADR-040).
///
/// The fixture is what makes it deterministic. Run against this repository,
/// the test would rest on it still carrying a warning of its own — and the
/// five it carries are exactly the kind someone eventually fixes.
#[test]
fn a_bare_run_counts_the_warnings_and_the_flag_lists_them() {
    let repo = warning_only_repo();
    let run = |args: &[&str]| {
        let out = superdev()
            .current_dir(repo.path())
            .args(args)
            .assert()
            .success();
        String::from_utf8_lossy(&out.get_output().stdout).into_owned()
    };
    let counted = run(&["validate"]);
    let listed = run(&["validate", "--warnings"]);

    assert!(
        !counted.contains("[warning]"),
        "a bare run lists no warning: {counted}"
    );
    assert!(
        listed.contains("[warning]"),
        "--warnings lists them: {listed}"
    );
    let summary = |text: &str| {
        text.lines()
            .next_back()
            .expect("a report ends with its summary")
            .to_string()
    };
    assert_eq!(summary(&counted), summary(&listed));
    let line = summary(&counted);
    assert!(line.starts_with("PASS (0 error(s), "), "{line}");
    let count: u32 = line
        .rsplit_once(", ")
        .and_then(|(_, tail)| tail.split_once(' '))
        .expect("the summary names a warning count")
        .0
        .parse()
        .expect("the warning count is a number");
    assert!(
        count > 0,
        "the warnings it did not list are counted: {line}"
    );
}

/// The `aokf` verb group is gone, alias and all. It was kept while the hook
/// marker and the lock key carried the old spelling; both now say `sokf`, so
/// there is nothing left for the alias to be compatible with.
#[test]
fn the_aokf_verb_group_is_gone() {
    for args in [
        vec!["aokf", "validate"],
        vec!["aokf", "index"],
        vec!["aokf", "hook", "validate"],
        vec!["mcp", "aokf"],
    ] {
        superdev()
            .current_dir(REPO_ROOT)
            .args(&args)
            .assert()
            .failure()
            .code(2);
    }
    // What replaced them resolves.
    superdev()
        .current_dir(REPO_ROOT)
        .args(["sokf", "index", "--help"])
        .assert()
        .success();
    superdev()
        .current_dir(REPO_ROOT)
        .args(["hook", "validate", "--help"])
        .assert()
        .success();
}

#[test]
fn validate_fails_broken_knowledge_with_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("kb")).unwrap();
    std::fs::write(
        dir.path().join("kb/a.md"),
        "---\ntype: T\nid: dup\n---\nx\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("kb/b.md"),
        "---\ntype: T\nid: dup\n---\nx\n",
    )
    .unwrap();
    // The knowledge directory is a flag: a positional path is the scope of
    // the run, and this repository keeps its knowledge somewhere other than
    // `knowledge`.
    superdev()
        .current_dir(dir.path())
        .args(["validate", "--knowledge", "kb"])
        .assert()
        .code(1);
}

/// Covers I012 criteria 1 and 2 (ADR-039): a link naming a file that is not
/// there fails the run on its own. It was a warning, so the run passed and
/// the finding went unread — 39 of them did, until someone happened to look.
#[test]
fn a_broken_link_alone_fails_the_run() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("kb")).unwrap();
    std::fs::write(
        dir.path().join("kb/a.md"),
        "---\ntype: T\nid: a\n---\n\nA [dangling](does-not-exist.md) link.\n",
    )
    .unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .args(["validate", "--knowledge", "kb"])
        .assert()
        .code(1);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("broken body link: does-not-exist.md"),
        "the finding names the target: {stdout}"
    );
}

/// A grammar error fails the run on its own, with no knowledge in the picture.
/// The temporary repository carries no `.agents/sokf/grammar.yaml` either,
/// so this is also the embedded grammar doing the checking (FR-11).
#[test]
fn validate_fails_a_broken_skill_with_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    let skill = dir.path().join(".claude/skills/broken");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "no frontmatter, no elements\n").unwrap();
    superdev()
        .current_dir(dir.path())
        .args(["validate", ".claude/skills"])
        .assert()
        .code(1);
}

/// An unreadable `PATH` fails naming itself (contract-002; I019 criterion 5).
#[test]
fn validate_fails_an_unreadable_path_naming_it() {
    let out = superdev()
        .current_dir(REPO_ROOT)
        .args(["validate", "no/such/file.md"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    // The path as it was given, not the absolute platform-separated spelling
    // the reader failed on: a substring check passes on the second by
    // accident everywhere `/` is the separator, and fails on Windows (I041).
    assert!(
        stderr.starts_with("error: no/such/file.md:"),
        "the path is not named as it was given: {stderr}"
    );
}

/// A named document is checked as what it is: the concept passes with no
/// skill-grammar finding, matching the bare run's verdict for that file
/// (I019 criterion 1, ADR-026).
#[test]
fn validate_checks_a_named_document_as_the_bare_run_does() {
    let out = superdev()
        .current_dir(REPO_ROOT)
        .args(["validate", "knowledge/architecture.md"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("PASS (0 error(s)"), "{stdout}");
    assert!(!stdout.contains("missing <"), "{stdout}");
}

/// For a concept, README.md and a skill alike, a named run reports exactly
/// the findings the bare run reports for that file (I019 criteria 1 and 2,
/// ADR-026).
///
/// Both runs ask for the warnings, because the only one of the three files
/// that carries a finding today carries warnings. Neither run asserts a
/// finding exists: this test binds bare-against-named parity, which holds
/// whether or not the tree has anything to report.
#[test]
fn a_named_runs_findings_equal_the_bare_runs_for_that_file() {
    let out = superdev()
        .current_dir(REPO_ROOT)
        .args(["validate", "--json", "--warnings"])
        .assert()
        .success();
    let bare: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    for file in [
        "knowledge/architecture.md",
        "README.md",
        ".claude/skills/handoff/SKILL.md",
    ] {
        let bare_for_file: Vec<&serde_json::Value> = bare["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["file"] == file)
            .collect();
        let out = superdev()
            .current_dir(REPO_ROOT)
            .args(["validate", "--json", "--warnings", file])
            .assert()
            .success();
        let named: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
        let named_findings: Vec<&serde_json::Value> =
            named["findings"].as_array().unwrap().iter().collect();
        assert_eq!(named_findings, bare_for_file, "{file}");
    }
}

#[test]
fn validate_json_is_machine_readable() {
    let out = superdev()
        .current_dir(REPO_ROOT)
        .args(["validate", "--json", "--warnings"])
        .assert()
        .success();
    let report: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    // One findings array over both checks. The live tree carries the five
    // portability warnings and no errors.
    let findings = report["findings"].as_array().unwrap();
    assert!(
        findings.iter().all(|f| f["severity"] == "warning"),
        "{report}"
    );
    assert_eq!(report["passed"], serde_json::json!(true));
    // The knowledge path is the CLI's to add: core omits it.
    let knowledge = report["knowledge"].as_str().unwrap();
    assert!(
        knowledge.ends_with("knowledge"),
        "unexpected knowledge path: {knowledge}"
    );
}

/// `--json` reports what the text output reports: both counts always, and the
/// findings the text run listed (ADR-040). A consumer that read the counts off
/// `findings` would now undercount, which is why the counts are there.
#[test]
fn validate_json_states_both_counts_and_lists_what_the_text_run_listed() {
    let repo = warning_only_repo();
    let report = |args: &[&str]| -> serde_json::Value {
        let out = superdev()
            .current_dir(repo.path())
            .args(args)
            .assert()
            .success();
        serde_json::from_slice(&out.get_output().stdout).unwrap()
    };
    let counted = report(&["validate", "--json"]);
    let listed = report(&["validate", "--json", "--warnings"]);

    for r in [&counted, &listed] {
        assert_eq!(r["errors"], serde_json::json!(0), "{r}");
        assert_eq!(
            r["warnings"],
            serde_json::json!(1),
            "the fixture's one warning is counted either way: {r}"
        );
    }
    assert_eq!(counted["warnings"], listed["warnings"]);
    assert!(
        counted["findings"].as_array().unwrap().is_empty(),
        "a default run lists no warning: {counted}"
    );
    assert_eq!(
        listed["findings"].as_array().unwrap().len() as u64,
        listed["warnings"].as_u64().unwrap(),
        "--warnings lists every one it counted: {listed}"
    );
}

/// The grammar is the only statement of the format, and `--doc` prints it.
#[test]
fn validate_doc_renders_the_grammar() {
    let out = superdev()
        .current_dir(REPO_ROOT)
        .args(["validate", "--doc"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.starts_with("# superdev grammar "), "{stdout}");
    assert!(stdout.contains("## Element order"), "{stdout}");
}

#[test]
fn sokf_index_rebuilds_and_reports_lexical_only() {
    let dir = tempfile::tempdir().unwrap();
    write_fixture_bundle(dir.path());
    // A manifest with no `[knowledge.embeddings]` table: the embedder comes
    // from the local default, which the blocked cache stops loading.
    std::fs::create_dir(dir.path().join(".superdev")).unwrap();
    std::fs::write(
        dir.path().join(".superdev/config.toml"),
        "blueprint = \"0.1.0\"\n",
    )
    .unwrap();
    // A file that cannot be parsed leaves the bundle silently, so `index` says so.
    std::fs::write(dir.path().join("knowledge/bad.md"), "no frontmatter here\n").unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", blocked_model_cache(dir.path()))
        .args(["sokf", "index"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("1 concept"), "no concept count: {stdout}");
    assert!(stdout.contains("lexical"), "no lexical-only note: {stdout}");
    assert!(
        stdout.contains("skipped 1"),
        "no broken-file note: {stdout}"
    );
    assert!(dir.path().join(".superdev/cache/sokf-index").is_dir());
}

#[test]
fn mcp_without_knowledge_fails_at_startup() {
    let dir = tempfile::tempdir().unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .args(["mcp", "sokf"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("no SOKF knowledge"), "unexpected: {stderr}");
}

#[test]
fn mcp_with_an_unusable_index_dir_fails_at_startup() {
    let dir = tempfile::tempdir().unwrap();
    write_fixture_bundle(dir.path());
    // A file where the index directory belongs: the startup sync cannot write it.
    std::fs::create_dir_all(dir.path().join(".superdev/cache")).unwrap();
    std::fs::write(dir.path().join(".superdev/cache/sokf-index"), "").unwrap();
    superdev()
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", blocked_model_cache(dir.path()))
        .args(["mcp", "sokf"])
        .assert()
        .code(2);
}

/// One `initialize`, the `initialized` notification, and one tool call, as raw
/// JSON-RPC — a smoke test needs no client library.
const MCP_REQUESTS: &str = concat!(
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"cli-test","version":"0"}}}"#,
    "\n",
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    "\n",
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sokf_overview","arguments":{}}}"#,
    "\n",
);

#[test]
fn mcp_server_initialises_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    write_fixture_bundle(dir.path());
    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin("superdev"))
        .args(["mcp", "sokf"])
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", blocked_model_cache(dir.path()))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(MCP_REQUESTS.as_bytes()).unwrap();
    stdin.flush().unwrap();

    // Read on a second thread: a server that never answers must fail the test,
    // not hang the run.
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                return;
            }
        }
    });
    let mut replies: Vec<String> = Vec::new();
    while !replies.iter().any(|line| line.contains("\"id\":2")) {
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(line) => replies.push(line),
            Err(e) => {
                let _ = child.kill();
                panic!("no reply to the tool call ({e}): {replies:?}");
            }
        }
    }
    // Closing stdin is how a client disconnects; the server must exit cleanly.
    drop(stdin);
    let status = child.wait().unwrap();
    assert!(status.success(), "server exited {status}");
    let replies = replies.join("\n");
    assert!(replies.contains("\"result\""), "no result in: {replies}");
    assert!(
        replies.contains("fixture-knowledge"),
        "no knowledge name in: {replies}"
    );
}

/// A one-concept bundle under `<dir>/knowledge`.
fn write_fixture_bundle(dir: &Path) {
    let bundle = dir.join("knowledge");
    std::fs::create_dir(&bundle).unwrap();
    std::fs::write(
        bundle.join("manifest.sokf.yaml"),
        "sokf: \"0.1\"\nname: fixture-knowledge\n",
    )
    .unwrap();
    std::fs::write(
        bundle.join("module.md"),
        "---\ntype: Module\nid: module-a\ntitle: Module A\n---\n\n# Role\n\nIt plans.\n",
    )
    .unwrap();
}

/// A cache root that is a regular file, so the local embedding model cannot be
/// downloaded into it: these tests stay offline and lexical-only.
fn blocked_model_cache(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("no-model-cache");
    std::fs::write(&path, "").unwrap();
    path
}

/// A repo with a `knowledge/` bundle: valid (level-2 clean) or broken.
fn hook_repo(valid: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let k = dir.path().join("knowledge");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::write(
        k.join("manifest.sokf.yaml"),
        "sokf: \"0.1\"\nname: fixture\n",
    )
    .unwrap();
    let concept = if valid {
        "---\ntype: Note\nid: alpha\n---\n\nBody.\n"
    } else {
        "---\nid: alpha\n---\n\nMissing type.\n"
    };
    std::fs::write(k.join("alpha.md"), concept).unwrap();
    dir
}

fn hook_payload(dir: &Path, rel: &str) -> String {
    serde_json::json!({
        "tool_input": { "file_path": dir.join(rel) }
    })
    .to_string()
}

#[test]
fn hook_validate_blocks_an_edit_that_broke_the_bundle() {
    let repo = hook_repo(false);
    let out = superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains("superdev validation failed after editing"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("alpha.md"), "stderr: {stderr}");
}

#[test]
fn hook_validate_passes_a_clean_bundle() {
    let repo = hook_repo(true);
    superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(0);
}

/// A knowledge carrying one error — a concept with no `type` — and one
/// warning, a non-core `rel` read as `relates-to`.
fn mixed_severity_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let k = dir.path().join("knowledge");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::write(
        k.join("manifest.sokf.yaml"),
        "sokf: \"0.1\"\nname: fixture\n",
    )
    .unwrap();
    std::fs::write(k.join("alpha.md"), "---\nid: alpha\n---\n\nMissing type.\n").unwrap();
    std::fs::write(
        k.join("beta.md"),
        "---\ntype: Note\nid: beta\nlinks:\n  - rel: made-up\n    to: gamma\n---\n\n\
         Beta names [gamma][sokf:gamma].\n\n<!-- sokf:links -->\n[sokf:gamma]: /knowledge/gamma.md\n",
    )
    .unwrap();
    std::fs::write(
        k.join("gamma.md"),
        "---\ntype: Note\nid: gamma\n---\n\nBody.\n",
    )
    .unwrap();
    dir
}

/// The PostToolUse hook defaults like a bare `validate`: it names the error,
/// counts the warning without naming it, and blocks as it did (ADR-040).
#[test]
fn hook_validate_counts_the_warnings_it_does_not_list() {
    let repo = mixed_severity_repo();
    let out = superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("[error]"), "the error is named: {stderr}");
    assert!(
        !stderr.contains("[warning]"),
        "no warning is listed: {stderr}"
    );
    assert!(
        stderr.contains("FAIL (1 error(s), 1 warning(s))"),
        "both counts stand: {stderr}"
    );
}

/// A knowledge whose only fault is one warning: a non-core `rel`, read as
/// `relates-to`. Deterministic, so a test about the two listings does not
/// rest on this repository still carrying a warning of its own.
fn warning_only_repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let k = repo.path().join("knowledge");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::write(
        k.join("manifest.sokf.yaml"),
        "sokf: \"0.1\"\nname: fixture\n",
    )
    .unwrap();
    std::fs::write(
        k.join("beta.md"),
        "---\ntype: Note\nid: beta\nlinks:\n  - rel: made-up\n    to: gamma\n---\n\n\
         Beta names [gamma][sokf:gamma].\n\n<!-- sokf:links -->\n[sokf:gamma]: /knowledge/gamma.md\n",
    )
    .unwrap();
    std::fs::write(
        k.join("gamma.md"),
        "---\ntype: Note\nid: gamma\n---\n\nBody.\n",
    )
    .unwrap();
    repo
}

/// A knowledge whose only fault is a warning leaves the hook at `0`, as it
/// always has: the default changes what is shown and never what is decided.
#[test]
fn hook_validate_still_passes_a_knowledge_whose_only_fault_is_a_warning() {
    let repo = warning_only_repo();
    superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/beta.md"))
        .assert()
        .code(0);
}

/// A knowledge whose only fault is a link the rest of the tree has not caught
/// up with. `kind` picks which of the two: a body link, or an index entry.
fn forward_reference_repo(kind: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let k = dir.path().join("knowledge");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::write(
        k.join("manifest.sokf.yaml"),
        "sokf: \"0.1\"\nname: fixture\n",
    )
    .unwrap();
    std::fs::write(
        k.join("alpha.md"),
        "---\ntype: Note\nid: alpha\n---\n\nBody.\n",
    )
    .unwrap();
    match kind {
        "link" => std::fs::write(
            k.join("beta.md"),
            "---\ntype: Note\nid: beta\n---\n\nA [later](not-yet.md) file.\n",
        )
        .unwrap(),
        _ => std::fs::write(k.join("index.md"), "# Index\n\n* [Later](not-yet.md)\n").unwrap(),
    }
    dir
}

/// Covers plan-022 slice 3 (ADR-039): the edit-time hook does not block on a
/// finding only the whole tree settles. It is handed one file and cannot see
/// whether the target arrives in the next edit; `hook run` holds the turn on
/// these instead, so they are not ignored.
#[test]
fn hook_validate_does_not_block_a_forward_reference() {
    for (kind, edited) in [
        ("link", "knowledge/beta.md"),
        ("index", "knowledge/index.md"),
    ] {
        let repo = forward_reference_repo(kind);
        let out = superdev()
            .args(["hook", "validate"])
            .env("CLAUDE_PROJECT_DIR", repo.path())
            .write_stdin(hook_payload(repo.path(), edited))
            .assert()
            .code(0);
        let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
        assert!(
            stderr.contains("the turn will not end while they stand"),
            "{kind}: the finding was hidden rather than reported: {stderr}"
        );
    }
}

/// Covers plan-022 slice 3: the hook was scoped, not disarmed. A fault the
/// edited file settles on its own still blocks, even beside a forward
/// reference.
#[test]
fn hook_validate_still_blocks_what_one_file_settles() {
    let repo = forward_reference_repo("link");
    std::fs::write(
        repo.path().join("knowledge/gamma.md"),
        "---\nid: gamma\n---\n\nMissing type.\n",
    )
    .unwrap();
    superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/gamma.md"))
        .assert()
        .code(2);
}

#[test]
fn hook_run_continues_an_armed_run_naming_next() {
    let repo = tempfile::tempdir().unwrap();
    let cache = repo.path().join(".superdev/cache");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::write(
        cache.join("run.toml"),
        "session_id = \"s1\"\nnext = \"build slice 2\"\ncontinues = 0\nstarted = \"2026-08-31T00:00:00Z\"\npid = 1\n",
    )
    .unwrap();
    let out = superdev()
        .args(["hook", "run"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(r#"{"session_id":"s1","hook_event_name":"Stop"}"#)
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("build slice 2"), "stderr: {stderr}");
    assert!(stderr.contains("superdev run advance"), "stderr: {stderr}");
}

/// The Stop hook defaults like a bare `validate` too: it holds the turn on
/// the error, names it, and states the warning count without naming the
/// warning (ADR-040).
#[test]
fn hook_run_counts_the_warnings_it_does_not_list() {
    let repo = mixed_severity_repo();
    let out = superdev()
        .args(["hook", "run"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(r#"{"session_id":"s1","hook_event_name":"Stop"}"#)
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains("the knowledge has findings"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("[error]"), "the error is named: {stderr}");
    assert!(
        !stderr.contains("[warning]"),
        "no warning is listed: {stderr}"
    );
    assert!(
        stderr.contains("FAIL (1 error(s), 1 warning(s))"),
        "both counts stand: {stderr}"
    );
}

#[test]
fn hook_run_is_invisible_without_a_run() {
    let repo = tempfile::tempdir().unwrap();
    superdev()
        .args(["hook", "run"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(r#"{"session_id":"s1","hook_event_name":"Stop"}"#)
        .assert()
        .code(0)
        .stderr("");
}

#[test]
fn hook_validate_ignores_paths_it_does_not_read() {
    // Even a broken bundle: an edit outside the bundle and the grammar's roots
    // is not the hook's business.
    let repo = hook_repo(false);
    superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "src/main.rs"))
        .assert()
        .code(0);
}

/// The hook covers the governed files too, so an edit that breaks a skill is
/// caught where the merge gate would catch it (FR-7).
#[test]
fn hook_validate_blocks_an_edit_that_broke_a_skill() {
    let repo = hook_repo(true);
    let skill = repo.path().join(".claude/skills/broken");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(skill.join("SKILL.md"), "no frontmatter, no elements\n").unwrap();
    let out = superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), ".claude/skills/broken/SKILL.md"))
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("SKILL.md"), "stderr: {stderr}");
}

#[test]
fn hook_validate_falls_back_to_the_working_directory() {
    let repo = hook_repo(false);
    superdev()
        .current_dir(repo.path())
        .args(["hook", "validate"])
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(2);
}

/// The working-directory fallback lands on a root the OS reports with
/// symlinks resolved, while the payload keeps the name the caller typed.
/// macOS hits this with every temp dir (`/private/var` vs `/var`); a
/// symlinked repo reproduces it anywhere.
#[cfg(unix)]
#[test]
fn hook_validate_follows_a_symlinked_working_directory() {
    let repo = hook_repo(false);
    let link = repo.path().join("link");
    std::os::unix::fs::symlink(repo.path(), &link).unwrap();
    superdev()
        .current_dir(&link)
        .args(["hook", "validate"])
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(hook_payload(&link, "knowledge/alpha.md"))
        .assert()
        .code(2);
}

#[test]
fn hook_validate_is_loud_on_a_malformed_payload() {
    let repo = hook_repo(true);
    let out = superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin("not json")
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("malformed"), "stderr: {stderr}");
}

#[test]
fn hook_validate_ignores_payloads_without_a_file_path() {
    let repo = hook_repo(false);
    superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(r#"{"tool_input":{}}"#)
        .assert()
        .code(0);
}

/// `init` a temp repo with only the skills capability (the others need
/// external binaries; skills needs none, so these tests run everywhere).
/// The SOKF knowledge comes too: it is part of superdev, and no flag skips
/// it.
fn skills_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    dir
}

#[test]
fn init_materialises_the_skill_pack() {
    let dir = skills_repo();
    for name in ["double-check", "template-update"] {
        let path = dir.path().join(format!(".claude/skills/{name}/SKILL.md"));
        assert!(path.is_file(), "missing {}", path.display());
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains(&format!("<skill name=\"{name}\"")),
            "{name} lacks its skill element"
        );
    }
    // The hook and the lifecycle skills belong to SOKF, which no flag
    // disables, so they are here beside the pack's.
    assert!(dir.path().join(".claude/settings.json").exists());
    assert!(dir.path().join(".claude/skills/maintain").exists());
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(
        lock.contains(".claude/skills/template-update/SKILL.md"),
        "{lock}"
    );
    assert!(lock.contains("superdev-skills"), "{lock}");
    // Straight after init there is nothing to do.
    superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
}

#[test]
fn adopting_a_repo_with_its_own_skills_keeps_them() {
    // goodbye-tinnitus's case: skills of the same name, written before the
    // pack existed. Adoption must not replace work superdev never wrote.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let theirs = dir.path().join(".claude/skills/template-update/SKILL.md");
    std::fs::create_dir_all(theirs.parent().unwrap()).unwrap();
    std::fs::write(&theirs, "# Ours, thanks\n").unwrap();

    let out = superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("kept your template-update"), "{stdout}");

    // Their file survives; the rest of the pack lands; the manifest records it.
    assert_eq!(
        std::fs::read_to_string(&theirs).unwrap(),
        "# Ours, thanks\n"
    );
    assert!(
        dir.path()
            .join(".claude/skills/double-check/SKILL.md")
            .is_file()
    );
    let config = std::fs::read_to_string(dir.path().join(".superdev/config.toml")).unwrap();
    assert!(
        config.contains("custom = [\"template-update\"]"),
        "{config}"
    );
    // No drift, and no lock hash claiming their file as superdev's.
    superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(
        !lock.contains(".claude/skills/template-update/SKILL.md"),
        "{lock}"
    );
}

/// Strip the `[[packs]]` entry `init` writes, leaving the manifest in the
/// shape a binary that knew nothing about packs produced. An absent array is
/// the migration case, and after this slice it is the only way to spell it.
fn as_a_pre_pack_manifest(root: &Path) -> String {
    let config = root.join(".superdev/config.toml");
    let existing = std::fs::read_to_string(&config).unwrap();
    let mut kept = String::new();
    let mut inside = false;
    for line in existing.lines() {
        if line.trim() == "[[packs]]" {
            inside = true;
            continue;
        }
        if inside {
            // The block runs to the next table header, and the blank line
            // before it belongs to the block.
            if line.starts_with('[') {
                inside = false;
            } else {
                continue;
            }
        }
        kept.push_str(line);
        kept.push('\n');
    }
    assert!(!kept.contains("packs"), "{kept}");
    std::fs::write(&config, &kept).unwrap();
    kept
}

/// Add `[[packs]]` entries to a repo's manifest, keeping the rest as written.
fn pin_packs(root: &Path, entries: &str) {
    let config = root.join(".superdev/config.toml");
    let existing = std::fs::read_to_string(&config).unwrap();
    let (first, rest) = existing.split_once('\n').expect("blueprint line");
    std::fs::write(&config, format!("{first}\n\n{entries}\n{rest}")).unwrap();
}

/// Test plan case 2: a pin naming exactly what this binary embeds is the
/// default path written out, and must cost no request. `init` writes that
/// entry, so this is every fresh repo. Run with `PATH` emptied, so a spawn of
/// any kind would fail: whatever this writes came from the binary itself, not
/// from the network.
#[test]
fn a_pin_at_the_embedded_rev_costs_no_request() {
    let dir = local_repo();
    let config = std::fs::read_to_string(dir.path().join(".superdev/config.toml")).unwrap();
    assert!(
        config.contains("[[packs]]")
            && config.contains("github:six5536/superdev")
            && config.contains("assets-v0.1.0"),
        "init writes the default entry explicitly: {config}"
    );

    superdev()
        .current_dir(dir.path())
        .env_clear()
        .env("HOME", dir.path())
        .env("PATH", "")
        .arg("sync")
        .assert()
        .success();
    // The embedded pack still supplies the items, and the entry survives.
    assert!(dir.path().join(".claude/skills/frame/SKILL.md").is_file());
    let after = std::fs::read_to_string(dir.path().join(".superdev/config.toml")).unwrap();
    assert_eq!(after, config, "sync leaves the pin exactly as it found it");
}

/// I007, inverted: the reproduction that worked, asserted not to.
///
/// `ext::<command>` makes git run that command as the connection. Whether it
/// may is `protocol.ext.allow`, which a stock git sets to refuse — so this
/// enables it, which is the configuration where superdev's own defence is the
/// only one left. A manifest arrives with a repository; cloning someone's
/// branch and running `sync` must not run their command.
///
/// Two of superdev's defences now stop this, and the assertion does not care
/// which: `parse` refuses the helper before the spawn, and the overrides
/// refuse the transport if it ever reaches git. Removing either alone leaves
/// this passing, which is the point of keeping it beside the two tests that
/// each pin one half — `a_manifest_naming_a_remote_helper_is_refused_before_anything_runs`
/// and `an_update_cannot_be_talked_into_running_a_manifests_command`.
#[cfg(unix)]
#[test]
fn a_manifest_cannot_talk_git_into_running_its_command() {
    let dir = local_repo();
    let marker = dir.path().join("PROOF");
    let gitconfig = dir.path().join("permissive.gitconfig");
    std::fs::write(&gitconfig, "[protocol \"ext\"]\n\tallow = always\n").unwrap();
    pin_packs(
        dir.path(),
        &format!(
            "[[packs]]\nsource = \"ext::touch {}\"\nrev = \"main\"\n",
            marker.display()
        ),
    );

    // Not `.success()`: the pack cannot resolve either way, and what is under
    // test is what did not happen while it failed.
    superdev()
        .current_dir(dir.path())
        .env("GIT_CONFIG_GLOBAL", &gitconfig)
        // Or a system config refusing the transport would make this pass
        // while proving nothing: the premise is that the transport IS
        // permitted, and only superdev's own override stands in the way.
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("sync")
        .assert()
        .failure();

    assert!(!marker.exists(), "superdev ran a command a manifest named");
}

/// I007 by the path `sync` alone does not cover: the query `update` makes.
///
/// A source `parse` approved can still reach git as something else.
/// `url.<base>.insteadOf` rewrites it *after* superdev has handed it over, so
/// a plain `https://` pin — which the transport allowlist accepts, and which
/// keys as superdev's own source, so `update` queries it — becomes an `ext::`
/// command on a machine whose config asks for that. The allowlist cannot see
/// the rewrite; only the named refusal in superdev's own overrides stops it.
///
/// This is the end-to-end half of `pack::fetch`'s `a_rewritten_url_still_runs_no_command`,
/// and it is what keeps the query path covered now that a manifest can no
/// longer name a helper outright — the case below.
///
/// The pin is a release the source does not carry, so both halves of the run
/// go out: the query, then the fetch the following `sync` attempts. Neither
/// may run the command, and the assertion does not distinguish them — read a
/// failure here as "some git call lost the override", not as a verdict on
/// which one.
#[cfg(unix)]
#[test]
fn an_update_cannot_be_talked_into_running_a_manifests_command() {
    let dir = local_repo();
    let marker = dir.path().join("PROOF");
    let gitconfig = dir.path().join("permissive.gitconfig");
    // A machine that has enabled the transport — which people do for custom
    // helpers — and rewrites superdev's own source into one. Both halves are
    // the user's own config, and neither is anything superdev can see.
    std::fs::write(
        &gitconfig,
        format!(
            "[protocol \"ext\"]\n\tallow = always\n\
             [url \"ext::touch {} \"]\n\tinsteadOf = https://github.com/\n",
            marker.display(),
        ),
    )
    .unwrap();
    // The pin keys as the default, so it must replace the entry `init` wrote
    // rather than sit beside it — two entries naming one source are refused,
    // and that refusal would pass this test for the wrong reason.
    as_a_pre_pack_manifest(dir.path());
    // A release tag the source does not carry, rather than whatever this
    // binary embeds: the pin has to be one `update` will query and one whose
    // outcome does not depend on the release phase. Embed a candidate and the
    // run reports "a candidate, and no release covers it yet" instead, which
    // says nothing about whether the query went out.
    pin_packs(
        dir.path(),
        "[[packs]]\nsource = \"https://github.com/six5536/superdev\"\nrev = \"assets-v9.9.9\"\n",
    );

    let out = superdev()
        .current_dir(dir.path())
        .env("GIT_CONFIG_GLOBAL", &gitconfig)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("update")
        .assert()
        // The sync that follows cannot resolve the pin either, and is refused
        // for the same reason. What matters is which of the two ran a command.
        .failure();

    assert!(
        !marker.exists(),
        "a git call under `update` ran a command the machine's config named — \
         the query or the fetch that follows it"
    );
    // The query has to have gone out, or this proves nothing: were the pin
    // ever short-circuited when it equals the embedded rev — which `sync`
    // already does — the `ext::` path would never be reached and the marker
    // would be absent for the wrong reason.
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("could not reach it"),
        "the source was never asked: {stdout}"
    );
}

/// A manifest may not name a remote helper at all, so the crafted source that
/// used to reach git no longer gets that far.
///
/// The transport allowlist refuses it in `PackSource::parse`, before anything
/// is spawned, and no config on the machine can lift that — where the
/// overrides above are what hold once a URL has reached git. Two different
/// failures, which is why both cases are kept. ADR-012.
#[cfg(unix)]
#[test]
fn a_manifest_naming_a_remote_helper_is_refused_before_anything_runs() {
    let dir = local_repo();
    let marker = dir.path().join("PROOF");
    let gitconfig = dir.path().join("permissive.gitconfig");
    std::fs::write(&gitconfig, "[protocol \"ext\"]\n\tallow = always\n").unwrap();
    as_a_pre_pack_manifest(dir.path());
    pin_packs(
        dir.path(),
        &format!(
            "[[packs]]\nsource = \"ext::touch {} ://@github.com/six5536/superdev\"\nrev = \"assets-v9.9.9\"\n",
            marker.display(),
        ),
    );

    let out = superdev()
        .current_dir(dir.path())
        .env("GIT_CONFIG_GLOBAL", &gitconfig)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .arg("sync")
        .assert()
        .failure();

    assert!(!marker.exists(), "superdev ran a command a manifest named");
    let reported = format!(
        "{}{}",
        stdout_of(&out),
        String::from_utf8_lossy(&out.get_output().stderr)
    );
    assert!(
        reported.contains("remote helper"),
        "the refusal does not say what is wrong with the source: {reported}"
    );
}

/// I005: the lock describes what is on disk, not only what this run wrote.
///
/// A backport ends with the live copy already matching the pack, so `sync`
/// plans nothing and writes nothing — and a hash is recorded only for a file
/// that gets written, so the lock keeps whatever it held before the edit.
/// The next run that does write that file then reports a user edit nobody
/// made. Simulated here by staling one entry directly, which is the state a
/// backport leaves behind.
#[test]
fn a_converged_run_brings_the_lock_up_to_what_is_on_disk() {
    let dir = local_repo();
    let lock = dir.path().join(".superdev/lock.toml");
    let key = ".claude/skills/double-check/SKILL.md";
    let real = std::fs::read_to_string(&lock).unwrap();
    let stale = real
        .lines()
        .map(|line| {
            if line.starts_with(&format!("\"{key}\"")) {
                format!("\"{key}\" = \"{}\"", "0".repeat(64))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(stale, real, "the fixture never staled anything");
    std::fs::write(&lock, format!("{stale}\n")).unwrap();

    // Nothing to do: the file on disk is already what superdev would write.
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    let after = std::fs::read_to_string(&lock).unwrap();
    assert!(
        !after.contains(&"0".repeat(64)),
        "the lock still describes a file that is not there: {after}"
    );
}

/// The other half of I005: reconciling must not swallow a real edit.
///
/// A hand-edited owned file differs from the lock *and* from what superdev
/// writes. Refresh the lock before the engine runs and the two agree by the
/// time it compares them, so the overwrite is reported as an ordinary write
/// and the user is never told their edit went into a backup. The order is the
/// whole of the fix, and nothing else would notice it being wrong.
#[test]
fn a_hand_edited_file_is_still_reported_as_one() {
    let dir = local_repo();
    let live = dir.path().join(".claude/skills/double-check/SKILL.md");
    let mine = format!(
        "{}\nmy own words\n",
        std::fs::read_to_string(&live).unwrap()
    );
    std::fs::write(&live, &mine).unwrap();

    let out = superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("overwrote a user-edited file (backed up)"),
        "the edit was overwritten with nothing said: {stdout}"
    );
}

/// I005 is not only about whole files. A mise pin and a JSON key are claims
/// too, and their hashes are recorded on the same terms — only when an action
/// applies. A stale one is worse here than on a file: the orphan pass reads
/// the current value, finds it does not match what the lock recorded, and
/// classifies superdev's own entry as *released* instead of removing it. The
/// registration then stays in the user's shared file for good, with a line
/// saying they changed it.
#[test]
fn a_converged_run_reconciles_a_json_key_too() {
    let dir = local_repo();
    let lock = dir.path().join(".superdev/lock.toml");
    let key = ".mcp.json:mcpServers.superdev-sokf";
    let real = std::fs::read_to_string(&lock).unwrap();
    assert!(real.contains(key), "the fixture claims no json key: {real}");
    let stale = real
        .lines()
        .map(|line| {
            if line.starts_with(&format!("\"{key}\"")) {
                format!("\"{key}\" = \"{}\"", "0".repeat(64))
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(stale, real, "the fixture never staled anything");
    std::fs::write(&lock, format!("{stale}\n")).unwrap();

    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    let after = std::fs::read_to_string(&lock).unwrap();
    assert!(
        !after.contains(&"0".repeat(64)),
        "the json key still carries a hash of nothing on disk: {after}"
    );
}

/// `update` is the one verb that reaches the network, and its own help said
/// it moved pins to this binary's defaults — which stopped being true when it
/// began asking the default pack source for a newer release. The description
/// is a doc comment clap renders into `--help`, the man page and the
/// completions, so it went stale in three places at once and nothing noticed.
/// I006.
#[test]
fn update_help_says_it_may_reach_the_source() {
    let out = superdev().args(["update", "--help"]).assert().success();
    let help = stdout_of(&out);

    // The man page is a one-line index per subcommand, so the detail cannot
    // live in `update`'s summary — a summary long enough to hold it runs off
    // the help table. It goes in the top-level description instead, which is
    // the man page's prose, and that is asserted here too.
    let manual = stdout_of(&superdev().arg("man").assert().success());
    assert!(
        manual.to_lowercase().contains("pack"),
        "the man page never mentions packs: {manual}"
    );

    // Not "does not say `this binary's defaults`" — that is still true of a
    // capability's pin, and the old text was wrong for saying it of every
    // pin. What has to be there is the part that was missing.
    let help = help.to_lowercase();
    assert!(
        help.contains("pack"),
        "the help does not mention the pack pin it may move: {help}"
    );
    assert!(
        help.contains("newest release"),
        "the help does not say the pack pin may go past this binary: {help}"
    );
}

/// A user who never reads the knowledge should still be able to find out
/// that content comes from somewhere and can be pointed elsewhere.
#[test]
fn the_readme_describes_packs() {
    let readme = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../README.md"),
    )
    .expect("the README");

    for expected in ["[[packs]]", "assets-v", "superdev update"] {
        assert!(
            readme.contains(expected),
            "the README never mentions `{expected}`"
        );
    }
}

/// Test plan case 19: a local-path pack is read from disk every run, so
/// editing it and syncing again updates the repo copy — no rebuild, which is
/// what lets this repo pin its own `/pack` and see an edit land.
#[test]
fn a_local_pack_updates_the_repo_copy_without_a_rebuild() {
    let dir = local_repo();
    let pack = dir.path().join("packs/acme");
    let skill = pack.join("knowledge/skills/acme-review");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(
        pack.join("pack.toml"),
        "format = 1\nname = \"acme\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    std::fs::write(skill.join("SKILL.md"), "# Acme review\n\nv1\n").unwrap();
    pin_packs(dir.path(), "[[packs]]\nsource = \"./packs/acme\"\n");

    let written = dir.path().join(".claude/skills/acme-review/SKILL.md");
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&written).unwrap(),
        "# Acme review\n\nv1\n"
    );

    // The same binary, a changed pack: the new bytes reach the repo.
    std::fs::write(skill.join("SKILL.md"), "# Acme review\n\nv2\n").unwrap();
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&written).unwrap(),
        "# Acme review\n\nv2\n"
    );
}

/// A local pack holding two skills, pinned into the repo and synced.
fn repo_with_a_pack() -> tempfile::TempDir {
    let dir = local_repo();
    let pack = dir.path().join("packs/acme");
    std::fs::write(
        {
            std::fs::create_dir_all(&pack).unwrap();
            pack.join("pack.toml")
        },
        "format = 1\nname = \"acme\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    for (name, body) in [
        ("acme-review", "# Acme review\n"),
        ("acme-plan", "# Acme plan\n"),
    ] {
        let skill = pack.join(format!("knowledge/skills/{name}"));
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.join("SKILL.md"), body).unwrap();
    }
    pin_packs(dir.path(), "[[packs]]\nsource = \"./packs/acme\"\n");
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();
    dir
}

/// Test plan case 20: with the default source unreachable, `update` moves the
/// pin no further than what this binary carries and says it could not check.
/// `PATH` emptied, so no `git` exists to ask — the same state as no network,
/// and `update` has to stay usable in it.
#[test]
fn an_offline_update_stops_at_the_blueprint_default() {
    let dir = local_repo();
    let config = dir.path().join(".superdev/config.toml");
    let before = std::fs::read_to_string(&config).unwrap();

    let out = superdev()
        .current_dir(dir.path())
        .env_clear()
        .env("HOME", dir.path())
        .env("PATH", "")
        .arg("update")
        .assert()
        .success();

    assert_eq!(
        std::fs::read_to_string(&config).unwrap(),
        before,
        "the pin moved with nothing to move it to"
    );
    let stdout = stdout_of(&out);
    assert!(stdout.contains("could not reach"), "{stdout}");
    assert!(stdout.contains("assets-v0.1.0"), "{stdout}");
}

/// Test plan case 22: a manifest an earlier binary wrote gains the default
/// entry on `update`. Offline, so what lands is the blueprint default — the
/// migration is the entry appearing, not where it points.
#[test]
fn an_update_writes_the_default_entry_into_a_pre_pack_manifest() {
    let dir = local_repo();
    let config = dir.path().join(".superdev/config.toml");
    as_a_pre_pack_manifest(dir.path());

    let out = superdev()
        .current_dir(dir.path())
        .env_clear()
        .env("HOME", dir.path())
        .env("PATH", "")
        .arg("update")
        .assert()
        .success();

    let after = std::fs::read_to_string(&config).unwrap();
    assert!(after.contains("[[packs]]"), "{after}");
    assert!(
        after.contains("source = \"github:six5536/superdev\""),
        "{after}"
    );
    assert!(after.contains("rev = \"assets-v0.1.0\""), "{after}");
    assert!(
        stdout_of(&out).contains("wrote the default entry"),
        "{after}"
    );
    // Still the same content: the entry names what was already compiled in.
    assert!(dir.path().join(".claude/skills/frame/SKILL.md").is_file());
}

/// A real git repository holding a pack, tagged, so the query and the fetch
/// run against git rather than a script of what git might say.
fn tagged_pack_repo(dir: &Path, tags: &[&str], skill: &str, body: &str) {
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git runs");
        assert!(out.status.success(), "git {args:?}: {out:?}");
    };
    std::fs::create_dir_all(dir.join(format!("pack/knowledge/skills/{skill}"))).unwrap();
    std::fs::write(
        dir.join("pack/pack.toml"),
        "format = 1\nname = \"fixture\"\nversion = \"9.9.9\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join(format!("pack/knowledge/skills/{skill}/SKILL.md")),
        body,
    )
    .unwrap();
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "fixture@example.com"]);
    git(&["config", "user.name", "fixture"]);
    // The developer's global config may sign commits and tags; a fixture must
    // not depend on a key being available.
    git(&["config", "commit.gpgsign", "false"]);
    git(&["config", "tag.gpgSign", "false"]);
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "pack"]);
    for tag in tags {
        git(&["tag", "-f", tag]);
    }
}

/// A directory holding a `git` that answers for the default source out of a
/// local repository, and delegates everything else to the real one.
///
/// The default source is a URL on the internet; the only honest way to test
/// that `update` asks it and fetches what it names is to put a repository
/// where it looks. One substitution does it — both `ls-remote` and `clone`
/// carry the URL as an argument.
#[cfg(unix)]
fn git_answering_for_the_default_source(origin: &Path) -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;

    let real = std::process::Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .expect("sh runs");
    let real = String::from_utf8_lossy(&real.stdout).trim().to_string();
    assert!(!real.is_empty(), "no git on PATH to delegate to");

    let dir = tempfile::tempdir().unwrap();
    let shim = dir.path().join("git");
    std::fs::write(
        &shim,
        format!(
            "#!/bin/sh\n\
             n=$#\n\
             i=0\n\
             while [ $i -lt $n ]; do\n\
             \x20 a=$1; shift\n\
             \x20 if [ \"$a\" = \"https://github.com/six5536/superdev\" ]; then\n\
             \x20   a=\"file://{origin}\"\n\
             \x20 fi\n\
             \x20 set -- \"$@\" \"$a\"\n\
             \x20 i=$((i+1))\n\
             done\n\
             exec {real} \"$@\"\n",
            origin = origin.display(),
        ),
    )
    .unwrap();
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

/// Test plan case 16: `update` asks the default source for its newest release
/// and moves the pin there, ahead of what this binary embeds. Without this the
/// feature reaches only repos whose owner edits the manifest by hand, which is
/// most of the point of shipping content separately from the binary.
///
/// The release then *replaces* layer 0 rather than layering over it, because
/// its identity is the default source's: what the rev no longer carries is
/// gone from the repo. That is the base-replacement rule end to end.
#[cfg(unix)]
#[test]
fn an_update_moves_the_default_pin_to_the_sources_newest_release() {
    let dir = local_repo();
    let origin = tempfile::tempdir().unwrap();
    // `assets-v0.9.0` sorts after `assets-v0.10.0` as a string, so a run that
    // picked the newest tag alphabetically would move the pin backwards.
    tagged_pack_repo(
        origin.path(),
        &["assets-v0.1.0", "assets-v0.9.0", "assets-v0.10.0", "v2.0.0"],
        "acme-review",
        "# From the release\n",
    );
    let shim = git_answering_for_the_default_source(origin.path());
    let path = format!(
        "{}:{}",
        shim.path().display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let out = superdev()
        .current_dir(dir.path())
        .env("PATH", &path)
        .arg("update")
        .assert()
        .success();

    let config = std::fs::read_to_string(dir.path().join(".superdev/config.toml")).unwrap();
    assert!(config.contains("rev = \"assets-v0.10.0\""), "{config}");
    assert!(
        stdout_of(&out).contains("moved to assets-v0.10.0"),
        "{}",
        stdout_of(&out)
    );

    // The release's content reached the repo, and replaced the embedded pack
    // rather than layering over it: what it does not carry is gone.
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".claude/skills/acme-review/SKILL.md")).unwrap(),
        "# From the release\n"
    );
    assert!(
        !dir.path().join(".claude/skills/frame/SKILL.md").exists(),
        "the pinned rev is the whole of layer 0"
    );

    // The lock records what was fetched, so a later run can prove it again.
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(lock.contains("assets-v0.10.0"), "{lock}");
    assert!(lock.contains("sha256:"), "{lock}");
}

/// Test plan case 13: ownership is unchanged by provenance. A hand-edited
/// pack-provided file is drift on exactly the terms an embedded one is —
/// reported, exit 1, and repaired by `sync` with the edit backed up.
#[test]
fn a_hand_edited_pack_file_is_drift_like_any_other() {
    let dir = repo_with_a_pack();
    let written = dir.path().join(".claude/skills/acme-review/SKILL.md");
    std::fs::write(&written, "# Mine now\n").unwrap();

    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(1);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains(".claude/skills/acme-review/SKILL.md"),
        "{stdout}"
    );

    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&written).unwrap(),
        "# Acme review\n",
        "sync repairs a pack-provided file"
    );
}

/// Test plan case 15: dropping a pack entry removes its files by the
/// existing orphan rule — pruned when they still hash to the locked value,
/// left and released when the user has changed them. A pack's files are
/// superdev's on the same terms as any other, right up to the point the user
/// makes one theirs.
#[test]
fn dropping_a_pack_entry_prunes_the_untouched_and_releases_the_edited() {
    let dir = repo_with_a_pack();
    let untouched = dir.path().join(".claude/skills/acme-review/SKILL.md");
    let edited = dir.path().join(".claude/skills/acme-plan/SKILL.md");
    std::fs::write(&edited, "# I edited this one\n").unwrap();

    // The record has to be there before its absence means anything: without
    // this, a build that never wrote one would pass the assertion below.
    let lock_path = dir.path().join(".superdev/lock.toml");
    assert!(
        std::fs::read_to_string(&lock_path)
            .unwrap()
            .contains("[[packs]]"),
        "the resolved pack is recorded while the entry stands"
    );

    // Drop the entry, leaving the pack directory on disk.
    let config = dir.path().join(".superdev/config.toml");
    let without = std::fs::read_to_string(&config)
        .unwrap()
        .replace("[[packs]]\nsource = \"./packs/acme\"\n\n", "");
    std::fs::write(&config, without).unwrap();

    let out = superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();

    assert!(!untouched.exists(), "an untouched pack file is pruned");
    assert_eq!(
        std::fs::read_to_string(&edited).unwrap(),
        "# I edited this one\n",
        "an edited one is left exactly where it is"
    );
    assert_eq!(
        stdout
            .lines()
            .filter(|l| l.contains("acme-plan") && l.contains("released from the lock"))
            .count(),
        1,
        "reported once, not per pass: {stdout}"
    );
    let lock = std::fs::read_to_string(&lock_path).unwrap();
    assert!(!lock.contains("acme"), "both leave the lock: {lock}");
    assert!(!lock.contains("[[packs]]"), "and so does the pack record");
}

/// A pack item of the same name supersedes the embedded one, and everything
/// the pack does not carry still comes from the embedded copy.
#[test]
fn a_pack_supersedes_the_embedded_item_of_the_same_name() {
    let dir = local_repo();
    let skill = dir.path().join("packs/acme/knowledge/skills/frame");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(
        dir.path().join("packs/acme/pack.toml"),
        "format = 1\nname = \"acme\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    std::fs::write(skill.join("SKILL.md"), "# Frame, the acme way\n").unwrap();
    pin_packs(dir.path(), "[[packs]]\nsource = \"./packs/acme\"\n");

    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".claude/skills/frame/SKILL.md")).unwrap(),
        "# Frame, the acme way\n"
    );
    assert!(
        dir.path()
            .join(".claude/skills/integrate/SKILL.md")
            .is_file(),
        "what the pack does not carry still comes from the embedded copy"
    );
}

/// Test plan case 21: a repo initialised by a binary that knew nothing about
/// packs must keep syncing from the embedded pack, and `sync` must not write
/// a pack entry into the manifest on its way past. An absent `[[packs]]` is
/// the default, not something to migrate — `update` is what fills it in.
#[test]
fn sync_leaves_a_pre_pack_manifest_without_a_pack_entry() {
    let dir = local_repo();
    let config = dir.path().join(".superdev/config.toml");

    // What an earlier binary wrote: no `[[packs]]` anywhere.
    let before = as_a_pre_pack_manifest(dir.path());
    assert!(!before.contains("packs"), "{before}");

    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .success();

    let after = std::fs::read_to_string(&config).unwrap();
    assert_eq!(after, before, "sync rewrote the manifest");
    assert!(
        !dir.path().join(".claude/skills/frame/SKILL.md").exists()
            || dir.path().join(".claude/skills/frame/SKILL.md").is_file(),
        "the embedded pack still supplies the files"
    );
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(!lock.contains("[[packs]]"), "{lock}");
}

/// Test plan case 1: the default path — no pack entry in the manifest —
/// resolves entirely from the pack compiled into the binary. Run with `PATH`
/// emptied, so a spawn of any kind would fail and a network client would have
/// nothing to reach for: whatever this writes came from the binary itself.
///
/// The assertion outlives this slice. Once a manifest can name a pack, the
/// absence of one must still mean the embedded snapshot and still touch
/// nothing outside the repo.
#[test]
fn the_default_path_needs_nothing_outside_the_binary() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .env_clear()
        .env("HOME", dir.path())
        .env("PATH", "")
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--template",
            "none",
        ])
        .assert()
        .success();

    let carried = [
        ".claude/skills/frame/SKILL.md",
        ".claude/skills/double-check/SKILL.md",
        "knowledge/index.md",
        "knowledge/schemas/adr.md",
        ".agents/sokf/grammar.yaml",
    ];
    for path in carried {
        assert!(dir.path().join(path).is_file(), "missing {path}");
    }

    // A manifest naming no pack at all — what an earlier binary wrote, and
    // still the shape this case is about — resolves the same way and reaches
    // for nothing either.
    as_a_pre_pack_manifest(dir.path());
    for path in carried {
        std::fs::remove_file(dir.path().join(path)).unwrap();
    }
    superdev()
        .current_dir(dir.path())
        .env_clear()
        .env("HOME", dir.path())
        .env("PATH", "")
        .arg("sync")
        .assert()
        .success();
    for path in carried {
        assert!(dir.path().join(path).is_file(), "missing {path} after sync");
    }
    let manifest = std::fs::read_to_string(dir.path().join(".superdev/config.toml")).unwrap();
    assert!(
        !manifest.contains("[[packs]]"),
        "sync added one: {manifest}"
    );
}

#[test]
fn init_no_skills_skips_the_pack() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index", "--no-skills"])
        .assert()
        .success();
    // The pack's skills are gone; SOKF's are not, because SOKF is not the
    // pack and no flag turns it off.
    assert!(!dir.path().join(".claude/skills/double-check").exists());
    assert!(!dir.path().join(".claude/skills/template-update").exists());
    assert!(dir.path().join(".claude/skills/maintain").exists());
    assert!(dir.path().join(".claude/settings.json").exists());
}

#[test]
fn init_ignores_a_cache_left_by_the_knowledge_tools() {
    // `superdev mcp sokf` writes .superdev/cache/ in repos it never initialised.
    // Only the manifest means initialised.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::create_dir_all(dir.path().join(".superdev/cache/sokf-index")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    assert!(dir.path().join(".superdev/config.toml").is_file());
}

#[test]
fn init_refuses_when_the_manifest_exists() {
    let dir = skills_repo();
    let out = superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("already initialised"), "{stderr}");
}

#[test]
fn a_drifted_skill_is_drift_until_marked_custom() {
    let dir = skills_repo();
    let skill = dir.path().join(".claude/skills/template-update/SKILL.md");
    std::fs::write(&skill, "# Mine now\n").unwrap();
    // Drift: status exits 1 and names the file.
    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(1);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("template-update"), "{stdout}");

    // Take it over: drift becomes a chosen state.
    let config_path = dir.path().join(".superdev/config.toml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    std::fs::write(
        &config_path,
        config.replace("[skills]", "[skills]\ncustom = [\"template-update\"]"),
    )
    .unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("skills: template-update custom, unmanaged"),
        "{stdout}"
    );

    // sync honours the takeover and prunes the lock entry.
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .code(0);
    assert_eq!(std::fs::read_to_string(&skill).unwrap(), "# Mine now\n");
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(
        !lock.contains(".claude/skills/template-update/SKILL.md"),
        "{lock}"
    );

    // Back under management: the next sync restores stock.
    let config = std::fs::read_to_string(&config_path).unwrap();
    std::fs::write(
        &config_path,
        config.replace("\ncustom = [\"template-update\"]", ""),
    )
    .unwrap();
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .code(0);
    assert!(
        std::fs::read_to_string(&skill)
            .unwrap()
            .contains("<skill name=\"template-update\"")
    );
}

#[test]
fn user_hook_entries_survive_a_sync() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::write(
        dir.path().join(".claude/settings.json"),
        r#"{"hooks":{"PostToolUse":[{"matcher":"Agent","hooks":[{"type":"command","command":"my-own-hook"}]}]},"permissions":{"deny":["Read(secrets/**)"]}}"#,
    )
    .unwrap();
    superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    let settings: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
    )
    .unwrap();
    let entries = settings["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .any(|e| e.to_string().contains("my-own-hook"))
    );
    assert_eq!(settings["permissions"]["deny"][0], "Read(secrets/**)");
}

#[test]
fn update_skills_to_an_explicit_version_is_refused() {
    let dir = skills_repo();
    let out = superdev()
        .current_dir(dir.path())
        .args(["update", "skills@9.9.9"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("skills"), "{stderr}");
}

#[test]
fn a_bare_at_sign_names_no_version() {
    let dir = skills_repo();
    let out = superdev()
        .current_dir(dir.path())
        .args(["update", "skills@"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("names no version"), "{stderr}");
}

#[test]
fn an_uninitialised_repo_says_to_run_init() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("not initialised"), "{stderr}");
    assert!(stderr.contains("superdev init"), "{stderr}");
}

/// `init` an existing repo with the two capabilities that need no external
/// binary: skills and knowledge. Nothing here plans a `Run` action.
fn init_local(root: &Path) {
    superdev()
        .current_dir(root)
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
}

/// A fresh git repo, initialised the same way.
fn local_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    init_local(dir.path());
    dir
}

/// Drop one `[table]` and its keys from a TOML manifest, leaving every other
/// line as it stands: the hand edit that turns a capability off.
fn remove_table(toml: &str, table: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for line in toml.lines() {
        if line.trim_start().starts_with('[') {
            skipping = line.trim() == table;
        }
        if !skipping {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// The one timestamped directory a run's backups sit under.
fn backup_dir(root: &Path) -> PathBuf {
    let mut stamps: Vec<PathBuf> = std::fs::read_dir(root.join(".superdev/cache/backup"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(stamps.len(), 1, "expected one backup stamp: {stamps:?}");
    stamps.pop().unwrap()
}

fn stdout_of(out: &assert_cmd::assert::Assert) -> String {
    String::from_utf8_lossy(&out.get_output().stdout).into_owned()
}

#[test]
fn disabling_skills_sweeps_them_and_releases_the_users_edit() {
    let dir = local_repo();
    let root = dir.path();
    // The bytes superdev shipped, trailer and all: what the backup must hold.
    let shipped =
        std::fs::read_to_string(root.join(".claude/skills/double-check/SKILL.md")).unwrap();
    let users_skill = root.join(".claude/skills/template-update/SKILL.md");
    std::fs::write(&users_skill, "mine now\n").unwrap();

    // Turn the capability off by hand, the way a user would: the rest of the
    // manifest stays byte-for-byte.
    let config_path = root.join(".superdev/config.toml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let edited = remove_table(&config, "[skills]");
    assert!(edited.contains("[knowledge]"), "{edited}");
    assert!(edited.contains("blueprint = "), "{edited}");
    assert!(!edited.contains("superdev-skills"), "{edited}");
    std::fs::write(&config_path, &edited).unwrap();

    let out = superdev().current_dir(root).arg("status").assert().code(1);
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("remove .claude/skills/double-check/SKILL.md (no longer in the blueprint)"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "orphan: .claude/skills/template-update/SKILL.md changed since superdev wrote it — left in place, released from the lock"
        ),
        "{stdout}"
    );

    superdev().current_dir(root).arg("sync").assert().code(0);

    // What superdev wrote goes, with a backup; what the user wrote stays.
    assert!(!root.join(".claude/skills/double-check/SKILL.md").exists());
    assert_eq!(
        std::fs::read_to_string(backup_dir(root).join(".claude/skills/double-check/SKILL.md"))
            .unwrap(),
        shipped
    );
    assert_eq!(std::fs::read_to_string(&users_skill).unwrap(), "mine now\n");

    // The hook belongs to SOKF, which is part of superdev, so it survives
    // the pack's departure — as do the SOKF-carried skills.
    let settings: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert!(
        settings["hooks"]["PostToolUse"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.to_string().contains("superdev hook validate")),
        "settings: {settings}"
    );

    let lock = std::fs::read_to_string(root.join(".superdev/lock.toml")).unwrap();
    assert!(!lock.contains(".claude/skills/double-check/"), "{lock}");
    assert!(!lock.contains(".claude/skills/template-update/"), "{lock}");
    assert!(lock.contains(".claude/skills/maintain/"), "{lock}");
    assert!(lock.contains("hooks.PostToolUse"), "{lock}");
    assert!(!lock.contains("[components.skills]"), "{lock}");
    // SOKF's files stay locked: the sweep is targeted at the pack alone.
    assert!(lock.contains(".agents/sokf/SPEC.md"), "{lock}");
    // No provider record, though — SOKF fills no slot, so there is no
    // provider choice for the lock to remember.
    assert!(!lock.contains("[components.knowledge]"), "{lock}");

    // A settled state is not drift.
    superdev().current_dir(root).arg("status").assert().code(0);
}

/// `--drift` narrows the exit code, never the report: on managed files it
/// answers exactly as bare `status`. The runs it drops are covered in core,
/// where a plan can hold one with nothing beside it.
#[test]
fn status_drift_answers_as_status_does_on_managed_files() {
    let dir = local_repo();
    let root = dir.path();
    superdev()
        .current_dir(root)
        .args(["status", "--drift"])
        .assert()
        .code(0);

    // An owned file the user rewrote is drift under either gate.
    let users_skill = root.join(".claude/skills/template-update/SKILL.md");
    std::fs::write(&users_skill, "mine now\n").unwrap();
    superdev().current_dir(root).arg("status").assert().code(1);
    let out = superdev()
        .current_dir(root)
        .args(["status", "--drift"])
        .assert()
        .code(1);
    assert!(
        stdout_of(&out).contains("write .claude/skills/template-update/SKILL.md"),
        "{}",
        stdout_of(&out)
    );
}

#[test]
fn claude_md_import_is_created_appended_and_restored() {
    // A: no CLAUDE.md at all — superdev writes one holding just the import.
    let fresh = local_repo();
    assert_eq!(
        std::fs::read_to_string(fresh.path().join("CLAUDE.md")).unwrap(),
        "@AGENTS.md\n"
    );

    // B: a CLAUDE.md of the user's own — the line is appended, their text intact.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let claude = dir.path().join("CLAUDE.md");
    std::fs::write(&claude, "# House rules\n").unwrap();
    init_local(dir.path());
    assert_eq!(
        std::fs::read_to_string(&claude).unwrap(),
        "# House rules\n@AGENTS.md\n"
    );

    // C: the line deleted — the same bargain as the .gitignore lines, added
    //    back when missing.
    std::fs::write(&claude, "# House rules\n").unwrap();
    superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(1);
    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .code(0);
    assert_eq!(
        std::fs::read_to_string(&claude).unwrap(),
        "# House rules\n@AGENTS.md\n"
    );

    // D: settled — no drift, and no second copy of the line.
    superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
    assert_eq!(
        std::fs::read_to_string(&claude).unwrap(),
        "# House rules\n@AGENTS.md\n"
    );
}

#[test]
fn a_stale_blueprint_reports_on_status_and_sync_stamps_it() {
    let dir = local_repo();
    let config_path = dir.path().join(".superdev/config.toml");
    let current = format!("blueprint = \"{}\"", env!("CARGO_PKG_VERSION"));
    let config = std::fs::read_to_string(&config_path).unwrap();
    std::fs::write(
        &config_path,
        config.replace(&current, "blueprint = \"0.0.1\""),
    )
    .unwrap();

    // An older blueprint is news, not drift: the repo is converged.
    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains(&format!(
            "blueprint 0.0.1, binary {} — sync will update it",
            env!("CARGO_PKG_VERSION")
        )),
        "{stdout}"
    );

    superdev()
        .current_dir(dir.path())
        .arg("sync")
        .assert()
        .code(0);
    let stamped = std::fs::read_to_string(&config_path).unwrap();
    assert!(stamped.contains(&current), "{stamped}");

    let out = superdev()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .code(0);
    let stdout = stdout_of(&out);
    assert!(!stdout.contains("sync will update it"), "{stdout}");
}

/// A template init end-to-end: cross-platform (no provider tools needed with
/// every capability disabled), non-TTY (so nothing prompts).
#[test]
fn init_with_a_template_seeds_the_repo_and_records_it() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    // A pre-existing file wins over its template counterpart.
    std::fs::write(dir.path().join("README.md"), "mine\n").unwrap();
    let assert = superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--template",
            "rust-npm",
            "--name",
            "My Tool",
            "--no-frontend",
            "--no-skills",
            "--no-code-index",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(
        stdout.contains("template rust-npm: kept README.md — already exists"),
        "{stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("README.md")).unwrap(),
        "mine\n",
        "existing files win"
    );
    // Substituted content and tokenised paths landed.
    let main_rs = dir.path().join("crates/app/my-tool/src/main.rs");
    let main_rs = std::fs::read_to_string(main_rs).unwrap();
    assert!(main_rs.contains("my_tool_core::"), "{main_rs}");
    let license = std::fs::read_to_string(dir.path().join("LICENSE")).unwrap();
    assert!(license.contains("the owners of My Tool"), "{license}");
    // The manifest records the template as provenance.
    let config = std::fs::read_to_string(dir.path().join(".superdev/config.toml")).unwrap();
    assert!(config.contains("[template]"), "{config}");
    assert!(config.contains("project-slug = \"my-tool\""), "{config}");
    assert!(
        config.contains(&format!("version = \"{}\"", env!("CARGO_PKG_VERSION"))),
        "{config}"
    );
    // Template files are scaffolds: not one of them is locked.
    let lock = std::fs::read_to_string(dir.path().join(".superdev/lock.toml")).unwrap();
    assert!(!lock.contains("LICENSE"), "{lock}");
}

#[test]
fn init_without_a_tty_and_without_the_flag_seeds_nothing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-skills", "--no-code-index"])
        .assert()
        .success();
    assert!(!dir.path().join("LICENSE").exists(), "no template files");
    let config = std::fs::read_to_string(dir.path().join(".superdev/config.toml")).unwrap();
    assert!(!config.contains("[template]"), "{config}");
}

#[test]
fn an_unknown_template_fails_before_anything_is_written() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .args(["init", "--template", "flying"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains("template must be one of: rust-npm"),
        "unexpected: {stderr}"
    );
    assert!(!dir.path().join(".superdev").exists(), "nothing written");
}

#[test]
fn template_list_names_every_shipped_template() {
    let out = superdev().args(["template", "list"]).assert().success();
    let stdout = stdout_of(&out);
    assert!(stdout.contains("rust-npm — "), "{stdout}");
}

/// `template render` is the template-update skill's window into the binary:
/// the substituted tree lands in the directory, and the token lines are the
/// `[template]` values verbatim, so the skill never re-derives a slug.
#[test]
fn template_render_writes_the_tree_and_prints_the_tokens() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("rendered");
    let out = superdev()
        .args(["template", "render", "rust-npm", "--name", "My Tool"])
        .arg("--dir")
        .arg(&target)
        .assert()
        .success();
    let stdout = stdout_of(&out);
    assert!(stdout.contains("rendered template rust-npm"), "{stdout}");
    assert!(stdout.contains("project-name = \"My Tool\""), "{stdout}");
    assert!(stdout.contains("project-slug = \"my-tool\""), "{stdout}");
    assert!(stdout.contains("project-ident = \"my_tool\""), "{stdout}");
    // Paths and contents both substituted; no repo state involved.
    let main_rs = target.join("crates/app/my-tool/src/main.rs");
    let main_rs = std::fs::read_to_string(main_rs).unwrap();
    assert!(main_rs.contains("my_tool_core::"), "{main_rs}");
    assert!(
        !dir.path().join(".superdev").exists(),
        "render is read-only"
    );

    // A second render into the same directory refuses: leftovers would read
    // as part of the render to whoever diffs against it.
    let out = superdev()
        .args(["template", "render", "rust-npm", "--name", "My Tool"])
        .arg("--dir")
        .arg(&target)
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("not empty"), "{stderr}");
}

#[test]
fn template_render_refuses_an_unknown_template() {
    let dir = tempfile::tempdir().unwrap();
    let out = superdev()
        .args(["template", "render", "flying", "--name", "x"])
        .arg("--dir")
        .arg(dir.path().join("out"))
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains("template must be one of: rust-npm"),
        "{stderr}"
    );
    assert!(!dir.path().join("out").join("LICENSE").exists());
}

/// SOKF is part of superdev, so every `init` seeds it and every `init`
/// prints the hint — there is no flag combination that suppresses either.
#[test]
fn init_always_hints_at_bootstrap() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("knowledge: run /bootstrap in Claude Code"),
        "{stdout}"
    );

    // Every other capability off changes nothing: the hint rides with SOKF.
    let bare = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(bare.path().join(".git")).unwrap();
    let out = superdev()
        .current_dir(bare.path())
        .args(["init", "--no-frontend", "--no-skills", "--no-code-index"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("knowledge: run /bootstrap in Claude Code"),
        "{stdout}"
    );
    assert!(bare.path().join("knowledge/index.md").is_file());
}

/// A repo whose knowledge carries the one fault `--fix` exists for: a body
/// link that names a concept by path.
fn path_link_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let k = dir.path().join("knowledge");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::write(
        k.join("manifest.sokf.yaml"),
        "sokf: \"0.4\"\nname: fixture\n",
    )
    .unwrap();
    std::fs::write(
        k.join("alpha.md"),
        "---\ntype: Note\nid: alpha\n---\n\nAlpha cites [beta](beta.md).\n",
    )
    .unwrap();
    std::fs::write(
        k.join("beta.md"),
        "---\ntype: Note\nid: beta\n---\n\nBody.\n",
    )
    .unwrap();
    dir
}

/// `--fix` converts the tree and names what it wrote; a second run has
/// nothing to do (NFR-4).
#[test]
fn validate_fix_converts_and_settles() {
    let repo = path_link_repo();
    let alpha = repo.path().join("knowledge/alpha.md");
    let out = superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("repaired: 1 file(s)"), "{stdout}");
    assert!(stdout.contains("knowledge/alpha.md"), "{stdout}");

    let text = std::fs::read_to_string(&alpha).unwrap();
    assert!(text.contains("[beta][sokf:beta]"), "{text}");
    assert!(text.contains("[sokf:beta]: /knowledge/beta.md"), "{text}");

    let out = superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("repaired: 0 file(s)"), "{stdout}");
    assert_eq!(std::fs::read_to_string(&alpha).unwrap(), text);
}

/// Covers I049 criteria 1, 2 and 4 (ADR-041): an include naming a source
/// region is written by `--fix` as a fenced block tagged by the extension, a
/// second run writes nothing, and an edit inside the region fails `validate`
/// naming the path and the region.
#[test]
fn validate_fix_materializes_a_source_include_and_validate_catches_its_drift() {
    let repo = tempfile::tempdir().unwrap();
    let k = repo.path().join("knowledge");
    std::fs::create_dir_all(&k).unwrap();
    std::fs::create_dir_all(repo.path().join("src")).unwrap();
    std::fs::write(
        k.join("manifest.sokf.yaml"),
        "sokf: \"0.4\"\nname: fixture\n",
    )
    .unwrap();
    let main_rs = repo.path().join("src/main.rs");
    std::fs::write(
        &main_rs,
        "use clap::Parser;\n\n// sokf:begin cli\n#[derive(Parser)]\nstruct Cli {}\n// sokf:end cli\n\nfn main() {}\n",
    )
    .unwrap();
    let contract = k.join("contract.md");
    std::fs::write(
        &contract,
        "---\ntype: Note\nid: contract\n---\n\n<!-- sokf:include /src/main.rs#cli -->\n<!-- /sokf:include -->\n",
    )
    .unwrap();

    let out = superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("repaired: 1 file(s)"), "{stdout}");
    let text = std::fs::read_to_string(&contract).unwrap();
    assert!(
        text.contains(
            "<!-- sokf:include /src/main.rs#cli -->\n```rust\n#[derive(Parser)]\nstruct Cli {}\n```\n<!-- /sokf:include -->\n"
        ),
        "{text}"
    );

    let out = superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("repaired: 0 file(s)"), "{stdout}");
    assert_eq!(std::fs::read_to_string(&contract).unwrap(), text);

    let edited = std::fs::read_to_string(&main_rs)
        .unwrap()
        .replace("struct Cli {}", "struct Cli { fix: bool }");
    std::fs::write(&main_rs, edited).unwrap();
    let out = superdev()
        .current_dir(repo.path())
        .args(["validate"])
        .assert()
        .code(1);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("the include block for `/src/main.rs#cli` is stale"),
        "{stdout}"
    );
}

/// Covers I049 criteria 2, 4 and 23 (ADR-042): the CLI contract's
/// definition is the `cli` regions, so a flag added to `ValidateArgs` fails
/// `validate` naming the contract's include, and `--fix` writes the flag
/// into the contract. Runs against a copy of the live contract and its
/// sources in a scratch repository, so this tree is never edited.
#[test]
fn a_flag_added_to_validate_args_fails_validate_and_fix_writes_it_into_the_contract() {
    const CONTRACT: &str = "knowledge/contracts/public/active/contract-002-cli-superdev.md";
    const SOURCES: [&str; 5] = [
        "crates/app/superdev/src/main.rs",
        "crates/app/superdev/src/manage.rs",
        "crates/app/superdev/src/validate_cli.rs",
        "crates/app/superdev/src/sokf_cli.rs",
        "crates/app/superdev/src/run.rs",
    ];
    let repo = tempfile::tempdir().unwrap();
    for rel in SOURCES.iter().chain([CONTRACT].iter()) {
        let to = repo.path().join(rel);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(Path::new(REPO_ROOT).join(rel), &to).unwrap();
    }
    std::fs::write(
        repo.path().join("knowledge/manifest.sokf.yaml"),
        "sokf: \"0.4\"\nname: fixture\n",
    )
    .unwrap();
    // The fixture carries the live contract's Definition — its includes —
    // and nothing else: the prose links concepts the fixture lacks.
    let contract = repo.path().join(CONTRACT);
    let live = std::fs::read_to_string(&contract).unwrap();
    let definition = &live[live.find("## Definition").unwrap()..live.find("## Behaviour").unwrap()];
    assert_eq!(
        definition.matches("<!-- sokf:include /crates/").count(),
        SOURCES.len(),
        "the live contract includes one region per source file"
    );
    std::fs::write(
        &contract,
        format!("---\ntype: Note\nid: contract-002-cli-superdev\n---\n\n# CLI\n\n{definition}"),
    )
    .unwrap();
    superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();

    let source = repo.path().join("crates/app/superdev/src/validate_cli.rs");
    let edited = std::fs::read_to_string(&source).unwrap().replace(
        "    /// Emit JSON instead of text\n",
        "    /// Do nothing new\n    #[arg(long)]\n    pub nothing: bool,\n    /// Emit JSON instead of text\n",
    );
    assert!(edited.contains("pub nothing: bool"), "the flag was added");
    std::fs::write(&source, edited).unwrap();

    let out = superdev()
        .current_dir(repo.path())
        .args(["validate"])
        .assert()
        .code(1);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains(
            "contract-002-cli-superdev.md: the include block for `/crates/app/superdev/src/validate_cli.rs#cli` is stale"
        ),
        "{stdout}"
    );
    assert!(
        !std::fs::read_to_string(&contract)
            .unwrap()
            .contains("pub nothing: bool"),
        "validate without --fix wrote the contract"
    );

    superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();
    let text = std::fs::read_to_string(&contract).unwrap();
    assert!(
        text.contains("    /// Do nothing new\n    #[arg(long)]\n    pub nothing: bool,\n"),
        "{text}"
    );
}

/// Covers I049 criteria 4 and 23 (ADR-042): the lock format contract's
/// definition is the `lock` regions of `lock.rs`, so a field renamed in the
/// lock struct fails `validate` naming the contract's include, and `--fix`
/// writes the new name into the contract. Runs against a copy of the live
/// contract and its source in a scratch repository, so this tree is never
/// edited.
#[test]
fn a_field_renamed_in_the_lock_struct_fails_validate_naming_the_contracts_include() {
    const CONTRACT: &str = "knowledge/contracts/public/active/contract-006-format-lock.md";
    const SOURCE: &str = "crates/lib/superdev-core/src/lock.rs";
    let repo = tempfile::tempdir().unwrap();
    for rel in [SOURCE, CONTRACT] {
        let to = repo.path().join(rel);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(Path::new(REPO_ROOT).join(rel), &to).unwrap();
    }
    std::fs::write(
        repo.path().join("knowledge/manifest.sokf.yaml"),
        "sokf: \"0.4\"\nname: fixture\n",
    )
    .unwrap();
    // The fixture carries the live contract's Definition — its include —
    // and nothing else: the prose links concepts the fixture lacks.
    let contract = repo.path().join(CONTRACT);
    let live = std::fs::read_to_string(&contract).unwrap();
    let definition = &live[live.find("## Definition").unwrap()..live.find("## Behaviour").unwrap()];
    assert_eq!(
        definition.matches("<!-- sokf:include /crates/").count(),
        1,
        "the live contract includes the lock regions as one"
    );
    assert!(
        definition.contains("pub struct PackLock") && definition.contains("pub struct Lock "),
        "the include carries both lock regions"
    );
    std::fs::write(
        &contract,
        format!("---\ntype: Note\nid: contract-006-format-lock\n---\n\n# Lock\n\n{definition}"),
    )
    .unwrap();
    superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();

    let source = repo.path().join(SOURCE);
    let edited = std::fs::read_to_string(&source).unwrap().replace(
        "    pub digest: Option<String>,\n",
        "    pub checksum: Option<String>,\n",
    );
    assert!(edited.contains("pub checksum"), "the field was renamed");
    std::fs::write(&source, edited).unwrap();

    let out = superdev()
        .current_dir(repo.path())
        .args(["validate"])
        .assert()
        .code(1);
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains(
            "contract-006-format-lock.md: the include block for `/crates/lib/superdev-core/src/lock.rs#lock` is stale"
        ),
        "{stdout}"
    );
    assert!(
        !std::fs::read_to_string(&contract)
            .unwrap()
            .contains("pub checksum"),
        "validate without --fix wrote the contract"
    );

    superdev()
        .current_dir(repo.path())
        .args(["validate", "--fix"])
        .assert()
        .success();
    let text = std::fs::read_to_string(&contract).unwrap();
    assert!(
        text.contains("    pub checksum: Option<String>,\n") && !text.contains("pub digest"),
        "{text}"
    );
}

/// Without the flag, `validate` reports and writes nothing.
#[test]
fn validate_without_fix_writes_nothing() {
    let repo = path_link_repo();
    let alpha = repo.path().join("knowledge/alpha.md");
    let before = std::fs::read_to_string(&alpha).unwrap();
    let _ = superdev()
        .current_dir(repo.path())
        .args(["validate"])
        .assert();
    assert_eq!(std::fs::read_to_string(&alpha).unwrap(), before);
}

/// The hook never repairs (D-7): it fires after an edit, so a hook that wrote
/// would rewrite the file the agent is still working in.
#[test]
fn hook_validate_writes_no_file() {
    let repo = path_link_repo();
    let alpha = repo.path().join("knowledge/alpha.md");
    let before = std::fs::read_to_string(&alpha).unwrap();
    let _ = superdev()
        .args(["hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert();
    assert_eq!(
        std::fs::read_to_string(&alpha).unwrap(),
        before,
        "the hook repaired a file"
    );
}
