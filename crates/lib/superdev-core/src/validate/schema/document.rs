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
use std::ops::Range;

use serde::Deserialize;

use super::Finding;
use super::grammar::Ordered;
use super::re;
use crate::sokf::{IncludeBlock, IncludeTarget, include_blocks};
use crate::validate::sokf;

// The declaration vocabulary is the document schemas contract's Definition
// (contract-010): the `document-schemas` regions below are every key a
// schema's YAML block may carry, as the structs that read it declare them,
// and the entry point that checks a document against them.
// sokf:begin document-schemas
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
    /// The shape of what sits under the heading, one of `CONTENT_KINDS`.
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
    /// The pattern, with one capture group, every top-level item of the
    /// section's list must match; the capture is the item's key, unique
    /// across every item of the document under a rule declaring one
    /// (ADR-047).
    #[serde(default, rename = "item-key")]
    pub item_key: Option<String>,
    /// The pattern that may match only inside a top-level item: a match on
    /// a body line outside every item is an error naming the line
    /// (ADR-047).
    #[serde(default, rename = "item-only-pattern")]
    pub item_only_pattern: Option<String>,
    /// The pattern no top-level item may match: a match is an error naming
    /// the item and the matched text (ADR-047).
    #[serde(default, rename = "item-prohibited-pattern")]
    pub item_prohibited_pattern: Option<String>,
    /// Whether an item that does not match `item-key` is unkeyed and
    /// exempt from `item-pattern` and the nested rules, rather than a
    /// finding — a wontfix issue's either-form lists (ADR-051).
    #[serde(default, rename = "item-key-optional")]
    pub item_key_optional: bool,
    /// The rule for the items one level below this rule's items, itself
    /// nesting a rule for the level below (ADR-051).
    #[serde(default)]
    pub nested: Option<Box<NestedRule>>,
    /// The variant values this rule applies to; empty applies to every
    /// variant (ADR-045).
    #[serde(default)]
    pub variants: Vec<String>,
}

/// The rule for one level of nested items: a marker of the section's list
/// kind indented one level past the item above, whose lines the item
/// above drops (ADR-051). The declarations read as the section rule's
/// item declarations do, at this level.
#[derive(Debug, Clone, Deserialize)]
pub struct NestedRule {
    /// Whether every item of the level above must carry at least one item
    /// of this level; absence is an error naming the item above.
    #[serde(default)]
    pub required: bool,
    /// The pattern every item of this level must match.
    #[serde(default, rename = "item-pattern")]
    pub item_pattern: Option<String>,
    /// The pattern, with one capture group, every item of this level must
    /// match; the capture is the item's key, unique with every other key of
    /// the document at every level.
    #[serde(default, rename = "item-key")]
    pub item_key: Option<String>,
    /// The pattern no item of this level may match.
    #[serde(default, rename = "item-prohibited-pattern")]
    pub item_prohibited_pattern: Option<String>,
    /// The rule for the level below this one.
    #[serde(default)]
    pub nested: Option<Box<NestedRule>>,
}

/// One `sections-prohibited` entry: a bare heading, banned in every
/// variant, or a `{heading, variants}` mapping banning it in the variants
/// named (ADR-045).
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "ProhibitedEntry")]
struct Prohibited {
    /// The heading that must not appear.
    heading: String,
    /// The variant values the ban applies to; empty applies to every variant.
    variants: Vec<String>,
}

/// The two forms a prohibited entry is written in.
#[derive(Deserialize)]
#[serde(untagged)]
enum ProhibitedEntry {
    Heading(String),
    Tagged {
        heading: String,
        #[serde(default)]
        variants: Vec<String>,
    },
}

// sokf:end document-schemas

impl From<ProhibitedEntry> for Prohibited {
    fn from(entry: ProhibitedEntry) -> Self {
        match entry {
            ProhibitedEntry::Heading(heading) => Prohibited {
                heading,
                variants: Vec::new(),
            },
            ProhibitedEntry::Tagged { heading, variants } => Prohibited { heading, variants },
        }
    }
}

// sokf:begin document-schemas
/// A schema's `example`: one document, or — with `variant-key` set — one
/// per variant value, keyed by it (ADR-045). Checked in place against the
/// declaring schema (ADR-024).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Example {
    /// One conforming document, for a schema without variants.
    One(String),
    /// One document per variant value, keyed by it, every enum value present.
    Keyed(Ordered<String>),
}

// sokf:end document-schemas

/// The content kinds an `item-pattern` may sit beside: the ones whose bodies
/// have items to bind (ADR-030).
const LIST_KINDS: [&str; 2] = ["bullet-list", "numbered-list"];

// sokf:begin document-schemas
/// The content kinds a section rule may declare — the closed vocabulary of
/// contract-010, which the grammar's `content` enum repeats. A kind outside
/// this set is reported on the schema and binds nothing.
pub(crate) const CONTENT_KINDS: [&str; 6] = [
    "prose",
    "bullet-list",
    "numbered-list",
    "table",
    "code",
    "include",
];

// sokf:end document-schemas

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

    /// Whether this rule and `other` name one heading: the same literal, or
    /// the same pattern, at one level. A rule with no level matches any
    /// depth, so it names the heading at every level (ADR-049). A literal and
    /// a pattern it matches are two headings: the literal wins the heading it
    /// names and the pattern names the rest (`check_one`), which is how a
    /// schema declares its fixed headings beside a catch-all for the
    /// author's own.
    fn names_same_heading(&self, other: &SectionRule) -> bool {
        let same_level = match (self.level, other.level) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        };
        same_level
            && match (&self.heading, &other.heading) {
                (Some(a), Some(b)) => a == b,
                (None, None) => {
                    self.heading_pattern.is_some() && self.heading_pattern == other.heading_pattern
                }
                _ => false,
            }
    }

    /// The rule's `variants` tag, for a finding: `tagged [a, b]`, or
    /// `untagged`.
    fn tags(&self) -> String {
        if self.variants.is_empty() {
            "untagged".to_string()
        } else {
            format!("tagged [{}]", self.variants.join(", "))
        }
    }
}

/// Two section rules naming one heading that their `variants` sets do not
/// separate (ADR-049): both indices into `sections`, in declared order, and
/// the value the sets share — `None` when one rule is untagged, so it binds
/// every variant.
struct HeadingConflict {
    first: usize,
    second: usize,
    shared: Option<String>,
}

// sokf:begin document-schemas
/// One frontmatter key's constraints, as a schema's `frontmatter:` block
/// declares them. A key declared with only a `description` deserialises to
/// an empty constraint and binds nothing — guidance, per ADR-022. Fields
/// outside these five (`description`) are ignored here.
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
    /// The variant values this constraint applies to; empty applies to every
    /// variant (ADR-045).
    #[serde(default)]
    variants: Vec<String>,
}

/// A schema's contract, as far as document checking needs it.
#[derive(Debug, Clone, Deserialize)]
pub struct DocSchema {
    /// The schema file's own name, filled in after parsing.
    #[serde(skip)]
    pub name: String,
    /// The glob that names this schema's documents when they carry no
    /// frontmatter; a `type` const names them otherwise.
    #[serde(default, rename = "target-files")]
    target_files: Option<String>,
    /// The line count a document must not exceed.
    #[serde(default, rename = "line-limit")]
    line_limit: Option<usize>,
    /// The frontmatter key whose value selects a variant (ADR-045).
    #[serde(default, rename = "variant-key")]
    variant_key: Option<String>,
    /// Whether the sections' first appearances must follow declaration
    /// order.
    #[serde(default, rename = "sections-ordered")]
    sections_ordered: bool,
    /// The section rules, in declaration order.
    #[serde(default)]
    sections: Vec<SectionRule>,
    /// The headings a document must not carry.
    #[serde(default, rename = "sections-prohibited")]
    sections_prohibited: Vec<Prohibited>,
    /// Every key's constraint block, in declaration order. `Option` because
    /// a schema may write a key with nothing under it — an empty contract,
    /// binding nothing, rather than a schema that fails to parse.
    #[serde(default)]
    frontmatter: Ordered<Option<KeyConstraint>>,
    /// The worked example — one document satisfying this schema, or one per
    /// variant, checked in place by `check_examples` (ADR-024). Absence is
    /// the grammar's schema check's finding, not this module's.
    #[serde(default)]
    example: Option<Example>,
}

// sokf:end document-schemas

impl DocSchema {
    /// The constraints declared for `key`, when there are any.
    fn constraint(&self, key: &str) -> Option<&KeyConstraint> {
        self.frontmatter.get(key)?.as_ref()
    }

