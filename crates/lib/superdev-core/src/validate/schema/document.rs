//! document.rs — checking a document against the schema that governs it.
//!
//! A schema names its documents in one of two ways. Almost all of them
//! declare a `type` const, and a concept carrying that type is governed by
//! that schema: one type, one schema, resolved by lookup. The handful whose
//! documents carry no frontmatter at all — `CHANGELOG.md`, `README.md`, an
//! index — name theirs with a `target-files` glob instead.
//!
//! The glob never touches the filesystem. It is matched against a candidate
//! list the caller assembles, which is what bounds it: a pattern like
//! `**/*release-notes*.md` cannot reach into `node_modules/` because nothing
//! from `node_modules/` is ever a candidate. Scoping a filesystem walk would
//! have to enumerate what to exclude, forever; scoping the input needs one
//! rule.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::Finding;
use super::re;

/// One section rule from a schema's contract.
#[derive(Debug, Clone, Deserialize)]
pub struct SectionRule {
    /// The literal heading this rule names.
    #[serde(default)]
    pub heading: Option<String>,
    /// The pattern a heading must match, when the name is the author's.
    #[serde(default, rename = "heading-pattern")]
    pub heading_pattern: Option<String>,
    /// Heading level, `#` count.
    #[serde(default)]
    pub level: Option<usize>,
    /// Whether the document must carry it.
    #[serde(default)]
    pub required: bool,
    /// Whether it may appear more than once.
    #[serde(default)]
    pub repeatable: bool,
    /// The shape of what sits under the heading.
    #[serde(default)]
    pub content: Option<String>,
    /// A table's columns, in order.
    #[serde(default)]
    pub columns: Vec<String>,
}

impl SectionRule {
    /// How the rule names its section, for a finding.
    fn label(&self) -> String {
        match (&self.heading, &self.heading_pattern) {
            (Some(h), _) => format!("\"{h}\""),
            (_, Some(p)) => format!("matching /{p}/"),
            _ => "(unnamed)".to_string(),
        }
    }

    /// Does `heading` satisfy this rule? Level is checked when the rule
    /// states one — a schema that leaves it open accepts any depth.
    fn matches(&self, level: usize, text: &str) -> bool {
        if self.level.is_some_and(|l| l != level) {
            return false;
        }
        match (&self.heading, &self.heading_pattern) {
            (Some(h), _) => h == text,
            (_, Some(p)) => re::compile(p).is_some_and(|re| re.is_match(text)),
            _ => false,
        }
    }
}

/// The frontmatter constraints a schema declares. Only `type` is read here:
/// the rest is the SOKF half's business.
#[derive(Debug, Clone, Default, Deserialize)]
struct FrontmatterContract {
    #[serde(default)]
    r#type: Option<TypeConstraint>,
}

#[derive(Debug, Clone, Deserialize)]
struct TypeConstraint {
    #[serde(default)]
    r#const: Option<String>,
}

/// A schema's contract, as far as document checking needs it.
#[derive(Debug, Clone, Deserialize)]
pub struct DocSchema {
    /// The schema file's own name, filled in after parsing.
    #[serde(skip)]
    pub name: String,
    #[serde(default, rename = "target-files")]
    target_files: Option<String>,
    #[serde(default, rename = "line-limit")]
    line_limit: Option<usize>,
    #[serde(default, rename = "sections-ordered")]
    sections_ordered: bool,
    #[serde(default)]
    sections: Vec<SectionRule>,
    #[serde(default, rename = "sections-prohibited")]
    sections_prohibited: Vec<String>,
    #[serde(default)]
    frontmatter: FrontmatterContract,
}

impl DocSchema {
    /// The `type` this schema governs, when it dispatches by type.
    #[must_use]
    pub fn type_const(&self) -> Option<&str> {
        self.frontmatter.r#type.as_ref()?.r#const.as_deref()
    }

    /// Parse a schema document's yaml contract. `None` when the document
    /// carries no contract to read — `check_schema` reports that; this is not
    /// the place to report it twice.
    #[must_use]
    pub fn parse(name: &str, text: &str) -> Option<DocSchema> {
        let fences = super::read::extract_yaml(text);
        let mut schema: DocSchema = serde_yaml_ng::from_str(&fences.first()?.text).ok()?;
        schema.name = name.to_string();
        Some(schema)
    }
}

/// Every schema, indexed by how it names its documents.
#[derive(Debug, Default)]
pub struct SchemaSet {
    /// Schemas that name their documents by frontmatter `type`.
    by_type: BTreeMap<String, DocSchema>,
    /// Schemas that name theirs by glob, for documents with no frontmatter.
    by_glob: Vec<DocSchema>,
}

