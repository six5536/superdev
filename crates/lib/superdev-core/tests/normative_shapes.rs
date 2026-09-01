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

/// The ADR-032 assignment: which contract-kind sections declare that their
/// entries bind, and which are definitional and declare nothing. Kept here
/// rather than derived, so a section quietly losing its declaration fails.
/// `mcp` Tools left this set when it became a definition block (ADR-033): a
/// block defines the surface, and the keyword rule is for a list of promises.
const PROMISE_ITEMS: [(&str, &[&str]); 4] = [
    ("cli", &["Behaviour"]),
    ("data", &["Constraints"]),
    ("deployment", &["Health and lifecycle"]),
    ("events", &["Ordering and delivery"]),
];

const PROMISE_BODIES: [(&str, &[&str]); 15] = [
    ("authz", &["Boundaries", "Stability"]),
    ("cli", &["Stability"]),
    (
        "config",
        &["Sources and precedence", "Secrets", "Stability"],
    ),
    ("data", &["Migration", "Stability"]),
    // Runtime left this set when it became a definition block (ADR-033).
    ("deployment", &["Stability"]),
    ("events", &["Stability"]),
    ("text-format", &["Compatibility", "Stability"]),
    ("graphql", &["Errors", "Limits", "Stability"]),
    (
        "interface",
        &["Module boundaries", "Cross-cutting concerns"],
    ),
    ("library", &["Errors", "Stability"]),
    ("mcp", &["Server", "Errors", "Stability"]),
    ("rest", &["Authentication", "Stability"]),
    ("rpc", &["Authentication", "Stability"]),
    ("telemetry", &["Stability"]),
    ("ui", &["Stability"]),
];

/// The RFC 2119 keyword pattern ADR-032 declares: the uppercase forms alone
/// bind, so descriptive prose keeps its ordinary words.
const KEYWORDS: &str = r"\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b";

/// The section rule for `heading` in a contract-kind schema, as written.
fn rule_for<'a>(schema: &'a str, heading: &str) -> &'a str {
    let anchor = format!("  - heading: \"{heading}\"\n");
    let start = schema
        .find(&anchor)
        .unwrap_or_else(|| panic!("no rule for {heading}"))
        + anchor.len();
    let rest = &schema[start..];
    rest.find("\n  - heading").map_or(rest, |end| &rest[..end])
}

/// Covers I034 criterion 7: every promise-bearing section declares that its
/// entries bind, in the live tree and in the pack mirror (ADR-032).
#[test]
fn every_promise_section_declares_its_shape() {
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        for (kind, headings) in PROMISE_ITEMS {
            let text =
                std::fs::read_to_string(repo(&format!("{root}/contract-{kind}.md"))).unwrap();
            for heading in headings {
                let rule = rule_for(&text, heading);
                assert!(
                    rule.contains(&format!("item-pattern: '{KEYWORDS}'")),
                    "{root}/contract-{kind}.md: {heading} declares the keyword per item"
                );
            }
        }
        for (kind, headings) in PROMISE_BODIES {
            let text =
                std::fs::read_to_string(repo(&format!("{root}/contract-{kind}.md"))).unwrap();
            for heading in headings {
                let rule = rule_for(&text, heading);
                assert!(
                    rule.contains(&format!("content-pattern: '{KEYWORDS}'")),
                    "{root}/contract-{kind}.md: {heading} declares the keyword for its body"
                );
            }
        }
    }
}

/// Covers I034 criterion 7: a definitional section declares no shape — it
/// binds by form, and a keyword rule there would misfire (ADR-032).
#[test]
fn a_definitional_section_declares_no_shape() {
    for (kind, heading) in [
        ("cli", "Commands"),
        ("rest", "Endpoints"),
        ("interface", "Key flows"),
        ("ui", "Screens and states"),
        ("telemetry", "Metrics"),
        ("authz", "Permissions"),
        // Definition blocks: their shape is the block contract's to bind.
        ("cli", "Commands"),
        ("mcp", "Tools"),
    ] {
        for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
            let text =
                std::fs::read_to_string(repo(&format!("{root}/contract-{kind}.md"))).unwrap();
            let rule = rule_for(&text, heading);
            assert!(
                !rule.contains("item-pattern") && !rule.contains("content-pattern"),
                "{root}/contract-{kind}.md: {heading} is definitional and declares no pattern"
            );
        }
    }
}

