use super::*;

pub(super) fn command(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {args:?}");
}

pub(super) fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    command(dir.path(), &["init", "-q", "-b", "main"]);
    command(dir.path(), &["config", "user.email", "test@example.com"]);
    command(dir.path(), &["config", "user.name", "Test"]);
    command(dir.path(), &["config", "commit.gpgsign", "false"]);
    command(dir.path(), &["config", "merge.gpgsign", "false"]);
    std::fs::write(dir.path().join("file"), "base\n").unwrap();
    command(dir.path(), &["add", "file"]);
    command(dir.path(), &["commit", "-q", "-m", "base"]);
    dir
}

#[test]
fn canonical_knowledge_commit_refuses_unrelated_changes() {
    let dir = repository();
    std::fs::create_dir_all(dir.path().join("knowledge")).unwrap();
    std::fs::write(dir.path().join("knowledge/plan.md"), "plan\n").unwrap();
    let revision = commit_knowledge_changes(dir.path(), "chore: record evidence").unwrap();
    assert_eq!(revision, super::revision(dir.path(), "HEAD").unwrap());
    assert!(
        String::from_utf8_lossy(
            &git(dir.path(), &["show", "--format=", "--name-only", "HEAD"])
                .unwrap()
                .stdout
        )
        .contains("knowledge/plan.md")
    );

    std::fs::write(dir.path().join("knowledge/plan.md"), "changed\n").unwrap();
    std::fs::write(dir.path().join("unrelated"), "must remain\n").unwrap();
    assert!(commit_knowledge_changes(dir.path(), "chore: unsafe").is_err());
    assert!(
        !git(dir.path(), &["status", "--porcelain=v1"])
            .unwrap()
            .stdout
            .is_empty()
    );
}

#[test]
fn scope_baseline_is_recovered_from_transition_history_not_plan_prose() {
    let dir = repository();
    let root = dir.path();
    let baseline = revision(root, "HEAD").unwrap();
    command(root, &["switch", "-q", "-c", "work/001-history"]);
    std::fs::create_dir_all(root.join("knowledge/plans/open")).unwrap();
    let plan = root.join("knowledge/plans/open/plan-001-history.md");
    std::fs::write(&plan, "phase: scope\n").unwrap();
    command(root, &["add", "knowledge"]);
    command(
        root,
        &["commit", "-q", "-m", "chore(workflow): start scope"],
    );
    std::fs::write(root.join("file"), "forbidden product\n").unwrap();
    std::fs::write(&plan, "phase: scope\nScope product baseline: HEAD.\n").unwrap();
    command(root, &["add", "."]);
    command(
        root,
        &["commit", "-q", "-m", "chore(workflow): return to scope"],
    );

    assert_eq!(
        scope_base_from_history(root, "work/001-history", &plan).unwrap(),
        baseline
    );

    let checkpoint_parent = revision(root, "HEAD").unwrap();
    std::fs::write(
        &plan,
        "phase: scope\nScope product baseline: current attempt.\n",
    )
    .unwrap();
    command(root, &["add", "knowledge"]);
    command(root, &["commit", "-q", "-m", SCOPE_CHECKPOINT_MESSAGE]);
    assert_eq!(
        scope_base_from_history(root, "work/001-history", &plan).unwrap(),
        checkpoint_parent
    );
}

#[test]
fn expected_parent_refuses_a_racing_commit_without_publishing() {
    let dir = repository();
    let root = dir.path();
    let stale_parent = revision(root, "HEAD").unwrap();
    std::fs::create_dir_all(root.join("knowledge")).unwrap();
    std::fs::write(root.join("knowledge/plan.md"), "first\n").unwrap();
    let current = commit_knowledge_changes(root, "docs: concurrent change").unwrap();
    std::fs::write(root.join("knowledge/plan.md"), "approval\n").unwrap();

    assert!(commit_knowledge_changes_at(root, "docs: approve", &stale_parent).is_err());
    assert_eq!(revision(root, "HEAD").unwrap(), current);
    assert_eq!(
        std::fs::read_to_string(root.join("knowledge/plan.md")).unwrap(),
        "approval\n"
    );
}

