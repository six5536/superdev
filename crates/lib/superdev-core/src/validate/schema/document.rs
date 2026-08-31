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
use super::grammar::Ordered;
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

/// The content kinds a section rule may declare — the closed vocabulary of
/// contract-010. A kind outside this set is reported on the schema and binds
/// nothing.
const CONTENT_KINDS: [&str; 5] = ["prose", "bullet-list", "numbered-list", "table", "code"];

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

/// One frontmatter key's constraints, as a schema's `frontmatter:` block
/// declares them. A key declared with only a `description` deserialises to
/// an empty constraint and binds nothing — guidance, per ADR-022. Fields
/// outside these four (`description`) are ignored here.
#[derive(Debug, Clone, Default, Deserialize)]
struct KeyConstraint {
    /// Whether the key's absence is an error (ADR-022).
    #[serde(default)]
    required: bool,
    /// The one value the key must carry, when present.
    #[serde(default)]
    r#const: Option<String>,
    /// The anchored regex a present value must match.
    #[serde(default)]
    pattern: Option<String>,
    /// The values a present one must be among.
    #[serde(default)]
    r#enum: Vec<String>,
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
    /// Every key's constraint block, in declaration order. `Option` because
    /// a schema may write a key with nothing under it — an empty contract,
    /// binding nothing, rather than a schema that fails to parse.
    #[serde(default)]
    frontmatter: Ordered<Option<KeyConstraint>>,
}

impl DocSchema {
    /// The constraints declared for `key`, when there are any.
    fn constraint(&self, key: &str) -> Option<&KeyConstraint> {
        self.frontmatter.get(key)?.as_ref()
    }

    /// The `type` this schema governs, when it dispatches by type.
    #[must_use]
    pub fn type_const(&self) -> Option<&str> {
        self.constraint("type")?.r#const.as_deref()
    }

    /// Whether this schema names its documents by glob — the fallback for
    /// documents that carry no frontmatter to dispatch on.
    #[must_use]
    pub fn declares_glob(&self) -> bool {
        self.target_files.is_some()
    }

