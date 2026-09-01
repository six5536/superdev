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

use std::collections::{BTreeMap, BTreeSet};

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
    /// The pattern every top-level item of the section's list must match.
    #[serde(default, rename = "item-pattern")]
    pub item_pattern: Option<String>,
    /// The pattern the section's whole body must match.
    #[serde(default, rename = "content-pattern")]
    pub content_pattern: Option<String>,
}

/// The content kinds an `item-pattern` may sit beside: the ones whose bodies
/// have items to bind (ADR-030).
const LIST_KINDS: [&str; 2] = ["bullet-list", "numbered-list"];

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
            let patterns = [
                ("item-pattern", rule.item_pattern.as_deref()),
                ("content-pattern", rule.content_pattern.as_deref()),
            ];
            for (name, pattern) in patterns {
                if let Some(pattern) = pattern
                    && re::compile(pattern).is_none()
                {
                    findings.push(Finding {
                        file: file.clone(),
                        message: format!(
                            "schema: section {} declares {name} `{pattern}` — it does not \
                             compile, and binds nothing",
                            rule.label()
                        ),
                        fatal: true,
                    });
                }
            }
            // An item-pattern needs items to bind: without a list kind the
            // section's body has none, so the rule would pass in silence.
            if rule.item_pattern.is_some()
                && !rule
                    .content
                    .as_deref()
                    .is_some_and(|k| LIST_KINDS.contains(&k))
            {
                findings.push(Finding {
                    file: file.clone(),
                    message: format!(
                        "schema: section {} declares an item-pattern, and its content is not {}",
                        rule.label(),
                        LIST_KINDS.join(" or ")
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
        check_one(&doc, &schema, true, &mut broke);
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
/// into the knowledge is an error, and so is a `sokf:` destination — the
/// concept-link form misspelled. No id or target is resolved: a fictional
/// `sokf:` label passes, a URL or a repository path outside the knowledge
/// keeps its ordinary markdown form, and an image names a picture, never a
/// concept, so nothing is asked of one.
///
/// A reference link's destination comes from its definition line, so
/// definitions are read directly — which also catches one nothing
/// references, for which the markdown parser emits no event. The
/// `sokf:`-labelled definitions of the generated block stay exempt: their
/// knowledge paths are the accepted form's own plumbing.
fn check_link_form(file: &str, body: &[&str], findings: &mut Vec<Finding>) {
    let mut push = |message: String| {
        findings.push(Finding {
            file: file.to_string(),
            message,
            fatal: true,
        });
    };
    let scan = sokf::scan_body(&body.join("\n"));
    let image_dests: BTreeSet<&str> = scan
        .links
        .iter()
        .filter(|l| l.image)
        .map(|l| l.dest.as_str())
        .collect();
    for link in &scan.links {
        if link.id.is_some() || link.image {
            continue;
        }
        if link.dest.starts_with(sokf::ID_LABEL) {
            push(format!(
                "body link writes `{}` as a destination — a concept link takes the \
                 [text][sokf:<id>] reference form",
                link.dest
            ));
            continue;
        }
        if !link.inline {
            continue; // The destination is a definition's; reported below.
        }
        if let Some(path) = sokf::link_path(&link.dest)
            && into_knowledge(path)
        {
            push(format!(
                "body link names a path into the knowledge: {} — a concept link takes \
                 the [text][sokf:<id>] form",
                link.dest
            ));
        }
    }
    let fenced = super::read::fence_map(body);
    for (line, _) in body.iter().zip(&fenced).filter(|&(_, f)| !f) {
        let Some((label, target)) = link_definition(line) else {
            continue;
        };
        if label.starts_with(sokf::ID_LABEL) || image_dests.contains(target) {
            continue;
        }
        if let Some(path) = sokf::link_path(target)
            && into_knowledge(path)
        {
            push(format!(
                "body link names a path into the knowledge: {target} — a concept link takes \
                 the [text][sokf:<id>] form"
            ));
        }
    }
}

/// One `[label]: target` link definition line, footnotes excluded.
fn link_definition(line: &str) -> Option<(&str, &str)> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (label, rest) = rest.split_once("]:")?;
    if label.starts_with('^') {
        return None; // A footnote definition carries text, not a target.
    }
    Some((label, rest.split_whitespace().next().unwrap_or("")))
}

/// Whether a link target reads as a path into the knowledge — the
/// `knowledge/` directory, spelled from the repository root, bare, or behind
/// leading `./` and `../` segments. Form only, never resolved.
fn into_knowledge(path: &str) -> bool {
    let mut path = path;
    loop {
        let trimmed = path.trim_start_matches('/');
        path = match trimmed
            .strip_prefix("./")
            .or_else(|| trimmed.strip_prefix("../"))
        {
            Some(rest) => rest,
            None => break trimmed.starts_with("knowledge/"),
        };
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
        check_one(doc, schema, false, &mut findings);
    }
    findings
}

/// One document against one schema. `example` marks a schema's example,
/// which the filing check never reads, so `lifecycle` binds here rather
/// than being deferred to it.
fn check_one(doc: &Document<'_>, schema: &DocSchema, example: bool, findings: &mut Vec<Finding>) {
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
    // read as `read::fence_map` reads them everywhere else. The frontmatter
    // block is masked with the fences: a YAML comment opens with `#`, which
    // is a comment and not a heading, so nothing in the block may satisfy a
    // required section or trigger a prohibited one.
    let mut fenced = super::read::fence_map(&lines);
    if let Some(split) = super::read::split_frontmatter(&lines) {
        for masked in fenced.iter_mut().take(split.body_start) {
            *masked = true;
        }
    }
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
        let (start, end) = body_range(h, &headings, &positions, lines.len());
        let mut kind_failed = false;
        if let Some(kind) = rule.content.as_deref()
            && CONTENT_KINDS.contains(&kind)
            // A declared table's absence is check_columns' finding already.
            && (kind != "table" || rule.columns.is_empty())
            && !body_has(kind, &lines[start..end], &fenced[start..end])
        {
            push(format!(
                "section {} carries no {}, and {} declares {kind} content",
                rule.label(),
                form_of(kind),
                schema.name
            ));
            kind_failed = true;
        }
        // A section that failed its content kind has no body worth matching:
        // the pattern would report the same fault a second time.
        if !kind_failed {
            check_body_patterns(
                rule,
                &headings[h].1,
                &lines[start..end],
                &fenced[start..end],
                schema,
                &mut push,
            );
        }
    }

    check_columns(schema, &headings, &positions, &lines, &fenced, &mut push);
    check_frontmatter(&lines, schema, example, &mut push);
}

/// One section's body against the patterns its rule declares (ADR-030):
/// `content-pattern` over the body, `item-pattern` over each top-level item
/// of the list the section's kind names. Both are matched found-anywhere, so
/// a rule binds the ends by writing them. `heading` is the occurrence's own
/// text, so a finding on a repeatable rule names the section that failed
/// rather than the pattern that matched it.
fn check_body_patterns(
    rule: &SectionRule,
    heading: &str,
    body: &[&str],
    fenced: &[bool],
    schema: &DocSchema,
    push: &mut impl FnMut(String),
) {
    if let Some(pattern) = rule.content_pattern.as_deref() {
        // Fenced lines are not content, here as everywhere else: a keyword
        // inside a worked example is an example of one (contract-010).
        let text: Vec<&str> = body
            .iter()
            .zip(fenced)
            .filter(|&(_, &in_fence)| !in_fence)
            .map(|(line, _)| *line)
            .collect();
        if re::compile(pattern).is_some_and(|re| !re.is_match(&text.join("\n"))) {
            push(format!(
                "section \"{heading}\" does not match, and {} declares content-pattern \
                 `{pattern}`",
                schema.name
            ));
        }
    }
    let Some(pattern) = rule.item_pattern.as_deref() else {
        return;
    };
    let Some(kind) = rule.content.as_deref().filter(|k| LIST_KINDS.contains(k)) else {
        return; // `check_declarations` reports the mis-declaration.
    };
    let Some(re) = re::compile(pattern) else {
        return; // An unreadable pattern binds nothing.
    };
    for item in items_in(body, fenced, kind) {
        if !re.is_match(&item.text) {
            push(format!(
                "section \"{heading}\" item `{}` does not match, and {} declares item-pattern \
                 `{pattern}`",
                item.first, schema.name
            ));
        }
    }
}

/// One top-level item of a section's list, as `item-pattern` reads it.
struct Item {
    /// The item's first line, verbatim, for the finding to name it by.
    first: String,
    /// The item's own lines, marker stripped and continuations joined.
    text: String,
}

/// Every top-level item of `kind` in a section body (ADR-030).
///
/// The list's top level is the shallowest marker in the body, because a list
/// indented one or two spaces is still a top-level list and binding only the
/// unindented ones would let an author escape the rule by indenting it. An
/// item takes every following line indented past that, blank lines included,
/// so a bullet carrying several paragraphs is read whole; it takes an
/// unindented line too while its paragraph is still open, which is markdown's
/// lazy continuation. A nested item belongs to itself: its own lines are
/// dropped, and the parent resumes at the first line no deeper than the
/// nested marker. Fenced lines are skipped, as they are for every content
/// check.
fn items_in(body: &[&str], fenced: &[bool], kind: &str) -> Vec<Item> {
    let unfenced = || {
        body.iter()
            .zip(fenced)
            .filter(|&(_, &in_fence)| !in_fence)
            .map(|(line, _)| *line)
    };
    let marks = |line: &str| {
        let trimmed = line.trim_start();
        !is_thematic_break(trimmed) && (is_bullet(trimmed) || is_numbered(trimmed))
    };
    let Some(top) = unfenced().filter(|line| marks(line)).map(indent_of).min() else {
        return Vec::new();
    };

    let mut items: Vec<Item> = Vec::new();
    // The indentation of the nested marker whose lines are being dropped, and
    // whether the open item's paragraph is still running (a blank line ends
    // it, and only an indented line may reopen one).
    let mut nested: Option<usize> = None;
    let mut flowing = false;
    let mut open = false;
    for line in unfenced() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            flowing = false;
            continue;
        }
        let indent = indent_of(line);
        if let Some(at) = nested {
            // The nested item keeps its own lines; anything no deeper than
            // its marker returns to the item above it.
            if indent > at {
                continue;
            }
            nested = None;
        }
        if marks(line) {
            if indent > top {
                nested = Some(indent);
            } else if is_kind(trimmed, kind) {
                items.push(Item {
                    first: line.trim_end().to_string(),
                    text: strip_marker(trimmed).to_string(),
                });
                open = true;
                flowing = true;
            } else {
                open = false; // A sibling of the other marker kind.
                flowing = false;
            }
            continue;
        }
        // An indented line continues the open item; an unindented one does so
        // only while the paragraph is still running.
        if open && (indent > top || flowing) {
            if let Some(item) = items.last_mut() {
                item.text.push(' ');
                item.text.push_str(trimmed.trim_end());
            }
            flowing = true;
        } else {
            open = false;
            flowing = false;
        }
    }
    items
}

