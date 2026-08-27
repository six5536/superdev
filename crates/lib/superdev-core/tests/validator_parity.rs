//! validator_parity.rs — `aokf::validate` against the Python reference
//! validator, one fixture bundle per failure class.
//!
//! Each `tests/fixtures/aokf/<case>/` bundle has a `<case>.golden.json`
//! holding the output of `.agents/aokf/tools/validator.py` for it. That script
//! is gone; the goldens are the only remaining record of the behaviour it
//! defined, and there is nothing left to regenerate them from. Each was
//! captured as:
//!
//! ```sh
//! python3 .agents/aokf/tools/validator.py "$d" --json --repo-root "$d" \
//!   | jq 'del(.bundle)'
//! ```
//!
//! Treat a golden as fixed. Editing one changes the contract these tests
//! exist to hold, so it needs the same argument a deliberate behaviour change
//! would.
//!
//! The goldens were projected once, when the conformance ladder was removed
//! (ADR-017): `checked_level` and `achieved_level` were deleted from each
//! report, `error_at_level` from each finding, and the one message reading
//! "(required at Level 1)" became "(required)". Nothing else was touched —
//! every `severity`, `file` and other `message` is as the reference emitted
//! it, which the commit that did it shows as a diff of deletions. The goldens
//! were captured at the top level, where a finding was an error exactly when
//! it carried any level at all, so no verdict moved.
//!
//! Two normalisations, and nothing else:
//!
//! - `jq 'del(.bundle)'` drops the reference validator's `bundle` key. It
//!   echoes the path from its own command line; [`Report::to_json`] leaves
//!   that to the caller, which holds the string.
//! - Findings for a file that failed to parse compare on `(file, severity)`
//!   only: both sides quote their YAML parser, and the two
//!   parsers word the same complaint differently. [`PARSE_ERRORS`] lists the
//!   fixture files this applies to; their `message` is replaced on both sides
//!   before comparing. Every other message compares verbatim.
//!
//! Finding order is compared as emitted — the two implementations walk files
//! and checks in the same order, so neither side is sorted here.
//!
//! Known divergence, deliberately left: a manifest that parses to a falsy
//! non-mapping (`[]`, `false`). Python coerces it to `{}` and reports the
//! missing `aokf` and `name`; the Rust reports the parse error, or nothing,
//! and returns. No fixture exercises it.

use std::path::{Path, PathBuf};

use superdev_core::aokf::{load_bundle, validate};

/// Fixture files whose parse-error message is not compared. See the header.
const PARSE_ERRORS: [(&str, &str); 2] = [
    ("malformed-frontmatter", "bad-yaml.md"),
    ("malformed-frontmatter", "no-frontmatter.md"),
];

/// The fixture root.
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aokf")
}

/// Validate `case` and compare the JSON to its golden.
fn parity(case: &str) {
    let dir = fixtures().join(case);
    let golden = std::fs::read_to_string(dir.with_extension("golden.json")).unwrap();
    let golden: serde_json::Value = serde_json::from_str(&golden).unwrap();

    let bundle = load_bundle(&dir).unwrap();
    let ours = validate(&bundle, &dir).to_json();

    assert_eq!(
        blank_parse_errors(case, ours),
        blank_parse_errors(case, golden),
        "parity failure for {case}"
    );
}

/// Replace the message of every finding listed in [`PARSE_ERRORS`], so the two
/// implementations' YAML parser wording does not have to agree.
fn blank_parse_errors(case: &str, mut report: serde_json::Value) -> serde_json::Value {
    let Some(findings) = report["findings"].as_array_mut() else {
        return report;
    };
    for finding in findings {
        let file = finding["file"].as_str().unwrap_or_default();
        if PARSE_ERRORS.contains(&(case, file)) {
            finding["message"] = serde_json::Value::String("<parse error>".to_string());
        }
    }
    report
}

#[test]
fn clean() {
    parity("clean");
}

#[test]
fn broken_links() {
    parity("broken-links");
}

#[test]
fn duplicate_ids() {
    parity("duplicate-ids");
}

#[test]
fn malformed_frontmatter() {
    parity("malformed-frontmatter");
}

#[test]
fn missing_type() {
    parity("missing-type");
}

#[test]
fn stamped_field() {
    parity("stamped-field");
}

#[test]
fn bad_verified() {
    parity("bad-verified");
}

#[test]
fn unmirrored_link() {
    parity("unmirrored-link");
}

#[test]
fn no_manifest() {
    parity("no-manifest");
}

#[test]
fn custom_rel() {
    parity("custom-rel");
}

#[test]
fn footnote_mismatch() {
    parity("footnote-mismatch");
}

/// The repository's own bundle, validated where it lives. It changes with
/// every knowledge edit, so the assertion is the one thing that must hold:
/// the check CI already gates on.
#[test]
fn the_live_knowledge_bundle_conforms() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let bundle = load_bundle(&repo_root.join("knowledge")).unwrap();
    let report = validate(&bundle, &repo_root);
    assert!(report.passed(), "{:#?}", report.findings);
}
