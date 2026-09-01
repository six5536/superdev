//! normative_shapes.rs — the live repository's declared body patterns: the
//! shapes normative text must take are schema declarations the validator
//! reads, not guidance a writer may miss (I034).

use std::path::PathBuf;

use superdev_core::validate::schema::document::{Document, SchemaSet, check_documents};

fn repo(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../..", path]
        .iter()
        .collect()
}

/// Every schema in a tree, as `SchemaSet::load` takes them.
fn schemas(root: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(repo(root)).expect("the schema directory") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "md") {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            out.push((name, std::fs::read_to_string(&path).unwrap()));
        }
    }
    out
}

/// The findings a document draws from the live schema set.
fn findings_for(path: &str, text: &str) -> Vec<String> {
    let (set, load) = SchemaSet::load(&schemas("knowledge/schemas"));
    assert!(
        load.is_empty(),
        "the live schema set loads clean: {load:#?}"
    );
    let doc = Document {
        path,
        text,
        doc_type: Some("FeatureRequest"),
    };
    check_documents(&[doc], &set)
        .into_iter()
        .map(|f| f.message)
        .collect()
}

/// One feature-request body, with `criteria` as its acceptance criteria.
fn request(criteria: &str) -> String {
    format!(
        "---\ntype: FeatureRequest\nid: issue-999-feature-request-probe\ntitle: t\n\
         description: d\nlifecycle: open\n---\n\n# Feature: probe\n\n## Summary\n\nA line.\n\n\
         ## Motivation\n\nA line.\n\n## Proposed behaviour\n\nA line.\n\n\
         ## Acceptance criteria\n\n{criteria}\n## Alternatives considered\n\n- One.\n\n\
         ## Scope\n\n- In: one.\n"
    )
}

/// Covers I034 criterion 4: a criterion that does not open with an EARS
/// pattern tag fails validate, and the finding names the file, the section
/// and the criterion (ADR-031).
#[test]
fn a_criterion_without_its_ears_tag_fails_validate() {
    let found = findings_for(
        "probe.md",
        &request("1. WHEN the tag is missing THE SYSTEM SHALL be told so.\n"),
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("\"Acceptance criteria\""), "{found:#?}");
    assert!(found[0].contains("WHEN the tag is missing"), "{found:#?}");
    assert!(found[0].contains("item-pattern"), "{found:#?}");
}

/// Covers I034 criterion 4: each of the six EARS tags opens a criterion, and
/// an unsettled criterion reads TBD — the pattern admits both, and the frame
/// phase is what retires TBD.
#[test]
fn every_ears_tag_and_a_tbd_criterion_pass() {
    let criteria = "1. [ubiquitous] THE SYSTEM SHALL do it.\n\
                    2. [event] WHEN x THE SYSTEM SHALL do it.\n\
                    3. [state] WHILE x THE SYSTEM SHALL do it.\n\
                    4. [conditional] IF x THE SYSTEM SHALL do it.\n\
                    5. [optional] WHERE x THE SYSTEM SHALL do it.\n\
                    6. [complex] WHILE x WHEN y THE SYSTEM SHALL do it.\n\
                    7. TBD — whether it should.\n";
    let found = findings_for("probe.md", &request(criteria));
    assert!(found.is_empty(), "{found:#?}");
}

/// Covers I034 criteria 4 and 6: every feature-request on file conforms, in
/// the live tree, so the declaration lands on a corpus it already fits.
#[test]
fn every_feature_request_on_file_conforms() {
    let (set, load) = SchemaSet::load(&schemas("knowledge/schemas"));
    assert!(load.is_empty(), "{load:#?}");
    let mut checked = 0;
    for state in ["open", "done", "wontfix"] {
        let dir = repo(&format!("knowledge/issues/{state}"));
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if !name.contains("-feature-request-") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let doc = Document {
                path: &name,
                text: &text,
                doc_type: Some("FeatureRequest"),
            };
            let found = check_documents(&[doc], &set);
            assert!(found.is_empty(), "{name}: {found:#?}");
            checked += 1;
        }
    }
    assert!(checked >= 10, "the tracker's requests were read: {checked}");
}

/// The declaration ships: the live schema and the pack mirror carry the same
/// EARS pattern, so a managed repository is held to it too.
#[test]
fn the_ears_declaration_ships_to_managed_repositories() {
    let pattern = "item-pattern: '^\\[(ubiquitous|event|state|conditional|optional|complex)\\] \
                   |^TBD — '";
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        let text = std::fs::read_to_string(repo(&format!("{root}/feature-request.md"))).unwrap();
        assert!(text.contains(pattern), "{root} declares the EARS pattern");
    }
}
