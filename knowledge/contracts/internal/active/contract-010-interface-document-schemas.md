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
    to: adr-035-a-schema-declares-its-definition-blocks-contract
    note: Fixes the three definition-block declarations and what a missing key reports.
---

# Interface contract: document schemas

The vocabulary the schemas in `knowledge/schemas/` write and the
validator's document check reads. Schema authors — agents, in this
repository and every managed one — are one side of the interface; the
`validate::schema` module is the other. Every declaration listed here
binds: the validator checks it, and a declaration the validator cannot
read is reported on the schema itself. The decisions behind the
vocabulary's newest rows are
[ADR-022][sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag],
[ADR-023][sokf:adr-023-a-content-kind-binds-by-presence],
[ADR-024][sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema],
[ADR-025][sokf:adr-025-an-examples-links-bind-by-form-and-never-resolve],
[ADR-030][sokf:adr-030-a-section-rule-declares-body-patterns] and
[ADR-035][sokf:adr-035-a-schema-declares-its-definition-blocks-contract].

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
    pattern: '<re>'          # a present value must match it
    enum: [<v1>, <v2>]       # a present value must be one of them
    description: <guidance>  # prose, unchecked

sections-ordered: true       # first appearances follow declaration order
sections-prohibited: [<heading>]
sections:
  - heading: '<literal>'     # or heading-pattern: '<re>'
    level: <int>             # omitted: any depth
    required: true
    repeatable: true
    content: <kind>          # prose | bullet-list | numbered-list | table | code
    columns: [<c1>, <c2>]    # a declared table carries exactly these
    item-pattern: '<re>'     # each top-level item of the list kind must match
    content-pattern: '<re>'  # the section's body must match
    block-language: <tag>    # the fence tag the section's block carries
    block-keys: [<k1>]       # keys the block carries at its top level
    block-entry-keys: [<k1>] # keys every top-level entry of the block carries
    description: <guidance>  # prose, unchecked

example: |                   # one conforming document; checked (ADR-024)
  <a complete document satisfying this schema>
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
- **Body patterns** — `item-pattern` binds each top-level item of the
  section's declared list kind, `content-pattern` the section's whole
  body. Every pattern in this vocabulary is a regex matched
  found-anywhere; authors write `^` and `$` explicitly, and neither
  pattern reads a fenced block (ADR-030). An item-pattern finding names
  the file, the section and the item's first line; a content-pattern
  finding names the file and the section. Both name the failing
  occurrence's own heading, so a repeatable rule is locatable.
- **What an item is** — the list's top level is the shallowest marker in
  the body, so a list indented under its heading still binds. An item
  takes the following lines indented past that, blank lines included, and
  an unindented line while its paragraph is still running. A nested item
  is its own: its lines are dropped, and the item above it resumes at the
  first line no deeper than the nested marker. A marker of the other list
  kind opens no item, and a thematic break is not a marker.
- **The definition block** — `block-language` names the fence tag the
  section's block must carry; `block-keys` the keys it must carry at
  its top level; `block-entry-keys` the keys every top-level entry must
  carry (ADR-035). The validator parses the block in the declared
  language — YAML and JSON, the two the binary reads — and a missing
  key is an error naming the file, the section, the entry and the key.
  A block that does not parse is an error naming the parse failure. A
  block in a language the binary does not read declares no
  `block-language`; a drift test binds its completeness instead
  (ADR-036).
- **A mis-declared schema is its own finding** — a `content` outside
  the five kinds, a `pattern` that does not compile, an `item-pattern`
  on a section whose `content` is not a list kind, a `block-language`
  the validator cannot parse, or a block declaration on a section whose
  `content` is not `code`: reported against the schema file, and the
  unreadable rule binds nothing.
- **The example is checked in place** — the `example:` block is read as
  a document and run through this same check with the declaring schema
  handed to it, no dispatch; every failure, including an example that
  does not parse as a document, is a finding on the schema file
  (ADR-024).
- **An example's links bind by form, never by destination** — a concept
  link in an example takes the `[text][sokf:<id>]` form and a path link
  into the knowledge is an error, but no id or target is resolved: a
  fictional `sokf:` label passes, and a link outside the knowledge — a
  URL, a repository path — keeps its ordinary markdown form (ADR-025).
  This is the one place the link rules differ from a real document's,
  where ids must resolve.

## Module boundaries

- `validate::schema` owns parsing (`DocSchema::parse`) and checking
  (`check_documents`); schemas are data it reads, never code.
- Schema files own the declarations; a document-structure declaration
  MUST NOT live outside `knowledge/schemas/` and its pack mirror.
- The grammar (`.agents/sokf/grammar.yaml`) governs the schema files'
  own markdown shape; this contract governs what their YAML declares.

## Key flows

- validate: collect documents → dispatch each by `type` or glob →
  sections (presence, order, prohibition, columns, line limit) →
  content kinds → body patterns → definition blocks → frontmatter
  contract → findings grouped per file, one verdict.
- example check: parse each schema's `example:` block as a document →
  run the document check with the declaring schema → check link form →
  findings land on the schema file, in the same run and verdict.

## Cross-cutting concerns

- Security: every schema pattern MUST compile through the validator's own
  `re` wrapper; one that does not compile MUST be a schema finding, never a
  panic or a silently-passing rule.
- Performance: every check is one pass over the document's lines; the
  schema set parses once per run.
- Migration/rollout: the new declarations are additive — a schema
  without `required` marks or with its existing `content` lines keeps
  its current meaning, so old packs stay readable; the live tree's
  reconciliation lands with the feature (I018).
- Observability: every finding names the document, the rule and the
  schema, in the shape the section findings already use.

<!-- sokf:links -->
[sokf:adr-022-a-frontmatter-key-is-required-by-a-per-key-flag]: /knowledge/adrs/active/adr-022-a-frontmatter-key-is-required-by-a-per-key-flag.md
[sokf:adr-023-a-content-kind-binds-by-presence]: /knowledge/adrs/active/adr-023-a-content-kind-binds-by-presence.md
[sokf:adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema]: /knowledge/adrs/active/adr-024-a-schemas-example-is-checked-in-place-against-its-own-schema.md
[sokf:adr-025-an-examples-links-bind-by-form-and-never-resolve]: /knowledge/adrs/active/adr-025-an-examples-links-bind-by-form-and-never-resolve.md
[sokf:adr-030-a-section-rule-declares-body-patterns]: /knowledge/adrs/active/adr-030-a-section-rule-declares-body-patterns.md
[sokf:adr-035-a-schema-declares-its-definition-blocks-contract]: /knowledge/adrs/active/adr-035-a-schema-declares-its-definition-blocks-contract.md