impl SchemaSet {
    /// Build the set from the schema documents, reporting a type two schemas
    /// both claim — which would make dispatch a coin toss.
    ///
    /// `schemas` is (file name, text). A schema that declares neither a type
    /// const nor a glob governs nothing and is reported: it is a contract
    /// written for documents it can never reach.
    ///
    /// A schema with no documents *yet* is not reported. Several here govern
    /// kinds nobody has written — a postmortem, a migration guide — and a
    /// contract waiting for its first document is doing its job.
    #[must_use]
    pub fn load(schemas: &[(String, String)]) -> (SchemaSet, Vec<Finding>) {
        let mut set = SchemaSet::default();
        let mut findings = Vec::new();
        for (file, text) in schemas {
            let Some(schema) = DocSchema::parse(file, text) else {
                continue;
            };
            match (schema.type_const(), schema.target_files.as_deref()) {
                (Some(t), _) => {
                    if let Some(first) = set.by_type.get(t) {
                        findings.push(Finding {
                            file: file.clone(),
                            message: format!(
                                "schema: type `{t}` is already governed by {} — a type names one schema",
                                first.name
                            ),
                            fatal: true,
                        });
                        continue;
                    }
                    set.by_type.insert(t.to_string(), schema);
                }
                (None, Some(_)) => set.by_glob.push(schema),
                (None, None) => findings.push(Finding {
                    file: file.clone(),
                    message: "schema: governs nothing — declare a frontmatter `type` const, \
                              or a `target-files` glob for documents that carry no frontmatter"
                        .to_string(),
                    fatal: true,
                }),
            }
        }
        (set, findings)
    }

    /// The schema governing a document, and how it was found.
    fn governing(&self, path: &str, doc_type: Option<&str>) -> Option<&DocSchema> {
        if let Some(t) = doc_type {
            return self.by_type.get(t);
        }
        self.by_glob.iter().find(|s| {
            s.target_files
                .as_deref()
                .is_some_and(|g| glob_match(g, path))
        })
    }
}

/// One document to check: its repo-relative path, its text, and the `type`
/// its frontmatter declares.
pub struct Document<'a> {
    /// Repo-relative path, forward-slashed.
    pub path: &'a str,
    /// The whole file.
    pub text: &'a str,
    /// The frontmatter `type`, absent for a document that carries none.
    pub doc_type: Option<&'a str>,
}

