use super::*;

#[test]
fn completion_evidence_is_service_owned_and_recoverable() {
    let plan = "## Completion evidence\n\nScope evidence.\n\n<!-- sokf:links -->\n";
    let edit = completion_evidence_edit(
        plan,
        &[
            "Candidate revision: abcdef1.".into(),
            "Verified default revision: 1234567.".into(),
        ],
    )
    .unwrap();
    let changed = plan.replace(&edit.old_text, &edit.new_text);
    assert_eq!(
        evidence_revision(&changed, "Candidate revision").as_deref(),
        Some("abcdef1")
    );
    assert_eq!(
        evidence_revision(&changed, "Verified default revision").as_deref(),
        Some("1234567")
    );
}

#[test]
fn block_completion_is_scoped_to_the_requested_stable_block() {
    let plan = "### Block 1: first\n\n- [x] Done.\n\n### Block 2: second\n\n- [ ] Done.\n\n## Build state\n";
    assert!(block_is_done(plan, 1));
    assert!(!block_is_done(plan, 2));
    assert!(!block_is_done(plan, 3));
}

#[test]
fn block_scope_and_dependencies_are_derived_from_the_current_block() {
    let plan = "### Block 2: build\n\n- [ ] Done.\n- Dependencies: Blocks 1 and 3.\n- Areas: `src/workflow` and `knowledge/contracts/a.md`.\n- Verification: `cargo test -p app`.\n\n## Build state\n";
    let block = plan_block(plan, 2).unwrap();
    assert_eq!(block_dependencies(block), vec![1, 3]);
    assert_eq!(
        block_area_paths(block),
        vec!["src/workflow", "knowledge/contracts/a.md"]
    );
    assert_eq!(plan_verification_commands(block), vec!["cargo test -p app"]);
}

#[test]
fn retry_state_round_trips_legacy_and_fingerprinted_lines() {
    let legacy =
        parse_retry_state("Current block: 2. Attempts: 1. Final corrections: 3. Blocker: none.")
            .unwrap();
    assert_eq!(legacy.current_block, 2);
    assert_eq!(legacy.fingerprint, None);
    let rendered = render_retry_state(&RetryState {
        fingerprint: Some("abc123".into()),
        blocker: "retrying".into(),
        ..legacy
    });
    assert_eq!(parse_retry_state(&rendered).unwrap().attempts, 1);
    assert!(rendered.contains("Fingerprint: abc123."));
}

#[test]
fn successful_checkpoint_advances_and_resets_only_build_retry_state() {
    let plan = "### Block 1: first\n\n- [x] Done.\n\n### Block 2: second\n\n- [x] Done.\n\n### Block 3: third\n\n- [ ] Done.\n\n## Build state\n";
    let previous = RetryState {
        current_block: 1,
        attempts: 3,
        final_corrections: 2,
        fingerprint: Some("failure".into()),
        blocker: "stalled".into(),
    };
    let reset = retry_state_after_checkpoint(plan, &previous);
    assert_eq!(reset.current_block, 3);
    assert_eq!(reset.attempts, 0);
    assert_eq!(reset.final_corrections, 2);
    assert_eq!(reset.fingerprint, None);
    assert_eq!(reset.blocker, "none");
}

#[test]
fn acceptance_rejection_invalidates_only_final_evidence() {
    let plan = "## Completion evidence\n\nScope requirements review: clean.\n\nCandidate revision: abcdef1.\n\nVerified default revision: 1234567.\n\nFinal verification: passed for abcdef1.\n\nDocumentation verification: passed for abcdef1.\n\nFinal review: clean for abcdef1 by isolated session review.\n\n## Follow-up\n";
    let edit = invalidate_final_evidence_edit(plan).unwrap();
    let changed = plan.replace(&edit.old_text, &edit.new_text);
    assert!(changed.contains("Scope requirements review: clean."));
    assert!(!changed.contains("Candidate revision:"));
    assert!(!changed.contains("Final review:"));
    assert!(changed.contains("## Follow-up"));
}

#[test]
fn rescope_invalidates_prior_scope_evidence_and_replaces_the_product_baseline() {
    let plan = "## Completion evidence\n\nScope requirements review: clean by isolated session old.\n\nHuman scope approval: approved.\n\nScope product baseline: 1111111.\n\nBlock evidence remains.\n\n## Follow-up\n";
    let edit =
        invalidate_scope_evidence_edit(plan, "Scope product baseline: 2222222.", false).unwrap();
    let changed = plan.replace(&edit.old_text, &edit.new_text);
    assert!(!changed.contains("isolated session old"));
    assert!(!changed.contains("Human scope approval"));
    assert!(!changed.contains("1111111"));
    assert!(changed.contains("Scope product baseline: 2222222."));
    assert!(changed.contains("Block evidence remains."));
}

#[test]
fn unresolved_discoveries_are_limited_to_the_discovery_section() {
    let dir = tempfile::tempdir().unwrap();
    let issues = dir.path().join("knowledge/issues/open");
    fs::create_dir_all(&issues).unwrap();
    fs::write(
        issues.join("issue-001-test.md"),
        "## Discoveries\n\n- [x] settled\n\n## Comments\n\n- [ ] not a discovery\n",
    )
    .unwrap();
    assert!(!primary_issue_has_unresolved_discoveries(dir.path(), "issue-001-test").unwrap());
    fs::write(
        issues.join("issue-001-test.md"),
        "## Discoveries\n\n- [ ] unresolved\n\n## Comments\n\nnone\n",
    )
    .unwrap();
    assert!(primary_issue_has_unresolved_discoveries(dir.path(), "issue-001-test").unwrap());
}

#[test]
fn rejection_feedback_becomes_an_unresolved_discovery() {
    let issue = "## Behaviour\n\nExpected.\n\n## Comments\n\nPrior.\n";
    let edit =
        rescope_discovery_edit(issue, "ACCEPT rejection", "First line\nsecond line").unwrap();
    let changed = issue.replace(&edit.old_text, &edit.new_text);
    assert!(
        changed.contains(
            "## Discoveries\n\n- [ ] ACCEPT rejection: First line\n\n      second line\n"
        )
    );
    assert!(changed.find("## Discoveries").unwrap() < changed.find("## Comments").unwrap());
}

#[test]
fn rejection_appends_without_erasing_existing_discoveries() {
    let issue = "## Discoveries\n\n- [x] Existing.\n\n## Comments\n\nnone.\n";
    let edit = rescope_discovery_edit(issue, "ACCEPT rejection", "Rejected because X").unwrap();
    let changed = issue.replace(&edit.old_text, &edit.new_text);
    assert!(
        changed.contains("- [x] Existing.\n- [ ] ACCEPT rejection: Rejected because X"),
        "{changed}"
    );
}

#[test]
fn verification_commands_are_extracted_only_from_executable_entries() {
    let plan = "- Outcome: ignore `not-a-command`.\n- Verification: `cargo test -p one` and `npm test`.\n- Verification: prose only.\n";
    assert_eq!(
        plan_verification_commands(plan),
        vec!["cargo test -p one", "npm test"]
    );
}

#[test]
fn malformed_or_absent_evidence_is_not_recovered() {
    assert!(evidence_revision("Candidate revision: not-a-sha.", "Candidate revision").is_none());
    assert!(evidence_revision("none.", "Candidate revision").is_none());
}
