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

/// The findings an issue draws from the live schema set.
fn findings_for(path: &str, text: &str) -> Vec<String> {
    findings_of("Issue", path, text)
}

/// One issue body of `kind` in `lifecycle`, with `behaviour` under its
/// Behaviour heading and `tail` after Scope: the Resolution a settled one
/// carries, or nothing.
fn issue_in(kind: &str, lifecycle: &str, behaviour: &str, tail: &str) -> String {
    let word = match kind {
        "bug" => "Bug",
        "feature" => "Feature",
        _ => "Chore",
    };
    format!(
        "---\ntype: Issue\nid: issue-999-probe\ntitle: t\ndescription: d\nkind: {kind}\n\
         lifecycle: {lifecycle}\n---\n\n# {word}: probe\n\n## Summary\n\nA line.\n\n\
         ## Context\n\nA line.\n\n## Behaviour\n\n{behaviour}\n## Scope\n\nA line.\n\n\
         - In: one.\n\n{tail}"
    )
}

/// The Resolution section a settled issue carries.
const RESOLUTION: &str = "## Resolution\n\nShipped.\n";

/// The three lifecycle values the issue schema declares, in order (ADR-050).
const LIFECYCLES: [&str; 3] = ["open", "done", "wontfix"];

/// The three kinds an issue may be.
const ISSUE_KINDS: [&str; 3] = ["bug", "feature", "chore"];

/// The six headings of the template, in order.
const HEADINGS: [&str; 6] = [
    "Summary",
    "Context",
    "Behaviour",
    "Scope",
    "Resolution",
    "Comments",
];

/// Covers I052 AC_issue-schema and AC_issue-plain: under the live schema set
/// an open issue of each kind passes with prose, bullets after prose, or
/// both under Behaviour, carrying no key and no tag.
#[test]
fn an_open_issue_passes_with_prose_bullets_or_both() {
    for kind in ISSUE_KINDS {
        for behaviour in [
            "The report is one JSON object.\n",
            "What is expected:\n\n- The report is one JSON object.\n- The text output is unchanged.\n",
            "The flag emits the report.\n\n- When a finding is an error, validate exits non-zero.\n\nThe text output is unchanged.\n",
        ] {
            let found = findings_for("probe.md", &issue_in(kind, "open", behaviour, ""));
            assert!(found.is_empty(), "{kind}: {found:#?}");
        }
    }
}

/// Covers I052 AC_issue-resolution: an open issue carrying Resolution fails
/// naming the heading, and a done or wontfix issue without it fails naming
/// the heading; each settled state passes with it.
#[test]
fn resolution_is_refused_open_and_required_settled() {
    let found = findings_for(
        "probe.md",
        &issue_in("bug", "open", "A line.\n", RESOLUTION),
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("prohibited section \"Resolution\""),
        "{found:#?}"
    );
    for state in ["done", "wontfix"] {
        let found = findings_for("probe.md", &issue_in("feature", state, "A line.\n", ""));
        assert_eq!(found.len(), 1, "{state}: {found:#?}");
        assert!(
            found[0].contains("missing required section \"Resolution\""),
            "{state}: {found:#?}"
        );
        let found = findings_for(
            "probe.md",
            &issue_in("feature", state, "A line.\n", RESOLUTION),
        );
        assert!(found.is_empty(), "{state}: {found:#?}");
    }
}

/// An issue's `kind` is required and one of the three; the id carries no
/// kind segment.
#[test]
fn an_issue_carries_a_kind_and_a_plain_id() {
    let sound = issue_in("bug", "open", "A line.\n", "");
    let found = findings_for("probe.md", &sound.replace("kind: bug\n", ""));
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("`kind`"), "{found:#?}");
    let found = findings_for("probe.md", &sound.replace("kind: bug\n", "kind: defect\n"));
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("`kind`"), "{found:#?}");
    let found = findings_for(
        "probe.md",
        &sound.replace("id: issue-999-probe", "id: issue-999-bug-probe"),
    );
    assert!(found.is_empty(), "the pattern admits any slug: {found:#?}");
    let found = findings_for(
        "probe.md",
        &sound.replace("id: issue-999-probe", "id: bug-999"),
    );
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("`id`"), "{found:#?}");
}