    /// The `lifecycle` values this schema admits; `None` when it declares
    /// none and its documents are outside the filing check.
    #[must_use]
    pub fn lifecycle_enum(&self) -> Option<&[String]> {
        Some(self.constraint("lifecycle")?.r#enum.as_slice()).filter(|values| !values.is_empty())
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
            for rule in &schema.sections {
                if let Some(kind) = rule.content.as_deref()
                    && !CONTENT_KINDS.contains(&kind)
                {
                    findings.push(Finding {
                        file: file.clone(),
                        message: format!(
                            "schema: section {} declares content `{kind}` — the kinds are \
                             prose, bullet-list, numbered-list, table and code",
                            rule.label()
                        ),
                        fatal: true,
                    });
                }
            }
            for (key, constraint) in schema.frontmatter.iter() {
                if let Some(pattern) = constraint.as_ref().and_then(|c| c.pattern.as_deref())
                    && re::compile(pattern).is_none()
                {
                    findings.push(Finding {
                        file: file.clone(),
                        message: format!(
                            "schema: frontmatter `{key}` declares pattern `{pattern}` — it does \
                             not compile, and binds nothing"
                        ),
                        fatal: true,
                    });
                }
            }
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

    /// Whether any schema governs this document.
    #[must_use]
    pub fn governs(&self, path: &str, doc_type: Option<&str>) -> bool {
        self.governing(path, doc_type).is_some()
    }

    /// The `lifecycle` enum the schema governing `doc_type` declares, when it
    /// declares one.
    #[must_use]
    pub fn lifecycle_enum(&self, doc_type: &str) -> Option<&[String]> {
        self.by_type.get(doc_type)?.lifecycle_enum()
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
    // Counted as an editor counts them, so a document at exactly its limit
    // passes: `split` yields a trailing empty element for the final newline,
    // and reporting that as one line over is an off-by-one nobody can act on.
    let count = doc.text.lines().count();
    if let Some(limit) = schema.line_limit
        && count > limit
    {
        push(format!(
            "{count} lines, over {}'s limit of {limit} — split it rather than trimming it",
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
    let mut matched: Vec<(usize, usize)> = Vec::new();
    for (h, (level, text)) in headings.iter().enumerate() {
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
            matched.push((h, i));
        }
    }

    for (i, rule) in schema.sections.iter().enumerate() {
        if rule.required && !matched.iter().any(|&(_, r)| r == i) {
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
        for &(_, i) in &matched {
            if !first.contains(&i) {
                first.push(i);
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

    // Each matched section must carry the form its rule's `content` kind
    // names — presence, per ADR-023: one bullet, one numbered item, one
    // table, one fenced block, or one plain paragraph line; other content
    // beside the form is tolerated. The body runs to the next heading at the
    // section's own level or shallower, so a subsection's content counts. A
    // kind outside the vocabulary was reported on the schema at load and
    // binds nothing.
    let positions = heading_positions(&lines);
    for &(h, r) in &matched {
        let rule = &schema.sections[r];
        let Some(kind) = rule.content.as_deref() else {
            continue;
        };
        if !CONTENT_KINDS.contains(&kind) {
            continue;
        }
        // A declared table's absence is check_columns' finding already.
        if kind == "table" && !rule.columns.is_empty() {
            continue;
        }
        let start = positions[h] + 1;
        let end = headings
            .iter()
            .enumerate()
            .skip(h + 1)
            .find(|(_, (level, _))| *level <= headings[h].0)
            .map_or(lines.len(), |(j, _)| positions[j]);
        if !body_has(kind, &lines[start..end]) {
            push(format!(
                "section {} carries no {}, and {} declares {kind} content",
                rule.label(),
                form_of(kind),
                schema.name
            ));
        }
    }

    check_columns(schema, &headings, &lines, &mut push);
    check_frontmatter(doc, schema, &mut push);
}

/// Every frontmatter key against the contract its schema declares: `const`,
/// `pattern` and `enum` bind a present value, `required` makes absence an
/// error, and a key declared with only a `description` is guidance
/// (ADR-022). A constraint compares against the value's scalar string form;
/// a value with no scalar form — a list, a map, a folded block — cannot
/// satisfy a scalar constraint and is a mismatch like any other. `lifecycle`
/// is the filing check's (P011), which already reports a value outside its
/// enum; reading it here too would say one fault twice.
fn check_frontmatter(doc: &Document<'_>, schema: &DocSchema, push: &mut impl FnMut(String)) {
    let fm = frontmatter_block(doc.text);
    let entries = super::read::parse_frontmatter(&fm);
    for (key, constraint) in schema.frontmatter.iter() {
        if key == "lifecycle" {
            continue;
        }
        let Some(c) = constraint else { continue };
        // Written with nothing after the colon is as absent as no line.
        let entry = entries
            .iter()
            .find(|e| e.key == key)
            .filter(|e| e.scalar.is_some() || e.block.is_some());
        let Some(entry) = entry else {
            if c.required {
                push(format!(
                    "frontmatter `{key}` is absent, and {} requires it",
                    schema.name
                ));
            }
            continue;
        };
        if c.r#const.is_none() && c.pattern.is_none() && c.r#enum.is_empty() {
            continue;
        }
        let scalar = if entry.is_folded || entry.block.is_some() {
            None
        } else {
            entry.scalar.as_deref()
        };
        let spell = scalar.map_or("is not a scalar".to_string(), |v| format!("is `{v}`"));
        if let Some(want) = c.r#const.as_deref()
            && scalar != Some(want)
        {
            push(format!(
                "frontmatter `{key}` {spell}, and {} declares const `{want}`",
                schema.name
            ));
        }
        if let Some(pattern) = c.pattern.as_deref()
            && re::compile(pattern).is_some()
            && !scalar.is_some_and(|v| re::matches(pattern, v))
        {
            push(format!(
                "frontmatter `{key}` {spell}, and {} declares pattern `{pattern}`",
                schema.name
            ));
        }
        if !c.r#enum.is_empty() && !scalar.is_some_and(|v| c.r#enum.iter().any(|a| a == v)) {
            push(format!(
                "frontmatter `{key}` {spell}, and {} declares one of: {}",
                schema.name,
                c.r#enum.join(", ")
            ));
        }
    }
}

/// The frontmatter block's lines, between the opening `---` and the line
/// that closes it; empty when the document carries none.
fn frontmatter_block(text: &str) -> Vec<&str> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Vec::new();
    };
    let Some(end) = rest.find("\n---") else {
        return Vec::new();
    };
    rest[..end].split('\n').collect()
}

/// The form a content kind demands, as a finding names it.
fn form_of(kind: &str) -> &'static str {
    match kind {
        "bullet-list" => "bullet",
        "numbered-list" => "numbered item",
        "table" => "table",
        "code" => "fenced block",
        _ => "paragraph line",
    }
}

/// Whether the kind's form appears in a section body. Lines inside fenced
/// blocks are not content: they neither satisfy a kind nor break one — the
/// fence itself is what satisfies `code`.
fn body_has(kind: &str, body: &[&str]) -> bool {
    let mut fenced = false;
    for line in body {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if !fenced && kind == "code" {
                return true;
            }
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let found = match kind {
            "bullet-list" => is_bullet(trimmed),
            "numbered-list" => is_numbered(trimmed),
            "table" => trimmed.starts_with('|'),
            "prose" => {
                !trimmed.is_empty()
                    && !is_bullet(trimmed)
                    && !is_numbered(trimmed)
                    && !trimmed.starts_with('|')
            }
            // `code` is satisfied by a fence alone, handled above.
            _ => false,
        };
        if found {
            return true;
        }
    }
    false
}

/// `- `, `* ` or `+ ` opens a bullet.
fn is_bullet(trimmed: &str) -> bool {
    ["- ", "* ", "+ "].iter().any(|b| trimmed.starts_with(b))
}

/// Digits then `. ` or `) ` opens a numbered item.
fn is_numbered(trimmed: &str) -> bool {
    let digits = trimmed.len()
        - trimmed
            .trim_start_matches(|c: char| c.is_ascii_digit())
            .len();
    digits > 0 && (trimmed[digits..].starts_with(". ") || trimmed[digits..].starts_with(") "))
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
            "knowledge/issues/issue-001-bug-x.md"
        ));
        assert!(!glob_match(
            "knowledge/issues/*.md",
            "knowledge/issues/deep/issue-001-bug-x.md"
        ));
        assert!(glob_match("**/*postmortem*.md", "docs/a-postmortem-b.md"));
        assert!(glob_match("**/*postmortem*.md", "postmortem.md"));
        assert!(glob_match("CHANGELOG.md", "CHANGELOG.md"));
        assert!(!glob_match("CHANGELOG.md", "docs/CHANGELOG.md"));
        // A dot is a literal, not "any character".
        assert!(!glob_match("READMEx.md", "README-.md"));
    }

    /// Every character a regex would read as syntax is a literal here, and a
    /// pattern matches the whole path or nothing. A glob that quietly became
    /// a regex would govern documents nobody named.
    #[test]
    fn a_pattern_is_a_glob_and_not_a_regex() {
        for meta in [
            "a+b.md", "a(b).md", "a[b].md", "a{b}.md", "a^b$.md", "a|b.md",
        ] {
            assert!(glob_match(meta, meta), "{meta} should match itself");
        }
        assert!(!glob_match("a+b.md", "aab.md"), "`+` is not a repeat");
        // `**` away from a separator spans them; `?` is one, and never `/`.
        assert!(glob_match("a**b.md", "a/x/b.md"));
        assert!(glob_match("a?.md", "ab.md"));
        assert!(!glob_match("a?.md", "a/.md"));
        // Anchored at both ends: a bare name never matches a nested path.
        assert!(!glob_match("x.md", "a/x.md"));
        assert!(!glob_match("x", "x.md"));
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
                frontmatter: Ordered::default(),
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

    /// The limit is inclusive, and a trailing newline is not a line. A
    /// document written to exactly its limit passes; one line more does not.
    #[test]
    fn a_document_at_exactly_its_limit_passes() {
        let schema = schema_of("frontmatter:\n  type:\n    const: T\nline-limit: 3\n");
        let at = Document {
            path: "a.md",
            text: "one\ntwo\nthree\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&at, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        let over = Document {
            path: "a.md",
            text: "one\ntwo\nthree\nfour\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&over, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.starts_with("4 lines, over"));
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

    /// Covers I018 criterion 1: the finding names the document, the section
    /// and the schema.
    #[test]
    fn a_bullet_list_section_without_a_bullet_is_a_finding() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n    content: bullet-list\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\nOnly prose here.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "a.md");
        assert!(findings[0].message.contains("\"Items\""), "{findings:#?}");
        assert!(findings[0].message.contains("s declares"), "{findings:#?}");
        assert!(findings[0].message.contains("bullet-list"), "{findings:#?}");
    }

    /// Covers I018 criterion 2: the kind binds the section's substance, so a
    /// lead-in sentence before the list passes.
    #[test]
    fn a_lead_in_sentence_before_the_bullets_passes() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n    content: bullet-list\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\nThe list below is the substance:\n\n- one\n- two\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// One pass and one fail body per remaining kind — numbered-list, table,
    /// code, prose. Presence of the form satisfies; its absence is a finding.
    #[test]
    fn each_kind_binds_by_presence() {
        let cases = [
            ("numbered-list", "1. first\n2. second\n", "just prose\n"),
            ("table", "| A | B |\n|---|---|\n| 1 | 2 |\n", "just prose\n"),
            ("code", "```sh\nls\n```\n", "just prose\n"),
            ("prose", "A paragraph line.\n", "- only\n- bullets\n"),
        ];
        for (kind, pass, fail) in cases {
            let schema = schema_of(&format!(
                "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Body\n    level: 2\n    content: {kind}\n"
            ));
            let ok = Document {
                path: "a.md",
                text: &format!("# T\n\n## Body\n\n{pass}"),
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&ok, &schema, &mut findings);
            assert!(findings.is_empty(), "{kind}: {findings:#?}");

            let bad = Document {
                path: "a.md",
                text: &format!("# T\n\n## Body\n\n{fail}"),
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&bad, &schema, &mut findings);
            assert_eq!(findings.len(), 1, "{kind}: {findings:#?}");
            assert!(findings[0].message.contains(kind), "{kind}: {findings:#?}");
        }
    }

    /// A section's body runs to the next heading at its own level or
    /// shallower, so the form may sit in a subsection — "no bullet anywhere"
    /// is what the finding means.
    #[test]
    fn a_form_in_a_subsection_satisfies_the_section() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n    content: bullet-list\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n### Grouped\n\n- one\n\n## Next\n\nprose\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // A bullet after the section's end does not count.
        let after = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\nprose\n\n## Next\n\n- one\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&after, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
    }

    /// Lines inside a fenced block are not content: a `-` in a fence
    /// satisfies nothing, a `#` in a fence ends no section, and the bullet
    /// after the fence still counts.
    #[test]
    fn lines_inside_a_fence_are_not_content() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n    content: bullet-list\n",
        );
        let fenced_only = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n```\n- not a bullet\n```\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&fenced_only, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");