/// Covers I034 criterion 6: every contract-kind schema's worked example
/// satisfies the declarations that schema carries — the example is the shape
/// a contract writer copies, so it teaches the modal form or teaches nothing.
#[test]
fn every_contract_schema_example_passes_its_own_declarations() {
    let schemas = schemas("knowledge/schemas");
    let contracts: Vec<(String, String)> = schemas
        .into_iter()
        .filter(|(name, _)| name.starts_with("contract-"))
        .collect();
    assert_eq!(contracts.len(), 16, "every contract kind was read");
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

/// Covers I035 criterion 9: the file-format kind is retired, and the two
/// kinds that replace it each govern their own type (ADR-037).
#[test]
fn the_format_kind_is_split_and_the_old_one_is_gone() {
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        assert!(
            !repo(&format!("{root}/contract-file-format.md")).exists(),
            "{root} still ships the retired file-format schema"
        );
        for (kind, konst) in [
            ("text-format", "TextFormatContract"),
            ("binary-format", "BinaryFormatContract"),
        ] {
            let text =
                std::fs::read_to_string(repo(&format!("{root}/contract-{kind}.md"))).unwrap();
            assert!(
                text.contains(&format!("const: {konst}")),
                "{root}/contract-{kind}.md governs {konst}"
            );
        }
    }
}

/// Covers I035 criterion 9: nothing a writer builds against names the
/// retired kind. The records that say what changed — the ADR, the framed
/// issue, the review reports and the indexes that summarise them —
/// legitimately name it, so the hunt is scoped to the schemas and contracts
/// a writer reads to build.
#[test]
fn nothing_names_the_retired_format_kind() {
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
                    "schema-contract-file-format",
                    "-file-format-pack",
                    "-file-format-lock",
                    "-file-format-template",
                    // The kind's own name, not only its id token: prose
                    // naming a retired kind misleads as much as a link does.
                    "file-format contract",
                    "FileFormatContract",
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
    ] {
        walk(&repo(root), &mut found);
    }
    assert!(
        found.is_empty(),
        "the retired kind is still named where a writer builds: {found:#?}"
    );
}

/// The section rules a schema declares, parsed from its yaml contract.
fn section_rules(text: &str) -> Vec<serde_yaml_ng::Value> {
    let block = text
        .split("````yaml\n")
        .nth(1)
        .and_then(|rest| rest.split("\n````").next())
        .expect("a schema carries a yaml contract");
    let y: serde_yaml_ng::Value = serde_yaml_ng::from_str(block).expect("the yaml contract parses");
    y.get("sections")
        .and_then(|s| s.as_sequence())
        .cloned()
        .unwrap_or_default()
}

