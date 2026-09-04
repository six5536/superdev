//! normative_shapes.rs — the live repository's declared body patterns: the
//! shapes normative text must take are schema declarations the validator
//! reads, not guidance a writer may miss (I034).

use std::path::PathBuf;

use superdev_core::validate::schema::document::{Document, SchemaSet, check_documents};

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

/// The findings a document of `doc_type` draws from the live schema set.
fn findings_of(doc_type: &str, path: &str, text: &str) -> Vec<String> {
    let (set, load) = SchemaSet::load(&schemas("knowledge/schemas"));
    assert!(
        load.is_empty(),
        "the live schema set loads clean: {load:#?}"
    );
    let doc = Document {
        path,
        text,
        doc_type: Some(doc_type),
    };
    check_documents(&[doc], &set)
        .into_iter()
        .map(|f| f.message)
        .collect()
}

/// The findings a feature-request draws from the live schema set.
fn findings_for(path: &str, text: &str) -> Vec<String> {
    findings_of("FeatureRequest", path, text)
}

/// One feature-request body in `lifecycle`, with `criteria` as its
/// acceptance criteria; a settled one carries its verdict section, as the
/// tracker's convention puts it.
fn request_in(lifecycle: &str, criteria: &str) -> String {
    format!(
        "---\ntype: FeatureRequest\nid: issue-999-feature-request-probe\ntitle: t\n\
         description: d\nlifecycle: {lifecycle}\n---\n\n# Feature: probe\n\n{}\
         ## Summary\n\nA line.\n\n\
         ## Motivation\n\nA line.\n\n## Proposed behaviour\n\nA line.\n\n\
         ## Acceptance criteria\n\n{criteria}\n## Alternatives considered\n\n- One.\n\n\
         ## Scope\n\n- In: one.\n",
        verdict(lifecycle)
    )
}

/// The verdict section a settled issue opens with, and nothing otherwise.
fn verdict(lifecycle: &str) -> &'static str {
    match lifecycle {
        "done" => "## Resolved\n\nShipped.\n\n",
        "wontfix" => "## Won't fix\n\nDeclined.\n\n",
        _ => "",
    }
}

/// One framed feature-request body, with `criteria` as its acceptance
/// criteria.
fn request(criteria: &str) -> String {
    request_in("framed", criteria)
}

/// Covers I034 criterion 4: a criterion that does not open with an EARS
/// pattern tag fails validate, and the finding names the file, the section
/// and the criterion (ADR-031). The criterion carries its key, which
/// ADR-046 puts before the tag, so the one finding is the tag's.
#[test]
fn a_criterion_without_its_ears_tag_fails_validate() {
    let found = findings_for(
        "probe.md",
        &request("1. `AC_told` WHEN the tag is missing THE SYSTEM SHALL be told so.\n"),
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("\"Acceptance criteria\""), "{found:#?}");
    assert!(found[0].contains("WHEN the tag is missing"), "{found:#?}");
    assert!(found[0].contains("item-pattern"), "{found:#?}");
}

/// Covers I034 criterion 4 and I037 criterion 18: each of the six EARS tags
/// opens a criterion after its key. The `TBD` branch the pattern once
/// admitted here is gone: an unsettled criterion belongs to an unframed
/// issue, and `a_framed_request_departing_from_the_form_fails_naming_each`
/// shows a framed one refusing it (ADR-048).
#[test]
fn every_ears_tag_passes_on_a_framed_criterion() {
    let criteria = "1. `AC_c1` [ubiquitous] THE SYSTEM SHALL do it.\n\
                    2. `AC_c2` [event] WHEN x THE SYSTEM SHALL do it.\n\
                    3. `AC_c3` [state] WHILE x THE SYSTEM SHALL do it.\n\
                    4. `AC_c4` [conditional] IF x THE SYSTEM SHALL do it.\n\
                    5. `AC_c5` [optional] WHERE x THE SYSTEM SHALL do it.\n\
                    6. `AC_c6` [complex] WHILE x WHEN y THE SYSTEM SHALL do it.\n";
    let found = findings_for("probe.md", &request(criteria));
    assert!(found.is_empty(), "{found:#?}");
}

/// Covers I037 AC_c17: under the live feature-request schema a
/// keyless criterion, a criterion keyed with another kind's prefix and a
/// key used twice each fail `validate` with a finding naming the item —
/// the same schema set the tracker is checked with.
#[test]
fn a_criterion_departing_from_the_key_form_fails_naming_each_departure() {
    let criteria = "1. `AC_c1` [ubiquitous] THE SYSTEM SHALL do it.\n\
                    2. [event] WHEN the key is missing THE SYSTEM SHALL say so.\n\
                    3. `P_c3` [event] WHEN the prefix is a promise's THE SYSTEM SHALL say so.\n\
                    4. `AC_c1` [event] WHEN the key repeats THE SYSTEM SHALL say so.\n";
    let found = findings_for("probe.md", &request(criteria));
    let named = |text: &str| found.iter().filter(|f| f.contains(text)).count();
    assert_eq!(
        named("item `2. [event] WHEN the key is missing THE SYSTEM SHALL say so.` carries no key"),
        1,
        "{found:#?}"
    );
    assert_eq!(
        named(
            "item `3. `P_c3` [event] WHEN the prefix is a promise's THE SYSTEM SHALL say so.` carries no key"
        ),
        1,
        "{found:#?}"
    );
    let repeated = found
        .iter()
        .filter(|f| {
            f.contains("key `AC_c1`") && f.contains("4. `AC_c1` [event] WHEN the key repeats")
        })
        .count();
    assert_eq!(repeated, 1, "{found:#?}");
    // Items 2 and 3 each fail the key, and the pattern — which requires the
    // `AC_` key before the tag — is not run on an item the key reported:
    // one fault, said once. Item 4 fails the repeat; item 1 is sound.
    assert_eq!(found.len(), 3, "{found:#?}");
}

/// One bug-report body in `lifecycle`, with `steps` as its steps to
/// reproduce and `expected` as its expected behaviour.
fn bug_in(lifecycle: &str, steps: &str, expected: &str) -> String {
    format!(
        "---\ntype: BugReport\nid: issue-999-bug-probe\ntitle: t\ndescription: d\n\
         lifecycle: {lifecycle}\n---\n\n# Bug: probe\n\n{}## Summary\n\nA line.\n\n\
         ## Environment\n\n- One.\n\n## Steps to reproduce\n\n{steps}\n\
         ## Expected behaviour\n\n{expected}\n## Actual behaviour\n\nA line.\n\n\
         ## Root cause (if known)\n\nA line.\n\n## Proposed fix / workaround\n\n- One.\n\n\
         ## Regression risk\n\nA line.\n",
        verdict(lifecycle)
    )
}

/// One framed bug-report body, with `steps` as its steps to reproduce and
/// one sound expected-behaviour item.
fn bug(steps: &str) -> String {
    bug_in(
        "framed",
        steps,
        "1. `EX_c1` [ubiquitous] THE SYSTEM SHALL do it.\n",
    )
}

/// One chore body in `lifecycle`, with `done` as its definition of done.
fn chore_in(lifecycle: &str, done: &str) -> String {
    format!(
        "---\ntype: Chore\nid: issue-999-chore-probe\ntitle: t\ndescription: d\n\
         lifecycle: {lifecycle}\n---\n\n# Chore: probe\n\n{}## Summary\n\nA line.\n\n\
         ## Surfaces\n\n- One.\n\n## Definition of done\n\n{done}\n",
        verdict(lifecycle)
    )
}

/// One framed chore body, with `done` as its definition of done.
fn chore(done: &str) -> String {
    chore_in("framed", done)
}

/// Covers I037 AC_c18: a repro step and a done item each carry their key
/// and no EARS tag — a step is not a requirement (ADR-046) — so one with
/// no key fails naming it, and one carrying a tag after its key fails
/// naming the tag.
#[test]
fn a_repro_step_carries_a_key_and_no_tag() {
    let found = findings_of(
        "BugReport",
        "probe.md",
        &bug("1. `RS_c1` Run the probe.\n2. `RS_wait` Wait 30 seconds.\n"),
    );
    assert!(found.is_empty(), "{found:#?}");
    let found = findings_of("Chore", "probe.md", &chore("- `DD_c1` The probe runs.\n"));
    assert!(found.is_empty(), "{found:#?}");

    let found = findings_of("BugReport", "probe.md", &bug("1. Run the probe.\n"));
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("\"Steps to reproduce\"")
            && found[0].contains("item `1. Run the probe.` carries no key"),
        "{found:#?}"
    );

    let found = findings_of(
        "BugReport",
        "probe.md",
        &bug("1. `RS_c1` [event] WHEN run, the probe runs.\n"),
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("\"Steps to reproduce\"")
            && found[0].contains(
                "item `1. `RS_c1` [event] WHEN run, the probe runs.` matches ``RS_c1` [event]`"
            )
            && found[0].contains("item-prohibited-pattern"),
        "{found:#?}"
    );
    let found = findings_of(
        "Chore",
        "probe.md",
        &chore("- `DD_c1` [ubiquitous] The probe SHALL run.\n"),
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("\"Definition of done\"")
            && found[0].contains("matches ``DD_c1` [ubiquitous]`")
            && found[0].contains("item-prohibited-pattern"),
        "{found:#?}"
    );
}

