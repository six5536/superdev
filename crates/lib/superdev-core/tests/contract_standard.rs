//! contract_standard.rs — the contract standard is prose in the one contract
//! schema, in the live tree and in the pack mirror (ADR-043): the one read a
//! contract writer is guaranteed to make carries it.

use std::path::PathBuf;

fn repo(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../..", path]
        .iter()
        .collect()
}

/// `text` with its line endings made uniform, for comparison against a
/// literal written with LF.
///
/// The comparison is where CRLF and LF are made the same — nothing normalises
/// on the way in, so what these tests read is what is on disk (I040). The
/// product needs none of this: its checks read a line at a time through
/// `validate::lines`, and a line is the same line whatever ends it.
fn same(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// The contract schema's prose before its yaml contract, where the standard
/// is stated, in both trees.
fn standard_copies() -> [(String, String); 2] {
    [
        "knowledge/schemas/contract.md",
        "pack/knowledge/schemas/contract.md",
    ]
    .map(|p| {
        let text = std::fs::read_to_string(repo(p)).expect("the contract schema is on file");
        let prose = same(&text)
            .split("````yaml")
            .next()
            .expect("the schema opens with prose")
            .to_string();
        (p.to_string(), prose)
    })
}

/// The standard states the definition rules and the binding obligation
/// (ADR-033, ADR-042).
#[test]
fn the_schema_states_the_definition_rules() {
    for (p, prose) in standard_copies() {
        for rule in [
            "a contract defines its interface",
            "MUST be one or more source includes",
            "MUST describe and MUST NOT define",
            "MUST NOT state how the",
            "bound by its include",
            "MUST be proved current by a test",
            "MUST NOT\n  restate the ADR's reasoning",
        ] {
            assert!(prose.contains(rule), "{p} is missing: {rule}");
        }
    }
}

/// Covers I049 criteria 16 and 17: the standard states that a doc comment in
/// an included region is contract text, that Behaviour carries what no
/// element and no include reaches and the project binds it by a test, and
/// that `PENDING` applies to prose alone (ADR-042, ADR-044). The standard is
/// prose in the schema: no include block carries it, and no fragment remains
/// to drift from.
#[test]
fn the_schema_states_the_doc_comment_and_pending_rules() {
    for (p, prose) in standard_copies() {
        for rule in [
            "A doc comment inside an included region is contract text",
            "binds as a promise under Behaviour does",
            "Behaviour MUST carry what no\n  single element can say and what no include reaches",
            "The project MUST bind each\n  Behaviour promise by a test of the behaviour it promises",
            "A Behaviour or Stability promise whose behaviour is unbuilt MAY\n  carry `PENDING`",
            "a\n  definition element carries none",
        ] {
            assert!(prose.contains(rule), "{p} is missing: {rule}");
        }
        assert!(
            !prose.contains("sokf:include"),
            "{p} carries the standard through an include block"
        );
    }
    assert!(
        !repo("knowledge/schemas/fragments/contract-style.md").exists()
            && !repo("pack/knowledge/schemas/fragments/contract-style.md").exists(),
        "the contract-style fragment is still on file"
    );
}

/// Covers I037 criteria 1, 8, 11 and 15: the standard states the promise
/// form — one keyed bullet with an EARS tag and one verb from SHALL, SHOULD
/// and MAY, the retired verbs, the interface element as subject, prose with
/// no modal verb, the numbered list as a sequence, the stable key, no `TBD`
/// — and the citation form, citing ADR-046 and ADR-047 beside the ADRs it
/// already cited; the schema alone carries the form, so nothing else needs
/// to (ADR-046).
#[test]
fn the_schema_states_the_promise_form() {
    for (p, prose) in standard_copies() {
        let one_line = prose.lines().map(str::trim).collect::<Vec<_>>().join(" ");
        for rule in [
            "ADR-044, ADR-046, ADR-047",
            "A promise MUST be one bullet under Behaviour or Stability, at any heading depth",
            "`[ubiquitous]`, `[event]`, `[state]`, `[conditional]`, `[optional]` or `[complex]`",
            "the interface element as the subject",
            "`SHALL` or `SHALL NOT` for a requirement, `SHOULD` or `SHOULD NOT` for a recommendation, or `MAY` for an option",
            "`MUST`, `REQUIRED`, `RECOMMENDED` and `OPTIONAL` are retired from contracts",
            "A key MUST be `P_` followed by a slug of lowercase letters and digits joined by hyphens, unique within the contract across both sections",
            "a rewording keeps it",
            "the key is not reused",
            "MUST describe and MUST NOT carry a modal verb",
            "A numbered list is a sequence — the steps of a flow — and never a promise",
            "cited by its bare key where the contract is the subject",
            "by the contract's id followed by the key elsewhere: `contract-002-cli-superdev P_init-outside-git`",
            "A contract MUST NOT carry a `TBD` item",
        ] {
            assert!(one_line.contains(rule), "{p} is missing: {rule}");
        }
        assert!(
            !one_line.contains("RFC 2119 modal verb")
                && !one_line.contains("one requirement per sentence"),
            "{p} still states the keyword-per-sentence rule ADR-046 replaced"
        );
    }
}
