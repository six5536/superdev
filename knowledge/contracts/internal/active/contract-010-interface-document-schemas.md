---
type: Contract
id: contract-010-interface-document-schemas
kind: interface
title: Interface contract for document schemas
description: The declaration vocabulary a document schema may carry — frontmatter constraints, section rules and content kinds — and what each declaration obliges the validator to check.
lifecycle: active
resource: /crates/lib/superdev-core/src/validate/schema/document.rs
links:
  - rel: references
    to: adr-022-a-frontmatter-key-is-required-by-a-per-key-flag
    note: Fixes the required-key half of the frontmatter vocabulary.
  - rel: references
    to: adr-023-a-content-kind-binds-by-presence
    note: Fixes what a content kind demands of a section's body.
  - rel: references
    to: adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema
    note: Fixes where and how the example check runs.
  - rel: references
    to: adr-025-an-examples-links-bind-by-form-and-never-resolve
    note: Fixes the example check's reach over body links.
  - rel: references
    to: adr-030-a-section-rule-declares-body-patterns
    note: Fixes the two body-pattern declarations and their found-anywhere semantics.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: Fixes the sixth content kind, `include`, withdraws the three definition-block declarations, and makes the structs that read a schema the declaration this contract includes.
  - rel: references
    to: adr-045-a-schema-declares-variants
    note: Fixes `variant-key`, the `variants` tag on any rule, and the keyed `example`.
  - rel: references
    to: adr-047-a-section-rule-declares-item-keys-and-item-bounds
    note: Fixes `item-key`, `item-only-pattern` and `item-prohibited-pattern`, the three item declarations.
---

# Interface contract: document schemas

The vocabulary the schemas in `knowledge/schemas/` write and the
validator's document check reads. Schema authors — agents, in this
repository and every managed one — are one side of the interface; the
`validate::schema` module is the other. The Definition is the vocabulary
as the structs that read a schema's YAML block declare it: a key a
schema may write is a field with the doc comment that says what it
obliges, and a key with no field is one the validator cannot read.
Every declaration binds: the validator checks it, and a declaration the
validator cannot read is reported on the schema itself. The decisions
behind the vocabulary's newest rows are
[ADR-022][sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag],
[ADR-023][sokf:adr-023-a-content-kind-binds-by-presence],
[ADR-024][sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema],
[ADR-025][sokf:adr-025-an-examples-links-bind-by-form-and-never-resolve],
[ADR-030][sokf:adr-030-a-section-rule-declares-body-patterns],
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source],
[ADR-045][sokf:adr-045-a-schema-declares-variants] and
[ADR-047][sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds].

## Definition

<!-- sokf:include /crates/lib/superdev-core/src/validate/schema/document.rs#document-schemas -->
```rust
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
    /// The variant values this rule applies to; empty applies to every
    /// variant (ADR-045).
    #[serde(default)]
    pub variants: Vec<String>,
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
```
<!-- /sokf:include -->

## Behaviour

### Declarations

What each declaration in the Definition obliges the validator to check.
A schema writes the vocabulary in YAML — `heading-pattern` for
`heading_pattern`, `variant-key` for `variant_key` — under the
`serde` renames the Definition shows; `description` beside any key is
prose.

- `P_description-unchecked` [ubiquitous] The validator SHALL NOT check
  the `description` beside a key.

**Dispatch** — a document's frontmatter `type` names the schema whose
`frontmatter.type.const` equals it; `target-files` globs catch the
frontmatter-less.

- `P_unknown-type` [event] WHEN a document's `type` names no schema,
  the validator SHALL report an error.

**Frontmatter** — `const`, `pattern` and `enum` bind a key's present
value; `required` makes absence an error; a key declared with only a
`description` is guidance (ADR-022). A constraint compares against the
value's scalar string form, so a value with no scalar form — a list, a
map, a folded block — cannot satisfy one. `lifecycle` belongs to the
filing check (P011), which reports its value against the enum and its
folder, so one fault is said once.