/// Covers I030 AC_unframed-form: under the live schema set an unframed
/// feature request whose criteria are a plain sentence, a `TBD` and one
/// keyed item passes; so do an unframed bug whose steps and expected
/// behaviour are plain sentences and `TBD`s, and an unframed chore whose
/// done items are — the list kind is all the unframed rule checks (ADR-048).
#[test]
fn an_unframed_issue_with_plain_tbd_and_keyed_items_passes() {
    let criteria = "1. The report is one JSON object.\n\
                    2. TBD — whether the counts are in it.\n\
                    3. `AC_error-exit` [event] WHEN a finding is an error THE SYSTEM SHALL exit non-zero.\n";
    let found = findings_for("probe.md", &request_in("unframed", criteria));
    assert!(found.is_empty(), "{found:#?}");

    let found = findings_of(
        "BugReport",
        "probe.md",
        &bug_in(
            "unframed",
            "1. Run the probe.\n2. TBD — how long to wait.\n",
            "1. The probe runs.\n2. TBD — whether it reports the wait.\n",
        ),
    );
    assert!(found.is_empty(), "{found:#?}");

    let found = findings_of(
        "Chore",
        "probe.md",
        &chore_in(
            "unframed",
            "- The probe runs.\n- TBD — the command that says it is finished.\n",
        ),
    );
    assert!(found.is_empty(), "{found:#?}");
}

/// Covers I030 AC_framed-form: under the live schema set a framed feature
/// request with a keyless criterion, a `TBD` criterion and a tagless
/// criterion fails naming each, and a sound criterion beside them is not
/// reported.
#[test]
fn a_framed_request_departing_from_the_form_fails_naming_each() {
    let criteria = "1. `AC_c1` [ubiquitous] THE SYSTEM SHALL do it.\n\
                    2. [event] WHEN the key is missing THE SYSTEM SHALL say so.\n\
                    3. `AC_open` TBD — whether it should.\n\
                    4. `AC_tagless` WHEN the tag is missing THE SYSTEM SHALL say so.\n";
    let found = findings_for("probe.md", &request(criteria));
    let named = |text: &str| found.iter().filter(|f| f.contains(text)).count();
    assert_eq!(
        named("item `2. [event] WHEN the key is missing THE SYSTEM SHALL say so.` carries no key"),
        1,
        "{found:#?}"
    );
    assert_eq!(
        named("item `3. `AC_open` TBD — whether it should.` does not match"),
        1,
        "{found:#?}"
    );
    assert_eq!(
        named(
            "item `4. `AC_tagless` WHEN the tag is missing THE SYSTEM SHALL say so.` does not match"
        ),
        1,
        "{found:#?}"
    );
    assert!(
        !found.iter().any(|f| f.contains("AC_c1")),
        "a sound criterion is not reported: {found:#?}"
    );
    assert_eq!(found.len(), 3, "{found:#?}");
}

/// Covers I030 AC_framed-form: under the live schema set a framed bug whose
/// Expected behaviour is prose, whose steps include a keyless one and whose
/// expected behaviour, once a list, includes an untagged `EX_` item fails
/// naming each; and a framed chore with a keyless done item fails naming
/// it.
#[test]
fn a_framed_bug_or_chore_departing_from_the_form_fails_naming_each() {
    let found = findings_of(
        "BugReport",
        "probe.md",
        &bug_in(
            "framed",
            "1. `RS_c1` Run the probe.\n2. Wait 30 seconds.\n",
            "The probe runs to completion.\n",
        ),
    );
    let named = |text: &str| found.iter().filter(|f| f.contains(text)).count();
    assert_eq!(
        named("section \"Expected behaviour\" carries no numbered item"),
        1,
        "{found:#?}"
    );
    assert_eq!(
        named("section \"Steps to reproduce\" item `2. Wait 30 seconds.` carries no key"),
        1,
        "{found:#?}"
    );
    assert_eq!(found.len(), 2, "{found:#?}");

    let found = findings_of(
        "BugReport",
        "probe.md",
        &bug_in(
            "framed",
            "1. `RS_c1` Run the probe.\n",
            "1. `EX_c1` [ubiquitous] THE SYSTEM SHALL run.\n\
             2. `EX_c2` THE SYSTEM SHALL report the wait.\n\
             3. `EX_c3` TBD — whether it reports the wait.\n",
        ),
    );
    let named = |text: &str| found.iter().filter(|f| f.contains(text)).count();
    assert_eq!(
        named("item `2. `EX_c2` THE SYSTEM SHALL report the wait.` does not match"),
        1,
        "{found:#?}"
    );
    assert_eq!(
        named("item `3. `EX_c3` TBD — whether it reports the wait.` does not match"),
        1,
        "{found:#?}"
    );
    assert_eq!(found.len(), 2, "{found:#?}");

    let found = findings_of(
        "Chore",
        "probe.md",
        &chore("- `DD_c1` The probe runs.\n- The command that says it is finished.\n"),
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("\"Definition of done\"")
            && found[0].contains("item `- The command that says it is finished.` carries no key"),
        "{found:#?}"
    );
}

/// Covers I030 AC_settled-form: under the live schema set a done and a
/// wontfix issue of each kind are held to the framed rules — a keyless
/// criterion, a prose Expected behaviour and a keyless done item each fail
/// naming the fault — and the sound framed body of each kind passes in
/// both states.
#[test]
fn a_done_and_a_wontfix_issue_are_held_to_the_framed_rules() {
    for state in ["done", "wontfix"] {
        let found = findings_for(
            "probe.md",
            &request_in(state, "1. `AC_c1` [ubiquitous] THE SYSTEM SHALL do it.\n"),
        );
        assert!(found.is_empty(), "{state}: {found:#?}");
        let found = findings_for(
            "probe.md",
            &request_in(state, "1. THE SYSTEM SHALL do it.\n"),
        );
        assert_eq!(found.len(), 1, "{state}: {found:#?}");
        assert!(
            found[0].contains("item `1. THE SYSTEM SHALL do it.` carries no key"),
            "{state}: {found:#?}"
        );

        let steps = "1. `RS_c1` Run the probe.\n";
        let found = findings_of(
            "BugReport",
            "probe.md",
            &bug_in(
                state,
                steps,
                "1. `EX_c1` [ubiquitous] THE SYSTEM SHALL run.\n",
            ),
        );
        assert!(found.is_empty(), "{state}: {found:#?}");
        let found = findings_of("BugReport", "probe.md", &bug_in(state, steps, "It runs.\n"));
        assert_eq!(found.len(), 1, "{state}: {found:#?}");
        assert!(
            found[0].contains("section \"Expected behaviour\" carries no numbered item"),
            "{state}: {found:#?}"
        );

        let found = findings_of(
            "Chore",
            "probe.md",
            &chore_in(state, "- `DD_c1` It runs.\n"),
        );
        assert!(found.is_empty(), "{state}: {found:#?}");
        let found = findings_of("Chore", "probe.md", &chore_in(state, "- It runs.\n"));
        assert_eq!(found.len(), 1, "{state}: {found:#?}");
        assert!(
            found[0].contains("item `- It runs.` carries no key"),
            "{state}: {found:#?}"
        );
    }
}

/// The four tracker lists a plan case cites and the key prefix each
/// declares (ADR-046, ADR-048): the heading, the schema that carries it,
/// the list's marker kind, the prefix, and whether the item is a
/// requirement — an EARS tag follows the key of one that is, and is
/// forbidden after the key of one that is not.
const CITED_LISTS: [(&str, &str, &str, &str, bool); 4] = [
    (
        "Acceptance criteria",
        "feature-request.md",
        "numbered",
        "AC_",
        true,
    ),
    (
        "Steps to reproduce",
        "bug-report.md",
        "numbered",
        "RS_",
        false,
    ),
    (
        "Expected behaviour",
        "bug-report.md",
        "numbered",
        "EX_",
        true,
    ),
    ("Definition of done", "chore.md", "bullet", "DD_", false),
];

/// The tracker's four lifecycle values, in the order the schemas declare
/// them (ADR-048).
const LIFECYCLES: [&str; 4] = ["unframed", "framed", "done", "wontfix"];

/// The three tracker schemas.
const TRACKER_SCHEMAS: [&str; 3] = ["feature-request.md", "bug-report.md", "chore.md"];

/// The `variants` line the framed rule carries.
const FRAMED_VARIANTS: &str = "variants: [framed, done, wontfix]";

/// The `variants` line the unframed rule carries.
const UNFRAMED_VARIANTS: &str = "variants: [unframed]";

