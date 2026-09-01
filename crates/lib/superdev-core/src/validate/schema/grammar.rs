//! grammar.rs — the grammar, as types.
//!
//! Every struct is `#[serde(deny_unknown_fields)]`, so a typo in the grammar
//! is a deserialisation error naming the key. That is the job the JSON
//! meta-schema used to do for the Node reference: the types are the
//! meta-schema now, and the compiler enforces the half of it that was a
//! required-field list.
//!
//! Mappings keep the order they were written in. Several findings list the
//! declared names — "unit files use only skill/goal/constraints/…" — and the
//! reference lists them in grammar order, so a sorted map would change those
//! messages and break parity.

use std::fmt;
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

/// A YAML mapping that keeps its written order.
///
/// `BTreeMap` would sort, and `IndexMap` would be a dependency; the grammar's
/// maps are small enough that a linear scan costs nothing.
#[derive(Debug, Clone)]
pub struct Ordered<V>(pub Vec<(String, V)>);

// Written out rather than derived: the derive would demand `V: Default`, and
// an empty mapping needs nothing of its value type.
impl<V> Default for Ordered<V> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<V> Ordered<V> {
    /// The value written under `key`, if any.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&V> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Whether `key` was written.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// The keys, in the order the grammar writes them.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(k, _)| k.as_str())
    }

    /// Every pair, in the order the grammar writes them.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &V)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// How many pairs were written.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing was written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for Ordered<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct InOrder<V>(PhantomData<V>);

        impl<'de, V: Deserialize<'de>> Visitor<'de> for InOrder<V> {
            type Value = Ordered<V>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a mapping")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut out = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((k, v)) = map.next_entry::<String, V>()? {
                    out.push((k, v));
                }
                Ok(Ordered(out))
            }
        }

        deserializer.deserialize_map(InOrder(PhantomData))
    }
}

/// The whole grammar.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grammar {
    /// Always `superdev`; the file says which grammar it is.
    pub grammar: String,
    /// The grammar's own version, bumped when the vocabulary changes.
    pub version: String,
    /// What the grammar is, in its own words.
    pub doc: String,
    /// Where a bare run looks.
    pub roots: Roots,
    /// The one condition attribute and its closed vocabulary.
    pub conditions: Conditions,
    /// The tool roster a `tool_call` may name.
    pub tools: Tools,
    /// The three file kinds and their rules.
    pub kinds: Kinds,
    /// The cross-file one-home-per-statement check.
    pub duplication: Duplication,
}

/// Where a bare run looks for files a kind claims.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roots {
    /// Why the list is an allowlist rather than a set of exclusions.
    pub doc: String,
    /// The directories, repo-root relative.
    pub paths: Vec<String>,
}

/// The one condition attribute and the closed set of forms it takes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conditions {
    /// The attribute name: one across the whole grammar.
    pub attribute: String,
    /// What a condition means and why there is only one attribute.
    pub doc: String,
    /// The anchored regex every condition value must match.
    pub pattern: String,
    /// The four forms, spelled out for the doc renderer.
    pub forms: Vec<String>,
    /// Spellings the grammar refuses by name, so a rename is reported rather
    /// than read as an unknown attribute.
    #[serde(default, rename = "renamedFrom")]
    pub renamed_from: Ordered<String>,
}

/// The tool roster a `tool_call` may name.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    /// Why the roster is closed.
    pub doc: String,
    /// The tool names a `tool_call` may use.
    pub roster: Vec<String>,
    /// Whether a `tool_call` naming a tool outside the roster is a finding.
    #[serde(rename = "enforceOnToolCall")]
    pub enforce_on_tool_call: bool,
}

/// The three file kinds.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kinds {
    /// Skills and prompts.
    pub unit: UnitKind,
    /// The structural contract of one produced artifact.
    pub schema: SchemaKind,
    /// The repository's one core file.
    pub core: CoreKind,
}