/// Check every document against the schema its type or path names.
///
/// A document whose type names no schema is reported: the type is a claim
/// about which contract applies, and a claim that resolves to nothing is
/// worse than none, because it reads as governed.
#[must_use]
pub fn check_documents(docs: &[Document<'_>], set: &SchemaSet) -> Vec<Finding> {
    let mut findings = Vec::new();
    for doc in docs {
        let Some(schema) = set.governing(doc.path, doc.doc_type) else {
            if let Some(t) = doc.doc_type {
                findings.push(Finding {
                    file: doc.path.to_string(),
                    message: format!("type `{t}` names no schema"),
                    fatal: true,
                });
            }
            continue;
        };
        check_one(doc, schema, &mut findings);
    }
    findings
}

/// One document against one schema.
fn check_one(doc: &Document<'_>, schema: &DocSchema, findings: &mut Vec<Finding>) {
    let mut push = |message: String| {
        findings.push(Finding {
            file: doc.path.to_string(),
            message,
            fatal: true,
        });
    };

    let lines: Vec<&str> = doc.text.split('\n').collect();
    if let Some(limit) = schema.line_limit
        && lines.len() > limit
    {
        push(format!(
            "{} lines, over {}'s limit of {limit} — split it rather than trimming it",
            lines.len(),
            schema.name
        ));
    }

    let headings = headings(&lines);

    for banned in &schema.sections_prohibited {
        if headings.iter().any(|(_, h)| h == banned) {
            push(format!(
                "prohibited section \"{banned}\" ({} forbids it)",
                schema.name
            ));
        }
    }

    // Which rule each heading satisfies, in document order, so ordering can
    // be judged against the contract's own order. A rule naming the heading
    // literally wins over one matching it by pattern, whatever order the
    // schema declares them in: otherwise a catch-all pattern for
    // author-named sections would swallow every literal section after it,
    // and the schema's declaration order would be load-bearing in a way no
    // author would expect.
    let mut matched: Vec<usize> = Vec::new();
    for (level, text) in &headings {
        let literal = schema
            .sections
            .iter()
            .position(|r| r.heading.is_some() && r.matches(*level, text));
        let by_pattern = || {
            schema
                .sections
                .iter()
                .position(|r| r.heading.is_none() && r.matches(*level, text))
        };
        if let Some(i) = literal.or_else(by_pattern) {
            matched.push(i);
        }
    }

    for (i, rule) in schema.sections.iter().enumerate() {
        if rule.required && !matched.contains(&i) {
            push(format!(
                "missing required section {} ({})",
                rule.label(),
                schema.name
            ));
        }
    }

    if schema.sections_ordered {
        // A repeatable rule may recur, so compare only the first appearance
        // of each: a plan with three workstreams is ordered, not out of order.
        let mut first: Vec<usize> = Vec::new();
        for i in &matched {
            if !first.contains(i) {
                first.push(*i);
            }
        }
        for pair in first.windows(2) {
            if pair[0] > pair[1] {
                push(format!(
                    "section {} comes after {}, and {} orders them the other way",
                    schema.sections[pair[1]].label(),
                    schema.sections[pair[0]].label(),
                    schema.name
                ));
                break;
            }
        }
    }

    check_columns(schema, &headings, &lines, &mut push);
}

/// Every table a rule declares columns for must carry exactly those columns,
/// in that order: the columns are the contract a reader relies on.
fn check_columns(
    schema: &DocSchema,
    headings: &[(usize, String)],
    lines: &[&str],
    push: &mut impl FnMut(String),
) {
    for rule in &schema.sections {
        if rule.columns.is_empty() {
            continue;
        }
        for (index, (level, text)) in heading_positions(lines).into_iter().zip(headings.iter()) {
            if !rule.matches(*level, text) {
                continue;
            }
            let Some(header) = table_header(lines, index) else {
                push(format!(
                    "section {} carries no table, and {} declares its columns",
                    rule.label(),
                    schema.name
                ));
                continue;
            };
            if header != rule.columns {
                push(format!(
                    "section {} has columns {header:?}, and {} declares {:?}",
                    rule.label(),
                    schema.name,
                    rule.columns
                ));
            }
        }
    }
}

/// The first table's header cells after `from`, stopping at the next heading.
fn table_header(lines: &[&str], from: usize) -> Option<Vec<String>> {
    let mut fenced = false;
    for line in lines.iter().skip(from + 1) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if trimmed.starts_with('#') {
            return None;
        }
        if trimmed.starts_with('|') {
            return Some(
                trimmed
                    .trim_matches('|')
                    .split('|')
                    .map(|c| c.trim().to_string())
                    .collect(),
            );
        }
    }
    None
}

/// Every heading outside a fenced block, as (level, text).
fn headings(lines: &[&str]) -> Vec<(usize, String)> {
    heading_positions(lines)
        .into_iter()
        .map(|i| parse_heading(lines[i]).expect("a heading line parses"))
        .collect()
}

/// The line index of every heading outside a fenced block.
fn heading_positions(lines: &[&str]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut fenced = false;
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if !fenced && parse_heading(line).is_some() {
            out.push(i);
        }
    }
    out
}

/// `## Title` as (2, "Title").
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let hashes = line.len() - line.trim_start_matches('#').len();
    if hashes == 0 || !line.starts_with('#') {
        return None;
    }
    let rest = line[hashes..].strip_prefix(' ')?;
    Some((hashes, rest.trim_end().to_string()))
}