/// Covers I030 AC_lifecycle-values and AC_one-schema-per-kind: each tracker
/// schema declares the four values, `variant-key: lifecycle`, and one
/// example per value, in the live tree and in the pack mirror; every
/// example passes its own schema's check. `--fix` filing by the value is
/// `validate::fix`'s refile test, and the live tree's filing is
/// `every_issue_on_file_sits_in_its_lifecycles_folder`.
#[test]
fn every_tracker_schema_varies_by_the_four_lifecycle_values() {
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        for schema in TRACKER_SCHEMAS {
            let path = format!("{root}/{schema}");
            let text = same(&std::fs::read_to_string(repo(&path)).unwrap());
            assert!(
                text.contains("\nvariant-key: lifecycle\n"),
                "{path}: lifecycle is the variant key"
            );
            assert!(
                text.contains("    enum: [unframed, framed, done, wontfix]\n"),
                "{path}: lifecycle admits the four values"
            );
            let block = fenced_block(&text, "yaml").expect("the schema carries a yaml contract");
            let y: serde_yaml_ng::Value = serde_yaml_ng::from_str(&block).unwrap();
            let keys: Vec<&str> = y["example"]
                .as_mapping()
                .unwrap_or_else(|| panic!("{path}: example is keyed by lifecycle"))
                .keys()
                .map(|k| k.as_str().unwrap())
                .collect();
            assert_eq!(keys, LIFECYCLES, "{path}: one example per value");
        }
    }
    let tracker: Vec<(String, String)> = schemas("knowledge/schemas")
        .into_iter()
        .filter(|(name, _)| TRACKER_SCHEMAS.contains(&name.as_str()))
        .collect();
    assert_eq!(tracker.len(), 3);
    let found = superdev_core::validate::schema::document::check_examples(&tracker);
    assert!(found.is_empty(), "{found:#?}");
}

/// Covers I037 AC_c17, AC_c18 and AC_c19 and I030 AC_unframed-form and
/// AC_framed-form as declared: each cited list is declared twice — the
/// unframed rule with its list kind and no key or pattern, the framed rule
/// (framed, done, wontfix) with `item-key` for its prefix, the tag required
/// after the key of a requirement and forbidden after the key of a step or
/// a done item, and no `TBD` branch — and each framed rule and the plan
/// schema's case rule state the citation in keys, in the live tree and in
/// the pack mirror.
#[test]
fn every_cited_list_declares_its_key_and_the_plan_cites_keys() {
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        for (heading, schema, kind, prefix, requirement) in CITED_LISTS {
            let path = format!("{root}/{schema}");
            let text = std::fs::read_to_string(repo(&path)).unwrap();
            let rules = rules_for(&text, heading);
            assert_eq!(
                rules.len(),
                2,
                "{path}: {heading} is declared once per state"
            );
            let content = format!("content: {kind}-list");
            let unframed = rules
                .iter()
                .find(|rule| rule.contains(UNFRAMED_VARIANTS))
                .unwrap_or_else(|| panic!("{path}: {heading} has no unframed rule"));
            assert!(
                unframed.contains(&content)
                    && !unframed.contains("item-key")
                    && !unframed.contains("item-pattern")
                    && !unframed.contains("item-prohibited-pattern"),
                "{path}: the unframed {heading} binds its list kind alone: {unframed}"
            );
            let framed = rules
                .iter()
                .find(|rule| rule.contains(FRAMED_VARIANTS))
                .unwrap_or_else(|| panic!("{path}: {heading} has no framed rule"));
            assert!(
                framed.contains(&content),
                "{path}: {heading} keeps its kind"
            );
            let key = format!("item-key: '^`({prefix}[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'");
            assert!(framed.contains(&key), "{path}: {heading} lacks `{key}`");
            let tag = "\\[(ubiquitous|event|state|conditional|optional|complex)\\]";
            if requirement {
                let tagged = format!("item-pattern: '^`{prefix}[a-z0-9-]+` {tag} '");
                assert!(
                    framed.contains(&tagged) && !framed.contains("TBD — )"),
                    "{path}: {heading} requires the tag after the key and admits no TBD: {framed}"
                );
            } else {
                let tagged = format!("item-prohibited-pattern: '^`{prefix}[a-z0-9-]+` {tag}'");
                assert!(
                    !framed.contains("item-pattern") && framed.contains(&tagged),
                    "{path}: {heading} binds the key alone and forbids a tag after it: {framed}"
                );
            }
            let description = folded(framed);
            assert!(
                description.contains("bare key where the issue is the subject")
                    && description.contains("the issue's id followed by the key elsewhere"),
                "{path}: {heading} does not state the citation form: {description}"
            );
            assert!(
                description.contains("`c<n>`"),
                "{path}: {heading} does not name the sweep's slug: {description}"
            );
        }
        let plan =
            same(&std::fs::read_to_string(repo(&format!("{root}/feature-plan.md"))).unwrap());
        let slice = plan
            .split("  - heading-pattern: '^Slice \\d+: .+$'\n")
            .nth(1)
            .and_then(|rest| rest.split("\n  - heading").next())
            .expect("the plan schema carries the slice rule");
        let slice = folded(slice);
        for phrase in [
            "naming the keys of the acceptance criteria it covers",
            "\"covers AC_c1, AC_stale-include\"",
            "keyed repro steps (`RS_`) and expected behaviour (`EX_`)",
        ] {
            assert!(
                slice.contains(phrase),
                "{root}/feature-plan.md: the slice rule lacks `{phrase}`: {slice}"
            );
        }
        assert!(
            !plan.contains("covers 1"),
            "{root}/feature-plan.md: the example's cases cite keys, not numbers"
        );
    }
}

/// Every issue on file: its folder, its file name and its text, read
/// straight from the tracker's four lifecycle folders.
fn issues_on_file() -> Vec<(&'static str, String, String)> {
    let mut out = Vec::new();
    for state in LIFECYCLES {
        let dir = repo(&format!("knowledge/issues/{state}"));
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let text = same(&std::fs::read_to_string(&path).unwrap());
            out.push((state, name, text));
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// The `lifecycle` an issue's frontmatter declares.
fn lifecycle_of(text: &str) -> Option<&str> {
    text.strip_prefix("---\n")?
        .split("\n---\n")
        .next()?
        .lines()
        .find_map(|line| line.strip_prefix("lifecycle: "))
        .map(str::trim)
}

/// Every top-level item of every cited list in an issue's body, with the
/// list's prefix and whether the item is a requirement; a fenced block is
/// skipped.
fn cited_items(text: &str) -> Vec<(String, &'static str, bool)> {
    let mut items = Vec::new();
    let mut section: Option<(&str, &str, bool)> = None;
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if line.starts_with("# ") || line.starts_with("## ") {
            section = CITED_LISTS
                .iter()
                .find(|(heading, ..)| line == format!("## {heading}"))
                .map(|(_, _, kind, prefix, requirement)| (*kind, *prefix, *requirement));
            continue;
        }
        let Some((kind, prefix, requirement)) = section else {
            continue;
        };
        let item = match kind {
            "numbered" => {
                let digits =
                    line.len() - line.trim_start_matches(|c: char| c.is_ascii_digit()).len();
                (digits > 0 && line[digits..].starts_with(". ")).then(|| &line[digits + 2..])
            }
            _ => line.strip_prefix("- "),
        };
        if let Some(item) = item {
            items.push((item.to_string(), prefix, requirement));
        }
    }
    items
}

/// Whether `item` opens with a key of `prefix` and, where `requirement`,
/// an EARS tag after it.
fn keyed_and_tagged(item: &str, prefix: &str, requirement: bool) -> bool {
    if !item.starts_with(&format!("`{prefix}")) {
        return false;
    }
    let after_key = item.split_once("` ").map_or("", |(_, rest)| rest);
    !requirement || (after_key.starts_with('[') && after_key.contains("] "))
}

/// Covers I037 AC_c20 and I030 AC_sweep: every framed, done or wontfix
/// issue on file carries a key on each top-level item of the lists a plan
/// case cites — `AC_` on a feature-request's criteria, `RS_` on a bug's
/// repro steps, `EX_` on its expected behaviour, `DD_` on a chore's
/// definition of done — and an EARS tag after the key of each `AC_` and
/// `EX_` item; the sweep's proof, read straight from the tracker rather
/// than through the schema. An unframed issue's items are read and not
/// held to the form (ADR-048).
#[test]
fn every_issue_on_file_carries_a_key_on_each_cited_item() {
    let mut keyed = 0;
    let mut unframed = 0;
    let issues = issues_on_file();
    for (state, name, text) in &issues {
        for (item, prefix, requirement) in cited_items(text) {
            if *state == "unframed" {
                unframed += 1;
                continue;
            }
            assert!(
                keyed_and_tagged(&item, prefix, requirement),
                "{name}: item `{item}` carries no {prefix} key{}",
                if requirement { " and EARS tag" } else { "" }
            );
            keyed += 1;
        }
    }
    assert!(issues.len() >= 50, "the tracker was read: {}", issues.len());
    assert!(keyed >= 200, "the tracker's cited items were read: {keyed}");
    assert!(
        unframed >= 1,
        "an unframed issue's items were read: {unframed}"
    );
}

/// Covers I030 AC_sweep: every issue on file sits in the folder its
/// `lifecycle` names; I042, the one open issue that kept a `TBD` item, is
/// `unframed` (I030, framed before the sweep with no `TBD` left, went
/// `framed` and then `done` at acceptance — a state this test does not
/// pin); and every bug report's Expected behaviour is a numbered list
/// whose items open with `EX_c1`, `EX_c2`, … and an EARS tag.
#[test]
fn every_issue_on_file_sits_in_its_lifecycles_folder() {
    let issues = issues_on_file();
    let mut bugs = 0;
    for (state, name, text) in &issues {
        assert_eq!(
            lifecycle_of(text),
            Some(*state),
            "{name}: filed under {state}/"
        );
        if name.starts_with("issue-042-") {
            assert_eq!(*state, "unframed", "{name}");
        }
        // The kind is the id's segment after the number: I015 names "a bug
        // report" in its slug and is a feature request.
        if !name.starts_with("issue-") || &name[10..14] != "bug-" {
            continue;
        }
        bugs += 1;
        let expected: Vec<String> = cited_items(text)
            .into_iter()
            .filter(|(_, prefix, _)| *prefix == "EX_")
            .map(|(item, ..)| item)
            .collect();
        assert!(
            !expected.is_empty(),
            "{name}: Expected behaviour carries no numbered item"
        );
        for (n, item) in expected.iter().enumerate() {
            let key = format!("EX_c{}", n + 1);
            assert!(
                item.starts_with(&format!("`{key}` ")) && keyed_and_tagged(item, "EX_", true),
                "{name}: expected-behaviour item `{item}` is not keyed `{key}` and tagged"
            );
        }
    }
    assert!(issues.len() >= 50, "the tracker was read: {}", issues.len());
    assert!(bugs >= 24, "the tracker's bug reports were read: {bugs}");
}

/// Covers I034 criteria 4 and 6: every feature-request on file conforms, in
/// the live tree, so the declaration lands on a corpus it already fits.
#[test]
fn every_feature_request_on_file_conforms() {
    let (set, load) = SchemaSet::load(&schemas("knowledge/schemas"));
    assert!(load.is_empty(), "{load:#?}");
    let mut checked = 0;
    for (_, name, text) in issues_on_file() {
        if !name.contains("-feature-request-") {
            continue;
        }
        let doc = Document {
            path: &name,
            text: &text,
            doc_type: Some("FeatureRequest"),
        };
        let found = check_documents(&[doc], &set);
        assert!(found.is_empty(), "{name}: {found:#?}");
        checked += 1;
    }
    assert!(checked >= 10, "the tracker's requests were read: {checked}");
}

/// The declaration ships: the live schema and the pack mirror carry the same
/// EARS patterns, so a managed repository is held to them too. The criterion's
/// pattern is the keyed one of ADR-046 — the `AC_` key before the tag — on
/// the framed rule, with the `TBD` branch of plan-025 slice 7 retired by
/// ADR-048; the expected-behaviour pattern is its `EX_` twin.
#[test]
fn the_ears_declaration_ships_to_managed_repositories() {
    let tag = "\\[(ubiquitous|event|state|conditional|optional|complex)\\]";
    for (schema, prefix) in [("feature-request.md", "AC_"), ("bug-report.md", "EX_")] {
        let pattern =
            format!("item-pattern: '^`{prefix}[a-z0-9-]+` {tag} '\n    {FRAMED_VARIANTS}\n");
        for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
            let text = same(&std::fs::read_to_string(repo(&format!("{root}/{schema}"))).unwrap());
            assert!(
                text.contains(&pattern),
                "{root}/{schema} declares the {prefix} EARS pattern on the framed rule"
            );
            assert!(!text.contains("TBD — )"), "{root}/{schema} admits no TBD");
        }
    }
}

