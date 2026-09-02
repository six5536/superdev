---
type: Decision
id: adr-043-one-contract-schema-and-twelve-kinds
title: One contract schema and twelve kinds
description: One schema governs every contract — a Definition of source includes, a Behaviour of normative prose, a Stability promise — and a contract's kind is one of twelve chosen by what a reader asks for, carried in the frontmatter and the id, with what each kind's Behaviour must cover stated in a checklist the agent reads rather than in sixteen section lists the validator enforced.
lifecycle: active
links:
  - rel: supersedes
    to: adr-032-contract-promise-sections-declare-their-shape
    note: The principle survives — promise sections declare that they bind — on the two sections one schema has, not sixteen per-kind assignments.
  - rel: supersedes
    to: adr-037-the-file-format-kind-splits-into-text-and-binary
    note: Text and binary formats are one reader's question, `format`; the encoding is in the definition, which is now the source.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: Once every definition is a source include, nothing distinguishes sixteen shapes but section names.
  - rel: references
    to: adr-027-an-include-block-materializes-shared-content-in-place
    note: Its first consumer — the contract style included into sixteen schemas — becomes plain prose in one; the source include is its next.
---

# ADR-043: One contract schema and twelve kinds

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

Sixteen schemas governed contracts, one per kind, and every one
required the same three things under different names: a definition
section, one or more behaviour sections, and a stability section. The
per-kind section lists were a checklist — a REST contract must state
its Authentication — and an inconsistent one: `rest` required it and
`mcp` did not, though an MCP server can have auth. Four kinds were one
reader's question, "the API", split by the file type their definition
copy took;
[ADR-037][sokf:adr-037-the-file-format-kind-splits-into-text-and-binary]
split a fifth by encoding. Four defined in tables that were hand copies
in table form: roles live in a policy file, metrics in
`register_counter!` calls.

[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
makes every definition a source include. After it, nothing
distinguishes sixteen shapes but the names of their sections, and
[ADR-032][sokf:adr-032-contract-promise-sections-declare-their-shape]'s
per-kind assignment of promise patterns has sixteen targets for what is
two sections. Under one schema a kind does almost nothing in
validation: it is the id's segment, the title's prefix, and the key
into a checklist. So the right basis for cutting kinds is what a reader
asks for, and the check on each cut is whether the Behaviour checklist
genuinely differs.

## Decision

We will govern every contract by one schema, `Contract`. It requires a
title opening with the kind's display name and "contract:", a
Definition section declaring `content: include`, a Behaviour section of
normative prose under the RFC 2119 pattern with `###` subsections free,
and a Stability section under the same pattern. A `kind` in the
frontmatter is one of twelve, and the id's third segment must equal
it: `api`, `events`, `cli`, `library`, `interface`, `ui`, `data`,
`format`, `config`, `telemetry`, `authz`, `deployment`. The display
names the title opens with are `API`, `Events`, `CLI`, `Library`,
`Interface`, `UI`, `Data`, `Format`, `Config`, `Telemetry`, `Authz`
and `Deployment` — so `# CLI contract: superdev`, `# API contract:
sokf over MCP`.

`api` absorbs `rest`, `graphql`, `rpc` and `mcp`: one question, four
file types, the type in the definition and the title. `format` absorbs
`binary-format` and `text-format`. `events` stays beside `api` for its
ordering and delivery semantics. `interface` stays beside `library`
because "how is this module bounded" is not "what can I import", and
separating them costs nothing under one schema. Environment variables
sit in `config`, the environment being one of its sources.

What the sixteen required section by section becomes section rules
in the one schema, each tagged with the kinds it applies to
([ADR-045][sokf:adr-045-a-schema-declares-variants]): `### Exit codes`
required for `cli`, `### Errors` required for `api` and `library`,
`### Prompting` optional for `cli`. The schema is the one read every
writer is guaranteed to make, so the kind's sections reach the writer
with the shape, and the validator enforces the required ones. A rule's
`description` carries what the section must say. A section is required
only where every contract of the kind has it; the rest are declared
optional, and the judgement step at integration asks after those.
Shape is the validator's; what a section says is the agent's.

A surface the twelve do not name is one line in the enum and one
section in the checklist. The fifteen schemas the one replaces are
deleted with their pack mirrors, and the nine contracts on file take
their kind and regroup their sections.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| One schema, twelve kinds by reader's question, per-kind sections as tagged rules | One shape to write and maintain; kinds match how a system is asked about; the kind's sections reach every writer with the schema and the required ones are enforced | Adding a kind edits an owned schema and adds an example |
| Keep sixteen schemas, change only their definition sections | No migration of kinds | Sixteen edits for one change; the per-kind lists stay inconsistent; the file-type splits stay with their reason gone |
| Kinds by audience — caller, operator, developer, user | Four kinds | Too coarse: a caller asks for the API, the CLI and the events by name |
| Keep `rest`, `graphql`, `rpc`, `mcp` distinct | Familiar names | One reader's question split by file type, which the definition and title already carry |
| Merge `interface` into `library` | Eleven kinds | Different questions, and an internal boundary makes no stability promise to callers |
| An open `kind` | No enum to extend | Costs the id pattern and the checklist lookup for a case nothing on file has |
| The checklist as prose in the schema, unenforced | Nothing to build | A `cli` contract with no Exit codes passes; the sixteen schemas' one merit given up |
| The checklist as a separate concept the skills read | The schema stays short | A skill must be modified to fetch it, and a writer outside the skill never sees it — the failure the sixteen schemas were the answer to |

## Consequences

- Positive: one schema; kinds a reader recognises; contract style is
  prose in one file rather than
  [ADR-027][sokf:adr-027-an-include-block-materializes-shared-content-in-place]'s
  include into sixteen; adding a kind is one line and one checklist
  section.
- Negative: a required section bites, so the split between required
  and optional per kind is a decision the schema's author owns; a
  migration of nine contracts and the deletion of fifteen schemas and
  their mirrors.
- Follow-ups: the contract schema's per-kind sections as tagged rules
  once ADR-045's vocabulary lands, with its prose checklist as their
  source until then; the migration; the integrate skill's judgement
  step asks after the optional sections; the ADR index and the schemas
  index reflect one schema.

<!-- sokf:links -->
[sokf:adr-027-an-include-block-materializes-shared-content-in-place]: /knowledge/adrs/active/adr-027-an-include-block-materializes-shared-content-in-place.md
[sokf:adr-032-contract-promise-sections-declare-their-shape]: /knowledge/adrs/deprecated/adr-032-contract-promise-sections-declare-their-shape.md
[sokf:adr-037-the-file-format-kind-splits-into-text-and-binary]: /knowledge/adrs/deprecated/adr-037-the-file-format-kind-splits-into-text-and-binary.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
[sokf:adr-045-a-schema-declares-variants]: /knowledge/adrs/active/adr-045-a-schema-declares-variants.md