/// Covers I052 AC_old-kinds-gone: a document typed by a retired tracker
/// kind names no schema, and no retired schema file ships in either tree.
#[test]
fn a_retired_tracker_type_names_no_schema() {
    for retired in ["BugReport", "FeatureRequest", "Chore"] {
        let found = findings_of(retired, "probe.md", "---\ntype: x\n---\n# x\n");
        assert_eq!(found.len(), 1, "{retired}: {found:#?}");
        assert!(
            found[0].contains(&format!("type `{retired}` names no schema")),
            "{retired}: {found:#?}"
        );
    }
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        for retired in ["bug-report.md", "feature-request.md", "chore.md"] {
            assert!(
                !repo(&format!("{root}/{retired}")).exists(),
                "{root} still ships {retired}"
            );
        }
        let index = std::fs::read_to_string(repo(&format!("{root}/index.md"))).unwrap();
        for retired in [
            "schema-bug-report",
            "schema-feature-request",
            "schema-chore",
        ] {
            assert!(!index.contains(retired), "{root}/index.md lists {retired}");
        }
        assert!(
            index.contains("[sokf:schema-issue]"),
            "{root}/index.md does not list the issue schema"
        );
    }
}

/// Covers I052 AC_issue-schema and AC_live-lifecycles as declared: the
/// issue schema declares `variant-key: lifecycle` over the three values,
/// `kind` over the three kinds, the six headings in order with prose
/// content and no item declaration, and one example per value that passes
/// its own check — in the live tree and in the pack mirror, byte-equal.
#[test]
fn the_issue_schema_declares_the_template() {
    let copies = [
        "knowledge/schemas/issue.md",
        "pack/knowledge/schemas/issue.md",
    ]
    .map(|p| (p, same(&std::fs::read_to_string(repo(p)).unwrap())));
    assert_eq!(
        copies[0].1, copies[1].1,
        "the pack copy differs from the live one"
    );
    for (path, text) in &copies {
        assert!(
            text.contains("\nvariant-key: lifecycle\n"),
            "{path}: lifecycle is the variant key"
        );
        assert!(
            text.contains("    enum: [open, done, wontfix]\n"),
            "{path}: lifecycle admits the three values"
        );
        assert!(
            text.contains("  kind:\n    required: true\n    enum: [bug, feature, chore]\n"),
            "{path}: kind is required and one of three"
        );
        assert!(
            text.contains("    const: Issue\n"),
            "{path}: the type is Issue"
        );
        assert!(
            text.contains("    pattern: '^issue-\\d{3}-[a-z0-9-]+$'\n"),
            "{path}: the id carries no kind segment"
        );
        let block = fenced_block(text, "yaml").expect("the schema carries a yaml contract");
        let y: serde_yaml_ng::Value = serde_yaml_ng::from_str(&block).unwrap();
        let sections = y["sections"].as_sequence().expect("sections");
        let headings: Vec<&str> = sections
            .iter()
            .filter_map(|s| s["heading"].as_str())
            .collect();
        assert_eq!(headings, HEADINGS, "{path}: the six headings in order");
        for section in sections {
            for key in [
                "item-key",
                "item-pattern",
                "item-only-pattern",
                "item-prohibited-pattern",
                "item-key-optional",
                "nested",
            ] {
                assert!(
                    section.get(key).is_none(),
                    "{path}: {} declares `{key}`",
                    section["heading"].as_str().unwrap_or("the title")
                );
            }
            if section.get("heading").is_some() {
                assert_eq!(
                    section["content"].as_str(),
                    Some("prose"),
                    "{path}: {} is not prose",
                    section["heading"].as_str().unwrap()
                );
            }
        }
        let keys: Vec<&str> = y["example"]
            .as_mapping()
            .unwrap_or_else(|| panic!("{path}: example is keyed by lifecycle"))
            .keys()
            .map(|k| k.as_str().unwrap())
            .collect();
        assert_eq!(keys, LIFECYCLES, "{path}: one example per value");
    }
    let schema: Vec<(String, String)> = schemas("knowledge/schemas")
        .into_iter()
        .filter(|(name, _)| name == "issue.md")
        .collect();
    assert_eq!(schema.len(), 1);
    let found = superdev_core::validate::schema::document::check_examples(&schema);
    assert!(found.is_empty(), "{found:#?}");
}