/// The four declarations ADR-047 puts on a promise section, as the contract
/// schema writes them: the `P_` key, the tag-and-verb item, the modal verb
/// bound to items, and the retired or repeated verb no item may carry.
const PROMISE_DECLARATIONS: [&str; 4] = [
    "item-key: '^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'",
    r"item-pattern: '(?s)^`P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b'",
    r"item-only-pattern: '\b(SHALL|SHOULD|MAY|MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b'",
    r"item-prohibited-pattern: '\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\b|(?s)\b(SHALL|SHOULD|MAY)\b.*\b(SHALL|SHOULD|MAY)\b'",
];

/// The twelve kinds the one contract schema admits (ADR-043).
const KINDS: [&str; 12] = [
    "api",
    "events",
    "cli",
    "library",
    "interface",
    "ui",
    "data",
    "format",
    "config",
    "telemetry",
    "authz",
    "deployment",
];

/// The contract schema as shipped: its live copy and its pack source, which
/// `sync` keeps byte-equal.
fn contract_schema_copies() -> [(String, String); 2] {
    [
        "knowledge/schemas/contract.md",
        "pack/knowledge/schemas/contract.md",
    ]
    .map(|p| {
        let text = std::fs::read_to_string(repo(p)).expect("the contract schema is on file");
        (p.to_string(), text)
    })
}

/// The section rule for `heading` in a schema, as written — the first,
/// where a heading is declared once per variant (ADR-049).
fn rule_for(schema: &str, heading: &str) -> String {
    rules_for(schema, heading)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no rule for {heading}"))
}

/// Every section rule for `heading` in a schema, in declared order.
fn rules_for(schema: &str, heading: &str) -> Vec<String> {
    let schema = same(schema);
    let anchor = format!("  - heading: \"{heading}\"\n");
    schema
        .match_indices(&anchor)
        .map(|(at, _)| {
            let rest = &schema[at + anchor.len()..];
            rest.find("\n  - heading")
                .map_or(rest, |end| &rest[..end])
                .to_string()
        })
        .collect()
}

/// Covers I034 criterion 7, I049 criterion 8 and I037 criteria 1 and 11:
/// the two promise sections the one schema has, Behaviour and Stability,
/// are bullet lists carrying the four ADR-047 declarations and no
/// `content-pattern`, and each rule's description states the citation form
/// — the bare key where the contract is the subject, the contract's id then
/// the key elsewhere — in the live tree and in the pack mirror (ADR-046).
#[test]
fn every_promise_section_declares_its_shape() {
    for (p, text) in contract_schema_copies() {
        for heading in ["Behaviour", "Stability"] {
            let rule = same(&rule_for(&text, heading));
            assert!(
                rule.contains("content: bullet-list"),
                "{p}: {heading} is a bullet list of promises"
            );
            assert!(
                !rule.contains("content-pattern"),
                "{p}: {heading} binds its items, not a keyword somewhere in its body"
            );
            for declaration in PROMISE_DECLARATIONS {
                assert!(
                    rule.contains(declaration),
                    "{p}: {heading} lacks `{declaration}`"
                );
            }
            let description = folded(&rule);
            assert!(
                description.contains("cited bare where the contract is the subject")
                    && description.contains("after the contract's id elsewhere"),
                "{p}: {heading} does not state the citation form: {description}"
            );
        }
        let behaviour = folded(&rule_for(&text, "Behaviour"));
        for phrase in [
            "`contract-002-cli-superdev P_init-outside-git`",
            "MUST, REQUIRED, RECOMMENDED and OPTIONAL are retired",
            "numbered list is a sequence, never a promise",
            "no item reads TBD",
        ] {
            assert!(
                behaviour.contains(phrase),
                "{p}: Behaviour's description lacks `{phrase}`"
            );
        }
    }
}

/// A section rule with its folded `description` read as the one line YAML
/// makes of it: the line breaks and their indentation become one space.
fn folded(rule: &str) -> String {
    same(rule)
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" ")
}

/// One contract of kind `cli` whose Behaviour is `behaviour` and whose
/// Stability is `stability`, under the base sections every kind carries.
fn contract(behaviour: &str, stability: &str) -> String {
    format!(
        "---\ntype: Contract\nid: contract-999-cli-probe\nkind: cli\ntitle: t\n\
         description: d\nlifecycle: active\n---\n\n# CLI contract: probe\n\n## Definition\n\n\
         <!-- sokf:include /src/main.rs#cli -->\n```rust\npub struct Cli;\n```\n\
         <!-- /sokf:include -->\n\n## Behaviour\n\n{behaviour}\n## Stability\n\n{stability}"
    )
}

