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
use crate::validate::sokf;

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
    /// The worked example — one document satisfying this schema, checked in
    /// place by `check_examples` (ADR-024). Absence is the grammar's schema
    /// check's finding, not this module's.
    #[serde(default)]
    example: Option<String>,
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

    /// Parse a schema document's yaml contract. `Ok(None)` when the document
    /// carries no contract to read — `check_schema` reports that; this is not
    /// the place to report it twice. `Err` when the contract is there but
    /// does not deserialize into this vocabulary — a schema that would
    /// otherwise drop silently and govern nothing.
    ///
    /// # Errors
    ///
    /// The deserialization error, worded by serde.
    pub fn parse(name: &str, text: &str) -> Result<Option<DocSchema>, String> {
        let fences = super::read::extract_yaml(text);
        let Some(fence) = fences.first() else {
            return Ok(None);
        };
        let mut schema: DocSchema =
            serde_yaml_ng::from_str(&fence.text).map_err(|e| e.to_string())?;
        schema.name = name.to_string();
        Ok(Some(schema))
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
            let schema = match DocSchema::parse(file, text) {
                Ok(Some(schema)) => schema,
                // No contract to read; `check_schema` reports that.
                Ok(None) => continue,
                // A contract that does not deserialize would otherwise drop
                // silently, leaving its documents ungoverned with only a
                // misleading "type X names no schema" to show for it.
                Err(e) => {
                    findings.push(Finding {
                        file: file.clone(),
                        message: format!("schema: the contract does not deserialize — {e}"),
                        fatal: true,
                    });
                    continue;
                }
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

/// The declarations the document checks cannot read, reported on the schema
/// itself: a `content` kind outside the vocabulary, a `pattern` that does
/// not compile. Each binds nothing. `validate` reports these through the
/// grammar's own schema check, so it does not call this — one fault, said
/// once; this is for callers checking documents without that pass.
#[must_use]
pub fn check_declarations(schemas: &[(String, String)]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (file, text) in schemas {
        let Ok(Some(schema)) = DocSchema::parse(file, text) else {
            continue; // `SchemaSet::load` reports a contract that fails.
        };
        for rule in &schema.sections {
            if let Some(kind) = rule.content.as_deref()
                && !CONTENT_KINDS.contains(&kind)
            {
                findings.push(Finding {
                    file: file.clone(),
                    message: format!(
                        "schema: section {} declares content `{kind}` — the kinds are {}",
                        rule.label(),
                        CONTENT_KINDS.join(", ")
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
    }
    findings
}

/// Every schema's example against the schema that declares it — in place,
/// with no dispatch, per ADR-024. Every failure is a finding on the schema
/// file, prefixed `example:` so a reader sees the example broke rather than
/// the schema's own shape. An example that does not parse as a document is a
/// finding too: a type-dispatched schema's example must open with a
/// frontmatter block whose text is YAML, while a glob-dispatched schema's
/// documents carry no frontmatter, so its example owes none.
#[must_use]
pub fn check_examples(schemas: &[(String, String)]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (file, text) in schemas {
        let Ok(Some(schema)) = DocSchema::parse(file, text) else {
            continue; // `SchemaSet::load` reports a contract that fails.
        };
        let Some(example) = schema.example.as_deref() else {
            continue; // The grammar's schema check reports a missing example.
        };
        let mut push = |message: String| {
            findings.push(Finding {
                file: file.clone(),
                message,
                fatal: true,
            });
        };
        let lines: Vec<&str> = example
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .collect();
        let body_start = match super::read::split_frontmatter(&lines) {
            Some(split) => {
                let fm = split.fm.join("\n");
                if let Err(e) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&fm) {
                    push(format!(
                        "example: does not parse as a document — the frontmatter is not YAML: {e}"
                    ));
                    continue;
                }
                split.body_start
            }
            None => {
                if schema.type_const().is_some() {
                    push(
                        "example: does not parse as a document — no frontmatter block".to_string(),
                    );
                    continue;
                }
                0
            }
        };
        let doc = Document {
            path: file,
            text: example,
            doc_type: None,
        };
        let mut broke = Vec::new();
        check_one(&doc, &schema, &mut broke);
        check_link_form(file, &lines[body_start..], &mut broke);
        findings.extend(broke.into_iter().map(|f| Finding {
            message: format!("example: {}", f.message),
            ..f
        }));
    }
    findings
}

/// The form of an example body's links, per ADR-025: a concept link takes
/// the `[text][sokf:<id>]` reference form, so a link whose target is a path
/// into the knowledge — the `knowledge/` directory, in either the repo-root
/// or the bare form — is an error. No id or target is resolved: a fictional
/// `sokf:` label passes, a URL or a repository path outside the knowledge
/// keeps its ordinary markdown form, and an image names a picture, never a
/// concept, so nothing is asked of one.
fn check_link_form(file: &str, body: &[&str], findings: &mut Vec<Finding>) {
    for link in sokf::scan_body(&body.join("\n")).links {
        if link.id.is_some() || link.image {
            continue;
        }
        let Some(path) = sokf::link_path(&link.dest) else {
            continue;
        };
        if path.trim_start_matches('/').starts_with("knowledge/") {
            findings.push(Finding {
                file: file.to_string(),
                message: format!(
                    "body link names a path into the knowledge: {} — a concept link takes \
                     the [text][sokf:<id>] form",
                    link.dest
                ),
                fatal: true,
            });
        }
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

    // `\r` is stripped so a CRLF document reads as its LF twin: the same
    // frontmatter, the same headings, the same content.
    let lines: Vec<&str> = doc
        .text
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
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

    // One fence reading for the whole check: nested and `~~~` fences are
    // read as `read::fence_map` reads them everywhere else.
    let fenced = super::read::fence_map(&lines);
    let headings = headings(&lines, &fenced);

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
    // kind outside the vocabulary is reported on the schema — by the
    // grammar's schema check — and binds nothing.
    let positions = heading_positions(&lines, &fenced);
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
        let (start, end) = body_range(h, &headings, &positions, lines.len());
        if !body_has(kind, &lines[start..end], &fenced[start..end]) {
            push(format!(
                "section {} carries no {}, and {} declares {kind} content",
                rule.label(),
                form_of(kind),
                schema.name
            ));
        }
    }

    check_columns(schema, &headings, &positions, &lines, &fenced, &mut push);
    check_frontmatter(&lines, schema, &mut push);
}

/// The body of the section whose heading is the `h`-th: from the line after
/// its heading to the next heading at the section's own level or shallower,
/// so a subsection's content counts (contract-010).
fn body_range(
    h: usize,
    headings: &[(usize, String)],
    positions: &[usize],
    total: usize,
) -> (usize, usize) {
    let end = headings
        .iter()
        .enumerate()
        .skip(h + 1)
        .find(|(_, (level, _))| *level <= headings[h].0)
        .map_or(total, |(j, _)| positions[j]);
    (positions[h] + 1, end)
}

/// Every frontmatter key against the contract its schema declares: `const`,
/// `pattern` and `enum` bind a present value, `required` makes absence an
/// error, and a key declared with only a `description` is guidance
/// (ADR-022). A constraint compares against the value's scalar string form;
/// a value with no scalar form — a list, a map, a folded block — cannot
/// satisfy a scalar constraint and is a mismatch like any other. A
/// `lifecycle` key with an enum is the filing check's (P011), which reports
/// absence, the value against the enum, and the folder; reading it here too
/// would say one fault twice. Without an enum nothing else reads the key,
/// and its constraints bind here like any other's.
fn check_frontmatter(lines: &[&str], schema: &DocSchema, push: &mut impl FnMut(String)) {
    let fm = super::read::split_frontmatter(lines).map_or_else(Vec::new, |s| s.fm);
    let entries = super::read::parse_frontmatter(&fm);
    for (key, constraint) in schema.frontmatter.iter() {
        if key == "lifecycle" && schema.lifecycle_enum().is_some() {
            continue;
        }
        let Some(c) = constraint else { continue };
        let entry = entries.iter().find(|e| e.key == key);
        // What the key carries: the value on its own line as YAML reads it —
        // comments stripped, quotes removed — or the block under it, whose
        // comment-only lines are comments rather than a block. A key with
        // nothing else after the colon is as absent as no line.
        let folded = entry.is_some_and(|e| e.is_folded);
        let block = entry.is_some_and(|e| {
            e.block
                .as_ref()
                .is_some_and(|b| b.iter().any(|l| !l.trim_start().starts_with('#')))
        });
        let value = entry
            .filter(|_| !folded)
            .and_then(|e| line_scalar(rest_of(&fm, e)));
        if !folded && !block && value.is_none() {
            if c.required {
                push(format!(
                    "frontmatter `{key}` is absent, and {} requires it",
                    schema.name
                ));
            }
            continue;
        }
        if c.r#const.is_none() && c.pattern.is_none() && c.r#enum.is_empty() {
            continue;
        }
        let scalar = if folded || block {
            None
        } else {
            value.as_deref()
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

/// What follows the colon on an entry's own line, verbatim.
fn rest_of<'a>(fm: &[&'a str], entry: &super::read::FmEntry) -> &'a str {
    fm[entry.line - 1]
        .split_once(':')
        .map_or("", |(_, rest)| rest)
}

/// The value on a key's own line, as YAML reads it: a leading quote runs to
/// its closing quote, and in a plain scalar a `#` — first on the line, or
/// preceded by whitespace — opens a comment. `None` when nothing but a
/// comment follows the colon.
fn line_scalar(rest: &str) -> Option<String> {
    let rest = rest.trim();
    for quote in ['"', '\''] {
        if let Some(body) = rest.strip_prefix(quote) {
            return body.split_once(quote).map(|(value, _)| value.to_string());
        }
    }
    let mut end = rest.len();
    for (i, _) in rest.match_indices('#') {
        if i == 0 || rest.as_bytes()[i - 1].is_ascii_whitespace() {
            end = i;
            break;
        }
    }
    let value = rest[..end].trim_end();
    (!value.is_empty()).then(|| value.to_string())
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

/// Whether the kind's form appears in a section body. `fenced` is the
/// body's slice of the document's fence map: lines inside fenced blocks are
/// not content — they neither satisfy a kind nor break one — and the fence
/// itself is what satisfies `code`.
fn body_has(kind: &str, body: &[&str], fenced: &[bool]) -> bool {
    for (line, &in_fence) in body.iter().zip(fenced) {
        if in_fence {
            // A section heading is never inside a fence, so a fenced line
            // here means a block opened within this body.
            if kind == "code" {
                return true;
            }
            continue;
        }
        let trimmed = line.trim_start();
        let found = match kind {
            "bullet-list" => is_bullet(trimmed),
            "numbered-list" => is_numbered(trimmed),
            "table" => trimmed.starts_with('|'),
            "prose" => is_paragraph(trimmed),
            // `code` is satisfied by a fence alone, handled above.
            _ => false,
        };
        if found {
            return true;
        }
    }
    false
}

/// A plain paragraph line: words, on a line that is not a list item, a
/// table row, a deeper heading, an HTML comment, a `[label]: target` link
/// definition, or a divider.
fn is_paragraph(trimmed: &str) -> bool {
    !trimmed.is_empty()
        && !is_bullet(trimmed)
        && !is_numbered(trimmed)
        && !trimmed.starts_with('|')
        && !trimmed.starts_with('#')
        && !trimmed.starts_with("<!--")
        && !is_link_definition(trimmed)
        && trimmed.chars().any(char::is_alphanumeric)
}

/// `[label]: target` — a markdown link reference definition.
fn is_link_definition(trimmed: &str) -> bool {
    trimmed
        .strip_prefix('[')
        .and_then(|rest| rest.split_once(']'))
        .is_some_and(|(_, after)| after.starts_with(':'))
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
/// in that order: the columns are the contract a reader relies on. The table
/// is sought in the same body range the content check reads, so one in a
/// subsection counts (contract-010).
fn check_columns(
    schema: &DocSchema,
    headings: &[(usize, String)],
    positions: &[usize],
    lines: &[&str],
    fenced: &[bool],
    push: &mut impl FnMut(String),
) {
    for rule in &schema.sections {
        if rule.columns.is_empty() {
            continue;
        }
        for (h, (level, text)) in headings.iter().enumerate() {
            if !rule.matches(*level, text) {
                continue;
            }
            let (start, end) = body_range(h, headings, positions, lines.len());
            let Some(header) = table_header(&lines[start..end], &fenced[start..end]) else {
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

/// The first table's header cells in a section body, fenced lines skipped.
fn table_header(body: &[&str], fenced: &[bool]) -> Option<Vec<String>> {
    for (line, &in_fence) in body.iter().zip(fenced) {
        if in_fence {
            continue;
        }
        let trimmed = line.trim_start();
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
fn headings(lines: &[&str], fenced: &[bool]) -> Vec<(usize, String)> {
    heading_positions(lines, fenced)
        .into_iter()
        .map(|i| parse_heading(lines[i]).expect("a heading line parses"))
        .collect()
}

/// The line index of every heading outside a fenced block.
fn heading_positions(lines: &[&str], fenced: &[bool]) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|&(i, line)| !fenced[i] && parse_heading(line).is_some())
        .map(|(i, _)| i)
        .collect()
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
                example: None,
            }],
        };
        // Nothing from node_modules is a candidate, so nothing is governed.
        let findings = check_documents(&[], &set);
        assert!(findings.is_empty());
    }

    fn schema_of(yaml: &str) -> DocSchema {
        let text = format!("---\ntype: Schema\n---\n\n````yaml\n{yaml}\n````\n");
        DocSchema::parse("s", &text)
            .expect("the contract deserializes")
            .expect("the contract is there")
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

    /// A fence inside a fence stays fenced: the nested marker the schemas'
    /// own worked examples use does not flip the reading, so fence-interior
    /// lines are never content, and a `~~~` fence satisfies `code`.
    #[test]
    fn a_nested_fence_stays_fenced() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n    content: bullet-list\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n````yaml\n```\n- inside a fence\n```\n````\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.contains("no bullet"), "{findings:#?}");

        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Run\n    level: 2\n    content: code\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Run\n\n~~~sh\nls\n~~~\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// A table in a subsection satisfies its section's columns rule: the
    /// table is sought in the same body range the content check reads.
    #[test]
    fn a_table_in_a_subsection_satisfies_a_columns_rule() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Data\n    level: 2\n\
             \x20   content: table\n    columns: [A, B]\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Data\n\nA lead-in line.\n\n### Sub\n\n| A | B |\n|---|---|\n| 1 | 2 |\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Link definitions, HTML comments, deeper headings and dividers are not
    /// paragraph lines, so a prose section of nothing else is a finding.
    #[test]
    fn link_definitions_and_comments_are_not_prose() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Notes\n    level: 2\n    content: prose\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Notes\n\n---\n\n<!-- sokf:links -->\n[sokf:x]: /knowledge/x.md\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("no paragraph line"),
            "{findings:#?}"
        );
    }

    /// Covers I018 criterion 5: a kind outside the five is reported on the
    /// schema file — by `check_declarations`; `validate` says it through the
    /// grammar's schema check instead — and the unreadable rule binds
    /// nothing.
    #[test]
    fn a_kind_outside_the_five_is_reported_on_the_schema_file() {
        let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    sections:\n  - heading: Body\n    level: 2\n    content: essay\n````\n";
        let schemas = [("s.md".to_string(), text.to_string())];
        let (set, findings) = SchemaSet::load(&schemas);
        assert!(findings.is_empty(), "{findings:#?}");
        let findings = check_declarations(&schemas);
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

    /// A YAML comment is not part of the value: a trailing `# …` is stripped
    /// before the comparison, and a comment line under a key is a comment,
    /// not a block.
    #[test]
    fn a_yaml_comment_is_not_part_of_the_value() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\n  status:\n    enum: [draft, stable]\n",
        );
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T # accepted 2026\nstatus: draft # note\n# a full-line comment\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // A key carrying only a comment is absent.
        let schema = schema_of("frontmatter:\n  type:\n    const: T\n  id:\n    required: true\n");
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\nid: # to be assigned\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("`id` is absent"),
            "{findings:#?}"
        );
    }

    /// A CRLF document reads as its LF twin: the frontmatter parses and the
    /// checks bind.
    #[test]
    fn a_crlf_document_reads_as_its_lf_twin() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\n  id:\n    required: true\n\
             \x20   pattern: '^t-\\d{3}$'\n",
        );
        let doc = Document {
            path: "a.md",
            text: "---\r\ntype: T\r\nid: t-001\r\n---\r\n\r\n# A\r\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// `lifecycle` with an enum belongs to the filing check; without one,
    /// its constraints bind here like any other key's (one fault, said once).
    #[test]
    fn lifecycle_without_an_enum_binds_like_any_other_key() {
        let with_enum = schema_of(
            "frontmatter:\n  type:\n    const: T\n  lifecycle:\n    required: true\n\
             \x20   enum: [open, done]\n",
        );
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &with_enum, &mut findings);
        assert!(
            findings.is_empty(),
            "the filing check reports it: {findings:#?}"
        );

        let without_enum =
            schema_of("frontmatter:\n  type:\n    const: T\n  lifecycle:\n    required: true\n");
        let mut findings = Vec::new();
        check_one(&doc, &without_enum, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("`lifecycle` is absent"),
            "{findings:#?}"
        );
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
    /// on the schema file — by `check_declarations` — and the unreadable
    /// rule binds nothing.
    #[test]
    fn an_uncompilable_pattern_is_reported_on_the_schema_file() {
        let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    \x20 id:\n    pattern: '(unclosed'\n````\n";
        let schemas = [("s.md".to_string(), text.to_string())];
        let (set, findings) = SchemaSet::load(&schemas);
        assert!(findings.is_empty(), "{findings:#?}");
        let findings = check_declarations(&schemas);
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

    /// A contract that does not deserialize is a finding, never a schema
    /// that silently governs nothing.
    #[test]
    fn a_contract_that_does_not_deserialize_is_a_finding() {
        let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    \x20 status:\n    enum:\n      - draft: what it means\n````\n";
        let (_, findings) = SchemaSet::load(&[("s.md".to_string(), text.to_string())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0].message.contains("does not deserialize"),
            "{findings:#?}"
        );
    }

    /// One schema file carrying the given yaml contract and an `example:`
    /// block holding `example`, as `check_examples` takes them.
    fn with_example(contract: &str, example: &str) -> Vec<(String, String)> {
        let indented: String = example
            .split('\n')
            .map(|l| {
                if l.is_empty() {
                    String::new()
                } else {
                    format!("  {l}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let text =
            format!("---\ntype: Schema\n---\n\n````yaml\n{contract}example: |\n{indented}\n````\n");
        vec![("s.md".to_string(), text)]
    }

    /// Covers I022 criterion 1: an example whose `id` breaks the declaring
    /// schema's own pattern is an error naming the schema file, prefixed so a
    /// reader sees the example broke.
    #[test]
    fn an_example_id_breaking_its_own_pattern_is_a_finding_on_the_schema_file() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\n  id:\n    pattern: '^t-\\d{3}$'\n",
            "---\ntype: T\nid: t-1\n---\n\n# A\n",
        );
        let findings = check_examples(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0].message.starts_with("example: "),
            "{findings:#?}"
        );
        assert!(
            findings[0].message.contains("`id` is `t-1`")
                && findings[0].message.contains("pattern"),
            "{findings:#?}"
        );
    }

    /// Covers I022 criterion 1: an example lacking a key the declaring schema
    /// marks required is an error naming the schema file.
    #[test]
    fn an_example_lacking_a_required_key_is_a_finding_on_the_schema_file() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    required: true\n    const: T\n  id:\n    required: true\n",
            "---\ntype: T\n---\n\n# A\n",
        );
        let findings = check_examples(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0]
                .message
                .contains("example: frontmatter `id` is absent"),
            "{findings:#?}"
        );
    }

    /// Covers I022 criterion 2: an example missing a required section, and
    /// one whose section body lacks its declared content kind, are each
    /// errors naming the schema file.
    #[test]
    fn an_example_breaking_the_section_rules_is_a_finding_on_the_schema_file() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Context\n    level: 2\n\
             \x20   required: true\n  - heading: Items\n    level: 2\n    content: bullet-list\n",
            "---\ntype: T\n---\n\n# A\n\n## Items\n\nprose only\n",
        );
        let findings = check_examples(&schemas);
        let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
        assert_eq!(findings.len(), 2, "{findings:#?}");
        assert!(findings.iter().all(|f| f.file == "s.md"), "{findings:#?}");
        assert!(
            messages
                .iter()
                .any(|m| m.contains("example: missing required section \"Context\"")),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("example: section \"Items\" carries no bullet")),
            "{messages:?}"
        );
    }

    /// Covers I022 criterion 6: an example satisfying its schema yields no
    /// finding — one dispatched by type with its frontmatter, and one
    /// dispatched by glob, whose documents carry no frontmatter at all.
    #[test]
    fn a_conforming_example_yields_no_finding() {
        let by_type = with_example(
            "frontmatter:\n  type:\n    required: true\n    const: T\n  id:\n    required: true\n\
             \x20   pattern: '^t-\\d{3}$'\nsections:\n  - heading: Context\n    level: 2\n\
             \x20   required: true\n    content: prose\n",
            "---\ntype: T\nid: t-001\n---\n\n# A\n\n## Context\n\nWhy it exists.\n",
        );
        let findings = check_examples(&by_type);
        assert!(findings.is_empty(), "{findings:#?}");

        let by_glob = with_example(
            "target-files: 'README.md'\nsections:\n  - heading: Install\n    level: 2\n\
             \x20   required: true\n    content: code\n",
            "# X\n\n## Install\n\n```sh\nls\n```\n",
        );
        let findings = check_examples(&by_glob);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I022 criterion 5: an example with no frontmatter block, and one
    /// whose frontmatter is not YAML, are each errors naming the schema file.
    /// The block is owed only where the schema dispatches by type — a
    /// glob-dispatched schema's documents carry no frontmatter to show.
    #[test]
    fn an_example_that_does_not_parse_as_a_document_is_a_finding() {
        let no_block = with_example("frontmatter:\n  type:\n    const: T\n", "# A\n\nprose\n");
        let findings = check_examples(&no_block);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0]
                .message
                .contains("example: does not parse as a document")
                && findings[0].message.contains("no frontmatter block"),
            "{findings:#?}"
        );

        let not_yaml = with_example(
            "frontmatter:\n  type:\n    const: T\n",
            "---\ntype: [unclosed\n---\n\n# A\n",
        );
        let findings = check_examples(&not_yaml);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0]
                .message
                .contains("example: does not parse as a document")
                && findings[0].message.contains("not YAML"),
            "{findings:#?}"
        );
    }

    /// Covers I022 criterion 3: an example body link whose target is a path
    /// into the knowledge is an error naming the schema file — the
    /// `[text][sokf:<id>]` form is the accepted form for a concept link
    /// (ADR-025). Inline and reference forms are both refused.
    #[test]
    fn an_example_link_into_the_knowledge_is_a_finding_on_the_schema_file() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\n",
            "---\ntype: T\n---\n\n# A\n\nSee [the plan](/knowledge/plans/open/plan-001-x.md)\n\
             and [the config][cfg].\n\n[cfg]: knowledge/config.md\n",
        );
        let findings = check_examples(&schemas);
        assert_eq!(findings.len(), 2, "{findings:#?}");
        assert!(findings.iter().all(|f| f.file == "s.md"), "{findings:#?}");
        let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
        assert!(
            messages.iter().any(|m| m.contains(
                "example: body link names a path into the knowledge: \
                 /knowledge/plans/open/plan-001-x.md"
            ) && m.contains("[text][sokf:<id>]")),
            "{messages:?}"
        );
        assert!(
            messages.iter().any(|m| m.contains(
                "example: body link names a path into the knowledge: knowledge/config.md"
            )),
            "{messages:?}"
        );
    }

    /// Covers I022 criterion 4: a `[text][sokf:<id>]` link naming no real
    /// concept passes — no id is resolved, with or without a definition
    /// block, and the definition's knowledge path is never read as a target.
    #[test]
    fn a_fictional_sokf_label_in_an_example_passes() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\n",
            "---\ntype: T\n---\n\n# A\n\nSee [config][sokf:no-such-concept] and \
             [ghost][sokf:undefined-label].\n\n<!-- sokf:links -->\n\
             [sokf:no-such-concept]: /knowledge/config.md\n",
        );
        let findings = check_examples(&schemas);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I022 criterion 4: a URL link and a repository-path link outside
    /// the knowledge each pass in their ordinary markdown form.
    #[test]
    fn a_url_and_a_repository_path_outside_the_knowledge_pass() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\n",
            "---\ntype: T\n---\n\n# A\n\nSee [docs](https://docs.rs/clap), \
             [the source](/crates/lib/superdev-core/src/lib.rs) and [the readme](README.md).\n",
        );
        let findings = check_examples(&schemas);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// A schema with no example is the grammar check's finding, not this
    /// one's — and a contract that does not deserialize is `SchemaSet::load`'s
    /// — so neither is said twice here.
    #[test]
    fn a_missing_example_and_a_broken_contract_are_not_reported_here() {
        let text =
            "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n````\n";
        let findings = check_examples(&[("s.md".to_string(), text.to_string())]);
        assert!(findings.is_empty(), "{findings:#?}");

        let broken = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                      \x20 status:\n    enum:\n      - draft: what it means\n````\n";
        let findings = check_examples(&[("s.md".to_string(), broken.to_string())]);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn headings_inside_a_fence_are_not_headings() {
        let lines: Vec<&str> = "# Real\n\n```\n# Fenced\n```\n\n## Also real\n"
            .split('\n')
            .collect();
        let fenced = super::super::read::fence_map(&lines);
        let found: Vec<String> = headings(&lines, &fenced)
            .into_iter()
            .map(|(_, h)| h)
            .collect();
        assert_eq!(found, ["Real", "Also real"]);
    }
}