/// Every issue on file: its folder, its file name and its text, read
/// straight from the tracker's three lifecycle folders.
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

/// The value of `key` in an issue's frontmatter.
fn frontmatter_of<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}: ");
    text.strip_prefix("---\n")?
        .split("\n---\n")
        .next()?
        .lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(str::trim)
}

/// Covers I052 AC_issue-sweep: every issue on file is typed `Issue`, sits
/// in the folder its `lifecycle` names, carries one of the three kinds and
/// an id with no kind segment, and the tracker has no folder but the
/// three; the sweep's proof, read straight from the tracker rather than
/// through the schema.
#[test]
fn every_issue_on_file_is_typed_issue_in_its_lifecycles_folder() {
    let issues = issues_on_file();
    for (state, name, text) in &issues {
        assert_eq!(frontmatter_of(text, "type"), Some("Issue"), "{name}");
        assert_eq!(
            frontmatter_of(text, "lifecycle"),
            Some(*state),
            "{name}: filed under {state}/"
        );
        let kind = frontmatter_of(text, "kind").unwrap_or_else(|| panic!("{name}: no kind"));
        assert!(ISSUE_KINDS.contains(&kind), "{name}: kind `{kind}`");
        let id = frontmatter_of(text, "id").unwrap_or_else(|| panic!("{name}: no id"));
        assert_eq!(
            format!("{id}.md"),
            *name,
            "{name}: the file is named by its id"
        );
        let slug = &id["issue-000-".len()..];
        assert!(
            !slug.starts_with("bug-")
                && !slug.starts_with("feature-request-")
                && !slug.starts_with("chore-"),
            "{name}: the id keeps its kind segment"
        );
    }
    let mut folders: Vec<String> = std::fs::read_dir(repo("knowledge/issues"))
        .unwrap()
        .filter_map(|e| {
            let e = e.unwrap();
            e.path()
                .is_dir()
                .then(|| e.file_name().to_str().unwrap().to_string())
        })
        .collect();
    folders.sort_unstable();
    let mut states = LIFECYCLES.to_vec();
    states.sort_unstable();
    assert_eq!(
        folders, states,
        "the tracker's folders are the three states"
    );
    assert!(issues.len() >= 52, "the tracker was read: {}", issues.len());
}

/// Covers I052 AC_issue-sweep: every issue on file conforms to the issue
/// schema, in the live tree, and none carries a tracker key on a bullet or
/// numbered item.
#[test]
fn every_issue_on_file_conforms() {
    let (set, load) = SchemaSet::load(&schemas("knowledge/schemas"));
    assert!(load.is_empty(), "{load:#?}");
    let issues = issues_on_file();
    for (_, name, text) in &issues {
        let doc = Document {
            path: name,
            text,
            doc_type: Some("Issue"),
        };
        let found = check_documents(&[doc], &set);
        assert!(found.is_empty(), "{name}: {found:#?}");
        let keyed: Vec<&str> = text
            .lines()
            .filter(|line| {
                let item = line.trim_start();
                let digits =
                    item.len() - item.trim_start_matches(|c: char| c.is_ascii_digit()).len();
                let item = item.strip_prefix("- ").or_else(|| {
                    (digits > 0 && item[digits..].starts_with(". ")).then(|| &item[digits + 2..])
                });
                item.is_some_and(|item| {
                    ["`AC_", "`RS_", "`EX_", "`DD_"]
                        .iter()
                        .any(|prefix| item.starts_with(prefix))
                })
            })
            .collect();
        assert!(keyed.is_empty(), "{name}: keyed items remain: {keyed:#?}");
    }
    assert!(issues.len() >= 52, "the tracker was read: {}", issues.len());
}