/// How a filename maps to a kind.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Match {
    /// Exact filenames.
    #[serde(default)]
    pub basename: Vec<String>,
    /// Filename endings.
    #[serde(default)]
    pub suffix: Vec<String>,
    /// Names of the immediate parent directory, the way SOKF lets a directory
    /// carry a concept's kind.
    #[serde(default)]
    pub dir: Vec<String>,
    /// Filenames this kind does not govern even where the rules above claim
    /// them — a directory index sits beside the concepts it lists.
    #[serde(default)]
    pub except: Vec<String>,
    /// The kind claimed when nothing else matches; exactly one carries it.
    #[serde(default)]
    pub default: bool,
}

/// A unit file: a skill or a prompt, written in the element vocabulary.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitKind {
    /// What a unit file is.
    pub doc: String,
    /// Which files are units.
    #[serde(rename = "match")]
    pub matches: Match,
    /// The host fields a unit carries.
    pub frontmatter: Frontmatter,
    /// The element every other one nests inside.
    pub root: String,
    /// The order the root's children must appear in.
    pub order: Vec<String>,
    /// Elements the grammar once had, kept so their return is reported as a
    /// migration rather than as an unknown tag.
    #[serde(default)]
    pub removed: Ordered<String>,
    /// The closed element vocabulary, in written order.
    pub elements: Ordered<Element>,
    /// Checks that are not part of the element vocabulary.
    pub checks: Checks,
}

/// A unit's YAML frontmatter: the host's fields, and which of them each kind
/// of unit file must carry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frontmatter {
    /// What the frontmatter is for and whose fields these are.
    pub doc: String,
    /// Everything a host reads as a boolean, written as it may appear.
    #[serde(rename = "booleanValues")]
    pub boolean_values: Vec<serde_yaml_ng::Value>,
    /// Every key the host defines, in written order.
    pub keys: Ordered<FrontmatterKey>,
    /// The shapes of unit file, each with its own required set.
    pub profiles: Vec<Profile>,
    /// The portable spec, and what to say about a key outside it.
    pub portability: Portability,
    /// Frontmatter keys that must equal an element's attribute.
    #[serde(default)]
    pub mirrors: Vec<Mirror>,
}

/// One frontmatter key the host defines.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontmatterKey {
    /// Optional: four of the nineteen keys carry none. The reference's
    /// meta-schema did not require it either, and the doc renderer does not
    /// print it, so the gap is invisible until someone reads the grammar.
    #[serde(default)]
    pub doc: Option<String>,
    /// `string`, `boolean`, or `list`.
    pub r#type: String,
    /// An anchored regex the value must match.
    #[serde(default)]
    pub pattern: Option<String>,
    /// The longest value the host will read.
    #[serde(default, rename = "maxLength")]
    pub max_length: Option<usize>,
    /// The closed set of values, when there is one.
    #[serde(default)]
    pub r#enum: Vec<serde_yaml_ng::Value>,
    /// Whether the portable spec carries this key, or only the host does.
    #[serde(default)]
    pub portable: bool,
}

/// One shape of unit file, and what its frontmatter must carry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// How the profile is named in a finding.
    pub name: String,
    /// Which host reads this shape, and where the file lives.
    pub doc: String,
    /// Which files take this profile; absent on the default.
    #[serde(default, rename = "match")]
    pub matches: Option<Match>,
    /// Keys the profile's host will not do without.
    pub required: Vec<String>,
    /// When present, the only keys this profile's host reads.
    #[serde(default)]
    pub allow: Vec<String>,
    /// Whether the frontmatter `name` must equal the containing directory,
    /// which for a skill is what the command is called.
    #[serde(default, rename = "nameMatchesDirectory")]
    pub name_matches_directory: bool,
    /// The profile used when no other matches; exactly one carries it.
    #[serde(default)]
    pub default: bool,
}

/// The portable subset, and what to say about a key outside it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Portability {
    /// The portable spec's name.
    pub spec: String,
    /// Where the spec lives.
    pub url: String,
    /// What a finding says about a key outside it.
    pub warn: String,
}