/// Match a `target-files` glob against a candidate path.
///
/// Supports the two forms the schemas use: `**/` for any directory prefix,
/// and `*` for any run of characters within one segment. Translated to a
/// regex rather than walked, because the candidate list is the scope.
#[must_use]
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let mut regex = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                // `**/` matches any number of leading segments, none included.
                if chars.peek() == Some(&'/') {
                    chars.next();
                    regex.push_str("(?:[^/]+/)*");
                } else {
                    regex.push_str(".*");
                }
            }
            '*' => regex.push_str("[^/]*"),
            '?' => regex.push_str("[^/]"),
            // Everything else is a literal. A `.` in particular must not
            // become "any character": `READMEx.md` is not `README-.md`.
            c => {
                if "\\.+()[]{}^$|".contains(c) {
                    regex.push('\\');
                }
                regex.push(c);
            }
        }
    }
    regex.push('$');
    re::compile(&regex).is_some_and(|re| re.is_match(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glob_stays_inside_one_segment_unless_told_otherwise() {
        assert!(glob_match(
            "knowledge/issues/*.md",
            "knowledge/issues/I001-x.md"
        ));
        assert!(!glob_match(
            "knowledge/issues/*.md",
            "knowledge/issues/deep/I001-x.md"
        ));
        assert!(glob_match("**/*postmortem*.md", "docs/a-postmortem-b.md"));
        assert!(glob_match("**/*postmortem*.md", "postmortem.md"));
        assert!(glob_match("CHANGELOG.md", "CHANGELOG.md"));
        assert!(!glob_match("CHANGELOG.md", "docs/CHANGELOG.md"));
        // A dot is a literal, not "any character".
        assert!(!glob_match("READMEx.md", "README-.md"));
    }

    /// The reach that made the old globs dangerous is closed by the candidate
    /// list, not by the pattern: `**/*release-notes*.md` still matches such a
    /// path, and no such path is ever offered.
    #[test]
    fn a_glob_matches_widely_and_is_only_ever_given_candidates() {
        assert!(glob_match(
            "**/*release-notes*.md",
            "node_modules/diff/release-notes.md"
        ));
        let set = SchemaSet {
            by_type: BTreeMap::new(),
            by_glob: vec![DocSchema {
                name: "release-notes".into(),
                target_files: Some("**/*release-notes*.md".into()),
                line_limit: None,
                sections_ordered: false,
                sections: Vec::new(),
                sections_prohibited: Vec::new(),
                frontmatter: FrontmatterContract::default(),
            }],
        };
        // Nothing from node_modules is a candidate, so nothing is governed.
        let findings = check_documents(&[], &set);
        assert!(findings.is_empty());
    }

    fn schema_of(yaml: &str) -> DocSchema {
        let text = format!("---\ntype: Schema\n---\n\n````yaml\n{yaml}\n````\n");
        DocSchema::parse("s", &text).expect("the contract parses")
    }

    #[test]
    fn a_missing_required_section_is_a_finding() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Context\n    level: 2\n    required: true\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Other\n\nx\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.contains("missing required section"));
    }

    #[test]
    fn sections_out_of_order_are_reported_once() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections-ordered: true\nsections:\n\
             \x20 - heading: One\n    level: 2\n    required: true\n\
             \x20 - heading: Two\n    level: 2\n    required: true\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Two\n\nx\n\n## One\n\ny\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.contains("orders them the other way"));
    }

    #[test]
    fn a_prohibited_section_and_an_over_limit_document_are_findings() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nline-limit: 3\nsections-prohibited:\n  - Summary\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Summary\n\nx\n\ny\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
        assert!(
            messages.iter().any(|m| m.contains("prohibited section")),
            "{messages:?}"
        );
        assert!(messages.iter().any(|m| m.contains("over")), "{messages:?}");
    }

    #[test]
    fn declared_columns_must_match_the_table() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Rows\n    level: 2\n\
             \x20   required: true\n    content: table\n    columns: [ID, Name]\n",
        );
        let mut findings = Vec::new();
        check_one(
            &Document {
                path: "a.md",
                text: "# T\n\n## Rows\n\n| ID | Other |\n|----|-------|\n| 1 | x |\n",
                doc_type: Some("T"),
            },
            &schema,
            &mut findings,
        );
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.contains("declares"));

        // The declared columns, in order, pass.
        let mut ok = Vec::new();
        check_one(
            &Document {
                path: "a.md",
                text: "# T\n\n## Rows\n\n| ID | Name |\n|----|------|\n| 1 | x |\n",
                doc_type: Some("T"),
            },
            &schema,
            &mut ok,
        );
        assert!(ok.is_empty(), "{ok:#?}");
    }

    /// A type that resolves to nothing reads as governed and is not. That is
    /// worse than carrying no type at all, so it is reported.
    #[test]
    fn a_type_naming_no_schema_is_a_finding() {
        let (set, findings) = SchemaSet::load(&[]);
        assert!(findings.is_empty());
        let found = check_documents(
            &[Document {
                path: "a.md",
                text: "x",
                doc_type: Some("Invented"),
            }],
            &set,
        );
        assert_eq!(found.len(), 1);
        assert!(found[0].message.contains("names no schema"));
    }

    #[test]
    fn two_schemas_claiming_one_type_is_a_finding() {
        let one = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n````\n";
        let (_, findings) =
            SchemaSet::load(&[("a.md".into(), one.into()), ("b.md".into(), one.into())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.contains("a type names one schema"));
    }

    #[test]
    fn a_schema_governing_nothing_is_a_finding() {
        let text = "---\ntype: Schema\n---\n\n````yaml\ndescription: x\n````\n";
        let (_, findings) = SchemaSet::load(&[("a.md".into(), text.into())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.contains("governs nothing"));
    }

    #[test]
    fn headings_inside_a_fence_are_not_headings() {
        let lines: Vec<&str> = "# Real\n\n```\n# Fenced\n```\n\n## Also real\n"
            .split('\n')
            .collect();
        let found: Vec<String> = headings(&lines).into_iter().map(|(_, h)| h).collect();
        assert_eq!(found, ["Real", "Also real"]);
    }
}