        let bullet_after_fence = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n```\n# not a heading\n```\n\n- a bullet\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&bullet_after_fence, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I018 criterion 5: a kind outside the five is reported on the
    /// schema file, and the unreadable rule binds nothing.
    #[test]
    fn a_kind_outside_the_five_is_reported_on_the_schema_file() {
        let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    sections:\n  - heading: Body\n    level: 2\n    content: essay\n````\n";
        let (set, findings) = SchemaSet::load(&[("s.md".into(), text.into())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(findings[0].message.contains("essay"), "{findings:#?}");

        // The rule binds nothing: a body of any shape passes.
        let found = check_documents(
            &[Document {
                path: "a.md",
                text: "# T\n\n## Body\n\nanything\n",
                doc_type: Some("T"),
            }],
            &set,
        );
        assert!(found.is_empty(), "{found:#?}");
    }

    /// Covers I018 criterion 3: the finding names the document, the key and
    /// the schema.
    #[test]
    fn a_present_value_breaking_its_pattern_is_a_finding() {
        let schema =
            schema_of("frontmatter:\n  type:\n    const: T\n  id:\n    pattern: '^t-\\d{3}$'\n");
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\nid: t-1\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "a.md");
        assert!(
            findings[0].message.contains("`id` is `t-1`"),
            "{findings:#?}"
        );
        assert!(
            findings[0].message.contains("s declares pattern"),
            "{findings:#?}"
        );

        let ok = Document {
            path: "a.md",
            text: "---\ntype: T\nid: t-001\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&ok, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I018 criterion 3: a value outside its `enum`, and one differing
    /// from its `const`, are each findings.
    #[test]
    fn a_value_outside_its_enum_or_differing_from_its_const_is_a_finding() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\n  status:\n    enum: [draft, stable]\n\
             \x20 category:\n    const: reference\n",
        );
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\nstatus: stale\ncategory: opinion\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
        assert_eq!(findings.len(), 2, "{findings:#?}");
        assert!(
            messages
                .iter()
                .any(|m| m.contains("`status` is `stale`") && m.contains("one of: draft, stable")),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("`category` is `opinion`") && m.contains("const `reference`")),
            "{messages:?}"
        );
    }