/// Covers I037 AC_c2, AC_c5, AC_c6, AC_c7 and AC_c8: under the final contract schema a
/// keyless bullet, a `MUST` item, a two-verb item, a `SHALL` in a paragraph
/// and a `TBD` item each fail `validate` with a finding naming the item or
/// the line — the same schema set the live tree is checked with, so what
/// fails here fails a filed contract.
#[test]
fn a_contract_departing_from_the_promise_form_fails_naming_each_departure() {
    let behaviour = "The probe SHALL be described here.\n\n\
                     ### Exit codes\n\n\
                     - `P_exit-zero` [event] WHEN the probe succeeds, the probe SHALL exit 0.\n\
                     - [event] WHEN the key is missing, the probe SHALL say so.\n\
                     - `P_retired-verb` [ubiquitous] The probe MUST exit 2 on a usage error.\n\
                     - `P_two-verbs` [state] WHILE running, the probe SHALL answer and\n  \
                       SHOULD answer quickly.\n\
                     - TBD — whether the probe exits 3.\n\n\
                     ### Streams\n\n\
                     - `P_stdout` [ubiquitous] The probe SHALL write its report to stdout.\n";
    let stability = "- `P_unreleased` [ubiquitous] Every command above MAY change.\n";
    let found = findings_of("Contract", "probe.md", &contract(behaviour, stability));
    let named = |text: &str| found.iter().filter(|f| f.contains(text)).count();
    assert!(
        named("line `The probe SHALL be described here.` matches outside a top-level item") == 1,
        "{found:#?}"
    );
    assert!(
        named("item `- [event] WHEN the key is missing, the probe SHALL say so.` carries no key")
            == 1,
        "{found:#?}"
    );
    assert!(
        named(
            "item `- `P_retired-verb` [ubiquitous] The probe MUST exit 2 on a usage error.` matches `MUST`"
        ) == 1,
        "{found:#?}"
    );
    assert!(
        named(
            "item `- `P_two-verbs` [state] WHILE running, the probe SHALL answer and` matches `SHALL answer and SHOULD`"
        ) == 1,
        "{found:#?}"
    );
    assert!(
        named("item `- TBD — whether the probe exits 3.` carries no key") == 1,
        "{found:#?}"
    );
    let sound = found
        .iter()
        .filter(|f| {
            f.contains("P_exit-zero") || f.contains("P_stdout") || f.contains("P_unreleased")
        })
        .collect::<Vec<_>>();
    assert!(
        sound.is_empty(),
        "a conforming promise is not reported: {sound:#?}"
    );
}

/// Covers I037 AC_c1: a numbered flow under Behaviour is a sequence
/// and never a promise (ADR-046), so a contract carrying one beside its
/// keyed bullets, with prose and a table beside them, passes the final
/// schema.
#[test]
fn a_numbered_flow_beside_keyed_promises_passes() {
    let behaviour = "Every command acts on the current directory.\n\n\
                     ### Exit codes\n\n\
                     | Code | Meaning |\n|------|---------|\n| 0 | success |\n\n\
                     - `P_exit-codes` [ubiquitous] The probe SHALL exit with the code the\n  \
                       table names.\n\n\
                     ### Streams\n\n\
                     A run proceeds in this order:\n\n\
                     1. The probe reads its input.\n\
                     2. The probe writes its report.\n\n\
                     - `P_stdout` [ubiquitous] The probe SHALL write its report to stdout.\n";
    let stability = "Unreleased.\n\n\
                     - `P_unreleased` [ubiquitous] Every command above MAY change without\n  \
                       notice.\n";
    let found = findings_of("Contract", "probe.md", &contract(behaviour, stability));
    assert!(found.is_empty(), "{found:#?}");
}

/// Covers I034 criterion 7 and I049 criterion 9: the Definition is
/// materialised, never authored — its rule declares `content: include`, no
/// pattern and none of the withdrawn `block-*` keys, so nothing inside it is
/// read (ADR-042).
#[test]
fn the_definition_declares_an_include_and_no_shape() {
    for (p, text) in contract_schema_copies() {
        let rule = rule_for(&text, "Definition");
        assert!(
            rule.contains("content: include"),
            "{p}: Definition is materialised from source"
        );
        for key in ["item-pattern", "content-pattern", "block-"] {
            assert!(
                !rule.contains(key),
                "{p}: Definition is definitional and declares no `{key}`"
            );
        }
    }
}

/// Covers I034 criterion 6 and I049 criterion 14: the one contract schema
/// carries a worked example per kind, and every example satisfies the base
/// rules and its own variant's — the example is the shape a contract writer
/// copies, so it teaches the modal form or teaches nothing.
#[test]
fn every_contract_example_passes_its_own_declarations() {
    let schemas = schemas("knowledge/schemas");
    let contracts: Vec<(String, String)> = schemas
        .into_iter()
        .filter(|(name, _)| name.starts_with("contract"))
        .collect();
    assert_eq!(
        contracts
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>(),
        ["contract.md"],
        "one contract schema, and no per-kind one"
    );
    let block = fenced_block(&contracts[0].1, "yaml").expect("the schema carries a yaml contract");
    let y: serde_yaml_ng::Value = serde_yaml_ng::from_str(&block).unwrap();
    let mut keys: Vec<&str> = y["example"]
        .as_mapping()
        .expect("example is keyed by kind")
        .keys()
        .map(|k| k.as_str().unwrap())
        .collect();
    keys.sort_unstable();
    let mut kinds = KINDS.to_vec();
    kinds.sort_unstable();
    assert_eq!(keys, kinds, "one example per kind");
    let found = superdev_core::validate::schema::document::check_examples(&contracts);
    assert!(found.is_empty(), "{found:#?}");
}

/// Covers I034 criterion 3: a schema declaring an `item-pattern` beside a
/// content kind with no items is reported on the schema file, and the rule
/// binds nothing. `validate` reports this through the grammar; this pins the
/// document layer's own guard, which no other test reaches.
#[test]
fn an_item_pattern_without_a_list_kind_is_a_schema_finding() {
    let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: Probe\n\
                sections:\n  - heading: Body\n    level: 2\n    content: prose\n\
                \x20   item-pattern: 'MUST'\n````\n";
    let found = superdev_core::validate::schema::document::check_declarations(&[(
        "probe.md".into(),
        text.into(),
    )]);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].message.contains("content is not"), "{found:#?}");
}

/// Covers I037 AC_c14: the validator reads `item-key`,
/// `item-only-pattern` and `item-prohibited-pattern`, so contract-010's
/// Behaviour still declares each and no clause about them is `PENDING`
/// any more. A clause about a later feature's declaration may be
/// (ADR-044); this test reads the three it covers.
#[test]
fn contract_010_no_longer_defers_the_item_declarations() {
    let path = "knowledge/contracts/internal/active/contract-010-interface-document-schemas.md";
    let text = same(&std::fs::read_to_string(repo(path)).unwrap());
    let behaviour = text
        .split("\n## Behaviour\n")
        .nth(1)
        .and_then(|rest| rest.split("\n## Stability\n").next())
        .expect("contract-010 carries Behaviour before Stability");
    let clauses = behaviour
        .replace('\n', " ")
        .split(['.', ';'])
        .map(str::to_string)
        .collect::<Vec<_>>();
    let deferred: Vec<&String> = clauses
        .iter()
        .filter(|clause| {
            clause.contains("PENDING")
                && ["item-key", "item-only-pattern", "item-prohibited-pattern"]
                    .iter()
                    .any(|declaration| clause.contains(declaration))
                // ADR-051's declarations reuse the three names one level down;
                // a promise about them may run ahead of its code (ADR-044).
                && !clause.contains("nested")
                && !clause.contains("item-key-optional")
        })
        .collect();
    assert!(deferred.is_empty(), "{deferred:#?}");
    for declaration in [
        "`item-key`",
        "`item-only-pattern`",
        "`item-prohibited-pattern`",
    ] {
        assert!(
            clauses.iter().any(|clause| clause.contains(declaration)),
            "contract-010's Behaviour still declares {declaration}"
        );
    }
}

/// Covers I030 AC_one-schema-per-kind: the validator now selects one rule
/// per heading per variant (ADR-049), so contract-010's two promises about
/// it, `P_heading-per-variant` and `P_heading-rules-overlap`, are declared
/// and neither is `PENDING` any more.
#[test]
fn contract_010_no_longer_defers_the_per_variant_heading() {
    let path = "knowledge/contracts/internal/active/contract-010-interface-document-schemas.md";
    let text = same(&std::fs::read_to_string(repo(path)).unwrap());
    let behaviour = text
        .split("\n## Behaviour\n")
        .nth(1)
        .and_then(|rest| rest.split("\n## Stability\n").next())
        .expect("contract-010 carries Behaviour before Stability");
    let items: Vec<&str> = behaviour
        .split("\n- ")
        .filter(|item| item.starts_with("`P_heading-"))
        .collect();
    for key in ["`P_heading-per-variant`", "`P_heading-rules-overlap`"] {
        let item = items
            .iter()
            .find(|item| item.starts_with(key))
            .unwrap_or_else(|| panic!("contract-010's Behaviour declares {key}"));
        assert!(!item.contains("PENDING"), "{key} is still deferred: {item}");
    }
}

/// Covers I049 criterion 8: one schema governs every contract, and neither
/// tree ships a per-kind one — the sixteen ADR-043 retired, `contract-cli`
/// through `contract-ui`, and the `file-format` one ADR-037 retired before
/// them.
#[test]
fn no_kind_schema_is_on_file() {
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        let mut names: Vec<String> = std::fs::read_dir(repo(root))
            .expect("the schema directory")
            .map(|e| e.unwrap().file_name().to_str().unwrap().to_string())
            .filter(|n| n.starts_with("contract"))
            .collect();
        names.sort_unstable();
        assert_eq!(
            names,
            ["contract.md"],
            "{root} ships a per-kind contract schema"
        );
        assert!(
            !repo(&format!("{root}/fragments/contract-style.md")).exists(),
            "{root} still ships the contract-style fragment"
        );
    }
}