/// A frontmatter key that must equal an element's attribute.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mirror {
    /// The frontmatter key.
    pub key: String,
    /// The element carrying the attribute it must equal.
    pub element: String,
    /// The attribute.
    pub attr: String,
    /// Only checked for files with this basename.
    pub basename: String,
}

/// One element of the unit vocabulary.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Element {
    /// What the element is for.
    pub doc: String,
    /// `block`, `self-closing`, or `either`.
    pub form: String,
    /// The element it nests inside: absent for the root, one name, or several.
    #[serde(default)]
    pub parent: Parent,
    /// Its attributes, in written order.
    #[serde(default)]
    pub attrs: Ordered<Attr>,
    /// How many times it may appear.
    #[serde(default)]
    pub occurs: Option<Occurs>,
    /// Attributes or `body`, exactly one of which must be present.
    #[serde(default, rename = "exactlyOneOf")]
    pub exactly_one_of: Vec<String>,
    /// Attributes or `body`, at most one of which may be present.
    #[serde(default, rename = "atMostOneOf")]
    pub at_most_one_of: Vec<String>,
    /// Children it must have at least one of.
    #[serde(default, rename = "mustContain")]
    pub must_contain: Option<MustContain>,
    /// Whether it must carry a body.
    #[serde(default, rename = "bodyRequired")]
    pub body_required: Option<bool>,
    /// A body pattern it refuses.
    #[serde(default, rename = "bodyForbid")]
    pub body_forbid: Option<BodyForbid>,
    /// What this element contributes to the duplication check.
    #[serde(default)]
    pub compare: Option<Compare>,
}

/// An element's parent: the file root, one element, or any of several.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
pub enum Parent {
    /// No parent: this is the file's root element.
    #[default]
    Root,
    /// One element.
    One(String),
    /// Any of several.
    Any(Vec<String>),
}

impl Parent {
    /// The names this parent allows; empty means the file root.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        match self {
            Self::Root => Vec::new(),
            Self::One(name) => vec![name.as_str()],
            Self::Any(names) => names.iter().map(String::as_str).collect(),
        }
    }
}

/// One attribute an element may carry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attr {
    /// Optional, as above: three attributes carry none.
    #[serde(default)]
    pub doc: Option<String>,
    /// Whether the element must carry it.
    #[serde(default)]
    pub required: bool,
    /// The attribute takes the condition vocabulary rather than free text.
    #[serde(default)]
    pub condition: bool,
    /// The one value it may take.
    #[serde(default)]
    pub r#const: Option<String>,
    /// The closed set of values it may take.
    #[serde(default)]
    pub r#enum: Vec<String>,
}

/// How many times an element may appear under its parent.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Occurs {
    /// Fewest allowed.
    pub min: usize,
    /// Most allowed.
    pub max: usize,
}

/// Children an element must have at least one of.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MustContain {
    /// Element names; one of them suffices.
    #[serde(rename = "anyOf")]
    pub any_of: Vec<String>,
}

/// A body pattern an element refuses, with the reason.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyForbid {
    /// The regex the body must not match.
    pub pattern: String,
    /// Regex flags, `i` for case-insensitive.
    #[serde(default)]
    pub flags: Option<String>,
    /// What the finding says.
    pub message: String,
}

/// What an element contributes to the duplication check, and how it is named
/// when a pair is reported.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compare {
    /// Attributes and `body`, joined to make the comparable.
    pub parts: Vec<String>,
    /// How the comparable is named when a pair is reported.
    pub label: String,
    /// When this matches, the element contributes no comparable.
    #[serde(default, rename = "skipIf")]
    pub skip_if: Option<SkipIf>,
}

/// A comparable this element does not contribute after all.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkipIf {
    /// The part to test.
    pub part: String,
    /// When it matches, the element contributes nothing.
    pub pattern: String,
}

