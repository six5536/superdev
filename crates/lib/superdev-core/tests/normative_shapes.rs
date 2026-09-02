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
/// opens a criterion after its key, and an unsettled criterion reads TBD
/// after its key — the pattern admits both, and the frame phase is what
/// retires TBD.
#[test]
fn every_ears_tag_and_a_tbd_criterion_pass() {
    let criteria = "1. `AC_c1` [ubiquitous] THE SYSTEM SHALL do it.\n\
                    2. `AC_c2` [event] WHEN x THE SYSTEM SHALL do it.\n\
                    3. `AC_c3` [state] WHILE x THE SYSTEM SHALL do it.\n\
                    4. `AC_c4` [conditional] IF x THE SYSTEM SHALL do it.\n\
                    5. `AC_c5` [optional] WHERE x THE SYSTEM SHALL do it.\n\
                    6. `AC_c6` [complex] WHILE x WHEN y THE SYSTEM SHALL do it.\n\
                    7. `AC_x` TBD — whether it should.\n";
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

/// One bug-report body, with `steps` as its steps to reproduce.
fn bug(steps: &str) -> String {
    format!(
        "---\ntype: BugReport\nid: issue-999-bug-probe\ntitle: t\ndescription: d\n\
         lifecycle: open\n---\n\n# Bug: probe\n\n## Summary\n\nA line.\n\n\
         ## Environment\n\n- One.\n\n## Steps to reproduce\n\n{steps}\n\
         ## Expected behaviour\n\nA line.\n\n## Actual behaviour\n\nA line.\n\n\
         ## Root cause (if known)\n\nA line.\n\n## Proposed fix / workaround\n\n- One.\n\n\
         ## Regression risk\n\nA line.\n"
    )
}

/// One chore body, with `done` as its definition of done.
fn chore(done: &str) -> String {
    format!(
        "---\ntype: Chore\nid: issue-999-chore-probe\ntitle: t\ndescription: d\n\
         lifecycle: open\n---\n\n# Chore: probe\n\n## Summary\n\nA line.\n\n\
         ## Surfaces\n\n- One.\n\n## Definition of done\n\n{done}\n"
    )
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

/// The three tracker lists a plan case cites and the key prefix each
/// declares (ADR-046): the heading, the schema that carries it, the list's
/// marker kind and the prefix.
const CITED_LISTS: [(&str, &str, &str, &str); 3] = [
    (
        "Acceptance criteria",
        "feature-request.md",
        "numbered",
        "AC_",
    ),
    ("Steps to reproduce", "bug-report.md", "numbered", "RS_"),
    ("Definition of done", "chore.md", "bullet", "DD_"),
];

/// Covers I037 AC_c17, AC_c18 and AC_c19: the three tracker schemas declare
/// `item-key` with their prefix on the list a plan case cites — the
/// criterion's `item-pattern` admitting the key before the tag or `TBD`,
/// the step's and the done item's rule binding the key alone — and each
/// rule and the plan schema's case rule state the citation in keys, in the
/// live tree and in the pack mirror.
#[test]
fn every_cited_list_declares_its_key_and_the_plan_cites_keys() {
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        for (heading, schema, _, prefix) in CITED_LISTS {
            let path = format!("{root}/{schema}");
            let text = std::fs::read_to_string(repo(&path)).unwrap();
            let rule = rule_for(&text, heading);
            let key = format!("item-key: '^`({prefix}[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`'");
            assert!(rule.contains(&key), "{path}: {heading} lacks `{key}`");
            let description = folded(&rule);
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
        let request = std::fs::read_to_string(repo(&format!("{root}/feature-request.md"))).unwrap();
        let criteria = rule_for(&request, "Acceptance criteria");
        assert!(
            criteria.contains(
                "item-pattern: '^`AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` (\\[(ubiquitous|event|state|conditional|optional|complex)\\] |TBD — )'"
            ),
            "{root}: the criterion's key precedes its tag or its TBD: {criteria}"
        );
        for schema in ["bug-report.md", "chore.md"] {
            let text = std::fs::read_to_string(repo(&format!("{root}/{schema}"))).unwrap();
            let (heading, _, _, prefix) = CITED_LISTS
                .iter()
                .find(|(_, s, _, _)| *s == schema)
                .unwrap();
            let rule = rule_for(&text, heading);
            let tagged = format!(
                "item-prohibited-pattern: '^`{prefix}[a-z0-9-]+` \\[(ubiquitous|event|state|conditional|optional|complex)\\]'"
            );
            assert!(
                !rule.contains("item-pattern") && rule.contains(&tagged),
                "{root}/{schema}: {heading} binds the key alone and forbids a tag after it: {rule}"
            );
        }
        let bug_schema = std::fs::read_to_string(repo(&format!("{root}/bug-report.md"))).unwrap();
        let expected = rule_for(&bug_schema, "Expected behaviour");
        assert!(
            expected.contains("content: prose") && !expected.contains("item-key"),
            "{root}: Expected behaviour stays prose with no key (plan-025, deferred)"
        );
        let plan = std::fs::read_to_string(repo(&format!("{root}/feature-plan.md"))).unwrap();
        let slice = plan
            .split("  - heading-pattern: '^Slice \\d+: .+$'\n")
            .nth(1)
            .and_then(|rest| rest.split("\n  - heading").next())
            .expect("the plan schema carries the slice rule");
        let slice = folded(slice);
        for phrase in [
            "naming the keys of the acceptance criteria it covers",
            "\"covers AC_c1, AC_stale-include\"",
            "keyed repro steps (`RS_`)",
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

/// Covers I037 AC_c20: every issue on file carries a key on each
/// top-level item of the lists a plan case cites — `AC_` on a
/// feature-request's criteria, `RS_` on a bug's repro steps, `DD_` on a
/// chore's definition of done — the sweep's proof, read straight from the
/// tracker rather than through the schema.
#[test]
fn every_issue_on_file_carries_a_key_on_each_cited_item() {
    let mut keyed = 0;
    let mut issues = 0;
    for state in ["open", "done", "wontfix"] {
        let dir = repo(&format!("knowledge/issues/{state}"));
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }
            issues += 1;
            let text = same(&std::fs::read_to_string(&path).unwrap());
            let mut section: Option<(&str, &str)> = None;
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
                        .map(|(_, _, kind, prefix)| (*kind, *prefix));
                    continue;
                }
                let Some((kind, prefix)) = section else {
                    continue;
                };
                let item = match kind {
                    "numbered" => {
                        let digits = line.len()
                            - line.trim_start_matches(|c: char| c.is_ascii_digit()).len();
                        (digits > 0 && line[digits..].starts_with(". "))
                            .then(|| &line[digits + 2..])
                    }
                    _ => line.strip_prefix("- "),
                };
                if let Some(item) = item {
                    assert!(
                        item.starts_with(&format!("`{prefix}")),
                        "{name}: item `{line}` carries no {prefix} key"
                    );
                    keyed += 1;
                }
            }
        }
    }
    assert!(issues >= 50, "the tracker was read: {issues}");
    assert!(keyed >= 200, "the tracker's cited items were read: {keyed}");
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
/// EARS pattern, so a managed repository is held to it too. The pattern is
/// the keyed one of ADR-046 — the `AC_` key before the tag or the `TBD` —
/// which replaced the tag-first pattern of ADR-031 in plan-025 slice 7.
#[test]
fn the_ears_declaration_ships_to_managed_repositories() {
    let pattern = "item-pattern: '^`AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \
                   (\\[(ubiquitous|event|state|conditional|optional|complex)\\] |TBD — )'";
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        let text = std::fs::read_to_string(repo(&format!("{root}/feature-request.md"))).unwrap();
        assert!(text.contains(pattern), "{root} declares the EARS pattern");
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

/// The section rule for `heading` in the contract schema, as written.
fn rule_for(schema: &str, heading: &str) -> String {
    let schema = same(schema);
    let anchor = format!("  - heading: \"{heading}\"\n");
    let start = schema
        .find(&anchor)
        .unwrap_or_else(|| panic!("no rule for {heading}"))
        + anchor.len();
    let rest = &schema[start..];
    rest.find("\n  - heading")
        .map_or(rest, |end| &rest[..end])
        .to_string()
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
/// Behaviour still declares each and no clause of it is `PENDING` any more.
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
        .filter(|clause| clause.contains("PENDING"))
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
    fn walk(dir: &std::path::Path, found: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, found);
            } else if path.extension().is_some_and(|e| e == "md") {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                for needle in [
                    "schema-contract-",
                    "-file-format-",
                    "file-format contract",
                    "FileFormatContract",
                    "TextFormatContract",
                    "BinaryFormatContract",
                    "sokf:include contract-style",
                ] {
                    if text.contains(needle) {
                        found.push(format!("{}: {needle}", path.display()));
                    }
                }
            }
        }
    }
    let mut found = Vec::new();
    for root in [
        "knowledge/schemas",
        "knowledge/contracts",
        "pack/knowledge/schemas",
        "pack/knowledge/concepts",
        "pack/knowledge/skills",
        ".claude/skills",
    ] {
        walk(&repo(root), &mut found);
    }
    // contract-008's link note records what its `format` kind replaced.
    found.retain(|f| !f.contains("contract-008-format-template.md: TextFormatContract"));
    assert!(
        found.is_empty(),
        "a retired kind schema is still named where a writer builds: {found:#?}"
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

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &std::path::Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|e| e == "rs") {
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
    rust_files(&repo("crates"), &mut files);
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
