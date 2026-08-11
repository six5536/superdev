//! End-to-end tests for the skeleton CLI: the real binary, real exit codes.

use assert_cmd::Command;

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