/// Covers I049 criterion 8: nothing a writer builds against names a retired
/// kind schema or a retired contract type. The records that say what
/// changed — the ADRs, the framed issue, the plans, the review reports and
/// the indexes that summarise them — legitimately name them, so the hunt is
/// scoped to the schemas, the contracts and the skills a writer reads to
/// build.
#[test]
fn nothing_names_a_retired_kind_schema() {
    let mut found = markdown_naming(
        &[
            "knowledge/schemas",
            "knowledge/contracts",
            "pack/knowledge/schemas",
            "pack/knowledge/concepts",
            "pack/knowledge/skills",
            ".claude/skills",
        ],
        &[
            "schema-contract-",
            "-file-format-",
            "file-format contract",
            "FileFormatContract",
            "TextFormatContract",
            "BinaryFormatContract",
            "sokf:include contract-style",
        ],
    );
    // contract-008's link note records what its `format` kind replaced.
    found.retain(|f| !f.contains("contract-008-format-template.md: TextFormatContract"));
    assert!(
        found.is_empty(),
        "a retired kind schema is still named where a writer builds: {found:#?}"
    );
}

/// Every `path: needle` pair where a markdown file under one of `roots` —
/// a file named directly, or any `.md` under a directory, recursively —
/// contains one of `needles`.
fn markdown_naming(roots: &[&str], needles: &[&str]) -> Vec<String> {
    fn walk(path: &std::path::Path, needles: &[&str], found: &mut Vec<String>) {
        if path.is_dir() {
            let Ok(entries) = std::fs::read_dir(path) else {
                return;
            };
            for entry in entries.flatten() {
                walk(&entry.path(), needles, found);
            }
        } else if path.extension().is_some_and(|e| e == "md") {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            for needle in needles {
                if text.contains(needle) {
                    found.push(format!("{}: {needle}", path.display()));
                }
            }
        }
    }
    let mut found = Vec::new();
    for root in roots {
        walk(&repo(root), needles, &mut found);
    }
    found.sort_unstable();
    found
}

/// Covers I030 `AC_backlog-retired`: no schema, skill, index or live
/// concept names the backlog (ADR-048). The records that say it retired —
/// the ADRs, the framed issue, the plans, the changelog, the four entries
/// that note where they came from — legitimately name it, so the hunt is
/// scoped to what a writer builds against: the schemas in both trees, the
/// skills in both trees, the pack's concept skeletons, the ideas index, and
/// the live knowledge root — its index and the concepts filed directly in
/// it.
#[test]
fn nothing_names_the_backlog() {
    let mut roots: Vec<String> = [
        "knowledge/schemas",
        "knowledge/ideas/index.md",
        "pack/knowledge/schemas",
        "pack/knowledge/concepts",
        "pack/knowledge/skills",
        ".claude/skills",
    ]
    .map(String::from)
    .to_vec();
    for entry in std::fs::read_dir(repo("knowledge")).expect("the knowledge root") {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "md") {
            roots.push(format!(
                "knowledge/{}",
                path.file_name().unwrap().to_str().unwrap()
            ));
        }
    }
    let roots: Vec<&str> = roots.iter().map(String::as_str).collect();
    let found = markdown_naming(&roots, &["backlog", "Backlog"]);
    assert!(
        found.is_empty(),
        "the backlog is still named where a writer builds: {found:#?}"
    );
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        assert!(
            !repo(&format!("{root}/backlog.md")).exists(),
            "{root} still ships the backlog schema"
        );
    }
    assert!(!repo("knowledge/backlog.md").exists());
    assert!(!repo("pack/knowledge/concepts/backlog.md").exists());
}

/// Covers I030 `AC_backlog-retired`: the backlog's four entries are on
/// file — its three under-consideration entries as ideas, listed in the
/// ideas index, and its decided-against entry as a `wontfix` chore under
/// `issues/wontfix/`, listed in the tracker's index — and each validates,
/// which `the_live_tree_passes` and the validator's own run over the tree
/// already hold; here each is read and its form checked.
#[test]
fn the_backlog_entries_are_ideas_and_a_wontfix_chore() {
    let ideas_index = std::fs::read_to_string(repo("knowledge/ideas/index.md")).unwrap();
    for id in [
        "idea-007-a-knowledge-capture-skill",
        "idea-008-templates-pre-fill-knowledge-skeletons",
        "idea-009-comment-preserving-manifest-stamping",
    ] {
        let path = format!("knowledge/ideas/{id}.md");
        let text = same(&std::fs::read_to_string(repo(&path)).expect(&path));
        assert!(text.contains("type: Idea"), "{path} is not an idea");
        assert!(
            text.contains(&format!("id: {id}")),
            "{path} carries another id"
        );
        assert!(
            text.contains("\n# Idea: "),
            "{path} lacks its `# Idea:` heading"
        );
        assert!(
            text.contains("\n## Motivation"),
            "{path} carries no Motivation"
        );
        assert!(
            ideas_index.contains(&format!("[sokf:{id}]")),
            "the ideas index does not list {id}"
        );
    }
    let id = "issue-051-chore-pin-node-in-the-managed-repo";
    let path = format!("knowledge/issues/wontfix/{id}.md");
    let text = same(&std::fs::read_to_string(repo(&path)).expect(&path));
    assert!(text.contains("type: Chore"));
    assert!(text.contains("lifecycle: wontfix"));
    assert!(
        text.contains("\n## Won't fix\n"),
        "{path} carries no verdict"
    );
    assert!(
        text.contains("\n- `DD_"),
        "{path}'s definition of done is not keyed"
    );
    let tracker_index = std::fs::read_to_string(repo("knowledge/issues/index.md")).unwrap();
    assert!(
        tracker_index.contains(&format!("[sokf:{id}]")),
        "the tracker index does not list {id}"
    );
}

/// The first fenced block carrying `tag`, without its markers.
///
/// Scanned a line at a time, so a CRLF checkout reads as its LF twin: a
/// fence is a fence whatever ends the line (I040). The closing marker is the
/// one that opened the block, so a ```` ```` ```` block may contain ``` ``` ````.
fn fenced_block(text: &str, tag: &str) -> Option<String> {
    let mut lines = text.lines();
    let marker = loop {
        let trimmed = lines.next()?.trim_start();
        let ticks = trimmed.len() - trimmed.trim_start_matches('`').len();
        if ticks >= 3 && trimmed[ticks..].trim() == tag {
            break trimmed[..ticks].to_string();
        }
    };
    let mut body = Vec::new();
    for line in lines {
        if line.trim_start().starts_with(&marker) {
            return Some(body.join("\n"));
        }
        body.push(line);
    }
    None
}

/// Covers I035 criterion 3: a schema demands a form and never a toolchain, so
/// its declarations hold whatever framework a managed repository builds on.
#[test]
fn no_schema_names_a_framework_or_a_toolchain() {
    // Named implementations, not interface description languages: a schema
    // may say "TypeSpec" or "protobuf" — those are forms a generator reads —
    // and may never say which library produced them.
    const TOOLCHAINS: [&str; 12] = [
        "clap",
        "commander",
        "cobra",
        "argparse",
        "click",
        "serde",
        "schemars",
        "rmcp",
        "axum",
        "express",
        "fastapi",
        "typer",
    ];
    let mut found = Vec::new();
    for entry in std::fs::read_dir(repo("knowledge/schemas")).expect("the schema directory") {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if name != "contract.md" {
            continue;
        }
        // Only what the schema demands: a worked example may name the
        // fictional library's own dependencies, which bind nobody.
        let whole = std::fs::read_to_string(&path).unwrap();
        let text = whole
            .split("\nexample:")
            .next()
            .unwrap_or_default()
            .to_lowercase();
        // Whole words only: "expressed" is not Express.
        let words: std::collections::BTreeSet<&str> =
            text.split(|c: char| !c.is_ascii_alphanumeric()).collect();
        for tool in TOOLCHAINS {
            if words.contains(tool) {
                found.push(format!("{name}: {tool}"));
            }
        }
    }
    assert!(found.is_empty(), "a schema names a toolchain: {found:#?}");
}

/// Every file with extension `ext` under `dir`, recursively.
fn files_with(dir: &std::path::Path, ext: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_with(&path, ext, found);
        } else if path.extension().is_some_and(|e| e == ext) {
            found.push(path);
        }
    }
}

