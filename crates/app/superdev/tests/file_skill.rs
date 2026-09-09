//! Filing remains a native skill, including upgrades from managed legacy assets.
use std::fs;

use assert_cmd::Command;
use superdev_core::lock::sha256_hex;

#[test]
fn the_removed_filing_cli_is_not_an_entry_point() {
    let output = Command::cargo_bin("superdev")
        .unwrap()
        .args(["file", "--help"])
        .assert()
        .code(2)
        .get_output()
        .clone();
    assert!(String::from_utf8_lossy(&output.stderr).contains("unrecognized subcommand 'file'"));
}

#[test]
fn sync_retires_managed_issue_skill_and_filing_prompt() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir(root.join(".git")).unwrap();
    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .args(["init", "--no-frontend", "--no-code-index"])
        .assert()
        .success();

    let retired = [
        ".pi/skills/issue/SKILL.md",
        ".pi/skills/file/SKILL.md",
        ".pi/skills/scope/SKILL.md",
        ".pi/skills/build/SKILL.md",
        ".pi/skills/accept/SKILL.md",
        ".pi/extensions/superdev/prompts/file.md",
    ];
    let body = "retired managed filing asset\n";
    let hash = sha256_hex(body.as_bytes());
    let lock_path = root.join(".superdev/lock.toml");
    let mut lock = fs::read_to_string(&lock_path).unwrap();
    for path in retired {
        let target = root.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, body).unwrap();
        lock = lock.replacen(
            "[files]\n",
            &format!("[files]\n\"{path}\" = \"{hash}\"\n"),
            1,
        );
    }
    fs::write(&lock_path, lock).unwrap();

    Command::cargo_bin("superdev")
        .unwrap()
        .current_dir(root)
        .arg("sync")
        .assert()
        .success();
    let lock = fs::read_to_string(lock_path).unwrap();
    for path in retired {
        assert!(!root.join(path).exists(), "retired asset remains: {path}");
        assert!(!lock.contains(path), "retired asset remains owned: {path}");
    }
    let file = ".pi/extensions/superdev/skills/file/SKILL.md";
    assert!(root.join(file).is_file());
    assert!(lock.contains(file));
}
