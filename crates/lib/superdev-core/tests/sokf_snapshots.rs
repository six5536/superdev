//! sokf_snapshots.rs — `validate::sokf::validate` over one fixture knowledge
//! tree per failure class, compared to a recorded report.
//!
//! Each `tests/fixtures/sokf/<case>/` tree has a `<case>.golden.json`
//! holding the report this validator produces for it. The goldens began as
//! captures from `.agents/aokf/tools/validator.py`, the Python reference this
//! code replaced, and were projected once when ADR-017 removed the
//! conformance ladder. That history is over: the reference is not the
//! authority here, this validator is, and a golden is simply its recorded
//! output.
//!
//! What they still are is the contract over 40-odd user-facing messages,
//! their severities and the order they arrive in — none of which the inline
//! tests in `validate.rs` pin. Change a message and this suite tells you which
//! reports move.
//!
//! Regenerate with:
//!
//! ```sh
//! UPDATE_GOLDENS=1 cargo test -p superdev-core --test sokf_snapshots
//! ```
//!
//! Then read the diff. A golden that changes because a message was reworded is
//! the diff doing its job; one that changes a severity, a verdict or a
//! finding's presence is a behaviour change, and wants the argument any
//! behaviour change wants.
//!
//! The report carries no knowledge-directory key: [`Report::to_json`] leaves
//! the path to the caller, which holds the string it was given.
//!
//! Every golden is captured with the warnings listed. What a run prints by
//! default is the CLI's decision (ADR-040); these record the findings, and a
//! severity a default run would not print is exactly the one nothing else
//! would catch.
//!
//! Known divergence from the old Python reference, deliberately left: a
//! manifest that parses to a falsy non-mapping (`[]`, `false`). Python coerced
//! it to `{}` and reported the missing `aokf` and `name`; this reports the
//! parse error, or nothing, and returns. No fixture exercises it.

use std::path::{Path, PathBuf};

use superdev_core::sokf::load_bundle;
use superdev_core::validate::sokf::{Warnings, validate};

/// The fixture root.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sokf")
}

/// Validate `case` and compare its report to the golden, or rewrite the golden
/// when `UPDATE_GOLDENS` is set.
fn snapshot(case: &str) {
    let dir = fixtures().join(case);
    let path = dir.with_extension("golden.json");

    let bundle = load_bundle(&dir).unwrap();
    let ours = validate(&bundle, &dir).to_json(Warnings::Listed);

    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        let mut text = serde_json::to_string_pretty(&ours).unwrap();
        text.push('\n');
        std::fs::write(&path, text).unwrap();
        return;
    }

    let golden: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(ours, golden, "{case} does not match its golden");
}

#[test]
fn clean() {
    snapshot("clean");
}

#[test]
fn broken_links() {
    snapshot("broken-links");
}

#[test]
fn duplicate_ids() {
    snapshot("duplicate-ids");
}

#[test]
fn malformed_frontmatter() {
    snapshot("malformed-frontmatter");
}

#[test]
fn missing_type() {
    snapshot("missing-type");
}

#[test]
fn stamped_field() {
    snapshot("stamped-field");
}

#[test]
fn bad_verified() {
    snapshot("bad-verified");
}

#[test]
fn unmirrored_link() {
    snapshot("unmirrored-link");
}

#[test]
fn no_manifest() {
    snapshot("no-manifest");
}

#[test]
fn custom_rel() {
    snapshot("custom-rel");
}

#[test]
fn footnote_mismatch() {
    snapshot("footnote-mismatch");
}

#[test]
fn path_links() {
    snapshot("path-links");
}

#[test]
fn unknown_label() {
    snapshot("unknown-label");
}

#[test]
fn stale_block() {
    snapshot("stale-block");
}

#[test]
fn duplicate_number() {
    snapshot("duplicate-number");
}

/// The five ways a source include fails (I049 criteria 4 and 5), one tree
/// each: the block is empty or stale; the path is missing or resolves outside
/// the repository — here through `..` into a sibling fixture — or the file
/// carries no region of that name. The fixture directory is the repository
/// root, so `/src/main.rs` is the file beside the host.
#[test]
fn source_include_empty() {
    snapshot("source-include-empty");
}

#[test]
fn source_include_stale() {
    snapshot("source-include-stale");
}

#[test]
fn source_include_missing_path() {
    snapshot("source-include-missing-path");
}

#[test]
fn source_include_outside() {
    snapshot("source-include-outside");
}

#[test]
fn source_include_no_region() {
    snapshot("source-include-no-region");
}

/// The repository's own knowledge, validated where it lives. It changes with
/// every knowledge edit, so the assertion is the one thing that must hold:
/// the check CI already gates on.
#[test]
fn the_live_knowledge_conforms() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let bundle = load_bundle(&repo_root.join("knowledge")).unwrap();
    let report = validate(&bundle, &repo_root);
    assert!(report.passed(), "{:#?}", report.findings);
}