/// Covers I049 criteria 21 and 23: no test under `crates/` opens the CLI,
/// the MCP, the config, any of the three format or any of the three
/// interface contracts and reads a fenced block out of it. The Definition
/// is an include the validator binds; a test that opened a fence in any of
/// them would be comparing a copy that no longer exists (ADR-042).
///
/// What this binds is structural, not a helper's name: a source file is
/// split into its top-level items, and it fails when an item names one of
/// the contracts and an item that could serve it — the same one, or any
/// item that is not itself a test — carries a fence reader, which is a
/// literal fence opener (` ``` `) or an identifier carrying `fenced`. A test
/// that names a contract beside another test that writes a fence into its
/// own fixture is fine; a test that names a contract and calls a helper
/// which splits on a fence is not, whatever the helper is called. This file
/// names the contracts and the readers here and is skipped for it.
#[test]
fn no_test_compares_a_fenced_block_of_an_included_contract_to_the_binary() {
    const CONTRACTS: [&str; 10] = [
        "contract-002-cli-superdev.md",
        "contract-003-api-sokf.md",
        "contract-004-config-superdev.md",
        "contract-005-format-pack.md",
        "contract-006-format-lock.md",
        "contract-007-interface-pack-resolution.md",
        "contract-008-format-template.md",
        "contract-009-interface-run-state.md",
        "contract-010-interface-document-schemas.md",
        "knowledge/contracts/internal",
    ];
    const READERS: [&str; 2] = ["```", "fenced"];
    let mut files = Vec::new();
    files_with(&repo("crates"), "rs", &mut files);
    assert!(!files.is_empty(), "the crates were read");
    let mut found = Vec::new();
    for path in files {
        if path.ends_with("normative_shapes.rs") {
            continue;
        }
        // A Windows checkout carries CRLF, on which the LF-only split below
        // would read the whole file as one item.
        let text = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        // One chunk per top-level item: each ends at a `}` in column 0, and
        // carries the doc comment, attributes and constants written before
        // the item.
        let items: Vec<&str> = text.split("\n}\n").collect();
        let names_a_contract = |item: &str| CONTRACTS.iter().any(|c| item.contains(c));
        let reads_a_fence = |item: &str| READERS.iter().any(|r| item.contains(r));
        let is_test = |item: &str| item.contains("#[test]");
        let helper_reads_a_fence = items
            .iter()
            .any(|item| !is_test(item) && reads_a_fence(item));
        for item in items.iter().filter(|item| names_a_contract(item)) {
            if reads_a_fence(item) || helper_reads_a_fence {
                let name = item
                    .lines()
                    .filter_map(|l| l.strip_prefix("fn "))
                    .filter_map(|l| l.split('(').next())
                    .next_back()
                    .unwrap_or("<no fn>");
                found.push(format!("{}: {name}", path.display()));
            }
        }
    }
    assert!(
        found.is_empty(),
        "an item names a contract whose definition is an include, and reads a fence out of it or can call a helper that does: {found:#?}"
    );
}

/// Covers I035 criterion 13: the plan schema and the feature-plan skill both
/// state that a slice closing a contract-implementation gap sorts first
/// (ADR-044), in the live tree and in the pack mirror.
#[test]
fn the_plan_orders_a_contract_gap_first() {
    for p in [
        "knowledge/schemas/feature-plan.md",
        "pack/knowledge/schemas/feature-plan.md",
        ".claude/skills/feature-plan/SKILL.md",
        "pack/knowledge/skills/feature-plan/SKILL.md",
    ] {
        let text = std::fs::read_to_string(repo(p)).expect("the file is on file");
        assert!(
            text.contains("contract-implementation gap"),
            "{p} does not state the ordering rule"
        );
    }
}

/// A skill as shipped: its live copy and its pack source, which `sync`
/// keeps byte-equal.
fn skill_copies(name: &str) -> [(String, String); 2] {
    [
        format!(".claude/skills/{name}/SKILL.md"),
        format!("pack/knowledge/skills/{name}/SKILL.md"),
    ]
    .map(|p| {
        let text = std::fs::read_to_string(repo(&p)).expect("the skill is on file");
        (p, text)
    })
}

/// The `task` of the step named `name` in a skill file.
fn step_task<'a>(skill: &'a str, name: &str) -> &'a str {
    let anchor = format!("<step name=\"{name}\" task=\"");
    let start = skill
        .find(&anchor)
        .unwrap_or_else(|| panic!("no step named {name}"))
        + anchor.len();
    let rest = &skill[start..];
    &rest[..rest.find("\" />").expect("the step closes")]
}

/// Covers I049 criteria 19, 20 and 21: integrate carries a step that reads a
/// touched contract as its consumer would and reports the three judgements
/// — the region against the surface, the prose against the kind's
/// checklist, the document against a reader — as a judgement that blocks
/// nothing, and says so when no contract was touched (ADR-042).
#[test]
fn the_integrate_skill_judges_a_touched_contract() {
    for (p, text) in skill_copies("integrate") {
        let task = step_task(&text, "JUDGE THE CONTRACTS");
        for phrase in [
            "as its consumer would",
            "omits part of the promised surface",
            "optional",
            "checklist",
            "could not learn the interface",
            "what you checked",
            "judgement",
            "blocks nothing",
            "not a validator finding",
            "No contract touched?",
            "report nothing further",
        ] {
            assert!(
                task.contains(phrase),
                "{p}: the judgement step lacks `{phrase}`"
            );
        }
    }
}

/// Covers I049 criterion 18: contract-design writes a new definition element
/// into its source region with the behaviour unbuilt, and commits that
/// declaration under the approval the phase already requires (ADR-044).
#[test]
fn the_contract_design_skill_declares_in_source() {
    for (p, text) in skill_copies("contract-design") {
        let declare = step_task(&text, "DECLARE IN SOURCE");
        for phrase in ["marked source region", "behaviour unbuilt", "include"] {
            assert!(
                declare.contains(phrase),
                "{p}: the declaration step lacks `{phrase}`"
            );
        }
        let commit = step_task(&text, "COMMIT THE CONTRACTS");
        assert!(
            commit.contains("source declaration"),
            "{p}: the commit step does not name the source declaration"
        );
        assert!(
            text.contains("id=\"schema-contract\""),
            "{p} does not read the one contract schema"
        );
        assert!(
            !text.contains("schema-{kind}") && !text.contains("contract-interface"),
            "{p} still names a per-kind contract schema"
        );
    }
}

/// Covers I035 criteria 14 and 15 and I049 criteria 17 and 23: the pending
/// marker is declared where a contract writer meets it, acceptance refuses a
/// contract whose Behaviour or Stability still carries `PENDING`
/// (ADR-044), and no contract on file carries the withdrawn `pending:` key.
/// Whether a `PENDING` marker remains is the accept gate's to judge per
/// feature — a promise may outrun its code while a feature runs (ADR-044),
/// and contract-010 carries I037's — so this test does not assert its
/// absence.
#[test]
fn a_pending_promise_is_declared_and_bounded() {
    for (p, text) in contract_schema_copies() {
        let rule = rule_for(&text, "Behaviour");
        assert!(
            same(&rule).contains("carries PENDING beside its verb"),
            "{p} does not declare the pending marker where Behaviour is declared"
        );
    }
    for (p, text) in skill_copies("accept") {
        let gate = text
            .lines()
            .find(|l| l.contains("`PENDING`"))
            .unwrap_or_else(|| panic!("{p} does not refuse a pending marker at acceptance"));
        for phrase in ["<gate ", "Behaviour", "Stability", "ADR-044"] {
            assert!(
                gate.contains(phrase),
                "{p}: the pending gate lacks `{phrase}`"
            );
        }
        assert!(
            !text.contains("ADR-038"),
            "{p} still cites the superseded ADR-038"
        );
    }
    // The marker is `PENDING` in prose (ADR-044); the YAML `pending:` key
    // went with the authored blocks.
    let mut checked = Vec::new();
    for dir in [
        "knowledge/contracts/public/active",
        "knowledge/contracts/internal/active",
    ] {
        for entry in std::fs::read_dir(repo(dir)).expect("the contracts are on file") {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            assert!(
                !text.contains("\n  pending:"),
                "{} still carries the withdrawn `pending:` key",
                path.display()
            );
            checked.push(path.file_name().unwrap().to_str().unwrap().to_string());
        }
    }
    assert!(
        checked
            .iter()
            .any(|name| name == "contract-010-interface-document-schemas.md"),
        "contract-010 was not among the contracts read: {checked:?}"
    );
}

/// Covers I030 AC_file-issue, AC_file-idea and AC_promote-idea: `/file`
/// names the four kinds, writes the minimum record `unframed`, files an
/// idea per `schema-idea`, promotes an idea with a `references` link, and
/// says it does not interview, branch or invent (ADR-048).
#[test]
fn the_file_skill_files_without_framing() {
    for (p, text) in skill_copies("file") {
        for phrase in [
            "bug, feature request, chore or idea",
            "numbered after the highest issue across all of the tracker's folders",
            "`lifecycle: unframed`",
            "Summary and, where the kind carries it, Motivation in the user's words",
            "`TBD — <the open question>`",
            "no criterion, step or done item the user did not state",
            "per `schema-idea` into `knowledge/ideas/`, listed in its index",
            "`references` link to the idea",
            "the idea stays on file",
            "superdev validate --fix",
        ] {
            assert!(text.contains(phrase), "{p} lacks `{phrase}`");
        }
        assert!(
            text.contains(
                "<rule level=\"MUST NOT\">interview the user, create a branch, or invent a criterion"
            ),
            "{p} does not refuse to interview, branch or invent"
        );
        assert!(
            text.contains("<rule level=\"MUST NOT\">frame the issue"),
            "{p} does not leave framing to /frame"
        );
    }
}

/// Covers I030 AC_file-asks: a missing or unknown kind is asked for, and
/// nothing is filed.
#[test]
fn the_file_skill_asks_for_a_missing_kind() {
    for (p, text) in skill_copies("file") {
        let gate = text
            .lines()
            .find(|l| l.contains("The kind is given and is one of the four"))
            .unwrap_or_else(|| panic!("{p} has no gate on the kind"));
        for phrase in ["<gate ", "ask the user for the kind", "file nothing"] {
            assert!(gate.contains(phrase), "{p}: the kind gate lacks `{phrase}`");
        }
    }
}