/// Covers I035 criterion 1: every contract kind declares how its interface is
/// defined — a section whose content is a code block, or the tables that are
/// its native structured form — and that section demands the whole surface.
#[test]
fn every_contract_kind_declares_a_definition_form() {
    // These kinds define through tables rather than a fenced block: a
    // permission matrix, a byte layout, a metric set, a route list.
    const BY_TABLE: [&str; 4] = ["authz", "binary-format", "telemetry", "ui"];
    let mut seen = 0;
    for root in ["knowledge/schemas", "pack/knowledge/schemas"] {
        for entry in std::fs::read_dir(repo(root)).expect("the schema directory") {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            if !name.starts_with("contract-") {
                continue;
            }
            let kind = name
                .trim_start_matches("contract-")
                .trim_end_matches(".md")
                .to_string();
            let text = std::fs::read_to_string(&path).unwrap();
            if root == "knowledge/schemas" {
                seen += 1;
            }
            let rules = section_rules(&text);
            let wanted = if BY_TABLE.contains(&kind.as_str()) {
                "table"
            } else {
                "code"
            };
            let definitions: Vec<&serde_yaml_ng::Value> = rules
                .iter()
                .filter(|r| r.get("content").and_then(|c| c.as_str()) == Some(wanted))
                .collect();
            assert!(
                !definitions.is_empty(),
                "{root}/{name} declares no section whose content is {wanted}"
            );
            // Where the validator can read the block's language, the
            // section must declare what the block carries — otherwise the
            // block is a fence the validator opens and does not check.
            // Completeness against reality is the drift test's to bind; no
            // wording in a description can decide it.
            for rule in &definitions {
                let Some(language) = rule.get("block-language").and_then(|l| l.as_str()) else {
                    continue;
                };
                assert!(
                    ["yaml", "json"].contains(&language),
                    "{root}/{name}: block-language `{language}` is one the validator cannot read"
                );
                let keys = rule.get("block-keys").is_some();
                let entry_keys = rule.get("block-entry-keys").is_some();
                assert!(
                    keys || entry_keys,
                    "{root}/{name}: the block is parsed and nothing about it is checked"
                );
            }
            // A table definition binds by its columns, which the schema
            // engine checks.
            if wanted == "table" {
                assert!(
                    definitions.iter().any(|r| r
                        .get("columns")
                        .is_some_and(|c| c.as_sequence().is_some_and(|s| s.len() >= 2))),
                    "{root}/{name}: no table section declares the columns it binds"
                );
            }
        }
    }
    assert_eq!(seen, 16, "every contract kind was read");
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
        if !name.starts_with("contract-") {
            continue;
        }
        // Only what the schema demands: a worked example may name the
        // fictional library's own dependencies, which bind nobody.
        let whole = std::fs::read_to_string(&path).unwrap();
        let text = whole
            .split("example: |")
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

/// Covers I035 criterion 12: a drift test says which kind of red it is. An
/// element the implementation carries undeclared is a DEFECT; one the
/// contract promises and the implementation has yet to keep is PENDING
/// (ADR-038). The wording is the mechanism, so it is pinned here.
#[test]
fn every_drift_test_names_the_direction_it_failed_in() {
    const BINDINGS: [(&str, &[&str]); 4] = [
        (
            "crates/app/superdev/src/contract.rs",
            &["DEFECT —", "PENDING —", "DRIFT —"],
        ),
        (
            "crates/lib/superdev-core/src/sokf/mcp.rs",
            &["DEFECT —", "PENDING —", "DRIFT —"],
        ),
        (
            "crates/lib/superdev-core/tests/contract_files.rs",
            &["DEFECT —"],
        ),
        (
            "crates/lib/superdev-core/tests/contract_interfaces.rs",
            &["PENDING —"],
        ),
    ];
    for (path, wordings) in BINDINGS {
        let text = std::fs::read_to_string(repo(path)).expect("the drift test is on file");
        for wording in wordings {
            assert!(
                text.contains(wording),
                "{path} reports a drift without saying it is `{wording}`"
            );
        }
    }
}

/// Covers I035 criterion 13: the plan schema and the feature-plan skill both
/// state that a slice closing a contract-implementation gap sorts first
/// (ADR-038), in the live tree and in the pack mirror.
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

/// Covers I035 criteria 14 and 15: the pending marker is declared where a
/// contract writer meets it, acceptance refuses a contract still carrying
/// one, and no contract on file carries one (ADR-038).
#[test]
fn a_pending_promise_is_declared_bounded_and_absent() {
    for p in [
        "knowledge/schemas/contract-cli.md",
        "pack/knowledge/schemas/contract-cli.md",
    ] {
        let text = std::fs::read_to_string(repo(p)).unwrap();
        assert!(
            text.contains("carries\n      `pending`"),
            "{p} does not declare the pending marker"
        );
    }
    for p in [
        ".claude/skills/accept/SKILL.md",
        "pack/knowledge/skills/accept/SKILL.md",
    ] {
        let text = std::fs::read_to_string(repo(p)).unwrap();
        assert!(
            text.contains("still marks an element `pending`"),
            "{p} does not refuse a pending marker at acceptance"
        );
    }
    // A promise may outrun its code while a feature runs, never once it
    // settles — and this feature has settled.
    for dir in [
        "knowledge/contracts/public/active",
        "knowledge/contracts/internal/active",
    ] {
        for entry in std::fs::read_dir(repo(dir)).expect("the contracts are on file") {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            assert!(
                !text.contains("\n  pending:"),
                "{} still promises something unbuilt",
                path.display()
            );
        }
    }
}