#[test]
fn failed_commit_construction_preserves_head_index_and_worktree() {
    let dir = repository();
    let root = dir.path();
    let head = revision(root, "HEAD").unwrap();
    std::fs::create_dir_all(root.join("knowledge")).unwrap();
    std::fs::write(root.join("knowledge/record.md"), "pending\n").unwrap();
    command(root, &["config", "user.name", ""]);
    command(root, &["config", "user.email", ""]);

    assert!(commit_knowledge_changes(root, "chore: must fail").is_err());
    assert_eq!(revision(root, "HEAD").unwrap(), head);
    assert_eq!(
        std::fs::read_to_string(root.join("knowledge/record.md")).unwrap(),
        "pending\n"
    );
    assert!(
        Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn block_commit_accepts_declared_product_and_knowledge_paths() {
    let dir = repository();
    let root = dir.path();
    std::fs::create_dir_all(root.join("knowledge/plans/open")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("knowledge/plans/open/plan.md"), "done\n").unwrap();
    std::fs::write(root.join("src/lib.rs"), "pub fn built() {}\n").unwrap();
    let areas = vec!["knowledge/plans".into(), "src".into()];
    commit_block_changes(root, "feat: checkpoint block", &areas).unwrap();
    require_clean(root).unwrap();
}

#[test]
fn block_commit_refuses_and_preserves_out_of_scope_changes() {
    let dir = repository();
    let root = dir.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/lib.rs"), "pub fn built() {}\n").unwrap();
    std::fs::write(root.join("unrelated"), "leave me\n").unwrap();
    assert!(commit_block_changes(root, "feat: unsafe", &["src".into()]).is_err());
    assert!(root.join("unrelated").exists());
    assert!(
        !git(root, &["status", "--porcelain=v1"])
            .unwrap()
            .stdout
            .is_empty()
    );
}

#[test]
fn work_branch_validation_is_strict() {
    assert!(validate_work_branch("work/059-scope-build-accept").is_ok());
    for bad in [
        "main",
        "work/59-x",
        "work/059-X",
        "work/059-../main",
        "--help",
    ] {
        assert!(validate_work_branch(bad).is_err(), "{bad}");
    }
}

#[test]
fn synchronization_merges_expected_tips_without_exposing_conflicts() {
    let dir = repository();
    let root = dir.path();
    create_work_branch(root, "work/059-test").unwrap();
    std::fs::write(root.join("work-file"), "work\n").unwrap();
    command(root, &["add", "work-file"]);
    command(root, &["commit", "-q", "-m", "work"]);
    let work = revision(root, "work/059-test").unwrap();
    command(root, &["switch", "-q", "main"]);
    std::fs::write(root.join("default-file"), "default\n").unwrap();
    command(root, &["add", "default-file"]);
    command(root, &["commit", "-q", "-m", "default"]);
    let default = revision(root, "main").unwrap();
    command(root, &["switch", "-q", "work/059-test"]);

    let synchronized = synchronize_default(root, "main", &default, "work/059-test", &work).unwrap();
    assert_eq!(revision(root, "work/059-test").unwrap(), synchronized);
    assert!(is_ancestor(root, &default, &synchronized).unwrap());
    assert!(root.join("default-file").is_file());
    require_clean(root).unwrap();

    command(root, &["switch", "-q", "main"]);
    std::fs::write(root.join("file"), "default conflict\n").unwrap();
    command(root, &["commit", "-q", "-am", "default conflict"]);
    let conflicting_default = revision(root, "main").unwrap();
    command(root, &["switch", "-q", "work/059-test"]);
    std::fs::write(root.join("file"), "work conflict\n").unwrap();
    command(root, &["commit", "-q", "-am", "work conflict"]);
    let conflicting_work = revision(root, "work/059-test").unwrap();

    assert!(
        synchronize_default(
            root,
            "main",
            &conflicting_default,
            "work/059-test",
            &conflicting_work,
        )
        .is_err()
    );
    assert_eq!(revision(root, "work/059-test").unwrap(), conflicting_work);
    require_clean(root).unwrap();
}

#[test]
fn synchronization_refuses_dirty_or_stale_tips_without_moving_the_branch() {
    let dir = repository();
    let root = dir.path();
    let default = revision(root, "main").unwrap();
    create_work_branch(root, "work/059-test").unwrap();
    std::fs::write(root.join("work-file"), "work\n").unwrap();
    command(root, &["add", "work-file"]);
    command(root, &["commit", "-q", "-m", "work"]);
    let work = revision(root, "work/059-test").unwrap();

    std::fs::write(root.join("dirty"), "leave this alone\n").unwrap();
    assert!(synchronize_default(root, "main", &default, "work/059-test", &work).is_err());
    assert_eq!(revision(root, "work/059-test").unwrap(), work);
    assert!(root.join("dirty").is_file());
    std::fs::remove_file(root.join("dirty")).unwrap();

    assert!(synchronize_default(root, "main", &work, "work/059-test", &work).is_err());
    assert!(synchronize_default(root, "main", &default, "work/059-test", &default).is_err());
    assert_eq!(revision(root, "work/059-test").unwrap(), work);
    require_clean(root).unwrap();
}

#[test]
fn integration_creates_a_no_ff_merge_commit() {
    let dir = repository();
    let root = dir.path();
    let default = revision(root, "main").unwrap();
    create_work_branch(root, "work/059-test").unwrap();
    std::fs::write(root.join("file"), "work\n").unwrap();
    command(root, &["commit", "-q", "-am", "work"]);
    let work = revision(root, "work/059-test").unwrap();
    let hook = root.join(".git/hooks/pre-merge-commit");
    std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&hook).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&hook, permissions).unwrap();
    }
    command(root, &["config", "commit.gpgsign", "true"]);
    command(root, &["switch", "-q", "main"]);

    integrate_no_ff(root, "main", &default, "work/059-test", &work).unwrap();
    assert_eq!(current_branch(root).unwrap(), "main");
    assert_eq!(
        std::fs::read_to_string(root.join("file")).unwrap(),
        "work\n"
    );

    let parents = git(root, &["rev-list", "--parents", "-n", "1", "HEAD"]).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&parents.stdout)
            .split_whitespace()
            .count(),
        3
    );
}