/// The issues index lists every issue on file by its id and no other.
#[test]
fn the_issues_index_lists_every_issue_on_file() {
    let index = same(&std::fs::read_to_string(repo("knowledge/issues/index.md")).unwrap());
    let issues = issues_on_file();
    for (state, name, _) in &issues {
        let id = name.strip_suffix(".md").unwrap();
        assert!(
            index.contains(&format!("[sokf:{id}]: /knowledge/issues/{state}/{name}")),
            "the tracker index does not list {id} under {state}/"
        );
    }
    let listed = index
        .lines()
        .filter(|line| line.starts_with("[sokf:issue-"))
        .count();
    assert_eq!(listed, issues.len(), "the index lists an issue not on file");
    assert!(
        !index.contains("unframed/") && !index.contains("framed/"),
        "the index names a retired folder"
    );
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

/// Covers I052 AC_contract-criteria: a promise carries a nested bullet list
/// of the criteria that check it, each opening with its `AC_` key and its
/// EARS tag (ADR-050, ADR-051), and the contract passes the live schema set.
#[test]
fn a_contract_whose_promise_nests_keyed_criteria_passes() {
    let behaviour = "### Exit codes\n\n\
                     - `P_exit-codes` [ubiquitous] The probe SHALL exit with the code its\n  \
                       report names.\n  \
                       - `AC_exit-zero` [event] WHEN the probe succeeds, the probe SHALL\n    \
                         exit 0.\n  \
                       - `AC_exit-one` [event] WHEN a check fails, the probe SHALL exit 1.\n\n\
                     ### Streams\n\n\
                     - `P_stdout` [ubiquitous] The probe SHALL write its report to stdout.\n";
    let stability = "- `P_unreleased` [ubiquitous] Every command above MAY change.\n  \
                     - `AC_change-is-noted` [event] WHEN a command changes, the changelog\n    \
                       SHALL carry the change.\n";
    let found = findings_of("Contract", "probe.md", &contract(behaviour, stability));
    assert!(found.is_empty(), "{found:#?}");
}

/// Covers I052 AC_contract-criteria: a nested item lacking its key, one
/// lacking its tag, and a criterion key used twice across the contract each
/// fail the live schema set with a finding naming the item — the repeat
/// naming both items, under whichever promises they sit. A key is its whole
/// `<PREFIX>_<slug>`, so `AC_exit-codes` beside `P_exit-codes` is two keys
/// and no finding.
#[test]
fn a_nested_criterion_departing_from_the_form_fails_naming_each_departure() {
    let behaviour = "### Exit codes\n\n\
                     - `P_exit-codes` [ubiquitous] The probe SHALL exit with the code its\n  \
                       report names.\n  \
                       - [event] WHEN the probe succeeds, the probe SHALL exit 0.\n  \
                       - `AC_exit-one` WHEN a check fails, the probe SHALL exit 1.\n  \
                       - `AC_exit-codes` [event] WHEN a usage error occurs, the probe SHALL\n    \
                         exit 2.\n\n\
                     ### Streams\n\n\
                     - `P_stdout` [ubiquitous] The probe SHALL write its report to stdout.\n  \
                       - `AC_exit-codes` [event] WHEN the report is written, the probe SHALL\n    \
                         flush stdout.\n";
    let stability = "- `P_unreleased` [ubiquitous] Every command above MAY change.\n";
    let found = findings_of("Contract", "probe.md", &contract(behaviour, stability));
    let named = |text: &str| found.iter().filter(|f| f.contains(text)).count();
    assert!(
        named(
            "nested item `- [event] WHEN the probe succeeds, the probe SHALL exit 0.` carries no key"
        ) == 1,
        "{found:#?}"
    );
    assert!(
        named(
            "nested item `- `AC_exit-one` WHEN a check fails, the probe SHALL exit 1.` does not match"
        ) == 1,
        "{found:#?}"
    );
    let repeat = found
        .iter()
        .find(|f| f.contains("repeats key `AC_exit-codes`"))
        .unwrap_or_else(|| panic!("the repeated key is reported: {found:#?}"));
    assert!(
        repeat.contains(
            "nested item `- `AC_exit-codes` [event] WHEN the report is written, the probe SHALL`"
        ) && repeat.contains(
            "nested item `- `AC_exit-codes` [event] WHEN a usage error occurs, the probe SHALL`"
        ),
        "the finding names both items: {repeat}"
    );
    assert_eq!(found.len(), 3, "{found:#?}");
}

/// Covers I052 AC_contract-criteria-optional: the contracts on file, none
/// nesting a criterion, pass the live schema set unchanged.
#[test]
fn every_contract_on_file_passes_without_nested_criteria() {
    let mut paths = Vec::new();
    files_with(&repo("knowledge/contracts"), "md", &mut paths);
    let contracts: Vec<PathBuf> = paths
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n != "index.md"))
        .collect();
    assert_eq!(contracts.len(), 9, "{contracts:#?}");
    for path in contracts {
        let text = std::fs::read_to_string(&path).unwrap();
        let found = findings_of("Contract", path.to_str().unwrap(), &text);
        assert!(found.is_empty(), "{}: {found:#?}", path.display());
        assert!(
            !same(&text).contains("\n  - `AC_"),
            "{} nests a criterion",
            path.display()
        );
    }
}

