//! End-to-end tests for the skeleton CLI: the real binary, real exit codes.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc;
use std::time::Duration;

use assert_cmd::Command;

/// The repository root: where `aokf validate` finds the live `knowledge/`.
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
fn aokf_validate_passes_the_live_bundle() {
    superdev()
        // workspace root, so `aokf validate` resolves <cwd>/knowledge
        .current_dir(REPO_ROOT)
        .args(["aokf", "validate"])
        .assert()
        .success();
}

#[test]
fn aokf_validate_fails_a_broken_bundle_with_exit_1() {
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
    superdev()
        .current_dir(dir.path())
        .args(["aokf", "validate", "kb"])
        .assert()
        .code(1);
}

#[test]
fn aokf_validate_json_is_machine_readable() {
    let out = superdev()
        .current_dir(REPO_ROOT)
        .args(["aokf", "validate", "--json"])
        .assert()
        .success();
    let report: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    // Zero findings, warnings included: the bundle's own links use only core
    // rels, which S013 made true by promoting `implements`.
    assert_eq!(report["findings"], serde_json::json!([]), "{report}");
    assert_eq!(report["passed"], serde_json::json!(true));
    // The bundle path is the CLI's to add: core omits it.
    let bundle = report["bundle"].as_str().unwrap();
    assert!(bundle.ends_with("knowledge"), "unexpected bundle: {bundle}");
}

#[test]
fn aokf_index_rebuilds_and_reports_lexical_only() {
    let dir = tempfile::tempdir().unwrap();
    write_fixture_bundle(dir.path());
    // A manifest with no `[knowledge.embeddings]` table: the embedder comes
    // from the local default, which the blocked cache stops loading.
    std::fs::create_dir(dir.path().join(".superdev")).unwrap();
    std::fs::write(
        dir.path().join(".superdev/config.toml"),
        "blueprint = \"0.1.0\"\n\n[knowledge]\nprovider = \"aokf\"\n",
    )
    .unwrap();
    // A file that cannot be parsed leaves the bundle silently, so `index` says so.
    std::fs::write(dir.path().join("knowledge/bad.md"), "no frontmatter here\n").unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", blocked_model_cache(dir.path()))
        .args(["aokf", "index"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(stdout.contains("1 concept"), "no concept count: {stdout}");
    assert!(stdout.contains("lexical"), "no lexical-only note: {stdout}");
    assert!(
        stdout.contains("skipped 1"),
        "no broken-file note: {stdout}"
    );
    assert!(dir.path().join(".superdev/cache/aokf-index").is_dir());
}

#[test]
fn mcp_without_a_bundle_fails_at_startup() {
    let dir = tempfile::tempdir().unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .args(["mcp", "aokf"])
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(stderr.contains("no AOKF bundle"), "unexpected: {stderr}");
}

#[test]
fn mcp_with_an_unusable_index_dir_fails_at_startup() {
    let dir = tempfile::tempdir().unwrap();
    write_fixture_bundle(dir.path());
    // A file where the index directory belongs: the startup sync cannot write it.
    std::fs::create_dir_all(dir.path().join(".superdev/cache")).unwrap();
    std::fs::write(dir.path().join(".superdev/cache/aokf-index"), "").unwrap();
    superdev()
        .current_dir(dir.path())
        .env("XDG_CACHE_HOME", blocked_model_cache(dir.path()))
        .args(["mcp", "aokf"])
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
    r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"aokf_overview","arguments":{}}}"#,
    "\n",
);

#[test]
fn mcp_server_initialises_over_stdio() {
    let dir = tempfile::tempdir().unwrap();
    write_fixture_bundle(dir.path());
    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin("superdev"))
        .args(["mcp", "aokf"])
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
        "no bundle name in: {replies}"
    );
}

/// A one-concept bundle under `<dir>/knowledge`.
fn write_fixture_bundle(dir: &Path) {
    let bundle = dir.join("knowledge");
    std::fs::create_dir(&bundle).unwrap();
    std::fs::write(
        bundle.join("manifest.aokf.yaml"),
        "aokf: \"0.1\"\nname: fixture-knowledge\n",
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
        k.join("manifest.aokf.yaml"),
        "aokf: \"0.1\"\nname: fixture\n",
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
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(2);
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains("AOKF validation failed after editing"),
        "stderr: {stderr}"
    );
    assert!(stderr.contains("alpha.md"), "stderr: {stderr}");
}

#[test]
fn hook_validate_passes_a_clean_bundle() {
    let repo = hook_repo(true);
    superdev()
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "knowledge/alpha.md"))
        .assert()
        .code(0);
}