**Content kinds** — the closed set `CONTENT_KINDS` names. A section
satisfies its kind when the form appears in its body: one bullet, one
numbered item, one table, one fenced block, one include block naming a
source path, or — for prose — one plain paragraph line; other content
beside the form is tolerated (ADR-023), with one exception for the
`include` kind. The body runs to the next heading at the section's own
level or shallower, so a subsection's content counts; lines inside
fenced blocks are not content.

- `P_include-carries-no-authored-block` [event] WHEN an `include`
  section carries a fenced block outside an include, the validator
  SHALL report an error naming the section (ADR-042).

**Body patterns** — `item-pattern` binds each top-level item of the
section's declared list kind, `content-pattern` the section's whole
body. Every pattern in this vocabulary is a regex matched
found-anywhere; authors write `^` and `$` explicitly, and neither
pattern reads a fenced block (ADR-030). An item-pattern finding names
the file, the section and the item's first line; a content-pattern
finding names the file and the section. Both name the failing
occurrence's own heading, so a repeatable rule is locatable.

**Item keys and bounds** — `item-key` is a regex with one capture
group whose capture is the item's key. `item-only-pattern` is a regex
that matches only inside a top-level item; on a section whose `content`
is not a list kind every body line is outside. `item-prohibited-pattern`
is a regex no top-level item matches. All three read an item as
`item-pattern` does and skip fenced blocks (ADR-047).

- `P_item-key-binds` [ubiquitous] `item-key` SHALL bind every top-level
  item of the section's declared list kind, the capture being the
  item's key.
- `P_item-key-unmatched` [event] WHEN an item has no `item-key` match,
  the validator SHALL report an error naming the section and the item.
- `P_item-key-repeated` [event] WHEN a key repeats across the document's
  items under rules declaring `item-key`, the validator SHALL report an
  error naming the key and both items.
- `P_item-only-outside` [event] WHEN `item-only-pattern` matches a body
  line outside every item — prose, a table row, an item of the other
  list kind — the validator SHALL report an error naming the section
  and the line.
- `P_item-prohibited-matched` [event] WHEN a top-level item matches
  `item-prohibited-pattern`, the validator SHALL report an error naming
  the item and the matched text.

**What an item is** — the list's top level is the shallowest marker in
the body, so a list indented under its heading still binds. An item
takes the following lines indented past that, blank lines included, and
an unindented line while its paragraph is still running. A nested item
is its own: its lines are dropped, and the item above it resumes at the
first line no deeper than the nested marker. A marker of the other list
kind opens no item, and a thematic break is not a marker.

**Variants** — `variant-key` names the frontmatter key whose value
selects a variant; any rule carrying `variants` applies only to the
values it lists, and a rule without it to all (ADR-045). A document
whose value is absent, or outside the key's enum, sees the untagged
rules alone, and the frontmatter check reports the value. With
`variant-key` set, `example` is a map keyed by value, every enum value
present, each checked against the base and its own variant's rules, its
value equal to its key; its findings carry the key, `example
`<value>`:`.

- `P_variant-selects-rules` [ubiquitous] The validator SHALL check a
  document against the rules its variant value selects, in declared
  order (ADR-045).

**The definition is not parsed** — the `include` kind asks only that
an include block naming a source path is present; what the block
carries is the SOKF validator's to keep current (ADR-041) and no
concern of this check. The former `block-language`, `block-keys` and
`block-entry-keys` declarations are withdrawn (ADR-042); a schema
still carrying one is mis-declared.

**A mis-declared schema is its own finding** — the unreadable rule
binds nothing.

- `P_misdeclared-schema` [event] WHEN a schema carries a `content`
  outside `CONTENT_KINDS`, a `pattern` that does not compile, an
  `item-pattern` on a section whose `content` is not a list kind, a
  withdrawn `block-*` declaration, a `variants` tag naming a value
  outside the discriminator's enum, a tag with no `variant-key`, a
  `variant-key` naming a frontmatter key with no enum, or an `example`
  of the wrong shape — one document under a `variant-key`, a map
  without one, a variant with no example, a key the enum does not
  carry — the validator SHALL report it against the schema file.