/// A line's indentation in columns, a tab counting as four.
fn indent_of(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

/// Does this marker open an item of the declared list kind?
fn is_kind(trimmed: &str, kind: &str) -> bool {
    if kind == "numbered-list" {
        is_numbered(trimmed)
    } else {
        is_bullet(trimmed)
    }
}

/// `***`, `- - -`, `___` — a thematic break, which opens no item however much
/// its first two characters look like a bullet.
fn is_thematic_break(trimmed: &str) -> bool {
    let mut chars = trimmed.chars().filter(|c| !c.is_whitespace()).peekable();
    let Some(&first) = chars.peek() else {
        return false;
    };
    if !matches!(first, '-' | '*' | '_') {
        return false;
    }
    let count = chars.clone().count();
    count >= 3 && chars.all(|c| c == first)
}

/// An item's text without its `- `, `* `, `+ ` or `1. ` marker — the marker
/// itself and the space after it, never a character of the text.
fn strip_marker(trimmed: &str) -> &str {
    let digits = trimmed
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let after = if digits > 0 {
        // A numbered marker: the digits, then the `.` or `)` that closes it.
        trimmed[digits..]
            .strip_prefix(['.', ')'])
            .unwrap_or(trimmed)
    } else {
        trimmed.strip_prefix(['-', '*', '+']).unwrap_or(trimmed)
    };
    after.trim_start()
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
/// would say one fault twice. That deferral covers real, filed documents
/// only: inside an `example`, which the filing check never reads, the key
/// binds here or nowhere. Without an enum nothing else reads the key,
/// and its constraints bind here like any other's.
fn check_frontmatter(
    lines: &[&str],
    schema: &DocSchema,
    example: bool,
    push: &mut impl FnMut(String),
) {
    let fm = super::read::split_frontmatter(lines).map_or_else(Vec::new, |s| s.fm);
    let entries = super::read::parse_frontmatter(&fm);
    for (key, constraint) in schema.frontmatter.iter() {
        if !example && key == "lifecycle" && schema.lifecycle_enum().is_some() {
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&at, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        let over = Document {
            path: "a.md",
            text: "one\ntwo\nthree\nfour\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&over, &schema, false, &mut findings);
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
            false,
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
            false,
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
            check_one(&ok, &schema, false, &mut findings);
            assert!(findings.is_empty(), "{kind}: {findings:#?}");

            let bad = Document {
                path: "a.md",
                text: &format!("# T\n\n## Body\n\n{fail}"),
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&bad, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // A bullet after the section's end does not count.
        let after = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\nprose\n\n## Next\n\n- one\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&after, &schema, false, &mut findings);
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
        check_one(&fenced_only, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");

        let bullet_after_fence = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n```\n# not a heading\n```\n\n- a bullet\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&bullet_after_fence, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// A schema declaring an item-pattern on a list section, for the
    /// body-pattern tests (ADR-030).
    fn items_schema(pattern: &str) -> DocSchema {
        schema_of(&format!(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n\
             \x20   content: bullet-list\n    item-pattern: '{pattern}'\n"
        ))
    }

    /// Covers I034 criterion 1: the finding names the document, the section
    /// and the item that failed.
    #[test]
    fn an_item_failing_its_pattern_is_a_finding_naming_the_item() {
        let schema = items_schema("^\\[x\\] ");
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- [x] tagged\n- untagged\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "a.md");
        assert!(findings[0].message.contains("\"Items\""), "{findings:#?}");
        assert!(findings[0].message.contains("- untagged"), "{findings:#?}");
        assert!(
            findings[0].message.contains("item-pattern"),
            "{findings:#?}"
        );
    }

    /// Covers I034 criterion 1: an item's continuation lines join before the
    /// pattern reads it, and a nested item belongs to itself — so a parent
    /// whose child carries the required text still fails.
    #[test]
    fn an_items_text_joins_its_continuations_and_excludes_its_children() {
        let schema = items_schema("covers \\d");
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a long item that wraps\n  onto a second line, covers 1.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a parent item\n  - a child, covers 1.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("- a parent item"),
            "{findings:#?}"
        );
    }

    /// Covers I034 criterion 1: an item's later paragraphs are its own — a
    /// blank line separates them, and only an unindented line ends the item.
    /// The prose a list runs into belongs to no item.
    #[test]
    fn an_item_carries_every_paragraph_indented_under_it() {
        let schema = items_schema("covers \\d");
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- an item opening a paragraph\n\n  and closing on a \
                   second, covers 1.\n\nProse after the list, covers 9.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // The trailing prose is nobody's item: an item that never matches
        // stays the only finding, and the prose adds none of its own.
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- an item\n\n  and its second paragraph.\n\nProse after \
                   the list, covers 9.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].message.contains("- an item"), "{findings:#?}");
    }

    /// Covers I034 criterion 2: the finding names the document and the
    /// section, and the body is read whole — a match anywhere in it passes.
    #[test]
    fn a_body_failing_its_content_pattern_is_a_finding() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Promise\n    level: 2\n\
             \x20   content: prose\n    content-pattern: '\\bMUST\\b'\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Promise\n\nThe server starts on demand.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "a.md");
        assert!(findings[0].message.contains("\"Promise\""), "{findings:#?}");
        assert!(
            findings[0].message.contains("content-pattern"),
            "{findings:#?}"
        );

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Promise\n\nA lead-in.\n\nThe server MUST start on demand.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I034 criteria 1 and 2: both patterns are matched
    /// found-anywhere, so a rule binds the ends only by writing them
    /// (ADR-030).
    #[test]
    fn a_pattern_matches_anywhere_until_it_is_anchored() {
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a tail marker here\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("marker"), false, &mut findings);
        assert!(findings.is_empty(), "found anywhere: {findings:#?}");

        let mut findings = Vec::new();
        check_one(&doc, &items_schema("^marker"), false, &mut findings);
        assert_eq!(findings.len(), 1, "anchored: {findings:#?}");
    }

    /// Covers I034 criterion 3: a pattern that does not compile, and an
    /// item-pattern beside a kind with no items, are findings on the schema
    /// file — and each binds nothing.
    #[test]
    fn a_mis_declared_body_pattern_is_a_finding_on_the_schema() {
        let broken = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                      sections:\n  - heading: Items\n    level: 2\n    content: bullet-list\n\
                      \x20   item-pattern: '['\n````\n";
        let findings = check_declarations(&[("s.md".into(), broken.into())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0].message.contains("does not compile"),
            "{findings:#?}"
        );

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- anything at all\n",
            doc_type: Some("T"),
        };
        let mut found = Vec::new();
        check_one(&doc, &items_schema("["), false, &mut found);
        assert!(
            found.is_empty(),
            "an unreadable rule binds nothing: {found:#?}"
        );

        let misplaced = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                         sections:\n  - heading: Body\n    level: 2\n    content: prose\n\
                         \x20   item-pattern: 'x'\n````\n";
        let findings = check_declarations(&[("s.md".into(), misplaced.into())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("content is not"),
            "{findings:#?}"
        );
    }

    /// Covers I034 criterion 1: a list indented one, two or three spaces, or
    /// a tab, is still the section's top-level list. Binding only the
    /// unindented markers would let an author escape every item-pattern by
    /// indenting the list.
    #[test]
    fn an_indented_list_is_still_the_sections_top_level() {
        for lead in ["  ", " ", "   ", "\t"] {
            let doc_text =
                format!("# T\n\n## Items\n\n{lead}- no keyword\n{lead}- none here either\n");
            let doc = Document {
                path: "a.md",
                text: &doc_text,
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&doc, &items_schema("MUST"), false, &mut findings);
            assert_eq!(findings.len(), 2, "lead {lead:?}: {findings:#?}");
        }
    }

    /// Covers I034 criterion 1: a nested item's own lines are dropped, and
    /// the item above it resumes at the first line no deeper than the nested
    /// marker — the parent's later paragraphs are still the parent's.
    #[test]
    fn a_parent_item_resumes_after_its_nested_list() {
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a parent item\n  - a child\n    with its own line\n\n  \
                   The parent MUST hold here.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // The child's text is the child's: a keyword only it carries does not
        // satisfy the parent.
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a parent item\n  - a child that MUST not count\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
    }

    /// Covers I034 criterion 1: an unindented line right under an item
    /// continues it — markdown's lazy continuation — while a blank line first
    /// ends the list, so the prose after it belongs to no item.
    #[test]
    fn a_lazy_continuation_belongs_to_the_item_and_later_prose_does_not() {
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a wrapped item\ncontinuing with a MUST unindented\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), false, &mut findings);
        assert!(findings.is_empty(), "lazy continuation: {findings:#?}");

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- an item with no keyword\n\nProse after the list MUST \
                   not rescue it.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
    }

    /// Covers I034 criterion 1: an item-pattern binds the items of the kind
    /// its rule declares, and a marker of the other kind is not one.
    #[test]
    fn an_item_pattern_binds_only_its_declared_marker_kind() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n\
             \x20   content: numbered-list\n    item-pattern: 'MUST'\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n1. a numbered item that MUST hold\n- a bullet beside it\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I034 criteria 1 and 2: a fenced block is not content, so a
    /// keyword inside a worked example neither satisfies a pattern nor opens
    /// an item, and a thematic break is not a bullet.
    #[test]
    fn a_fence_and_a_thematic_break_are_not_content() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Promise\n    level: 2\n\
             \x20   content: prose\n    content-pattern: 'MUST'\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Promise\n\nA plain line.\n\n```text\nThe example MUST not count.\n\
                   ```\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "a fenced keyword: {findings:#?}");

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- an item that MUST hold\n\n* * *\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), false, &mut findings);
        assert!(findings.is_empty(), "a thematic break: {findings:#?}");

        // A bullet inside a fenced example is an example of one, and opens no
        // item of the section's own list.
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- an item that MUST hold\n\n```text\n- a fenced bullet \
                   with no keyword\n```\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), false, &mut findings);
        assert!(findings.is_empty(), "a fenced bullet: {findings:#?}");
    }

    /// Covers I034 criteria 1 and 2: a repeatable rule's finding names the
    /// occurrence that failed, not the pattern that matched it.
    #[test]
    fn a_finding_on_a_repeatable_rule_names_the_failing_section() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading-pattern: '^Item: '\n\
             \x20   level: 2\n    repeatable: true\n    content: prose\n\
             \x20   content-pattern: 'MUST'\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Item: one\n\nIt MUST hold.\n\n## Item: two\n\nIt does not say.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("\"Item: two\""),
            "{findings:#?}"
        );
    }

    /// Covers I034 criterion 2: a section that already failed its content
    /// kind does not fail its pattern too — one fault is said once.
    #[test]
    fn an_empty_section_reports_its_kind_and_not_its_pattern() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Promise\n    level: 2\n\
             \x20   content: prose\n    content-pattern: 'MUST'\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Promise\n\n- only a bullet\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("prose content"),
            "{findings:#?}"
        );
    }

    /// Covers I034 criterion 5: a section rule declaring neither pattern is
    /// checked exactly as it was before ADR-030.
    #[test]
    fn a_section_declaring_no_pattern_gains_no_finding() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n\
             \x20   content: bullet-list\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- anything at all\n- in any shape\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I034 criterion 1: a schema's own example is checked against the
    /// patterns that schema declares, on the ADR-024 path.
    #[test]
    fn an_example_is_checked_against_its_schemas_item_pattern() {
        let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    sections:\n  - heading: Items\n    level: 2\n    content: bullet-list\n\
                    \x20   item-pattern: '^\\[x\\] '\nexample: |\n  ---\n  type: T\n  ---\n\n\
                    \x20 # T\n\n  ## Items\n\n  - untagged\n````\n";
        let findings = check_examples(&[("s.md".into(), text.into())]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(findings[0].message.starts_with("example:"), "{findings:#?}");
        assert!(
            findings[0].message.contains("item-pattern"),
            "{findings:#?}"
        );
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&ok, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&broken, &schema, false, &mut findings);
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
        check_one(&ok, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // A key carrying only a comment is absent.
        let schema = schema_of("frontmatter:\n  type:\n    const: T\n  id:\n    required: true\n");
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\nid: # to be assigned\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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
        check_one(&doc, &with_enum, false, &mut findings);
        assert!(
            findings.is_empty(),
            "the filing check reports it: {findings:#?}"
        );

        let without_enum =
            schema_of("frontmatter:\n  type:\n    const: T\n  lifecycle:\n    required: true\n");
        let mut findings = Vec::new();
        check_one(&doc, &without_enum, false, &mut findings);
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
        check_one(&doc, &schema, false, &mut findings);
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

    /// Review finding 1 (code-review-002): an example's `lifecycle` binds
    /// here. The filing check owns the key only for real, filed documents,
    /// which an example is not — deferring it inside the example check made
    /// the constraint bind nothing.
    #[test]
    fn an_example_lifecycle_breaking_its_enum_is_a_finding_on_the_schema_file() {
        let contract = "frontmatter:\n  type:\n    const: T\n  lifecycle:\n    required: true\n\
                        \x20   enum: [open, done]\n";
        let wrong = with_example(contract, "---\ntype: T\nlifecycle: bananas\n---\n\n# A\n");
        let findings = check_examples(&wrong);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("example: frontmatter `lifecycle` is `bananas`"),
            "{findings:#?}"
        );

        let absent = with_example(contract, "---\ntype: T\n---\n\n# A\n");
        let findings = check_examples(&absent);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("example: frontmatter `lifecycle` is absent"),
            "{findings:#?}"
        );
    }

    /// Review finding 2 (code-review-002): a knowledge path behind leading
    /// `./` or `../` segments is still a path into the knowledge.
    #[test]
    fn a_dot_segmented_knowledge_path_in_an_example_is_a_finding() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\n",
            "---\ntype: T\n---\n\n# A\n\nSee [a](./knowledge/a.md) and [b](../knowledge/b.md).\n",
        );
        let findings = check_examples(&schemas);
        assert_eq!(findings.len(), 2, "{findings:#?}");
        assert!(
            findings
                .iter()
                .all(|f| f.message.contains("path into the knowledge")),
            "{findings:#?}"
        );
    }

    /// Review finding 3 (code-review-002): a YAML frontmatter comment is a
    /// comment, not a heading — it neither satisfies a required section nor
    /// triggers a prohibited one.
    #[test]
    fn a_frontmatter_comment_is_not_a_heading() {
        let text = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    sections-prohibited: [Banned]\nsections:\n  - heading: A\n    level: 1\n\
                    \x20   required: true\n````\n";
        let (set, findings) = SchemaSet::load(&[("s.md".to_string(), text.to_string())]);
        assert!(findings.is_empty(), "{findings:#?}");
        let found = check_documents(
            &[Document {
                path: "a.md",
                text: "---\ntype: T\n# A\n# Banned\n---\n\nprose\n",
                doc_type: Some("T"),
            }],
            &set,
        );
        assert_eq!(found.len(), 1, "{found:#?}");
        assert!(
            found[0].message.contains("missing required section \"A\""),
            "{found:#?}"
        );
    }

    /// Review finding 4 (code-review-002): an inline `(sokf:<id>)`
    /// destination is the concept-link form misspelled — the reference form
    /// `[text][sokf:<id>]` is the accepted one.
    #[test]
    fn an_inline_sokf_destination_in_an_example_is_a_finding() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\n",
            "---\ntype: T\n---\n\n# A\n\nSee [config](sokf:config).\n",
        );
        let findings = check_examples(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("sokf:config")
                && findings[0].message.contains("[text][sokf:<id>]"),
            "{findings:#?}"
        );
    }

    /// Review finding 5 (code-review-002): a link definition naming a
    /// knowledge path is caught even when nothing references it — while the
    /// `sokf:`-labelled definitions of the generated block stay exempt, being
    /// the accepted form's own plumbing.
    #[test]
    fn an_unreferenced_definition_naming_a_knowledge_path_is_a_finding() {
        let schemas = with_example(
            "frontmatter:\n  type:\n    const: T\n",
            "---\ntype: T\n---\n\n# A\n\nprose\n\n[stray]: /knowledge/x.md\n",
        );
        let findings = check_examples(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("path into the knowledge: /knowledge/x.md"),
            "{findings:#?}"
        );
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