#[test]
fn hook_validate_ignores_paths_outside_the_bundle() {
    // Even a broken bundle: an edit elsewhere is not the hook's business.
    let repo = hook_repo(false);
    superdev()
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(hook_payload(repo.path(), "src/main.rs"))
        .assert()
        .code(0);
}

#[test]
fn hook_validate_falls_back_to_the_working_directory() {
    let repo = hook_repo(false);
    superdev()
        .current_dir(repo.path())
        .args(["aokf", "hook", "validate"])
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
        .args(["aokf", "hook", "validate"])
        .env_remove("CLAUDE_PROJECT_DIR")
        .write_stdin(hook_payload(&link, "knowledge/alpha.md"))
        .assert()
        .code(2);
}

#[test]
fn hook_validate_is_loud_on_a_malformed_payload() {
    let repo = hook_repo(true);
    let out = superdev()
        .args(["aokf", "hook", "validate"])
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
        .args(["aokf", "hook", "validate"])
        .env("CLAUDE_PROJECT_DIR", repo.path())
        .write_stdin(r#"{"tool_input":{}}"#)
        .assert()
        .code(0);
}

/// `init` a temp repo with only the skills capability (the others need
/// external binaries; skills needs none, so these tests run everywhere).
fn skills_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
            "--no-knowledge",
        ])
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
                .contains("Project adaptations"),
            "{name} lacks the PROJECT.md trailer"
        );
    }
    // The hook and the lifecycle skills belong to knowledge, disabled here.
    assert!(!dir.path().join(".claude/settings.json").exists());
    assert!(!dir.path().join(".claude/skills/maintain").exists());
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
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
            "--no-knowledge",
        ])
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

#[test]
fn init_no_skills_skips_the_pack() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
            "--no-knowledge",
            "--no-skills",
        ])
        .assert()
        .success();
    assert!(!dir.path().join(".claude/skills").exists());
    assert!(!dir.path().join(".claude/settings.json").exists());
}

#[test]
fn init_ignores_a_cache_left_by_the_knowledge_tools() {
    // `superdev mcp aokf` writes .superdev/cache/ in repos it never initialised.
    // Only the manifest means initialised.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::create_dir_all(dir.path().join(".superdev/cache/aokf-index")).unwrap();
    superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
            "--no-knowledge",
        ])
        .assert()
        .success();
    assert!(dir.path().join(".superdev/config.toml").is_file());
}

#[test]
fn init_refuses_when_the_manifest_exists() {
    let dir = skills_repo();
    let out = superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
            "--no-knowledge",
        ])
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
            .contains("Project adaptations")
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
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
        ])
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
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
        ])
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

    // The hook belongs to the still-enabled knowledge capability, so it
    // survives the pack's departure — as do the aokf-carried skills.
    let settings: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join(".claude/settings.json")).unwrap())
            .unwrap();
    assert!(
        settings["hooks"]["PostToolUse"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.to_string().contains("superdev aokf hook validate")),
        "settings: {settings}"
    );

    let lock = std::fs::read_to_string(root.join(".superdev/lock.toml")).unwrap();
    assert!(!lock.contains(".claude/skills/double-check/"), "{lock}");
    assert!(!lock.contains(".claude/skills/template-update/"), "{lock}");
    assert!(lock.contains(".claude/skills/maintain/"), "{lock}");
    assert!(lock.contains("hooks.PostToolUse"), "{lock}");
    assert!(!lock.contains("[components.skills]"), "{lock}");
    // The capability still enabled keeps its record: the sweep is targeted.
    assert!(lock.contains(".agents/aokf/SPEC.md"), "{lock}");
    assert!(lock.contains("[components.knowledge]"), "{lock}");

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
            "--no-bash-output-filter",
            "--no-knowledge",
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
        .args([
            "init",
            "--no-frontend",
            "--no-skills",
            "--no-code-index",
            "--no-bash-output-filter",
            "--no-knowledge",
        ])
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

#[test]
fn init_hints_at_bootstrap_only_when_knowledge_is_enabled() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let out = superdev()
        .current_dir(dir.path())
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(
        stdout.contains("knowledge: run /bootstrap in Claude Code"),
        "{stdout}"
    );

    let off = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(off.path().join(".git")).unwrap();
    let out = superdev()
        .current_dir(off.path())
        .args([
            "init",
            "--no-frontend",
            "--no-code-index",
            "--no-bash-output-filter",
            "--no-knowledge",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    assert!(!stdout.contains("/bootstrap"), "{stdout}");
}