- `P_misdeclared-item-key` [event] WHEN an `item-key` sits on a section
  whose `content` is not a list kind, or carries a capture count other
  than one, the validator SHALL report it against the schema file
  (ADR-047).
- `P_misdeclared-item-prohibited` [event] WHEN an
  `item-prohibited-pattern` sits on a section whose `content` is not a
  list kind, the validator SHALL report it against the schema file
  (ADR-047).

**The example is checked in place** — the `example:` block is read as
a document and run through this same check with the declaring schema
handed to it, no dispatch; every failure, including an example that
does not parse as a document, is a finding on the schema file
(ADR-024).

**An example's links bind by form, never by destination** — a concept
link in an example takes the `[text][sokf:<id>]` form and a path link
into the knowledge is an error, but no id or target is resolved: a
fictional `sokf:` label passes, and a link outside the knowledge — a
URL, a repository path — keeps its ordinary markdown form (ADR-025).
This is the one place the link rules differ from a real document's,
where ids resolve.

### Module boundaries

`validate::schema` owns parsing (`DocSchema::parse`) and checking
(`check_documents`); schemas are data it reads, never code. Schema
files own the declarations. The grammar (`.agents/sokf/grammar.yaml`)
governs the schema files' own markdown shape; this contract governs
what their YAML declares.

- `P_declarations-live-in-schemas` [ubiquitous] A document-structure
  declaration SHALL NOT live outside `knowledge/schemas/` and its pack
  mirror.

### Key flows

1. validate: collect documents → dispatch each by `type` or glob →
   sections (presence, order, prohibition, columns, line limit) →
   content kinds → body patterns → frontmatter contract → findings
   grouped per file, one verdict.
2. example check: parse each schema's `example:` block as a document,
   one per variant where the schema declares them → run the document
   check with the declaring schema → check link form → findings land
   on the schema file, in the same run and verdict.

### Cross-cutting concerns

Security: a pattern is data a schema author wrote, compiled by the
validator alone.

- `P_pattern-compiles-through-wrapper` [ubiquitous] The validator SHALL
  compile every schema pattern through its own `re` wrapper.
- `P_uncompilable-pattern-is-finding` [event] WHEN a pattern does not
  compile, the validator SHALL report a schema finding, never a panic
  or a silently-passing rule.

Performance: every check is one pass over the document's lines; the
schema set parses once per run.

Migration/rollout: a schema without `required` marks or with its
existing `content` lines keeps its current meaning, so old packs stay
readable; the three `block-*` declarations are the one withdrawal, and
a pack still carrying them reports on its schemas rather than failing
to load (I049).

Observability: every finding names the document, the rule and the
schema, in the shape the section findings already use.

## Stability

Internal.

- `P_internal` [ubiquitous] Every item above MAY change with the crate.

<!-- sokf:links -->
[sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag]: /knowledge/adrs/active/adr-022-a-frontmatter-key-is-required-by-a-per-key-flag.md
[sokf:adr-023-a-content-kind-binds-by-presence]: /knowledge/adrs/active/adr-023-a-content-kind-binds-by-presence.md
[sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema]: /knowledge/adrs/active/adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema.md
[sokf:adr-025-an-examples-links-bind-by-form-and-never-resolve]: /knowledge/adrs/active/adr-025-an-examples-links-bind-by-form-and-never-resolve.md
[sokf:adr-030-a-section-rule-declares-body-patterns]: /knowledge/adrs/active/adr-030-a-section-rule-declares-body-patterns.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
[sokf:adr-047-a-section-rule-declares-item-keys-and-item-bounds]: /knowledge/adrs/active/adr-047-a-section-rule-declares-item-keys-and-item-bounds.md