    /// The values a `variants` tag may name: the discriminator key's `enum`.
    /// `None` when the schema declares no `variant-key`, or names a key with
    /// no enum — `check_variants` reports the latter.
    fn variant_values(&self) -> Option<&[String]> {
        let key = self.variant_key.as_deref()?;
        Some(self.constraint(key)?.r#enum.as_slice()).filter(|values| !values.is_empty())
    }

    /// Whether a rule tagged `variants` binds a document whose discriminator
    /// carries `value`. An untagged rule binds every document; a tagged one
    /// binds when the value is among its tags — and never when the tag is
    /// unreadable, with no `variant-key` to read it against or a value the
    /// discriminator's enum does not carry (ADR-045).
    fn selects(&self, variants: &[String], value: Option<&str>) -> bool {
        if variants.is_empty() {
            return true;
        }
        let Some(values) = self.variant_values() else {
            return false;
        };
        variants.iter().all(|tag| values.contains(tag))
            && value.is_some_and(|v| variants.iter().any(|tag| tag == v))
    }

    /// Every pair of section rules naming one heading whose `variants` sets
    /// do not separate them (ADR-049): two rules with disjoint tags are one
    /// heading in two shapes, and any other pair is mis-declared.
    fn heading_conflicts(&self) -> Vec<HeadingConflict> {
        let mut conflicts = Vec::new();
        for (first, a) in self.sections.iter().enumerate() {
            for (second, b) in self.sections.iter().enumerate().skip(first + 1) {
                if !a.names_same_heading(b) {
                    continue;
                }
                let shared = if a.variants.is_empty() || b.variants.is_empty() {
                    None
                } else {
                    match a.variants.iter().find(|tag| b.variants.contains(tag)) {
                        Some(tag) => Some(tag.clone()),
                        None => continue,
                    }
                };
                conflicts.push(HeadingConflict {
                    first,
                    second,
                    shared,
                });
            }
        }
        conflicts
    }

    /// The section rules a document carrying `value` is checked against, in
    /// declared order: those its value selects, less both rules of every
    /// conflicting pair, which bind nothing — `check_variants` reports them
    /// (ADR-049). A heading declared once per variant is checked by the
    /// rule the value selects, at that rule's place in the order.
    fn sections_for(&self, value: Option<&str>) -> Vec<&SectionRule> {
        let unreadable: Vec<usize> = self
            .heading_conflicts()
            .iter()
            .flat_map(|c| [c.first, c.second])
            .collect();
        self.sections
            .iter()
            .enumerate()
            .filter(|(i, rule)| !unreadable.contains(i) && self.selects(&rule.variants, value))
            .map(|(_, rule)| rule)
            .collect()
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
/// not compile, a variant tag nothing reads. Each binds nothing. `validate`
/// reports all but the variant and item-key declarations through the
/// grammar's own schema check, so it calls only `check_variants` and
/// `check_item_keys` — one fault, said once; this is for callers checking
/// documents without that pass.
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
                ("item-key", rule.item_key.as_deref()),
                ("item-only-pattern", rule.item_only_pattern.as_deref()),
                (
                    "item-prohibited-pattern",
                    rule.item_prohibited_pattern.as_deref(),
                ),
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
            findings.extend(item_key_captures(file, rule));
            // An item declaration needs items to bind: without a list kind
            // the section's body has none, so the rule would pass in silence.
            // `item-only-pattern` is the exception — with no items, it binds
            // every line (ADR-047).
            let per_item = [
                ("item-pattern", rule.item_pattern.is_some()),
                ("item-key", rule.item_key.is_some()),
                (
                    "item-prohibited-pattern",
                    rule.item_prohibited_pattern.is_some(),
                ),
            ];
            let unlisted = !rule
                .content
                .as_deref()
                .is_some_and(|k| LIST_KINDS.contains(&k));
            for (name, _) in per_item
                .iter()
                .filter(|(_, declared)| *declared && unlisted)
            {
                findings.push(Finding {
                    file: file.clone(),
                    message: format!(
                        "schema: section {} declares an {name}, and its content is not {}",
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
    findings.extend(check_variants(schemas));
    findings
}

/// The `item-key` declarations the document check cannot read, reported on
/// the schema itself (ADR-047): a key pattern with no capture group, or more
/// than one, names no key and binds nothing. The grammar's schema check
/// reports a key that does not compile or sits beside no list kind; the
/// capture count is a fact about the compiled regex, so it is read here.
/// `validate` calls this beside the grammar check; `check_declarations`
/// reads the same fact from the schema it has already parsed.
#[must_use]
pub fn check_item_keys(schemas: &[(String, String)]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (file, text) in schemas {
        let Ok(Some(schema)) = DocSchema::parse(file, text) else {
            continue; // `SchemaSet::load` reports a contract that fails.
        };
        for rule in &schema.sections {
            findings.extend(item_key_captures(file, rule));
        }
    }
    findings
}

/// One rule's `item-key` capture count, as a finding on `file` when it is
/// not one. A key that does not compile is reported where every pattern's
/// compile failure is, and binds nothing here.
fn item_key_captures(file: &str, rule: &SectionRule) -> Option<Finding> {
    let pattern = rule.item_key.as_deref()?;
    let re = re::compile(pattern)?;
    let groups = re.captures_len() - 1;
    if groups == 1 {
        return None;
    }
    let counted = match groups {
        0 => "no capture group".to_string(),
        n => format!("{n} capture groups"),
    };
    Some(Finding {
        file: file.to_string(),
        message: format!(
            "schema: section {} declares item-key `{pattern}` with {counted} — the key is the \
             one capture, so the rule binds nothing",
            rule.label()
        ),
        fatal: true,
    })
}

/// The variant declarations the document check cannot read, reported on the
/// schema itself (ADR-045): a `variant-key` naming a frontmatter key with no
/// `enum`, a `variants` tag in a schema with no `variant-key`, and a tag
/// naming a value the discriminator's enum does not carry. Each binds
/// nothing. Two section rules naming one heading whose sets share a value,
/// or of which one is untagged, are reported the same way, and both bind
/// nothing (ADR-049). The keyed example's faults are `check_examples`'.
#[must_use]
pub fn check_variants(schemas: &[(String, String)]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (file, text) in schemas {
        let Ok(Some(schema)) = DocSchema::parse(file, text) else {
            continue; // `SchemaSet::load` reports a contract that fails.
        };
        let mut push = |message: String| {
            findings.push(Finding {
                file: file.clone(),
                message,
                fatal: true,
            });
        };
        let values = schema.variant_values();
        if let Some(key) = schema.variant_key.as_deref()
            && values.is_none()
        {
            push(format!(
                "schema: variant-key `{key}` names a frontmatter key with no enum — a \
                 variants tag is read against that enum, so every tag binds nothing"
            ));
        }
        let tagged = schema
            .sections
            .iter()
            .map(|rule| (format!("section {}", rule.label()), &rule.variants))
            .chain(schema.frontmatter.iter().filter_map(|(key, c)| {
                Some((format!("frontmatter `{key}`"), &c.as_ref()?.variants))
            }))
            .chain(
                schema
                    .sections_prohibited
                    .iter()
                    .map(|p| (format!("prohibited section \"{}\"", p.heading), &p.variants)),
            )
            .filter(|(_, variants)| !variants.is_empty());
        for (where_, variants) in tagged {
            let Some(key) = schema.variant_key.as_deref() else {
                push(format!(
                    "schema: {where_} declares variants [{}], and the schema declares no \
                     variant-key — the rule binds nothing",
                    variants.join(", ")
                ));
                continue;
            };
            let Some(values) = values else {
                continue; // Said once, on the key.
            };
            for tag in variants.iter().filter(|tag| !values.contains(tag)) {
                push(format!(
                    "schema: {where_} declares variant `{tag}`, and `{key}` admits {} — the \
                     rule binds nothing",
                    values.join(", ")
                ));
            }
        }
        for conflict in schema.heading_conflicts() {
            let (a, b) = (
                &schema.sections[conflict.first],
                &schema.sections[conflict.second],
            );
            let declared = format!(
                "schema: section {} is declared by two rules, {} and {}",
                a.label(),
                a.tags(),
                b.tags()
            );
            push(match conflict.shared {
                Some(value) => {
                    format!("{declared}, whose variants share `{value}` — both bind nothing")
                }
                None => format!(
                    "{declared} — an untagged rule binds every variant, so both bind nothing"
                ),
            });
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
///
/// With `variant-key` set the example is a map keyed by variant value, every
/// enum value present, and each is checked against the base rules and its
/// own variant's, prefixed `example <value>:`; an example whose
/// discriminator differs from its key, a key the enum does not carry, a
/// value with no example, and an example of the other form — one document
/// under a `variant-key`, a map without one — are each a finding (ADR-045).
#[must_use]
pub fn check_examples(schemas: &[(String, String)]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (file, text) in schemas {
        let Ok(Some(schema)) = DocSchema::parse(file, text) else {
            continue; // `SchemaSet::load` reports a contract that fails.
        };
        let Some(example) = schema.example.as_ref() else {
            continue; // The grammar's schema check reports a missing example.
        };
        let on_schema = |message: String| Finding {
            file: file.clone(),
            message,
            fatal: true,
        };
        match (schema.variant_key.as_deref(), example) {
            (None, Example::One(text)) => check_example(file, &schema, None, text, &mut findings),
            (None, Example::Keyed(_)) => findings.push(on_schema(
                "example: is keyed by variant value, and the schema declares no variant-key"
                    .to_string(),
            )),
            (Some(key), Example::One(_)) => findings.push(on_schema(format!(
                "example: is one document, and the schema declares variant-key `{key}` — \
                 write one example per value, keyed by it"
            ))),
            (Some(key), Example::Keyed(examples)) => {
                let values = schema.variant_values().unwrap_or_default();
                for value in values.iter().filter(|value| !examples.has(value)) {
                    findings.push(on_schema(format!(
                        "example: no example for {key} `{value}` — every value the \
                         discriminator admits has one"
                    )));
                }
                for (value, text) in examples.iter() {
                    if !values.iter().any(|v| v == value) {
                        findings.push(on_schema(format!(
                            "example `{value}`: names a value `{key}` does not admit"
                        )));
                        continue;
                    }
                    check_example(file, &schema, Some(value), text, &mut findings);
                }
            }
        }
    }
    findings
}

/// One example against its schema, as a document of the variant `key` names
/// (`None` for a schema without variants); every finding lands on the schema
/// file under the example's prefix.
fn check_example(
    file: &str,
    schema: &DocSchema,
    key: Option<&str>,
    example: &str,
    findings: &mut Vec<Finding>,
) {
    let prefix = key.map_or_else(|| "example".to_string(), |key| format!("example `{key}`"));
    let mut push = |message: String| {
        findings.push(Finding {
            file: file.to_string(),
            message: format!("{prefix}: {message}"),
            fatal: true,
        });
    };
    let lines: Vec<&str> = crate::fsutil::lines(example);
    let split = super::read::split_frontmatter(&lines);
    let body_start = match &split {
        Some(split) => {
            let fm = split.fm.join("\n");
            if let Err(e) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&fm) {
                push(format!(
                    "does not parse as a document — the frontmatter is not YAML: {e}"
                ));
                return;
            }
            split.body_start
        }
        None => {
            if schema.type_const().is_some() {
                push("does not parse as a document — no frontmatter block".to_string());
                return;
            }
            0
        }
    };
    // A keyed example is that variant's: its discriminator says so too, or
    // the key and the document disagree about what is being shown.
    if let (Some(key), Some(variant_key)) = (key, schema.variant_key.as_deref()) {
        let fm = split.as_ref().map_or(&[][..], |s| s.fm.as_slice());
        let entries = super::read::parse_frontmatter(fm);
        let value = carried(fm, &entries, variant_key);
        if value.as_ref().and_then(Option::as_deref) != Some(key) {
            push(format!(
                "frontmatter `{variant_key}` {}, and the example is keyed `{key}`",
                spell(value.as_ref().map(Option::as_deref))
            ));
        }
    }
    let doc = Document {
        path: file,
        text: example,
        doc_type: None,
    };
    let mut broke = Vec::new();
    check_one(&doc, schema, Subject::Example(key), &mut broke);
    check_link_form(file, &lines[body_start..], &mut broke);
    findings.extend(broke.into_iter().map(|f| Finding {
        message: format!("{prefix}: {}", f.message),
        ..f
    }));
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

// sokf:begin document-schemas
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
        check_one(doc, schema, Subject::Filed, &mut findings);
    }
    findings
}
// sokf:end document-schemas

/// What `check_one` is handed: a filed document, whose variant its own
/// discriminator value selects, or a schema's example — which the filing
/// check never reads, so `lifecycle` binds here rather than being deferred
/// to it, and whose variant is the key it is written under (ADR-045).
#[derive(Clone, Copy)]
enum Subject<'a> {
    Filed,
    Example(Option<&'a str>),
}

/// One document against one schema: the rules its variant selects, in the
/// schema's declared order, so ordering, presence, prohibition, columns and
/// the body patterns all run on that subsequence (ADR-045). A heading
/// declared once per variant is checked by the rule the value selects, its
/// shape that rule's own (ADR-049). A document with no discriminator value,
/// or one the enum does not carry, sees the untagged rules alone; the
/// frontmatter check reports the value.
fn check_one(
    doc: &Document<'_>,
    schema: &DocSchema,
    subject: Subject<'_>,
    findings: &mut Vec<Finding>,
) {
    let mut push = |message: String| {
        findings.push(Finding {
            file: doc.path.to_string(),
            message,
            fatal: true,
        });
    };

    let lines: Vec<&str> = crate::fsutil::lines(doc.text);
    let fm = super::read::split_frontmatter(&lines).map_or_else(Vec::new, |s| s.fm);
    let entries = super::read::parse_frontmatter(&fm);
    let variant: Option<String> = match subject {
        Subject::Filed => schema
            .variant_key
            .as_deref()
            .and_then(|key| carried(&fm, &entries, key).flatten()),
        Subject::Example(key) => key.map(str::to_string),
    };
    let variant = variant.as_deref();
    let sections = schema.sections_for(variant);
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

    for banned in schema
        .sections_prohibited
        .iter()
        .filter(|p| schema.selects(&p.variants, variant))
    {
        if headings.iter().any(|(_, h)| *h == banned.heading) {
            push(format!(
                "prohibited section \"{}\" ({} forbids it)",
                banned.heading, schema.name
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
        let literal = sections
            .iter()
            .position(|r| r.heading.is_some() && r.matches(*level, text));
        let by_pattern = || {
            sections
                .iter()
                .position(|r| r.heading.is_none() && r.matches(*level, text))
        };
        if let Some(i) = literal.or_else(by_pattern) {
            matched.push((h, i));
        }
    }

    for (i, rule) in sections.iter().enumerate() {
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
                    sections[pair[1]].label(),
                    sections[pair[0]].label(),
                    schema.name
                ));
                break;
            }
        }
    }

    // Each matched section must carry the form its rule's `content` kind
    // names — presence, per ADR-023: one bullet, one numbered item, one
    // table, one fenced block, one include block naming a source path, or
    // one plain paragraph line; other content beside the form is tolerated.
    // The body runs to the next heading at the section's own level or
    // shallower, so a subsection's content counts. A kind outside the
    // vocabulary is reported on the schema — by the grammar's schema check —
    // and binds nothing.
    //
    // Include blocks are found by byte offset, so the body's lines are
    // mapped to bytes once. The marker faults are the SOKF check's to
    // report; a block that does not close is no block here either.
    let (includes, _) = include_blocks(doc.text);
    let starts = line_starts(doc.text);
    let positions = heading_positions(&lines, &fenced);
    // Every key an `item-key` rule captures, in document order, so a repeat
    // anywhere in the document is found once the sections are read. A
    // level-2 body spans its level-3 subsections, so two keyed rules can
    // capture one item; the item counts once, by its line.
    let mut keys: Vec<Keyed> = Vec::new();
    for &(h, r) in &matched {
        let rule = sections[r];
        let (start, end) = body_range(h, &headings, &positions, lines.len());
        let at = |line: usize| starts.get(line).copied().unwrap_or(doc.text.len());
        let bytes = at(start)..at(end);
        let mut kind_failed = false;
        if let Some(kind) = rule.content.as_deref()
            && CONTENT_KINDS.contains(&kind)
            // A declared table's absence is check_columns' finding already.
            && (kind != "table" || rule.columns.is_empty())
            && !body_has(
                kind,
                &lines[start..end],
                &fenced[start..end],
                &bytes,
                &includes,
            )
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
            let body = &lines[start..end];
            let in_fence = &fenced[start..end];
            // The section's items are read once, and the three item
            // declarations read the same list, in the order `Items` states.
            let mut items = Items::read(rule, body, in_fence);
            for keyed in
                check_item_keys_in(rule, &headings[h].1, start, &mut items, schema, &mut push)
            {
                if !keys.iter().any(|k| k.line == keyed.line) {
                    keys.push(keyed);
                }
            }
            check_item_bounds(
                rule,
                &headings[h].1,
                body,
                in_fence,
                &mut items,
                schema,
                &mut push,
            );
            check_body_patterns(
                rule,
                &headings[h].1,
                body,
                in_fence,
                &items,
                schema,
                &mut push,
            );
        }
        if rule.content.as_deref() == Some("include") {
            check_authored_fences(
                &headings[h].1,
                &fenced[start..end],
                &starts[start..end],
                &includes,
                schema,
                &mut push,
            );
        }
    }

    // A key is unique across the document (ADR-047): each repeat names the
    // key, the item that carries it first, and itself.
    for (i, later) in keys.iter().enumerate() {
        if let Some(first) = keys[..i].iter().find(|k| k.key == later.key) {
            push(format!(
                "section \"{}\" item `{}` repeats key `{}`, carried by section \"{}\" item `{}`, \
                 and {} declares item-key",
                later.heading, later.first, later.key, first.heading, first.first, schema.name
            ));
        }
    }

    check_columns(
        &sections, schema, &headings, &positions, &lines, &fenced, &mut push,
    );
    let example = matches!(subject, Subject::Example(_));
    check_frontmatter(&fm, &entries, schema, variant, example, &mut push);
}

/// One section's body against the patterns its rule declares (ADR-030):
/// `content-pattern` over the body, `item-pattern` over each top-level item
/// of the list the section's kind names that no earlier declaration
/// reported. Both are matched found-anywhere, so a rule binds the ends by
/// writing them. `heading` is the occurrence's own text, so a finding on a
/// repeatable rule names the section that failed rather than the pattern
/// that matched it.
fn check_body_patterns(
    rule: &SectionRule,
    heading: &str,
    body: &[&str],
    fenced: &[bool],
    items: &Items,
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
    let Some(re) = re::compile(pattern) else {
        return; // An unreadable pattern binds nothing.
    };
    for item in items.unreported() {
        if !re.is_match(&item.text) {
            push(format!(
                "section \"{heading}\" item `{}` does not match, and {} declares item-pattern \
                 `{pattern}`",
                item.first, schema.name
            ));
        }
    }
}

/// One key an `item-key` rule captured: the key, the section that carries
/// the item, the item's first line, for the repeat finding to name, and the
/// document line the item opens on, so an item two rules capture is one.
struct Keyed {
    key: String,
    heading: String,
    first: String,
    line: usize,
}

/// One section's items against the `item-key` its rule declares (ADR-047):
/// every top-level item of the list the section's kind names must match,
/// and the one capture is its key. An item with no match is a finding
/// naming the section, the item and the form a key takes, so a malformed
/// key is told apart from none, and is marked reported for the later
/// declarations to skip; the keys are returned for the document-wide repeat
/// check. A key on a rule with no list kind, one that does not compile, or
/// one whose capture count is not one binds nothing — each is
/// `check_declarations`' or `check_item_keys`' finding on the schema.
fn check_item_keys_in(
    rule: &SectionRule,
    heading: &str,
    start: usize,
    items: &mut Items,
    schema: &DocSchema,
    push: &mut impl FnMut(String),
) -> Vec<Keyed> {
    let Some(pattern) = rule.item_key.as_deref() else {
        return Vec::new();
    };
    let Some(re) = re::compile(pattern).filter(|re| re.captures_len() == 2) else {
        return Vec::new();
    };
    let mut keys = Vec::new();
    for (item, reported) in items.unreported_mut() {
        match re.captures(&item.text).and_then(|c| c.get(1)) {
            Some(key) => keys.push(Keyed {
                key: key.as_str().to_string(),
                heading: heading.to_string(),
                first: item.first.clone(),
                line: start + item.lines[0],
            }),
            None => {
                push(format!(
                    "section \"{heading}\" item `{}` carries no key of the form `{pattern}`, \
                     and {} declares item-key",
                    item.first, schema.name
                ));
                *reported = true;
            }
        }
    }
    keys
}

/// One section's body against the item bounds its rule declares (ADR-047):
/// `item-only-pattern` may match only inside a top-level item of the list
/// the section's kind names, so a match on any other unfenced body line —
/// prose, a table row, a heading, an item of the other list kind, a nested
/// item — is a finding naming the section and the line; a rule with no list
/// kind has no items, so the pattern is forbidden on every line.
/// `item-prohibited-pattern` may match no top-level item, and a match on
/// one no earlier declaration reported is a finding naming the item and
/// the matched text, marked reported for the later declaration to skip; on
/// a rule with no list kind it binds nothing, as `check_declarations`
/// reports. A pattern that does not compile binds nothing either.
fn check_item_bounds(
    rule: &SectionRule,
    heading: &str,
    body: &[&str],
    fenced: &[bool],
    items: &mut Items,
    schema: &DocSchema,
    push: &mut impl FnMut(String),
) {
    if let Some(pattern) = rule.item_only_pattern.as_deref()
        && let Some(re) = re::compile(pattern)
    {
        let inside: BTreeSet<usize> = items
            .items
            .iter()
            .flat_map(|item| &item.lines)
            .copied()
            .collect();
        for (i, line) in body.iter().enumerate() {
            // An HTML comment is not content, here as in `is_paragraph`.
            if !fenced[i]
                && !inside.contains(&i)
                && !line.trim_start().starts_with("<!--")
                && re.is_match(line)
            {
                push(format!(
                    "section \"{heading}\" line `{}` matches outside a top-level item, and {} \
                     declares item-only-pattern `{pattern}`",
                    line.trim(),
                    schema.name
                ));
            }
        }
    }

    if let Some(pattern) = rule.item_prohibited_pattern.as_deref()
        && let Some(re) = re::compile(pattern)
    {
        for (item, reported) in items.unreported_mut() {
            if let Some(matched) = re.find(&item.text) {
                push(format!(
                    "section \"{heading}\" item `{}` matches `{}`, and {} declares \
                     item-prohibited-pattern `{pattern}`",
                    item.first,
                    matched.as_str(),
                    schema.name
                ));
                *reported = true;
            }
        }
    }
}

/// An `include` section's fenced blocks against ADR-042: every fence in the
/// body must sit inside an include block, because the section's content is
/// materialised and never authored — a fence outside one is the hand-written
/// copy the kind exists to forbid. `starts` is the byte offset of each body
/// line; a fence opener is a fenced line whose predecessor is not, and the
/// heading before the body is never fenced. What an include block carries is
/// not read here: keeping it current is the SOKF check's (ADR-041).
fn check_authored_fences(
    heading: &str,
    fenced: &[bool],
    starts: &[usize],
    includes: &[IncludeBlock],
    schema: &DocSchema,
    push: &mut impl FnMut(String),
) {
    let authored = fenced.iter().enumerate().any(|(i, &in_fence)| {
        in_fence
            && (i == 0 || !fenced[i - 1])
            && !includes
                .iter()
                .any(|block| (block.content_start..block.content_end).contains(&starts[i]))
    });
    if authored {
        push(format!(
            "section \"{heading}\" carries a fenced block outside an include, and {} declares \
             include content — a definition is materialised, never authored",
            schema.name
        ));
    }
}

/// The byte offset at which each line of `text` starts, one per line as
/// [`crate::fsutil::lines`] splits them.
fn line_starts(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// A section's top-level items, read once by `check_one` for the three
/// item declarations to share, and which of them a declaration has already
/// reported. One fault, said once: an item is checked by `item-key`, then
/// `item-prohibited-pattern`, then `item-pattern`, and an item that drew a
/// finding from an earlier declaration is not checked by a later one.
struct Items {
    items: Vec<Item>,
    reported: Vec<bool>,
}

impl Items {
    /// The top-level items of `body` under `rule`'s list kind. A rule with
    /// no list kind has no items; `check_declarations` reports an item
    /// declaration on one.
    fn read(rule: &SectionRule, body: &[&str], fenced: &[bool]) -> Self {
        let items = rule
            .content
            .as_deref()
            .filter(|k| LIST_KINDS.contains(k))
            .map_or_else(Vec::new, |kind| items_in(body, fenced, kind));
        let reported = vec![false; items.len()];
        Self { items, reported }
    }

    /// The items no declaration has reported yet.
    fn unreported(&self) -> impl Iterator<Item = &Item> {
        self.items
            .iter()
            .zip(&self.reported)
            .filter(|&(_, &reported)| !reported)
            .map(|(item, _)| item)
    }

    /// The items no declaration has reported yet, each with the flag a
    /// declaration sets when it reports the item.
    fn unreported_mut(&mut self) -> impl Iterator<Item = (&Item, &mut bool)> {
        self.items
            .iter()
            .zip(&mut self.reported)
            .filter(|(_, reported)| !**reported)
    }
}

/// One top-level item of a section's list, as the item declarations read
/// it.
struct Item {
    /// The item's first line, verbatim, for the finding to name it by.
    first: String,
    /// The item's own lines, marker stripped and continuations joined.
    text: String,
    /// The body indices of the item's own lines — the ones `text` joins —
    /// so a bound can tell a line inside an item from one outside (ADR-047).
    lines: Vec<usize>,
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
            .enumerate()
            .filter(|&(_, (_, &in_fence))| !in_fence)
            .map(|(i, (line, _))| (i, *line))
    };
    let marks = |line: &str| {
        let trimmed = line.trim_start();
        !is_thematic_break(trimmed) && (is_bullet(trimmed) || is_numbered(trimmed))
    };
    let Some(top) = unfenced()
        .filter(|(_, line)| marks(line))
        .map(|(_, line)| indent_of(line))
        .min()
    else {
        return Vec::new();
    };

    let mut items: Vec<Item> = Vec::new();
    // The indentation of the nested marker whose lines are being dropped, and
    // whether the open item's paragraph is still running (a blank line ends
    // it, and only an indented line may reopen one).
    let mut nested: Option<usize> = None;
    let mut flowing = false;
    let mut open = false;
    for (i, line) in unfenced() {
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
                    lines: vec![i],
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
        // only while the paragraph is still running, and a heading, a table
        // row or a comment ends the paragraph.
        if open && (indent > top || (flowing && !ends_paragraph(trimmed))) {
            if let Some(item) = items.last_mut() {
                item.text.push(' ');
                item.text.push_str(trimmed.trim_end());
                item.lines.push(i);
            }
            flowing = true;
        } else {
            open = false;
            flowing = false;
        }
    }
    items
}

/// An ATX heading, a table row or an HTML comment: a line that ends a
/// running paragraph, as CommonMark ends it, so it is never a list item's
/// lazy continuation.
fn ends_paragraph(trimmed: &str) -> bool {
    trimmed.starts_with('#') || trimmed.starts_with('|') || trimmed.starts_with("<!--")
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
/// and its constraints bind here like any other's. A constraint tagged
/// `variants` binds the variants it names (ADR-045).
fn check_frontmatter(
    fm: &[&str],
    entries: &[super::read::FmEntry],
    schema: &DocSchema,
    variant: Option<&str>,
    example: bool,
    push: &mut impl FnMut(String),
) {
    for (key, constraint) in schema.frontmatter.iter() {
        if !example && key == "lifecycle" && schema.lifecycle_enum().is_some() {
            continue;
        }
        let Some(c) = constraint else { continue };
        if !schema.selects(&c.variants, variant) {
            continue;
        }
        let Some(scalar) = carried(fm, entries, key) else {
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
        let scalar = scalar.as_deref();
        let spell = spell(Some(scalar));
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

/// What a frontmatter key carries: the value on its own line as YAML reads
/// it — comments stripped, quotes removed — or the block under it, whose
/// comment-only lines are comments rather than a block. `None` when the key
/// is absent, which a key with nothing after the colon is too; `Some(None)`
/// when it carries a value with no scalar form — a list, a map, a folded
/// block; `Some(Some(value))` otherwise.
fn carried(fm: &[&str], entries: &[super::read::FmEntry], key: &str) -> Option<Option<String>> {
    let entry = entries.iter().find(|e| e.key == key)?;
    let block = entry
        .block
        .as_ref()
        .is_some_and(|b| b.iter().any(|l| !l.trim_start().starts_with('#')));
    if entry.is_folded || block {
        return Some(None);
    }
    line_scalar(rest_of(fm, entry)).map(Some)
}

/// How a finding names what a key carries, as `carried` reads it.
fn spell(carried: Option<Option<&str>>) -> String {
    match carried {
        None => "is absent".to_string(),
        Some(None) => "is not a scalar".to_string(),
        Some(Some(value)) => format!("is `{value}`"),
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
        "include" => "include block naming a source path",
        _ => "paragraph line",
    }
}

/// Whether the kind's form appears in a section body. `fenced` is the
/// body's slice of the document's fence map: lines inside fenced blocks are
/// not content — they neither satisfy a kind nor break one — and the fence
/// itself is what satisfies `code`. A list kind is satisfied by a top-level
/// item as `items_in` reads one. `include` is satisfied by an include
/// block naming a source path whose content starts inside `bytes`, the
/// body's byte range; a concept include is shared prose, not a definition,
/// and satisfies nothing (ADR-042).
fn body_has(
    kind: &str,
    body: &[&str],
    fenced: &[bool],
    bytes: &Range<usize>,
    includes: &[IncludeBlock],
) -> bool {
    if kind == "include" {
        return includes.iter().any(|block| {
            matches!(block.target, IncludeTarget::Source { .. })
                && bytes.contains(&block.content_start)
        });
    }
    // A list is present when it has a top-level item as the item
    // declarations read one: a nested bullet or a `- - -` break is not one,
    // so a keyed rule never passes over an empty list.
    if LIST_KINDS.contains(&kind) {
        return !items_in(body, fenced, kind).is_empty();
    }
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
/// subsection counts (contract-010). `sections` is the rules the document's
/// variant selects.
fn check_columns(
    sections: &[&SectionRule],
    schema: &DocSchema,
    headings: &[(usize, String)],
    positions: &[usize],
    lines: &[&str],
    fenced: &[bool],
    push: &mut impl FnMut(String),
) {
    for rule in sections {
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
                variant_key: None,
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&at, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        let over = Document {
            path: "a.md",
            text: "one\ntwo\nthree\nfour\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&over, &schema, Subject::Filed, &mut findings);
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
            Subject::Filed,
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
            Subject::Filed,
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
            check_one(&ok, &schema, Subject::Filed, &mut findings);
            assert!(findings.is_empty(), "{kind}: {findings:#?}");

            let bad = Document {
                path: "a.md",
                text: &format!("# T\n\n## Body\n\n{fail}"),
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&bad, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // A bullet after the section's end does not count.
        let after = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\nprose\n\n## Next\n\n- one\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&after, &schema, Subject::Filed, &mut findings);
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
        check_one(&fenced_only, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");

        let bullet_after_fence = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n```\n# not a heading\n```\n\n- a bullet\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&bullet_after_fence, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a parent item\n  - a child, covers 1.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &items_schema("marker"), Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "found anywhere: {findings:#?}");

        let mut findings = Vec::new();
        check_one(
            &doc,
            &items_schema("^marker"),
            Subject::Filed,
            &mut findings,
        );
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
        check_one(&doc, &items_schema("["), Subject::Filed, &mut found);
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
            check_one(&doc, &items_schema("MUST"), Subject::Filed, &mut findings);
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
        check_one(&doc, &items_schema("MUST"), Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // The child's text is the child's: a keyword only it carries does not
        // satisfy the parent.
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- a parent item\n  - a child that MUST not count\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), Subject::Filed, &mut findings);
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
        check_one(&doc, &items_schema("MUST"), Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "lazy continuation: {findings:#?}");

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- an item with no keyword\n\nProse after the list MUST \
                   not rescue it.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "a fenced keyword: {findings:#?}");

        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- an item that MUST hold\n\n* * *\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &items_schema("MUST"), Subject::Filed, &mut findings);
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
        check_one(&doc, &items_schema("MUST"), Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("prose content"),
            "{findings:#?}"
        );
    }

    /// A schema declaring `item-key` on two bullet-list sections, for the
    /// item-key tests (ADR-047).
    fn keyed_schema(key: &str) -> DocSchema {
        schema_of(&format!(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Behaviour\n    level: 2\n\
             \x20   content: bullet-list\n    item-key: '{key}'\n  - heading: Stability\n\
             \x20   level: 2\n    content: bullet-list\n    item-key: '{key}'\n"
        ))
    }

    /// Covers I037 AC_c2 and AC_c3: an item that does not match `item-key`
    /// is a fatal finding naming the section, the item's first line and the
    /// form a key takes — a key of the wrong form is an item with no match,
    /// and the finding says which form it missed.
    #[test]
    fn an_item_without_a_key_is_a_finding_naming_the_item() {
        let schema = keyed_schema("^`(P_[a-z][a-z0-9-]*)`");
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\n- `P_one` [event] keyed\n- `P_Two` [event] malformed\n\
                   - untagged and unkeyed\n\n## Stability\n\n- `P_three` [event] keyed\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 2, "{findings:#?}");
        for finding in &findings {
            assert_eq!(finding.file, "a.md");
            assert!(finding.fatal, "{finding:#?}");
            assert!(finding.message.contains("\"Behaviour\""), "{finding:#?}");
            assert!(
                finding
                    .message
                    .contains("carries no key of the form `^`(P_[a-z][a-z0-9-]*)``")
                    && finding.message.contains("declares item-key"),
                "{finding:#?}"
            );
        }
        assert!(
            findings[0].message.contains("- `P_Two` [event] malformed"),
            "{findings:#?}"
        );
        assert!(
            findings[1].message.contains("- untagged and unkeyed"),
            "{findings:#?}"
        );
    }

    /// Covers I037 AC_c4: a key captured twice in one document, across
    /// two sections declaring `item-key`, is a finding naming the key and
    /// both items — and the same key in two documents is nobody's finding.
    #[test]
    fn a_key_repeated_across_a_documents_sections_is_a_finding() {
        let schema = keyed_schema("^`(P_[a-z][a-z0-9-]*)`");
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\n- `P_one` [event] first\n\n## Stability\n\n\
                   - `P_two` [event] second\n- `P_one` [event] repeated\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].fatal, "{findings:#?}");
        assert!(findings[0].message.contains("`P_one`"), "{findings:#?}");
        assert!(
            findings[0].message.contains("- `P_one` [event] first"),
            "{findings:#?}"
        );
        assert!(
            findings[0].message.contains("- `P_one` [event] repeated"),
            "{findings:#?}"
        );
        assert!(
            findings[0].message.contains("\"Behaviour\"")
                && findings[0].message.contains("\"Stability\""),
            "{findings:#?}"
        );

        let other = Document {
            path: "b.md",
            text: "# T\n\n## Behaviour\n\n- `P_one` [event] the same key elsewhere\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&other, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I037 AC_c1: a matching, unique key on every item passes,
    /// and a nested item's key is its own — neither bound nor counted.
    #[test]
    fn unique_keys_on_every_item_pass() {
        let schema = keyed_schema("^`(P_[a-z][a-z0-9-]*)`");
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\n- `P_one` [event] first\n  - a nested note\n\
                   - `P_two` [state] second\n\n## Stability\n\n- `P_three` [ubiquitous] third\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I037 AC_c14: an `item-key` with no capture group, or
    /// with two, and one on a `prose` rule, are each a finding on the schema
    /// file — and each binds nothing.
    #[test]
    fn a_mis_declared_item_key_is_a_finding_on_the_schema() {
        let head = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    sections:\n  - heading: Items\n    level: 2\n";
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\n- `P_one` keyed\n- `P_one` keyed again\n",
            doc_type: Some("T"),
        };

        for (key, groups) in [
            ("^`P_[a-z-]+`", "no capture group"),
            ("^`(P_)([a-z-]+)`", "2"),
        ] {
            let text = format!("{head}    content: bullet-list\n    item-key: '{key}'\n````\n");
            let findings = check_declarations(&[("s.md".into(), text.clone())]);
            assert_eq!(findings.len(), 1, "{key}: {findings:#?}");
            assert_eq!(findings[0].file, "s.md");
            assert!(findings[0].message.contains("item-key"), "{findings:#?}");
            assert!(findings[0].message.contains(groups), "{findings:#?}");
            let same = check_item_keys(&[("s.md".into(), text)]);
            assert_eq!(same.len(), 1, "{key}: {same:#?}");

            let schema = schema_of(&format!(
                "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n\
                 \x20   content: bullet-list\n    item-key: '{key}'\n"
            ));
            let mut found = Vec::new();
            check_one(&doc, &schema, Subject::Filed, &mut found);
            assert!(
                found.is_empty(),
                "an unreadable key binds nothing: {found:#?}"
            );
        }

        let misplaced = format!("{head}    content: prose\n    item-key: '(x)'\n````\n");
        let findings = check_declarations(&[("s.md".into(), misplaced)]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("item-key")
                && findings[0].message.contains("content is not"),
            "{findings:#?}"
        );
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n\
             \x20   content: prose\n    item-key: '(x)'\n",
        );
        let prose = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\nProse, then a list.\n\n- unkeyed\n",
            doc_type: Some("T"),
        };
        let mut found = Vec::new();
        check_one(&prose, &schema, Subject::Filed, &mut found);
        assert!(
            found.is_empty(),
            "a misplaced key binds nothing: {found:#?}"
        );
    }

    /// The ADR-047 example rule's four patterns, for the item-bound tests.
    const KEY: &str = "^`(P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*)`";
    const TAGGED: &str = "(?s)^`P_[a-z][a-z0-9]*(?:-[a-z0-9]+)*` \\[(ubiquitous|event|state|\
                          conditional|optional|complex)\\] .*\\b(SHALL|SHOULD|MAY)\\b";
    const VERB: &str = "\\b(SHALL|SHOULD|MAY|MUST|REQUIRED|RECOMMENDED|OPTIONAL)\\b";
    const RETIRED_OR_TWICE: &str = "\\b(MUST|REQUIRED|RECOMMENDED|OPTIONAL)\\b|(?s)\\b(SHALL|\
                                    SHOULD|MAY)\\b.*\\b(SHALL|SHOULD|MAY)\\b";

    /// A schema declaring the ADR-047 example rule — `item-key`,
    /// `item-pattern`, `item-only-pattern` and `item-prohibited-pattern` —
    /// on a bullet-list Behaviour section.
    fn bounded_schema() -> DocSchema {
        schema_of(&format!(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Behaviour\n    level: 2\n\
             \x20   content: bullet-list\n    item-key: '{KEY}'\n    item-pattern: '{TAGGED}'\n\
             \x20   item-only-pattern: '{VERB}'\n    item-prohibited-pattern: '{RETIRED_OR_TWICE}'\n"
        ))
    }

    /// Covers I037 AC_c5: a modal verb on a paragraph line, a table
    /// row, a subsection heading and a numbered item under a bullet-list
    /// rule is each a fatal finding naming the section and the line; one
    /// inside a bullet item, its continuation included, is not.
    #[test]
    fn a_match_outside_an_item_is_a_finding_naming_the_line() {
        let schema = bounded_schema();
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\nThe thing SHALL be described here.\n\n\
                   - `P_one` [event] WHEN asked, the thing\n  SHALL answer.\n\n\
                   | Verb | Use |\n|---|---|\n| MAY | an option |\n\n\
                   1. The thing SHOULD step.\n\n### What it MUST NOT do\n\n\
                   ```\nA fenced SHALL is an example of one.\n```\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 4, "{findings:#?}");
        for (finding, line) in findings.iter().zip([
            "The thing SHALL be described here.",
            "| MAY | an option |",
            "1. The thing SHOULD step.",
            "### What it MUST NOT do",
        ]) {
            assert_eq!(finding.file, "a.md");
            assert!(finding.fatal, "{finding:#?}");
            assert!(finding.message.contains("\"Behaviour\""), "{finding:#?}");
            assert!(
                finding.message.contains("item-only-pattern"),
                "{finding:#?}"
            );
            assert!(finding.message.contains(line), "{finding:#?}");
        }
    }

    /// Covers I037 AC_c5: a rule with no list `content` binds the
    /// pattern everywhere in its body — a bullet item is outside too.
    #[test]
    fn a_rule_without_a_list_kind_forbids_the_pattern_everywhere() {
        let schema = schema_of(&format!(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Notes\n    level: 2\n\
             \x20   content: prose\n    item-only-pattern: '{VERB}'\n"
        ));
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Notes\n\nA note.\n\n- the thing SHALL not promise here\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(findings[0].fatal, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("- the thing SHALL not promise here")
                && findings[0].message.contains("item-only-pattern"),
            "{findings:#?}"
        );
    }

    /// Covers I037 AC_c6 and AC_c7: an item carrying a retired verb, and
    /// one carrying two verbs, are each a fatal finding naming the item and
    /// the matched text; `SHALL NOT` is one verb, and passes. The retired
    /// verb sits beside an admitted one, so `item-pattern` is satisfied and
    /// the finding is the bound's alone.
    #[test]
    fn an_item_matching_the_prohibited_pattern_is_a_finding_naming_the_text() {
        let schema = bounded_schema();
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\n\
                   - `P_one` [event] WHEN asked, the thing SHALL answer, as it MUST.\n\
                   - `P_two` [event] WHEN told, the thing SHALL stop, and\n  SHALL stay stopped.\n\
                   - `P_three` [state] WHILE stopped, the thing SHALL NOT answer.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 2, "{findings:#?}");
        for finding in &findings {
            assert!(finding.fatal, "{finding:#?}");
            assert!(
                finding.message.contains("item-prohibited-pattern"),
                "{finding:#?}"
            );
        }
        assert!(
            findings[0]
                .message
                .contains("- `P_one` [event] WHEN asked, the thing SHALL answer, as it MUST.")
                && findings[0].message.contains("matches `MUST`"),
            "{findings:#?}"
        );
        assert!(
            findings[1]
                .message
                .contains("- `P_two` [event] WHEN told, the thing SHALL stop, and")
                && findings[1]
                    .message
                    .contains("matches `SHALL stop, and SHALL`"),
            "{findings:#?}"
        );
    }

    /// Covers I037 AC_c7: an item with a tag and no verb fails the
    /// ADR-047 `item-pattern`, and only that.
    #[test]
    fn an_item_with_a_tag_and_no_verb_fails_the_item_pattern() {
        let schema = bounded_schema();
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\n- `P_one` [event] WHEN asked, the thing answers.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("item-pattern")
                && findings[0]
                    .message
                    .contains("- `P_one` [event] WHEN asked, the thing answers."),
            "{findings:#?}"
        );
    }

    /// Covers I037 AC_c2, AC_c6 and AC_c7: an item draws one finding
    /// however many declarations it fails. `item-key` is checked first, then
    /// `item-prohibited-pattern`, then `item-pattern`, and an item reported
    /// by an earlier check is not checked by a later one — a keyless,
    /// tagless item carrying `MUST` is the key's finding alone, and a keyed,
    /// tagged item carrying `MUST` and no admitted verb is the bound's alone.
    #[test]
    fn an_item_failing_several_declarations_draws_one_finding() {
        let schema = bounded_schema();
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\n- the thing MUST answer.\n\
                   - `P_two` [event] WHEN told, the thing MUST stop.\n\
                   - `P_three` [event] WHEN asked, the thing answers.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 3, "{findings:#?}");
        assert!(
            findings[0].message.contains("- the thing MUST answer.")
                && findings[0].message.contains("declares item-key"),
            "{findings:#?}"
        );
        assert!(
            findings[1]
                .message
                .contains("- `P_two` [event] WHEN told, the thing MUST stop.")
                && findings[1].message.contains("item-prohibited-pattern"),
            "{findings:#?}"
        );
        assert!(
            findings[2]
                .message
                .contains("- `P_three` [event] WHEN asked, the thing answers.")
                && findings[2].message.contains("item-pattern"),
            "{findings:#?}"
        );
    }

    /// Covers I037 AC_c1 and AC_c2: a list kind is present only when the
    /// section carries a top-level item as the item declarations read one —
    /// a Behaviour whose bullets are all nested under a numbered step, or a
    /// `- - -` break, carries no bullet, so the kind's finding stands where
    /// the key and pattern checks would otherwise bind nothing.
    #[test]
    fn a_section_with_no_top_level_item_fails_its_list_kind() {
        let schema = bounded_schema();
        for body in [
            "1. A step.\n   - A nested bullet, which the thing SHALL not promise by.\n",
            "- - -\n",
        ] {
            let text = format!("# T\n\n## Behaviour\n\nThe thing does things.\n\n{body}");
            let doc = Document {
                path: "a.md",
                text: &text,
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&doc, &schema, Subject::Filed, &mut findings);
            assert_eq!(findings.len(), 1, "{body}: {findings:#?}");
            assert!(
                findings[0]
                    .message
                    .contains("section \"Behaviour\" carries no bullet"),
                "{body}: {findings:#?}"
            );
        }
    }

    /// Covers I037 AC_c5: a heading, a table row or an HTML comment directly
    /// under a bullet ends the item, as CommonMark ends a paragraph, rather
    /// than joining it as a lazy continuation — so a heading's verb is
    /// reported outside the item, and a comment's is not reported at all.
    #[test]
    fn a_heading_or_table_row_under_a_bullet_ends_the_item() {
        let schema = bounded_schema();
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\n\
                   - `P_one` [event] WHEN asked, the thing SHALL answer.\n\
                   <!-- the thing MUST answer, says a comment nobody reads -->\n\
                   - `P_two` [event] WHEN told, the thing SHALL stop.\n\
                   | Verb | Use |\n|---|---|\n| MAY | an option |\n\
                   - `P_three` [state] WHILE stopped, the thing SHALL NOT answer.\n\
                   ### When it MUST NOT answer\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 2, "{findings:#?}");
        for (finding, line) in findings
            .iter()
            .zip(["| MAY | an option |", "### When it MUST NOT answer"])
        {
            assert!(
                finding.message.contains("item-only-pattern")
                    && finding.message.contains("outside a top-level item")
                    && finding.message.contains(line),
                "{finding:#?}"
            );
        }
    }

    /// Covers I037 AC_c4: an item two keyed rules capture — a level-2 rule
    /// whose body spans the level-3 subsection another keyed rule matches —
    /// counts once, so it does not repeat its own key.
    #[test]
    fn an_item_captured_by_two_rules_counts_once() {
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Slices\n    level: 2\n\
             \x20   content: bullet-list\n    item-key: '^`(S_[a-z]+)`'\n\
             \x20 - heading-pattern: '^Slice \\d+$'\n    level: 3\n    repeatable: true\n\
             \x20   content: bullet-list\n    item-key: '^`(S_[a-z]+)`'\n",
        );
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Slices\n\n### Slice 1\n\n- `S_one` first\n\n### Slice 2\n\n\
                   - `S_two` second\n- `S_one` repeated\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("repeats key `S_one`")
                && findings[0].message.contains("- `S_one` first"),
            "{findings:#?}"
        );
    }

    /// Covers I037 AC_c9: an item carrying `PENDING` beside its verb,
    /// as ADR-044 places it, passes all four patterns.
    #[test]
    fn a_pending_item_passes_every_pattern() {
        let schema = bounded_schema();
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Behaviour\n\nEvery promise acts on the thing.\n\n\
                   - `P_one` [event] WHEN asked, the thing SHALL PENDING (I037) answer.\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I037 AC_c14: an `item-prohibited-pattern` on a `prose`
    /// rule, and an `item-only-pattern` or `item-prohibited-pattern` that
    /// does not compile, are each a finding on the schema file — and each
    /// binds nothing.
    #[test]
    fn a_mis_declared_item_bound_is_a_finding_on_the_schema() {
        let head = "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                    sections:\n  - heading: Items\n    level: 2\n";
        let doc = Document {
            path: "a.md",
            text: "# T\n\n## Items\n\nProse with MUST.\n\n- an item with MUST\n",
            doc_type: Some("T"),
        };

        let misplaced =
            format!("{head}    content: prose\n    item-prohibited-pattern: 'MUST'\n````\n");
        let findings = check_declarations(&[("s.md".into(), misplaced)]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0].message.contains("item-prohibited-pattern")
                && findings[0].message.contains("content is not"),
            "{findings:#?}"
        );
        let schema = schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n\
             \x20   content: prose\n    item-prohibited-pattern: 'MUST'\n",
        );
        let mut found = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut found);
        assert!(
            found.is_empty(),
            "a misplaced bound binds nothing: {found:#?}"
        );