/// The unit checks that are not part of the element vocabulary.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checks {
    /// An attribute value carrying a tag is a mis-nesting.
    #[serde(rename = "attributes-free-of-angle-brackets")]
    pub attributes_free_of_angle_brackets: SimpleCheck,
    /// A step that only reads belongs in bootstrap.
    #[serde(rename = "steps-are-not-pure-loads")]
    pub steps_are_not_pure_loads: PureLoadCheck,
    /// A named core block must exist.
    #[serde(rename = "core-block-references")]
    pub core_block_references: RefCheck,
}

/// A check with nothing to configure but its message.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimpleCheck {
    /// Whether the check runs.
    pub enabled: bool,
    /// What the finding says.
    pub message: String,
}

/// A step whose task begins with one of these verbs is a load, not a step.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PureLoadCheck {
    /// Whether the check runs.
    pub enabled: bool,
    /// Opening words that make a step a load.
    pub verbs: Vec<String>,
    /// What the finding says.
    pub message: String,
}

/// A reference to a core block the core must define.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefCheck {
    /// Whether the check runs.
    pub enabled: bool,
    /// Kinds the check reads; units only by default.
    #[serde(default, rename = "appliesTo")]
    pub applies_to: Vec<String>,
    /// The regex that finds a reference and captures the block name.
    pub pattern: String,
    /// What the finding says.
    pub message: String,
}

/// A schema file: an SOKF concept wrapping one artifact's contract.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaKind {
    /// What a schema file is.
    pub doc: String,
    /// Which files are schemas.
    #[serde(rename = "match")]
    pub matches: Match,
    /// What a schema's own frontmatter must carry.
    pub frontmatter: SchemaFrontmatter,
    /// The vocabulary of the contract inside it.
    pub document: DocumentVocab,
    /// Which of its text the duplication check reads.
    pub compare: SchemaCompare,
}

/// What a schema's own frontmatter must carry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaFrontmatter {
    /// Keys every schema must carry.
    pub required: Vec<String>,
    /// Keys whose value must be a slug.
    pub slug: Vec<String>,
}

/// Which of a schema's own text the duplication check reads.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaCompare {
    /// The contract key whose prose is compared.
    #[serde(rename = "descriptionKey")]
    pub description_key: String,
    /// Reading stops here: the example is illustration, not a statement.
    #[serde(rename = "stopAtKey")]
    pub stop_at_key: String,
}

/// The vocabulary of the fenced YAML contract inside a schema file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentVocab {
    /// What the contract is, and what it is to a governed document.
    pub doc: String,
    /// How many yaml fences a schema carries.
    pub fences: usize,
    /// The exact opening marker: four backticks, so the example can hold three.
    #[serde(rename = "fence-marker")]
    pub fence_marker: String,
    /// Which regex dialect every pattern is written in.
    #[serde(rename = "pattern-dialect")]
    pub pattern_dialect: String,
    /// The contract's top-level keys, in written order.
    pub keys: Ordered<KeyDef>,
    /// What a section may say.
    pub section: KeyTable,
    /// What the text before the first heading may say.
    pub preamble: KeyTable,
    /// What a schema may say about one frontmatter key.
    #[serde(rename = "frontmatter-constraint")]
    pub frontmatter_constraint: KeyTable,
}

/// A named table of keys, with whatever rule spans them.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyTable {
    /// What this table describes.
    pub doc: String,
    /// Keys exactly one of which must be present.
    #[serde(default, rename = "exactlyOneOf")]
    pub exactly_one_of: Vec<String>,
    /// The keys, in written order.
    pub keys: Ordered<KeyDef>,
}

/// One key a governed document's contract may carry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyDef {
    /// What the key means.
    pub doc: String,
    /// `string`, `integer`, `boolean`, `list`, or `map`.
    pub r#type: String,
    /// Whether the contract must carry it.
    #[serde(default)]
    pub required: bool,
    /// The closed set of values, when there is one.
    #[serde(default)]
    pub r#enum: Vec<serde_yaml_ng::Value>,
    /// `regex` when the value is compiled.
    #[serde(default)]
    pub format: Option<String>,
    /// The entry type of a list or map key.
    #[serde(default)]
    pub of: Option<String>,
    /// Another key this one is only allowed beside.
    #[serde(default)]
    pub requires: Option<Ordered<Requirement>>,
}

