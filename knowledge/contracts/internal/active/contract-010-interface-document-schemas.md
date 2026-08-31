---
type: InterfaceContract
id: contract-010-interface-document-schemas
title: Document Schemas Interface
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
---

# Interface contract: document schemas

The vocabulary the 53 schemas in `knowledge/schemas/` write and the
validator's document check reads. Schema authors — agents, in this
repository and every managed one — are one side of the interface; the
`validate::schema` module is the other. Every declaration listed here
binds: the validator checks it, and a declaration the validator cannot
read is reported on the schema itself. The decisions behind the
vocabulary's newest rows are
[ADR-022][sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag]
and [ADR-023][sokf:adr-023-a-content-kind-binds-by-presence].

## Data model & API

The declaration vocabulary, as a schema's fenced YAML block writes it:

```yaml
description: <what the governed document is>   # prose, unchecked
line-limit: <int>            # error when the document exceeds it

target-files: '<glob>'       # dispatch for frontmatter-less documents

frontmatter:
  <key>:
    required: true           # error when the key is absent
    const: <value>           # a present value must equal it
    pattern: '<anchored re>' # a present value must match it
    enum: [<v1>, <v2>]       # a present value must be one of them
    description: <guidance>  # prose, unchecked

sections-ordered: true       # first appearances follow declaration order
sections-prohibited: [<heading>]
sections:
  - heading: '<literal>'     # or heading-pattern: '<anchored re>'
    level: <int>             # omitted: any depth
    required: true
    repeatable: true
    content: <kind>          # prose | bullet-list | numbered-list | table | code
    columns: [<c1>, <c2>]    # a declared table carries exactly these
    description: <guidance>  # prose, unchecked
```

```rust
/// One document against the schema its `type` names; every finding
/// carries the document, the rule broken, and the schema's name.
pub fn check_documents(docs: &[Document<'_>], set: &SchemaSet) -> Vec<Finding>;
```

- **Dispatch** — a document's frontmatter `type` names the schema whose
  `frontmatter.type.const` equals it; `target-files` globs catch the
  frontmatter-less. A `type` naming no schema is an error.
- **Frontmatter** — `const`, `pattern` and `enum` bind a key's present
  value; `required` makes absence an error; a key declared with only a
  `description` is guidance (ADR-022). A constraint compares against the
  value's scalar string form, so a value with no scalar form — a list, a
  map, a folded block — cannot satisfy one. `lifecycle` belongs to the
  filing check (P011), which reports its value against the enum and its
  folder, so one fault is said once.
- **Content kinds** — a closed set of five. A section satisfies its
  kind when the form appears in its body: one bullet, one numbered
  item, one table, one fenced block, or — for prose — one plain
  paragraph line; other content beside the form is tolerated
  (ADR-023). The body runs to the next heading at the section's own
  level or shallower, so a subsection's content counts; lines inside
  fenced blocks are not content.
- **A mis-declared schema is its own finding** — a `content` outside
  the five kinds, a `pattern` that does not compile: reported against
  the schema file, and the unreadable rule binds nothing.

## Module boundaries

- `validate::schema` owns parsing (`DocSchema::parse`) and checking
  (`check_documents`); schemas are data it reads, never code.
- Schema files own the declarations; nothing outside `knowledge/schemas/`
  (and its pack mirror) declares document structure.
- The grammar (`.agents/sokf/grammar.yaml`) governs the schema files'
  own markdown shape; this contract governs what their YAML declares.

## Key flows

- validate: collect documents → dispatch each by `type` or glob →
  sections (presence, order, prohibition, columns, line limit) →
  content kinds → frontmatter contract → findings grouped per file,
  one verdict.

## Cross-cutting concerns

- Security: schema `pattern` values compile through the validator's own
  `re` wrapper; one that does not compile is a schema finding, never a
  panic or a silently-passing rule.
- Performance: every check is one pass over the document's lines; the
  53-schema set parses once per run.
- Migration/rollout: the new declarations are additive — a schema
  without `required` marks or with its existing `content` lines keeps
  its current meaning, so old packs stay readable; the live tree's
  reconciliation lands with the feature (I018).
- Observability: every finding names the document, the rule and the
  schema, in the shape the section findings already use.

<!-- sokf:links -->
[sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag]: /knowledge/adrs/active/adr-022-a-frontmatter-key-is-required-by-a-per-key-flag.md
[sokf:adr-023-a-content-kind-binds-by-presence]: /knowledge/adrs/active/adr-023-a-content-kind-binds-by-presence.md