/// Covers I052 AC_contract-criteria-optional: the Behaviour and Stability
/// rules declare the criteria a promise may nest — a `nested` rule keyed
/// `AC_`, tagged as a promise is, and not required — in the live tree and
/// in the pack mirror (ADR-050, ADR-051).
#[test]
fn the_promise_sections_declare_optional_nested_criteria() {
    for (p, text) in contract_schema_copies() {
        let block = fenced_block(&text, "yaml").expect("the schema carries a yaml contract");
        let y: serde_yaml_ng::Value = serde_yaml_ng::from_str(&block).unwrap();
        for heading in ["Behaviour", "Stability"] {
            let rule = y["sections"]
                .as_sequence()
                .unwrap()
                .iter()
                .find(|r| r["heading"].as_str() == Some(heading))
                .unwrap_or_else(|| panic!("{p}: no rule for {heading}"));
            let nested = &rule["nested"];
            assert_eq!(
                nested["item-key"].as_str(),
                Some("^`(AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`"),
                "{p}: {heading} keys its nested criteria `AC_`"
            );
            assert_eq!(
                nested["item-pattern"].as_str(),
                Some(
                    r"(?s)^`AC_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \[(ubiquitous|event|state|conditional|optional|complex)\] .*\b(SHALL|SHOULD|MAY)\b"
                ),
                "{p}: {heading} tags its nested criteria as it tags a promise"
            );
            assert_eq!(
                nested["required"].as_bool(),
                Some(false),
                "{p}: {heading} leaves the criteria optional"
            );
            let description = folded(&rule_for(&text, heading));
            assert!(
                description.contains("MAY carry"),
                "{p}: {heading}'s description does not say a promise MAY carry criteria: {description}"
            );
        }
        let behaviour = folded(&rule_for(&text, "Behaviour"));
        assert!(
            behaviour.contains("`contract-002-cli-superdev AC_init-outside-git`"),
            "{p}: Behaviour's description does not show how a criterion is cited: {behaviour}"
        );
    }
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

/// Covers I052 AC_contract-criteria: the validator reads `nested` and
/// `item-key-optional` (ADR-051), so contract-010's five promises about
/// them are declared and none is `PENDING` any more.
#[test]
fn contract_010_no_longer_defers_the_nested_declarations() {
    let path = "knowledge/contracts/internal/active/contract-010-interface-document-schemas.md";
    let text = same(&std::fs::read_to_string(repo(path)).unwrap());
    let behaviour = text
        .split("\n## Behaviour\n")
        .nth(1)
        .and_then(|rest| rest.split("\n## Stability\n").next())
        .expect("contract-010 carries Behaviour before Stability");
    let items: Vec<&str> = behaviour.split("\n- ").collect();
    for key in [
        "`P_nested-binds`",
        "`P_nested-required`",
        "`P_key-optional-unkeyed`",
        "`P_key-optional-keyed`",
        "`P_misdeclared-nested`",
    ] {
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
    let id = "issue-051-pin-node-in-the-managed-repo";
    let path = format!("knowledge/issues/wontfix/{id}.md");
    let text = same(&std::fs::read_to_string(repo(&path)).expect(&path));
    assert!(text.contains("type: Issue"));
    assert!(text.contains("kind: chore"));
    assert!(text.contains("lifecycle: wontfix"));
    assert!(
        text.contains("\n## Resolution\n"),
        "{path} carries no resolution"
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

/// Covers I030 AC_file-issue, AC_file-idea and AC_promote-idea and I052
/// AC_issue-schema: `/file` names the four kinds, writes the issue template
/// — `kind`, `lifecycle: open`, the six headings in prose and bullets with
/// no key — files an idea per `schema-idea`, promotes an idea with a
/// `references` link, and says it does not interview, branch or invent
/// (ADR-048, ADR-050).
#[test]
fn the_file_skill_files_without_framing() {
    for (p, text) in skill_copies("file") {
        for phrase in [
            "bug, feature request, chore or idea",
            "numbered after the highest issue across all of the tracker's folders",
            "`type: Issue`",
            "`kind`",
            "`lifecycle: open`",
            "Summary, Context, Behaviour, Scope, Resolution and Comments",
            "no Resolution",
            "a line of prose, and bullets beneath it",
            "no key",
            "no expectation the user did not state",
            "per `schema-idea` into `knowledge/ideas/`, listed in its index",
            "`references` link to the idea",
            "the idea stays on file",
            "superdev validate --fix",
        ] {
            assert!(text.contains(phrase), "{p} lacks `{phrase}`");
        }
        assert!(
            text.contains("<tool_call name=\"sokf_read\" id=\"schema-issue\""),
            "{p} does not read the issue schema"
        );
        for retired in [
            "bug-report",
            "feature-request.md",
            "schemas/chore",
            "TBD",
            "unframed",
        ] {
            assert!(!text.contains(retired), "{p} still names `{retired}`");
        }
        assert!(
            text.contains(
                "<rule level=\"MUST NOT\">interview the user, create a branch, or invent an expectation"
            ),
            "{p} does not refuse to interview, branch or invent"
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

/// Covers I052 AC_issue-schema as the tracker concept writes it: the live
/// concept and the pack's skeleton are one text, and it names the three
/// states, the three kinds, the six headings and no retired state or type.
#[test]
fn the_tracker_concept_describes_the_template() {
    let skeleton =
        same(&std::fs::read_to_string(repo("pack/knowledge/concepts/issue-tracker.md")).unwrap());
    let live = same(&std::fs::read_to_string(repo("knowledge/issue-tracker.md")).unwrap());
    assert_eq!(
        skeleton, live,
        "the pack's tracker skeleton is the live concept"
    );
    for phrase in [
        "`issue-<nnn>-<slug>`",
        "`issues/open/`",
        "`issues/done/`",
        "`issues/wontfix/`",
        "`bug`,\n  `feature` or `chore`",
        "Summary, Context, Behaviour, Scope, Resolution and\n  Comments",
        "`lifecycle: open`",
        "schema-issue",
    ] {
        assert!(
            live.contains(phrase),
            "the tracker concept lacks `{phrase}`"
        );
    }
    for retired in [
        "unframed",
        "framed",
        "BugReport",
        "FeatureRequest",
        "`Chore`",
        "`AC_`",
        "`RS_`",
        "`EX_`",
        "`DD_`",
    ] {
        assert!(
            !live.contains(retired),
            "the tracker concept still names `{retired}`"
        );
    }
}