/// What a `requires` entry admits: one value, or any of a set.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Requirement {
    /// The required key must carry this value.
    One(String),
    /// The required key must carry one of these.
    Any(Vec<String>),
}

impl Requirement {
    /// Does `value` satisfy the requirement?
    #[must_use]
    pub fn admits(&self, value: Option<&str>) -> bool {
        match self {
            Self::One(want) => value == Some(want.as_str()),
            Self::Any(wants) => value.is_some_and(|v| wants.iter().any(|w| w == v)),
        }
    }

    /// The requirement as a finding names it.
    #[must_use]
    pub fn spell(&self) -> String {
        match self {
            Self::One(want) => want.clone(),
            Self::Any(wants) => wants.join(" or "),
        }
    }
}

/// The core file: one per repository, and the only place blocks are defined.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreKind {
    /// What the core file is.
    pub doc: String,
    /// Which file is the core.
    #[serde(rename = "match")]
    pub matches: Match,
    /// Whether it must open with a level-1 heading.
    #[serde(rename = "requireH1")]
    pub require_h1: bool,
    /// Whether every block tag must close.
    #[serde(rename = "balancedTags")]
    pub balanced_tags: bool,
    /// Whether its block names are collected for the reference check.
    #[serde(rename = "collectBlocks")]
    pub collect_blocks: bool,
}