    /// Covers I018 criterion 3: a key declared with only a `description` is
    /// guidance and passes any value.
    #[test]
    fn a_key_with_only_a_description_passes_any_value() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\n  title:\n    description: What it is called.\n",
        );
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\ntitle: Anything at all\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I018 criterion 4: an absent key with constraints is not
    /// reported — requiring one is the `required` flag's business.
    #[test]
    fn an_absent_key_with_constraints_is_not_reported() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\n  id:\n    pattern: '^t-\\d{3}$'\n\
             \x20 status:\n    enum: [draft, stable]\n",
        );
        // `status:` with nothing after the colon is as absent as no line.
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\nstatus:\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I018 criterion 4: an absent key marked `required: true` is an
    /// error naming the document, the key and the schema (ADR-022). A key
    /// written with nothing after the colon is as absent as no line.
    #[test]
    fn an_absent_required_key_is_a_finding() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\n  id:\n    required: true\n\
             \x20   pattern: '^t-\\d{3}$'\n  title:\n    required: true\n",
        );
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\ntitle:\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
        assert_eq!(findings.len(), 2, "{findings:#?}");
        assert_eq!(findings[0].file, "a.md");
        assert!(
            messages
                .iter()
                .any(|m| m.contains("`id` is absent") && m.contains("s requires it")),
            "{messages:?}"
        );
        assert!(
            messages.iter().any(|m| m.contains("`title` is absent")),
            "{messages:?}"
        );
    }

    /// Covers I018 criteria 3 and 4: a present key marked required is not an
    /// absence finding, and its value checks bind as they would without the
    /// flag.
    #[test]
    fn a_present_required_key_gets_its_value_checks() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\n  id:\n    required: true\n\
             \x20   pattern: '^t-\\d{3}$'\n",
        );
        let broken = Document {
            path: "a.md",
            text: "---\ntype: T\nid: t-1\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&broken, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("`id` is `t-1`")
                && findings[0].message.contains("pattern"),
            "{findings:#?}"
        );

        let ok = Document {
            path: "a.md",
            text: "---\ntype: T\nid: t-001\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&ok, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// A value with no scalar form — a list, a folded block — cannot satisfy
    /// a scalar constraint, and the mismatch is reported like any other.
    #[test]
    fn a_non_scalar_value_with_a_scalar_constraint_is_a_finding() {
        let schema =
            schema_of("frontmatter:\n  type:\n    const: T\n  id:\n    pattern: '^t-\\d{3}$'\n");
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\nid:\n  - t-001\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("`id` is not a scalar"),
            "{findings:#?}"
        );
    }

    /// Covers I018 criterion 5: a `pattern` that does not compile is reported
    /// on the schema file, and the unreadable rule binds nothing.
    #[test]
    fn an_uncompilable_pattern_is_reported_on_the_schema_file() {
        let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    \x20 id:\n    pattern: '(unclosed'\n````\n";
        let (set, findings) = SchemaSet::load(&[("s.md".into(), text.into())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(findings[0].message.contains("(unclosed"), "{findings:#?}");

        // The rule binds nothing: an id of any shape passes.
        let found = check_documents(
            &[Document {
                path: "a.md",
                text: "---\ntype: T\nid: whatever\n---\n\n# A\n",
                doc_type: Some("T"),
            }],
            &set,
        );
        assert!(found.is_empty(), "{found:#?}");
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