/// Covers I030 AC_workflow-lists-file: the aggregator's source and its
/// rendered copy list `/file` outside the phases, and how-do-i's map names
/// it.
#[test]
fn the_workflow_lists_file_outside_the_phases() {
    let line = "<outside skill=\"/file\" when=\"an issue or an idea to record without framing it — /frame frames it when it is taken up\" />";
    for p in [
        "crates/lib/superdev-core/src/pipeline.rs",
        ".agents/superdev.md",
    ] {
        let text = std::fs::read_to_string(repo(p)).expect("the file is on file");
        assert!(
            text.contains(line),
            "{p} does not list /file outside the phases"
        );
        assert!(
            !text.contains("<phase name=\"FILE\""),
            "{p} lists /file as a phase"
        );
    }
    for (p, text) in skill_copies("how-do-i") {
        let map = step_body(&text, "MAP THE QUESTION");
        for phrase in ["`/file`", "without framing it", "`/frame` frames the issue"] {
            assert!(map.contains(phrase), "{p}: the map lacks `{phrase}`");
        }
    }
}

/// The body of the step named `name`, where the step carries its text
/// between its tags rather than in a `task` attribute.
fn step_body<'a>(skill: &'a str, name: &str) -> &'a str {
    let anchor = format!("<step name=\"{name}\">");
    let start = skill
        .find(&anchor)
        .unwrap_or_else(|| panic!("no step named {name}"))
        + anchor.len();
    let rest = &skill[start..];
    &rest[..rest.find("</step>").expect("the step closes")]
}

/// Covers I030 AC_skill-ships: `/file` is a knowledge-carried skill — the
/// pack source and the synced copy are byte-equal, and the lock claims the
/// copy.
#[test]
fn the_file_skill_ships_in_the_pack_and_the_lock() {
    let [(live, live_text), (pack, pack_text)] = skill_copies("file");
    assert_eq!(
        same(&live_text),
        same(&pack_text),
        "{live} differs from {pack}"
    );
    assert!(
        live_text.starts_with("---\nname: file\n"),
        "{live} does not open with the skill's name"
    );
    let lock = std::fs::read_to_string(repo(".superdev/lock.toml")).expect("the lock is on file");
    assert!(
        lock.contains("\".claude/skills/file/SKILL.md\" = \""),
        "the lock does not claim .claude/skills/file/SKILL.md"
    );
}

/// Covers I030 AC_frame-in-place: `/frame` reads a filed issue by id,
/// fetches an unframed one and frames it in that file, replaces every `TBD`,
/// keys and tags every criterion, and closes by setting `lifecycle: framed`
/// for `--fix` to refile (ADR-048).
#[test]
fn the_frame_skill_frames_an_unframed_issue_in_place() {
    for (p, text) in skill_copies("frame") {
        assert!(
            text.contains(
                "<tool_call name=\"sokf_read\" id=\"issue-{nnn}-{kind}-{slug}\" when=\"if an issue is given\" />"
            ),
            "{p} does not read a given issue by id"
        );
        let fetch = step_task(&text, "FILE OR FETCH THE ISSUE");
        for phrase in ["an unframed one `/file` filed", "in place"] {
            assert!(
                fetch.contains(phrase),
                "{p}: the fetch step lacks `{phrase}`"
            );
        }
        let criteria = step_task(&text, "WRITE ACCEPTANCE CRITERIA");
        for phrase in ["`AC_<slug>` [event]", "every `TBD`"] {
            assert!(
                criteria.contains(phrase),
                "{p}: the criteria step lacks `{phrase}`"
            );
        }
        let close = step_task(&text, "SET FRAMED");
        for phrase in [
            "`lifecycle: framed`",
            "superdev validate --fix",
            "`issues/framed/`",
        ] {
            assert!(
                close.contains(phrase),
                "{p}: the close-out lacks `{phrase}`"
            );
        }
        let (close_at, commit_at) = (
            text.find("<step name=\"SET FRAMED\"").unwrap(),
            text.find("<step name=\"COMMIT THE FRAME\"").unwrap(),
        );
        assert!(
            close_at < commit_at,
            "{p}: the close-out follows the commit"
        );
        assert!(
            !text.contains("`lifecycle: open`"),
            "{p} still files the issue `open`"
        );
    }
}

/// Covers I030 AC_frame-files: run with no issue, `/frame` creates it
/// `unframed` per its kind's schema and frames it in the same pass, so the
/// run ends `framed` (ADR-048).
#[test]
fn the_frame_skill_files_and_frames_in_one_pass() {
    for (p, text) in skill_copies("frame") {
        assert!(
            text.contains("input=\"a filed issue's id, or the new project or feature to frame\""),
            "{p} does not take a filed issue or a new feature"
        );
        let fetch = step_task(&text, "FILE OR FETCH THE ISSUE");
        for phrase in [
            "where none exists",
            "create it `lifecycle: unframed`",
            "per its kind's schema",
            "superdev validate --fix",
        ] {
            assert!(
                fetch.contains(phrase),
                "{p}: the fetch step lacks `{phrase}`"
            );
        }
        let (fetch_at, close_at) = (
            text.find("<step name=\"FILE OR FETCH THE ISSUE\"").unwrap(),
            text.find("<step name=\"SET FRAMED\"").unwrap(),
        );
        assert!(
            fetch_at < close_at,
            "{p}: the issue is filed after it is closed out"
        );
    }
}

/// Covers I030 AC_phases-refuse: contract-design, feature-plan and
/// execute-feature-plan each open their gates with one on the framed
/// issue's lifecycle, returning an unframed issue to `/frame` (ADR-048).
#[test]
fn the_later_phases_refuse_an_unframed_issue() {
    for (name, verb) in [
        ("contract-design", "designed"),
        ("feature-plan", "planned"),
        ("execute-feature-plan", "run"),
    ] {
        for (p, text) in skill_copies(name) {
            let first = text
                .lines()
                .find(|l| l.starts_with("<gate "))
                .unwrap_or_else(|| panic!("{p} has no gate"));
            assert!(
                first.contains("check=\"The framed issue's lifecycle is framed\""),
                "{p}: the first gate is not the lifecycle gate: {first}"
            );
            let on_fail =
                format!("on-fail=\"/frame — an unframed issue is framed before it is {verb}\"");
            assert!(
                first.contains(&on_fail),
                "{p}: the lifecycle gate lacks `{on_fail}`"
            );
        }
    }
    for (p, text) in skill_copies("execute-feature-plan") {
        let (lifecycle_at, contracts_at) = (
            text.find("lifecycle is framed").unwrap(),
            text.find("contracts are settled").unwrap(),
        );
        assert!(
            lifecycle_at < contracts_at,
            "{p}: the contracts gate precedes the lifecycle gate"
        );
    }
}

/// Covers I030 AC_lifecycle-values as the skills and the pack's concept
/// skeletons write it (code-review-010 findings 1 and 2): no skill in either
/// tree and no skeleton `superdev init` writes says `lifecycle: open` of an
/// issue — the value the tracker schemas refuse — or filters a search by
/// it. A plan is `open` while it runs, so a line naming a plan may say so;
/// the accept skill files a gap `unframed` and names `/frame`.
#[test]
fn no_skill_or_skeleton_writes_an_issue_open() {
    let mut found = Vec::new();
    let mut read = 0;
    for root in [
        "pack/knowledge/skills",
        ".claude/skills",
        "pack/knowledge/concepts",
    ] {
        let mut files = Vec::new();
        files_with(&repo(root), "md", &mut files);
        for path in files {
            read += 1;
            let text = std::fs::read_to_string(&path).unwrap();
            for (n, line) in text.lines().enumerate() {
                let says_open =
                    line.contains("lifecycle: open`") || line.contains("lifecycle: [\"open\"]");
                if says_open && !line.contains("plan-{nnn}") {
                    found.push(format!("{}:{}: {line}", path.display(), n + 1));
                }
            }
        }
    }
    assert!(read >= 20, "the skills and skeletons were read: {read}");
    assert!(found.is_empty(), "an issue is written `open`: {found:#?}");
    for (p, text) in skill_copies("accept") {
        let gaps = step_task(&text, "FILE GAPS");
        for phrase in ["`lifecycle: unframed`", "`/frame` frames it"] {
            assert!(gaps.contains(phrase), "{p}: FILE GAPS lacks `{phrase}`");
        }
    }
    for (p, text) in skill_copies("maintain") {
        assert!(
            text.contains("still `unframed` or `framed`"),
            "{p} does not audit an issue by the four states"
        );
    }
    let skeleton =
        same(&std::fs::read_to_string(repo("pack/knowledge/concepts/issue-tracker.md")).unwrap());
    let live = same(&std::fs::read_to_string(repo("knowledge/issue-tracker.md")).unwrap());
    assert_eq!(
        skeleton, live,
        "the pack's tracker skeleton is the live concept"
    );
}