/// One home per statement: the cross-file duplication check.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Duplication {
    /// What duplication means here and what a flagged pair asks for.
    pub doc: String,
    /// Comparables shorter than this are not compared.
    #[serde(rename = "minTokens")]
    pub min_tokens: usize,
    /// Containment at or above this is a finding.
    pub threshold: f64,
    /// Kinds whose comparables are compared against others in the same file.
    #[serde(rename = "withinFileKinds")]
    pub within_file_kinds: Vec<String>,
    /// `a|b` pairs of kinds compared across files.
    #[serde(rename = "crossPairs")]
    pub cross_pairs: Vec<String>,
    /// Elements exempt from cross-unit comparison.
    #[serde(rename = "exemptCrossUnitElements")]
    pub exempt_cross_unit_elements: Vec<String>,
    /// Why they are exempt.
    #[serde(rename = "exemptCrossUnitReason")]
    pub exempt_cross_unit_reason: String,
    /// Skeleton text every unit carries, which is not duplication.
    #[serde(rename = "skeletonConstants")]
    pub skeleton_constants: Vec<String>,
    /// Words carrying no signal, dropped before comparing.
    #[serde(rename = "stopWords")]
    pub stop_words: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The grammar as it ships, read from the repository this crate lives in.
    fn live() -> String {
        let path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "../../..",
            ".agents/sokf/grammar.yaml",
        ]
        .iter()
        .collect();
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
    }

    /// The types accept the real grammar. This is what retires the JSON
    /// meta-schema: everything it asserted about shape, `deny_unknown_fields`
    /// and the required fields assert here, against the file that ships.
    #[test]
    fn the_live_grammar_deserialises() {
        let g: Grammar = serde_yaml_ng::from_str(&live()).expect("the shipped grammar must parse");
        assert_eq!(g.grammar, "superdev");
        assert!(!g.roots.paths.is_empty());
        assert!(g.kinds.unit.elements.has("skill"));
        assert!(g.kinds.unit.elements.has("loop"));
    }

    /// Mappings keep grammar order. Findings list the declared names, and the
    /// reference lists them as written, so a sorted map would reword them.
    #[test]
    fn mappings_keep_the_order_they_were_written_in() {
        let text = live();
        let g: Grammar = serde_yaml_ng::from_str(&text).unwrap();
        let names: Vec<String> = g.kinds.unit.elements.names().map(str::to_string).collect();
        assert_eq!(
            names.first().map(String::as_str),
            Some("skill"),
            "the root element comes first"
        );

        // The order the file writes them in, read straight from the YAML.
        let raw: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).unwrap();
        let written: Vec<String> = raw["kinds"]["unit"]["elements"]
            .as_mapping()
            .unwrap()
            .keys()
            .map(|k| k.as_str().unwrap().to_string())
            .collect();
        assert_eq!(names, written);
    }

    /// A key the grammar does not declare fails, naming itself. This is the
    /// half of the meta-schema that was a closed-vocabulary check.
    #[test]
    fn an_undeclared_key_is_refused_by_name() {
        let mut yaml = live();
        yaml.push_str("\ninvented-key: nonsense\n");
        let err = serde_yaml_ng::from_str::<Grammar>(&yaml)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("invented-key"),
            "the error must name the key: {err}"
        );
    }

    /// A missing required key fails too, rather than defaulting quietly.
    #[test]
    fn a_missing_required_key_is_refused() {
        let yaml = live().replace("\nversion:", "\nnot-version:");
        let err = serde_yaml_ng::from_str::<Grammar>(&yaml)
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("version"),
            "the error must name the key: {err}"
        );
    }

    /// A `requires` entry admits one value or any of a set, and admits
    /// nothing when the key it names is absent — the guard `check_keys` runs
    /// on every schema, so a mistake here would silence a whole rule.
    #[test]
    fn a_requirement_admits_its_value_and_nothing_else() {
        let one: Requirement = serde_yaml_ng::from_str("table").unwrap();
        assert!(one.admits(Some("table")));
        assert!(!one.admits(Some("code")));
        assert!(!one.admits(None));
        assert_eq!(one.spell(), "table");

        let any: Requirement = serde_yaml_ng::from_str("[bullet-list, numbered-list]").unwrap();
        assert!(any.admits(Some("bullet-list")));
        assert!(any.admits(Some("numbered-list")));
        assert!(!any.admits(Some("prose")));
        assert!(!any.admits(None));
        assert_eq!(any.spell(), "bullet-list or numbered-list");
    }

    /// The languages the grammar admits for a definition block are the ones
    /// the document check can parse. They live in two files, so a third
    /// added to one and not the other would accept a block nothing reads.
    #[test]
    fn the_grammars_block_languages_are_the_ones_the_validator_reads() {
        let g: Grammar = serde_yaml_ng::from_str(&live()).expect("the grammar parses");
        let declared = g
            .kinds
            .schema
            .document
            .section
            .keys
            .get("block-language")
            .expect("the grammar declares block-language");
        let mut from_grammar: Vec<String> = declared
            .r#enum
            .iter()
            .filter_map(|v| v.as_str().map(ToString::to_string))
            .collect();
        from_grammar.sort();
        let mut from_code: Vec<String> = crate::validate::schema::document::BLOCK_LANGUAGES
            .iter()
            .map(ToString::to_string)
            .collect();
        from_code.sort();
        assert_eq!(
            from_grammar, from_code,
            "the grammar admits languages the document check does not read"
        );
    }
}

#[cfg(test)]
mod yaml_dialect {
    /// `yes` / `no` / `on` / `off` are strings in YAML 1.2 and booleans in 1.1.
    /// The Node reference reads the grammar with a 1.2 parser, so the port has
    /// to agree or `booleanValues` would differ and every boolean frontmatter
    /// key would be checked against the wrong set.
    #[test]
    fn unquoted_yes_and_no_are_strings_as_they_are_for_the_reference() {
        let v: serde_yaml_ng::Value =
            serde_yaml_ng::from_str("- \"true\"\n- yes\n- no\n- on\n- off\n- 1\n").unwrap();
        let kinds: Vec<&str> = v
            .as_sequence()
            .unwrap()
            .iter()
            .map(|x| {
                if x.is_string() {
                    "string"
                } else if x.is_bool() {
                    "bool"
                } else {
                    "other"
                }
            })
            .collect();
        assert_eq!(
            kinds,
            ["string", "string", "string", "string", "string", "other"]
        );
    }
}