#[test]
fn integration_refuses_a_default_checked_out_in_another_worktree() {
    let dir = repository();
    let root = dir.path();
    let default = revision(root, "main").unwrap();
    create_work_branch(root, "work/059-test").unwrap();
    std::fs::write(root.join("file"), "work\n").unwrap();
    command(root, &["commit", "-q", "-am", "work"]);
    let work = revision(root, "work/059-test").unwrap();
    let linked_parent = tempfile::tempdir().unwrap();
    let linked = linked_parent.path().join("default");
    command(
        root,
        &["worktree", "add", "-q", linked.to_str().unwrap(), "main"],
    );

    assert!(integrate_no_ff(root, "main", &default, "work/059-test", &work).is_err());
    assert_eq!(revision(root, "main").unwrap(), default);
    assert_eq!(revision(root, "work/059-test").unwrap(), work);
    command(
        root,
        &["worktree", "remove", "--force", linked.to_str().unwrap()],
    );
}

#[test]
fn integration_refuses_dirty_or_stale_tips() {
    let dir = repository();
    let root = dir.path();
    let default = revision(root, "main").unwrap();
    create_work_branch(root, "work/059-test").unwrap();
    std::fs::write(root.join("file"), "work\n").unwrap();
    command(root, &["commit", "-q", "-am", "work"]);
    let work = revision(root, "work/059-test").unwrap();
    command(root, &["switch", "-q", "main"]);

    std::fs::write(root.join("unrelated"), "dirty\n").unwrap();
    assert!(integrate_no_ff(root, "main", &default, "work/059-test", &work).is_err());
    std::fs::remove_file(root.join("unrelated")).unwrap();
    assert!(integrate_no_ff(root, "main", "0000000", "work/059-test", &work).is_err());
    assert!(integrate_no_ff(root, "main", &default, "work/059-test", &default).is_err());
}