        for name in ["item-only-pattern", "item-prohibited-pattern"] {
            let text = format!("{head}    content: bullet-list\n    {name}: '(unclosed'\n````\n");
            let findings = check_declarations(&[("s.md".into(), text)]);
            assert_eq!(findings.len(), 1, "{name}: {findings:#?}");
            assert!(
                findings[0].message.contains(name) && findings[0].message.contains("compile"),
                "{findings:#?}"
            );
            let schema = schema_of(&format!(
                "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Items\n    level: 2\n\
                 \x20   content: bullet-list\n    {name}: '(unclosed'\n"
            ));
            let mut found = Vec::new();
            check_one(&doc, &schema, Subject::Filed, &mut found);
            assert!(
                found.is_empty(),
                "an unreadable bound binds nothing: {found:#?}"
            );
        }
    }

    /// A schema declaring `include` content on one Definition section, for
    /// the ADR-042 tests.
    fn include_schema() -> DocSchema {
        schema_of(
            "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Definition\n\
             \x20   level: 2\n    content: include\n",
        )
    }

    /// One document carrying `body` as its Definition section.
    fn include_doc(body: &str) -> String {
        format!("# T\n\n## Definition\n\n{body}")
    }

    /// A source include over `content`, as `validate --fix` writes one.
    fn source_include(content: &str) -> String {
        format!("<!-- sokf:include /src/main.rs#cli -->\n{content}<!-- /sokf:include -->\n")
    }

    /// Covers I049 criterion 9: `content: include` is satisfied by an include
    /// block naming a source path, and not by one naming a concept — a
    /// concept include is shared prose, not a definition — nor by prose.
    #[test]
    fn an_include_section_is_satisfied_by_a_source_include_and_not_a_concept_include() {
        let schema = include_schema();
        let text = include_doc(&source_include("```rust\npub struct Cli;\n```\n"));
        let doc = Document {
            path: "a.md",
            text: &text,
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        let concept = include_doc(
            "<!-- sokf:include contract-style -->\nShared prose.\n<!-- /sokf:include -->\n",
        );
        for text in [concept, include_doc("Only prose here.\n")] {
            let doc = Document {
                path: "a.md",
                text: &text,
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&doc, &schema, Subject::Filed, &mut findings);
            assert_eq!(findings.len(), 1, "{findings:#?}");
            assert!(
                findings[0]
                    .message
                    .contains("carries no include block naming a source path"),
                "{findings:#?}"
            );
            assert!(
                findings[0].message.contains("\"Definition\""),
                "{findings:#?}"
            );
        }
    }

    /// Covers I049 criterion 10: a fenced block outside an include in an
    /// `include` section is an error naming the section, wherever in the
    /// body it sits; the fence inside the include is the materialised
    /// content, and is fine.
    #[test]
    fn a_fenced_block_outside_an_include_is_a_finding_naming_the_section() {
        let schema = include_schema();
        let materialised = source_include("```rust\npub struct Cli;\n```\n");
        let authored = "```yaml\nhand: written\n```\n";
        for body in [
            format!("{materialised}\n{authored}"),
            format!("{authored}\n{materialised}"),
            format!("{materialised}\n### Detail\n\n{authored}"),
        ] {
            let text = include_doc(&body);
            let doc = Document {
                path: "a.md",
                text: &text,
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&doc, &schema, Subject::Filed, &mut findings);
            assert_eq!(findings.len(), 1, "{body}: {findings:#?}");
            assert!(
                findings[0]
                    .message
                    .contains("section \"Definition\" carries a fenced block outside an include"),
                "{findings:#?}"
            );
        }

        // A fence in the next section is that section's, not this one's.
        let text = format!("{}\n## Notes\n\n{authored}", include_doc(&materialised));
        let doc = Document {
            path: "a.md",
            text: &text,
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I049 criterion 7: nothing inside an include is read. A block
    /// that parses in no language, and a `yaml` block lacking a key a schema
    /// once declared through `block-keys`, produce no finding; keeping the
    /// content current is the SOKF check's.
    #[test]
    fn nothing_inside_an_include_is_read() {
        let schema = include_schema();
        for content in [
            "```yaml\nflags: {}\n```\n",
            "```rust\n{{{ not: [valid, in any language\n```\n",
            "```\n- not a bullet\n# not a heading\n```\n",
        ] {
            let text = include_doc(&source_include(content));
            let doc = Document {
                path: "a.md",
                text: &text,
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&doc, &schema, Subject::Filed, &mut findings);
            assert!(findings.is_empty(), "{content}: {findings:#?}");
        }
    }

    /// Covers I049 criterion 7: the withdrawn `block-language`, `block-keys`
    /// and `block-entry-keys` are the grammar's unknown keys (`check.rs`);
    /// here they are not read, so no schema finding and no document finding
    /// follows from one — a `yaml` block missing a formerly declared key
    /// passes.
    #[test]
    fn a_withdrawn_block_declaration_binds_nothing() {
        for declaration in [
            "block-language: yaml",
            "block-keys: [commands]",
            "block-entry-keys: [about]",
        ] {
            let text = format!(
                "---\ntype: Schema\n---\n\n````yaml\nfrontmatter:\n  type:\n    const: T\n\
                 sections:\n  - heading: Commands\n    level: 2\n    content: code\n\
                 \x20   {declaration}\n````\n"
            );
            let schemas = [("s.md".to_string(), text)];
            let (set, findings) = SchemaSet::load(&schemas);
            assert!(findings.is_empty(), "{declaration}: {findings:#?}");
            let findings = check_declarations(&schemas);
            assert!(findings.is_empty(), "{declaration}: {findings:#?}");
            let found = check_documents(
                &[Document {
                    path: "a.md",
                    text: "# T\n\n## Commands\n\n```yaml\nflags: {}\n```\n",
                    doc_type: Some("T"),
                }],
                &set,
            );
            assert!(found.is_empty(), "{declaration}: {found:#?}");
        }
    }

    /// Covers I049 criterion 9: a schema's own example is checked against
    /// `include` content on the ADR-024 path, where the include markers sit
    /// inside the schema's fenced example and are still read as markers once
    /// the example is a document of its own.
    #[test]
    fn an_example_is_checked_against_its_schemas_include_content() {
        let contract = "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Definition\n\
                        \x20   level: 2\n    content: include\n";
        let passing = "---\ntype: T\n---\n\n# T\n\n## Definition\n\n\
                       <!-- sokf:include /src/main.rs#cli -->\n```rust\npub struct Cli;\n```\n\
                       <!-- /sokf:include -->\n";
        let findings = check_examples(&with_example(contract, passing));
        assert!(findings.is_empty(), "{findings:#?}");

        let authored = "---\ntype: T\n---\n\n# T\n\n## Definition\n\n```yaml\nhand: written\n```\n";
        let findings = check_examples(&with_example(contract, authored));
        assert_eq!(findings.len(), 2, "{findings:#?}");
        assert!(
            findings
                .iter()
                .all(|f| f.message.starts_with("example:") && f.message.contains("\"Definition\"")),
            "{findings:#?}"
        );
    }

    /// A schema declaring `variant-key: kind` over two kinds, with a section
    /// only kind `a` carries between two every kind carries, for the
    /// ADR-045 tests.
    const VARIANT_CONTRACT: &str = "frontmatter:\n  type:\n    const: T\n  kind:\n\
        \x20   required: true\n    enum: [a, b]\nvariant-key: kind\nsections-ordered: true\n\
        sections:\n  - heading: Shared\n    level: 2\n    required: true\n\
        \x20 - heading: Only A\n    level: 2\n    required: true\n    variants: [a]\n\
        \x20 - heading: Tail\n    level: 2\n    required: true\n";

    /// A document of `kind` whose body is the headings named.
    fn kind_doc(kind: &str, headings: &[&str]) -> String {
        let body: Vec<String> = headings.iter().map(|h| format!("## {h}\n\nx\n")).collect();
        format!(
            "---\ntype: T\nkind: {kind}\n---\n\n# T\n\n{}",
            body.join("\n")
        )
    }

    /// The findings a document of `kind` carrying `headings` gets from
    /// `schema`.
    fn findings_for(schema: &DocSchema, kind: &str, headings: &[&str]) -> Vec<Finding> {
        let text = kind_doc(kind, headings);
        let doc = Document {
            path: "a.md",
            text: &text,
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, schema, Subject::Filed, &mut findings);
        findings
    }

    /// Covers I049 criterion 13: a section tagged `[a]` is required for kind
    /// `a` and absent from kind `b`'s rules, so a `b` document lacking it
    /// passes; an untagged section applies to every kind.
    #[test]
    fn a_tagged_section_binds_its_variants_and_an_untagged_one_binds_all() {
        let schema = schema_of(VARIANT_CONTRACT);
        let findings = findings_for(&schema, "a", &["Shared", "Tail"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section \"Only A\""),
            "{findings:#?}"
        );
        let findings = findings_for(&schema, "a", &["Shared", "Only A", "Tail"]);
        assert!(findings.is_empty(), "{findings:#?}");
        let findings = findings_for(&schema, "b", &["Shared", "Tail"]);
        assert!(findings.is_empty(), "{findings:#?}");

        // The untagged sections bind kind `b` as they bind kind `a`.
        let findings = findings_for(&schema, "b", &["Shared"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section \"Tail\""),
            "{findings:#?}"
        );
    }

    /// Covers I049 criterion 13: `sections-ordered` holds on the subsequence
    /// a kind sees, so a kind `b` document is not faulted for a kind `a`
    /// section's position, while a kind `a` document is.
    #[test]
    fn sections_ordered_holds_on_the_subsequence_a_variant_sees() {
        let schema = schema_of(VARIANT_CONTRACT);
        let findings = findings_for(&schema, "b", &["Shared", "Tail", "Only A"]);
        assert!(findings.is_empty(), "{findings:#?}");
        let findings = findings_for(&schema, "a", &["Shared", "Tail", "Only A"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("orders them the other way"),
            "{findings:#?}"
        );
    }

    /// Covers I049 criterion 13: a frontmatter key tagged `[a]` is required
    /// for `a` and unchecked for `b`, and a prohibited entry tagged `[a]`
    /// bans its heading in `a` alone.
    #[test]
    fn a_tagged_frontmatter_key_and_a_tagged_prohibition_bind_their_variants() {
        let with_owner = VARIANT_CONTRACT.replace(
            "    enum: [a, b]\n",
            "    enum: [a, b]\n  owner:\n    required: true\n    variants: [a]\n",
        );
        let schema = schema_of(&format!(
            "{with_owner}sections-prohibited:\n  - heading: Notes\n    variants: [a]\n  - Draft\n"
        ));
        let mut findings = findings_for(&schema, "a", &["Shared", "Only A", "Tail", "Notes"]);
        findings.extend(findings_for(
            &schema,
            "a",
            &["Shared", "Only A", "Tail", "Draft"],
        ));
        let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
        assert_eq!(findings.len(), 4, "{findings:#?}");
        assert!(
            messages
                .iter()
                .filter(|m| m.contains("`owner` is absent"))
                .count()
                == 2,
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("prohibited section \"Notes\"")),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("prohibited section \"Draft\"")),
            "{messages:?}"
        );

        // Kind `b`: the owner is unchecked, Notes is allowed, Draft is not.
        let findings = findings_for(&schema, "b", &["Shared", "Tail", "Notes"]);
        assert!(findings.is_empty(), "{findings:#?}");
        let findings = findings_for(&schema, "b", &["Shared", "Tail", "Draft"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
    }

    /// A document whose discriminator is absent, or carries a value the enum
    /// does not admit, sees the untagged rules alone; the frontmatter check
    /// says what the value is.
    #[test]
    fn a_document_with_no_readable_variant_sees_the_untagged_rules_alone() {
        let schema = schema_of(VARIANT_CONTRACT);
        let findings = findings_for(&schema, "c", &["Shared", "Tail"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("`kind` is `c`"),
            "{findings:#?}"
        );

        let text = "---\ntype: T\n---\n\n# T\n\n## Shared\n\nx\n\n## Tail\n\nx\n";
        let doc = Document {
            path: "a.md",
            text,
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("`kind` is absent"),
            "{findings:#?}"
        );
    }

    /// One schema file carrying `contract` and a keyed `example:` block, one
    /// document per (key, text) pair.
    fn with_keyed_examples(contract: &str, examples: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut block = String::from("example:\n");
        for (key, text) in examples {
            block.push_str(&format!("  {key}: |\n"));
            for line in text.split('\n') {
                if line.is_empty() {
                    block.push('\n');
                } else {
                    block.push_str(&format!("    {line}\n"));
                }
            }
        }
        let text = format!("---\ntype: Schema\n---\n\n````yaml\n{contract}{block}````\n");
        vec![("s.md".to_string(), text)]
    }

    /// Covers I049 criterion 14: a keyed example is checked per key against
    /// the base rules and its own variant's — `a`'s must carry Only A, `b`'s
    /// need not — and an example whose discriminator differs from its key is
    /// a finding on the schema file.
    #[test]
    fn a_keyed_example_is_checked_per_key_against_its_own_variant() {
        let a = kind_doc("a", &["Shared", "Only A", "Tail"]);
        let b = kind_doc("b", &["Shared", "Tail"]);
        let findings = check_examples(&with_keyed_examples(
            VARIANT_CONTRACT,
            &[("a", &a), ("b", &b)],
        ));
        assert!(findings.is_empty(), "{findings:#?}");

        let a_lacking = kind_doc("a", &["Shared", "Tail"]);
        let findings = check_examples(&with_keyed_examples(
            VARIANT_CONTRACT,
            &[("a", &a_lacking), ("b", &b)],
        ));
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0]
                .message
                .starts_with("example `a`: missing required section \"Only A\""),
            "{findings:#?}"
        );

        // Keyed `b`, declaring itself `a`: the key and the document disagree
        // about what is shown, and the check runs against the key's variant.
        let findings = check_examples(&with_keyed_examples(
            VARIANT_CONTRACT,
            &[("a", &a), ("b", &a)],
        ));
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("example `b`: frontmatter `kind` is `a`, and the example is keyed `b`"),
            "{findings:#?}"
        );
    }

    /// Covers I049 criteria 13 and 14: a tag outside the enum, a tag without
    /// `variant-key`, a missing example key and a keyed example without
    /// `variant-key` are each a finding on the schema file — and the
    /// unreadable rule binds nothing.
    #[test]
    fn each_variant_mis_declaration_is_a_finding_on_the_schema_file() {
        let outside = VARIANT_CONTRACT.replace("variants: [a]", "variants: [a, c]");
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{outside}````\n"),
        )];
        let findings = check_variants(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(
            findings[0]
                .message
                .contains("section \"Only A\" declares variant `c`, and `kind` admits a, b"),
            "{findings:#?}"
        );
        let findings = findings_for(&schema_of(&outside), "a", &["Shared", "Tail"]);
        assert!(findings.is_empty(), "binds nothing: {findings:#?}");

        let untagged_schema = VARIANT_CONTRACT.replace("variant-key: kind\n", "");
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{untagged_schema}````\n"),
        )];
        let findings = check_variants(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains(
                "section \"Only A\" declares variants [a], and the schema declares no variant-key"
            ),
            "{findings:#?}"
        );
        let findings = findings_for(&schema_of(&untagged_schema), "a", &["Shared", "Tail"]);
        assert!(findings.is_empty(), "binds nothing: {findings:#?}");

        let a = kind_doc("a", &["Shared", "Only A", "Tail"]);
        let findings = check_examples(&with_keyed_examples(VARIANT_CONTRACT, &[("a", &a)]));
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("example: no example for kind `b`"),
            "{findings:#?}"
        );

        let findings = check_examples(&with_keyed_examples(&untagged_schema, &[("a", &a)]));
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains(
                "example: is keyed by variant value, and the schema declares no variant-key"
            ),
            "{findings:#?}"
        );
    }

    /// The remaining mis-declarations around the key: a `variant-key` naming
    /// a frontmatter key with no enum — the enum is what the tags are read
    /// against, so it is reported once and every tag binds nothing — one
    /// example under a `variant-key`, and an example keyed by a value the
    /// enum does not admit.
    #[test]
    fn a_variant_key_without_an_enum_and_a_mis_shaped_example_are_findings() {
        let no_enum = VARIANT_CONTRACT.replace("    enum: [a, b]\n", "");
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{no_enum}````\n"),
        )];
        let findings = check_variants(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("variant-key `kind` names a frontmatter key with no enum"),
            "{findings:#?}"
        );
        let findings = findings_for(&schema_of(&no_enum), "a", &["Shared", "Tail"]);
        assert!(findings.is_empty(), "binds nothing: {findings:#?}");

        let findings = check_examples(&with_example(
            VARIANT_CONTRACT,
            &kind_doc("a", &["Shared", "Only A", "Tail"]),
        ));
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("example: is one document, and the schema declares variant-key `kind`"),
            "{findings:#?}"
        );

        let a = kind_doc("a", &["Shared", "Only A", "Tail"]);
        let b = kind_doc("b", &["Shared", "Tail"]);
        let findings = check_examples(&with_keyed_examples(
            VARIANT_CONTRACT,
            &[("a", &a), ("b", &b), ("c", &b)],
        ));
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("example `c`: names a value `kind` does not admit"),
            "{findings:#?}"
        );
    }

    /// A contract declaring one heading twice (ADR-049): `Criteria` is a plain
    /// numbered list for `unframed`, and a keyed one for `framed` and `done`.
    const PER_VARIANT_CONTRACT: &str = "frontmatter:\n  type:\n    const: T\n  lifecycle:\n\
        \x20   required: true\n    enum: [unframed, framed, done]\nvariant-key: lifecycle\n\
        sections-ordered: true\nsections:\n  - heading: Context\n    level: 2\n    required: true\n\
        \x20 - heading: Criteria\n    level: 2\n    required: true\n    content: numbered-list\n\
        \x20   variants: [unframed]\n  - heading: Criteria\n    level: 2\n    required: true\n\
        \x20   content: numbered-list\n    item-key: '^`(AC_[a-z]+)`'\n\
        \x20   item-pattern: '^`AC_[a-z]+` \\[event\\] '\n    variants: [framed, done]\n\
        \x20 - heading: Tail\n    level: 2\n    required: true\n";

    /// A document of `lifecycle` whose Criteria section carries `items`, the
    /// other sections in the declared order unless `tail_first`.
    fn lifecycle_doc(lifecycle: &str, items: &str, tail_first: bool) -> String {
        let criteria = format!("## Criteria\n\n{items}\n");
        let tail = "## Tail\n\nx\n";
        let (second, third) = if tail_first {
            (tail, criteria.as_str())
        } else {
            (criteria.as_str(), tail)
        };
        format!(
            "---\ntype: T\nlifecycle: {lifecycle}\n---\n\n# T\n\n## Context\n\nx\n\n{second}\n{third}"
        )
    }

    /// The findings a document of `lifecycle` carrying `items` under Criteria
    /// gets from `schema`.
    fn lifecycle_findings(
        schema: &DocSchema,
        lifecycle: &str,
        items: &str,
        tail_first: bool,
    ) -> Vec<Finding> {
        let text = lifecycle_doc(lifecycle, items, tail_first);
        let doc = Document {
            path: "a.md",
            text: &text,
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, schema, Subject::Filed, &mut findings);
        findings
    }

    /// Covers I030 AC_one-schema-per-kind: two rules for one heading with
    /// disjoint variants — an unframed document sees the plain rule and none
    /// of the keyed rule's findings, a framed one sees the keyed rule and
    /// none of the plain rule's leniency (ADR-049).
    #[test]
    fn a_heading_declared_per_variant_binds_the_rule_the_value_selects() {
        let schema = schema_of(PER_VARIANT_CONTRACT);
        let findings = lifecycle_findings(&schema, "unframed", "1. plain and keyless", false);
        assert!(findings.is_empty(), "{findings:#?}");
        let findings = lifecycle_findings(&schema, "framed", "1. plain and keyless", false);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("section \"Criteria\" item `1. plain and keyless` carries no key"),
            "{findings:#?}"
        );
        let findings = lifecycle_findings(&schema, "done", "1. `AC_one` [event] keyed", false);
        assert!(findings.is_empty(), "{findings:#?}");
        // The heading is required in every variant: each rule says so.
        let text =
            "---\ntype: T\nlifecycle: framed\n---\n\n# T\n\n## Context\n\nx\n\n## Tail\n\nx\n";
        let doc = Document {
            path: "a.md",
            text,
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section \"Criteria\""),
            "{findings:#?}"
        );
    }

    /// Covers I030 AC_one-schema-per-kind: `sections-ordered` holds with the
    /// recurring heading at one position — Criteria after Tail is out of
    /// order in every variant, and in order in every variant otherwise.
    #[test]
    fn sections_ordered_holds_with_the_recurring_heading_at_one_position() {
        let schema = schema_of(PER_VARIANT_CONTRACT);
        for (lifecycle, items) in [
            ("unframed", "1. plain"),
            ("framed", "1. `AC_one` [event] keyed"),
        ] {
            let findings = lifecycle_findings(&schema, lifecycle, items, false);
            assert!(findings.is_empty(), "{lifecycle}: {findings:#?}");
            let findings = lifecycle_findings(&schema, lifecycle, items, true);
            assert_eq!(findings.len(), 1, "{lifecycle}: {findings:#?}");
            assert!(
                findings[0].message.contains(
                    "section \"Criteria\" comes after \"Tail\", and s orders them the other way"
                ),
                "{lifecycle}: {findings:#?}"
            );
        }
    }

    /// Covers I030 AC_one-schema-per-kind: two rules for one heading whose
    /// sets share a value are a finding on the schema naming the heading and
    /// the value; an untagged twin is a finding naming it; and in both cases
    /// neither rule binds — a framed document with a keyless item passes.
    #[test]
    fn overlapping_or_untagged_rules_for_one_heading_are_a_schema_finding_and_bind_nothing() {
        let overlapping =
            PER_VARIANT_CONTRACT.replace("variants: [unframed]", "variants: [unframed, done]");
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{overlapping}````\n"),
        )];
        let findings = check_variants(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert_eq!(findings[0].file, "s.md");
        assert!(findings[0].fatal);
        assert!(
            findings[0].message.contains(
                "section \"Criteria\" is declared by two rules, tagged [unframed, done] and \
                 tagged [framed, done], whose variants share `done` — both bind nothing"
            ),
            "{findings:#?}"
        );
        let findings = lifecycle_findings(&schema_of(&overlapping), "framed", "1. keyless", false);
        assert!(findings.is_empty(), "binds nothing: {findings:#?}");

        let untagged = PER_VARIANT_CONTRACT.replace("    variants: [unframed]\n", "");
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{untagged}````\n"),
        )];
        let findings = check_variants(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains(
                "section \"Criteria\" is declared by two rules, untagged and tagged [framed, \
                 done] — an untagged rule binds every variant, so both bind nothing"
            ),
            "{findings:#?}"
        );
        let findings = lifecycle_findings(&schema_of(&untagged), "framed", "1. keyless", false);
        assert!(findings.is_empty(), "binds nothing: {findings:#?}");

        // A heading declared once at each of two levels is two headings.
        let two_levels = "frontmatter:\n  type:\n    const: T\nsections:\n  - heading-pattern: '^.+$'\n\
                          \x20   level: 1\n  - heading-pattern: '^.+$'\n    level: 3\n";
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{two_levels}````\n"),
        )];
        assert!(check_variants(&schemas).is_empty());
    }

    /// A literal `heading` and a `heading-pattern` it matches are two
    /// headings, not one (code-review-010 finding 3): the disjointness check
    /// is by declaration form, the literal wins the heading it names, and the
    /// pattern names every other heading it matches — the shape 18 live
    /// schemas take, a fixed heading beside a catch-all `^.+$` at one level.
    /// A pattern that can name nothing beyond the literal is therefore a
    /// required rule no document satisfies, reported on the document.
    #[test]
    fn a_literal_beside_a_pattern_it_matches_is_two_headings() {
        let contract = "frontmatter:\n  type:\n    const: T\n  lifecycle:\n    required: true\n\
                        \x20   enum: [unframed, framed]\nvariant-key: lifecycle\nsections:\n\
                        \x20 - heading: Notes\n    level: 2\n    required: true\n    variants: [framed]\n\
                        \x20 - heading-pattern: '^.+$'\n    level: 2\n    required: true\n";
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{contract}````\n"),
        )];
        assert!(check_variants(&schemas).is_empty());
        let schema = schema_of(contract);
        let check = |text: &str| {
            let doc = Document {
                path: "a.md",
                text,
                doc_type: Some("T"),
            };
            let mut findings = Vec::new();
            check_one(&doc, &schema, Subject::Filed, &mut findings);
            findings
        };
        // The literal takes `Notes`; the catch-all takes `Other`.
        let findings = check(
            "---\ntype: T\nlifecycle: framed\n---\n\n# T\n\n## Notes\n\nx\n\n## Other\n\nx\n",
        );
        assert!(findings.is_empty(), "{findings:#?}");
        // With no other heading, the catch-all is the rule left unsatisfied.
        let findings = check("---\ntype: T\nlifecycle: framed\n---\n\n# T\n\n## Notes\n\nx\n");
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section matching /^.+$/"),
            "{findings:#?}"
        );

        // A pattern naming nothing beyond the literal is that case always:
        // the schema check is silent, and the document reports the pattern.
        let exact = contract.replace("'^.+$'", "'^Notes$'");
        let schemas = [(
            "s.md".to_string(),
            format!("---\ntype: Schema\n---\n\n````yaml\n{exact}````\n"),
        )];
        assert!(check_variants(&schemas).is_empty());
        let mut findings = Vec::new();
        check_one(
            &Document {
                path: "a.md",
                text: "---\ntype: T\nlifecycle: framed\n---\n\n# T\n\n## Notes\n\nx\n",
                doc_type: Some("T"),
            },
            &schema_of(&exact),
            Subject::Filed,
            &mut findings,
        );
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section matching /^Notes$/"),
            "{findings:#?}"
        );
    }

    /// Covers I030 AC_one-schema-per-kind: a keyed example map with one
    /// example per value passes when each example matches its own variant's
    /// rule for the shared heading, and fails naming the one that does not.
    #[test]
    fn a_keyed_example_per_value_passes_against_its_own_rule_for_the_shared_heading() {
        let contract = PER_VARIANT_CONTRACT
            .replace(
                "[unframed, framed, done]",
                "[unframed, framed, done, wontfix]",
            )
            .replace(
                "variants: [framed, done]",
                "variants: [framed, done, wontfix]",
            );
        let plain = lifecycle_doc("unframed", "1. plain", false);
        let keyed = |lifecycle: &str| lifecycle_doc(lifecycle, "1. `AC_one` [event] keyed", false);
        let (framed, done, wontfix) = (keyed("framed"), keyed("done"), keyed("wontfix"));
        let findings = check_examples(&with_keyed_examples(
            &contract,
            &[
                ("unframed", &plain),
                ("framed", &framed),
                ("done", &done),
                ("wontfix", &wontfix),
            ],
        ));
        assert!(findings.is_empty(), "{findings:#?}");

        let keyless = lifecycle_doc("framed", "1. plain", false);
        let findings = check_examples(&with_keyed_examples(
            &contract,
            &[
                ("unframed", &plain),
                ("framed", &keyless),
                ("done", &done),
                ("wontfix", &wontfix),
            ],
        ));
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.starts_with(
                "example `framed`: section \"Criteria\" item `1. plain` carries no key"
            ),
            "{findings:#?}"
        );
    }

    /// Covers I049 criterion 13: a schema with no `variant-key` and a string
    /// `example` is checked exactly as before — no variant finding, the
    /// example under its plain prefix, and a document against every rule.
    #[test]
    fn a_schema_without_variants_is_checked_as_before() {
        let contract = "frontmatter:\n  type:\n    const: T\nsections:\n  - heading: Context\n\
                        \x20   level: 2\n    required: true\n";
        let schemas = with_example(contract, "---\ntype: T\n---\n\n# A\n\n## Other\n\nx\n");
        assert!(check_variants(&schemas).is_empty());
        let findings = check_examples(&schemas);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .starts_with("example: missing required section \"Context\""),
            "{findings:#?}"
        );
        let findings = findings_for(&schema_of(contract), "a", &["Other"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section \"Context\""),
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0].message.contains("no paragraph line"),
            "{findings:#?}"
        );
    }

    /// Covers I018 criterion 5: a kind outside the six is reported on the
    /// schema file — by `check_declarations`; `validate` says it through the
    /// grammar's schema check instead — and the unreadable rule binds
    /// nothing.
    #[test]
    fn a_kind_outside_the_six_is_reported_on_the_schema_file() {
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&ok, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&broken, &schema, Subject::Filed, &mut findings);
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
        check_one(&ok, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
        assert!(findings.is_empty(), "{findings:#?}");

        // A key carrying only a comment is absent.
        let schema = schema_of("frontmatter:\n  type:\n    const: T\n  id:\n    required: true\n");
        let doc = Document {
            path: "a.md",
            text: "---\ntype: T\nid: # to be assigned\n---\n\n# A\n",
            doc_type: Some("T"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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
        check_one(&doc, &with_enum, Subject::Filed, &mut findings);
        assert!(
            findings.is_empty(),
            "the filing check reports it: {findings:#?}"
        );

        let without_enum =
            schema_of("frontmatter:\n  type:\n    const: T\n  lifecycle:\n    required: true\n");
        let mut findings = Vec::new();
        check_one(&doc, &without_enum, Subject::Filed, &mut findings);
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
        check_one(&doc, &schema, Subject::Filed, &mut findings);
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

/// The shipped contract schema, in its final form (ADR-043, ADR-045): the
/// per-kind sections it declares, and the title rule per kind.
#[cfg(test)]
mod contract_schema {
    use super::*;

    /// The twelve kinds, in the enum's order.
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

    /// `knowledge/schemas/contract.md` as the repository ships it.
    fn shipped() -> (String, String) {
        let path: std::path::PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "../../..",
            "knowledge/schemas/contract.md",
        ]
        .iter()
        .collect();
        (
            "contract.md".to_string(),
            std::fs::read_to_string(path).expect("the contract schema is on file"),
        )
    }

    fn schema() -> DocSchema {
        let (name, text) = shipped();
        DocSchema::parse(&name, &text)
            .expect("the contract deserializes")
            .expect("the contract is there")
    }

    /// A contract of `kind` titled `title`, carrying `sections` as `###`
    /// under Behaviour, each with one keyed promise (ADR-046), and the base
    /// sections every kind carries.
    fn contract(kind: &str, title: &str, sections: &[&str]) -> String {
        let body: Vec<String> = sections
            .iter()
            .map(|h| {
                let key = h.to_lowercase().replace(' ', "-");
                format!("### {h}\n\n- `P_{key}` [ubiquitous] The thing SHALL hold.\n")
            })
            .collect();
        format!(
            "---\ntype: Contract\nid: contract-001-{kind}-thing\nkind: {kind}\ntitle: T\n\
             description: D\n---\n\n# {title}\n\n## Definition\n\n\
             <!-- sokf:include /src/lib.rs#api -->\n```rust\npub struct Api;\n```\n\
             <!-- /sokf:include -->\n\n## Behaviour\n\n{}\n## Stability\n\n\
             - `P_change` [ubiquitous] The thing MAY change.\n",
            body.join("\n")
        )
    }

    fn findings_for(kind: &str, title: &str, sections: &[&str]) -> Vec<Finding> {
        let text = contract(kind, title, sections);
        let doc = Document {
            path: "c.md",
            text: &text,
            doc_type: Some("Contract"),
        };
        let mut findings = Vec::new();
        check_one(&doc, &schema(), Subject::Filed, &mut findings);
        findings
    }

    /// Covers I049 criterion 15: a `cli` contract without `### Exit codes`
    /// fails naming the section; with it, and without the optional
    /// `### Prompting`, it passes.
    #[test]
    fn a_cli_contract_carries_exit_codes_and_may_omit_prompting() {
        let findings = findings_for("cli", "CLI contract: thing", &["Streams"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section \"Exit codes\""),
            "{findings:#?}"
        );
        let findings = findings_for("cli", "CLI contract: thing", &["Exit codes", "Streams"]);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I049 criterion 15: `### Authentication` is required for `api`
    /// and is not a `cli` contract's rule.
    #[test]
    fn authentication_binds_an_api_contract_and_not_a_cli_one() {
        let findings = findings_for(
            "api",
            "API contract: thing",
            &["Transport", "Errors", "Limits", "Versioning"],
        );
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section \"Authentication\""),
            "{findings:#?}"
        );
        let findings = findings_for("cli", "CLI contract: thing", &["Exit codes", "Streams"]);
        assert!(findings.is_empty(), "{findings:#?}");
    }

    /// Covers I049 criterion 12: the title rule a kind sees names its own
    /// display name, so a `cli` contract titled as an API fails it.
    #[test]
    fn a_title_naming_another_kind_fails_the_kinds_title_rule() {
        let findings = findings_for("cli", "API contract: thing", &["Exit codes", "Streams"]);
        assert_eq!(findings.len(), 1, "{findings:#?}");
        assert!(
            findings[0]
                .message
                .contains("missing required section matching /^CLI contract: .+$/"),
            "{findings:#?}"
        );
    }

    /// Covers I049 criteria 8 and 14: the one schema carries one example
    /// per kind, and every example passes base plus its own variant.
    #[test]
    fn every_kinds_example_passes() {
        let schema = schema();
        assert_eq!(
            schema.variant_values().map(<[String]>::to_vec),
            Some(KINDS.iter().map(|k| (*k).to_string()).collect())
        );
        let Some(Example::Keyed(examples)) = &schema.example else {
            panic!("the example is keyed by kind");
        };
        let keys: Vec<&str> = examples.0.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, KINDS, "one example per kind, in the enum's order");
        let findings = check_examples(&[shipped()]);
        assert!(findings.is_empty(), "{findings:#?}");
    }
}
