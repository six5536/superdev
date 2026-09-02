---
type: FeaturePlan
id: plan-024-feature-a-contract-includes-its-definition
title: A contract includes its definition — feature plan
description: Slices delivering I049 — the source include, the sixth content kind, schema variants, the one contract schema, the skills' judgement and declaration steps, the migration of nine contracts, and the deletion of fifteen schemas and four copy-comparing tests.
lifecycle: done
links:
  - rel: implements
    to: issue-049-feature-request-a-contract-cannot-point-at-its-definition
    note: The framed feature whose twenty-four criteria these slices deliver.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Carries three PENDING promises; slices 2 and 3 close them, so they run before the slices that do not own them.
---

# Feature plan: A contract includes its definition

Request: [issue-049-feature-request-a-contract-cannot-point-at-its-definition][sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]

The mechanism first, because it is the riskiest and everything reads
through it. Then the two schema-layer changes that close
[contract-010][sokf:contract-010-interface-document-schemas]'s
PENDING promises, in the order their vocabulary is needed. Then the
one schema in its final form, the skills, and the nine migrations —
each deleting the test that compared the copy it replaces. The
fifteen schemas go last, because every contract must have left them
first.

Three trees are owned copies with a source elsewhere, and `sync`
overwrites the copy: `.agents/sokf/SPEC.md` from
`pack/sokf/agents/sokf/SPEC.md`; `.agents/sokf/grammar.yaml` from
`crates/lib/superdev-core/src/validate/schema/grammar.yaml`, embedded
in the binary; and every `knowledge/schemas/*.md` from
`pack/knowledge/schemas/`. A slice edits the source, runs `cargo run --
sync`, and commits the moved lock hashes; `superdev status` reporting
no drift is part of its done-check. Slice 2 found this — the plan first
named the pack as the grammar's source.

## Slices

### Slice 1: An include block names a source region

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `include_blocks` in
  `crates/lib/superdev-core/src/sokf/concept.rs` reads a `/`-rooted
  argument as a path with an optional `#region`, and `IncludeBlock`
  carries which it holds. A new function renders a source include's
  content: the file read under the repository root, refused when it
  resolves outside it; the region between `sokf:begin <name>` and
  `sokf:end <name>` found by substring, same-name regions concatenated
  in file order, the whole file when no region is named; a fenced block
  tagged by the extension through a small map (`rs`→`rust`,
  `yml`→`yaml`, `ts`→`typescript`, `py`→`python`), the extension
  verbatim otherwise, bare when none; a `sokf:generated-by` line in the
  file's leading lines kept. `check_include_blocks` in
  `validate/sokf.rs` and `materialize` in `validate/fix.rs` branch on
  the block's form and share the renderer, so the check and the repair
  cannot disagree. Errors name the path and the region: absent, empty
  or stale block; path missing or outside the repository; region the
  file does not carry. Nothing inside a region is parsed. The
  `sokf_snapshots` fixtures gain one case per error.
- Done-check: a fixture contract carrying
  `<!-- sokf:include /src/main.rs#cli -->` over a marked region
  materialises under `--fix` to a `rust`-tagged block equal to the
  region, passes `validate`, and fails it naming the path and region
  after one byte inside the region changes; `cargo nextest run -p
  superdev-core` passes; `superdev status` reports no drift.
- Cases:
  - unit: a `/path#region` argument parses to a source include and a
    bare id still parses to a concept include — covers 1.
  - unit: the renderer returns the region between the markers, fenced
    and tagged `rust` for a `.rs` file — covers 2, 3.
  - unit: two regions of one name concatenate in file order — covers 3.
  - unit: a path with no `#` renders the whole file — covers 2.
  - unit: `.proto` renders tagged `proto`, `.yml` renders `yaml`, a file
    with no extension renders bare — covers 2.
  - unit: a `sokf:generated-by` line in the file's first lines is the
    block's first line — covers 6.
  - unit: a region whose content is not valid in any language renders
    byte for byte, so nothing parsed it — covers 7.
  - unit: an absent block, an empty block and a stale block are each an
    error naming the path and the region — covers 4.
  - unit: a path that does not exist, a path resolving above the
    repository root through `..` or a symlink, and a region the file
    does not carry are each an error saying which — covers 5.
  - unit: `materialize` fills a source include with the renderer's text
    and leaves a concept include's behaviour unchanged — covers 2.
  - e2e: `superdev validate --fix` on a fixture knowledge writes the
    block, a second run writes nothing, and `superdev validate` after
    an edit inside the region exits 1 naming the path — covers 1, 2, 4.

### Slice 2: A sixth content kind, and the block declarations withdrawn

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `CONTENT_KINDS` in `validate/schema/document.rs` gains
  `include`; a section of that kind is satisfied by an include block
  whose argument is a `/`-rooted path, and a fenced block in such a
  section outside an include is an error naming the section.
  `block-language`, `block-keys` and `block-entry-keys` leave
  `SectionRule` and the block check with them, so a schema still
  declaring one is an unknown key the grammar reports. The grammar's
  section vocabulary in `pack/sokf/agents/sokf/grammar.yaml` drops the
  three keys and adds `include` to `content`, and `sync` writes the
  owned copy. `knowledge/schemas/contract.md` flips Definition from
  `code` to `include` and drops the sentence saying it would. contract-010
  drops the PENDING marks on the content-kinds and definition
  bullets, and `contract-002`'s block gains nothing — its `--fix`
  description already names include blocks. The `document_snapshots`
  goldens move where a message did.
- Done-check: a fixture schema declaring `content: include` passes a
  document whose section carries a source include and fails one whose
  section carries a bare fence, naming the section; a fixture schema
  declaring `block-keys` is reported as mis-declared;
  `knowledge/schemas/contract.md` validates with its example; `superdev
  status` reports no drift.
- Cases:
  - unit: `content: include` is satisfied by an include block naming a
    source path and not by one naming a concept — covers 9.
  - unit: a fenced block outside an include in an `include` section is
    an error naming the section — covers 10.
  - unit: a schema declaring `block-language`, `block-keys` or
    `block-entry-keys` is a finding on the schema, and no document
    finding follows from it — covers 7.
  - unit: a `yaml` block with a missing key in a section that formerly
    declared `block-keys` produces no finding — covers 7.
  - e2e: `superdev validate` on this repository passes with the contract
    schema's Definition declared `include` — covers 7, 9.

### Slice 3: A schema declares variants

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `DocSchema` gains `variant-key`; `SectionRule`, the
  frontmatter rule and a prohibited entry gain `variants`; `example`
  deserialises as one document or a map keyed by variant value. A
  document's rules are the schema's rules filtered by its
  discriminator value, in declared order, so `sections-ordered`,
  presence, prohibition and every body pattern run on the subsequence.
  The example check runs once per key, handing each example the
  variant its key names, and its discriminator value must equal the
  key. Mis-declared: a tag naming a value outside the discriminator's
  enum, a tag in a schema with no `variant-key`, a keyed example
  missing an enum value, a keyed `example` with no `variant-key`. The
  grammar gains the three keys at its pack source and `sync` writes the
  owned copy; contract-010 drops the PENDING marks on the variants
  bullet and the mis-declared list.
- Done-check: a fixture schema with `variant-key: kind` and a section
  tagged `[a]` passes a document of kind `a` carrying the section,
  fails one of kind `a` lacking it, and passes one of kind `b` lacking
  it; each mis-declaration is a finding on the schema; `superdev
  status` reports no drift.
- Cases:
  - unit: a section tagged `[a]` is required for kind `a` and absent
    from kind `b`'s rules — covers 13.
  - unit: an untagged section applies to every kind — covers 13.
  - unit: `sections-ordered` holds on the subsequence a kind sees, so a
    kind `b` document is not faulted for a kind `a` section's position
    — covers 13.
  - unit: a frontmatter key tagged `[a]` is required for `a` and
    unchecked for `b` — covers 13.
  - unit: a keyed example is checked per key against base plus its own
    variant, and an example whose discriminator differs from its key is
    a finding — covers 14.
  - unit: a tag outside the enum, a tag without `variant-key`, a missing
    example key and a keyed example without `variant-key` are each a
    finding on the schema — covers 13, 14.
  - unit: a schema with no `variant-key` and a string `example` is
    checked exactly as before — covers 13.

### Slice 4: The contract schema in its final form

- [x] Done — ticked by integrate at merge.
- Depends-on: 2, 3.
- Change: `knowledge/schemas/contract.md` declares `variant-key: kind`;
  each checklist bullet becomes a level-3 section rule tagged with its
  kinds, `required: true` where the bullet was marked required, its
  text as the rule's `description`; the title rule becomes twelve
  `heading-pattern` rules each tagged with its kind, so the display
  name agrees with the kind by construction; the prose checklist
  section goes; `example` becomes twelve, one per kind, each the
  shortest document that satisfies base plus variant. The filing check
  in `validate/lifecycle.rs` gains one rule for type `Contract`: the
  id's third segment equals `kind`. The pack mirror follows.
- Done-check: the schema validates with all twelve examples; a fixture
  `cli` contract lacking `### Exit codes` fails naming the section, and
  one with `kind: api` and an id segment `cli` fails naming both;
  `superdev status` reports no drift.
- Cases:
  - unit: a `cli` contract without `### Exit codes` fails; with it, and
    without `### Prompting`, passes — covers 15.
  - unit: an `api` contract without `### Authentication` fails and a
    `cli` contract without it passes — covers 15.
  - unit: a contract whose `kind` and id segment disagree is a filing
    finding naming both — covers 11.
  - unit: a `cli` contract titled `# API contract: …` fails the title
    rule for its kind — covers 12.
  - e2e: `superdev validate` on this repository passes with the schema
    carrying twelve examples — covers 8, 14, 15.

### Slice 5: The skills ask and declare

- [x] Done — ticked by integrate at merge.
- Depends-on: 4.
- Change: `.claude/skills/integrate/SKILL.md` gains a step, when a
  slice touched a contract, that reads the contract as its consumer
  would and reports where a marked region omits part of the promised
  surface, where an optional section the kind's checklist names is
  absent without reason, and where a reader could not learn the
  interface — naming what it checked, as a judgement that blocks
  nothing; a slice touching no contract says so.
  `.claude/skills/contract-design/SKILL.md` gains the declaration step:
  a new definition element is written into its source region with
  behaviour unbuilt, and the commit step names the source edit.
  `.claude/skills/accept/SKILL.md`'s pending gate reads Behaviour and
  Stability for `PENDING` and cites ADR-044. The three skills backport
  to the pack through `/pack-backport`.
- Done-check: `normative_shapes` and the skill grammar pass on the
  three files; `superdev status` reports no drift; the pack's copies
  equal the live ones.
- Cases:
  - unit: the integrate skill carries a step whose task names the three
    judgements and the no-contract case, checked by the normative-shape
    test the skills already have — covers 19, 20, 21.
  - unit: the contract-design skill's commit step names a source
    declaration — covers 18.
  - unit: the accept skill's pending gate names Behaviour, Stability and
    ADR-044 — covers 17.
  - integration: `superdev status` reports the three skills unchanged
    after `sync`, so the pack ships them — covers 22.

### Slice 6: The CLI and MCP contracts include their source

- [x] Done — ticked by integrate at merge.
- Depends-on: 1, 4.
- Change: `sokf:begin cli` / `sokf:end cli` around every clap struct
  and enum in `crates/app/superdev/src/` — `main.rs`, `validate_cli.rs`,
  `manage.rs` and the rest, one region name across files, one include
  per file — and `sokf:begin tools` / `sokf:end tools` around the tool
  registrations in `crates/lib/superdev-core/src/sokf/mcp.rs`.
  `contract-002` becomes `type: Contract`, `kind: cli`, `# CLI
  contract: superdev`, Definition of includes, its Behaviour and Exit
  codes and Streams regrouped as `###` under Behaviour; `contract-003`
  becomes `kind: api`, `# API contract: sokf over MCP`, likewise. The
  drift half of `contract.rs` — the surface comparison and
  `pending_commands` — is deleted; the tests that read the contract for
  usage-shape rules (integer exit keys, sorted commands) go with the
  YAML block they read. `mcp.rs`'s drift test is deleted.
  `contract_exit_codes.rs` stays; its reader moves from the YAML block's
  `exit:` maps to the `### Exit codes` table under Behaviour.
- Done-check: both contracts validate under the contract schema; the
  Definition of `contract-002` shows every clap struct with its doc
  comments; `cargo nextest run` passes with `contract.rs`'s drift tests
  and `mcp.rs`'s gone and `contract_exit_codes.rs` present; `--fix`
  after adding a flag to `validate_cli.rs` rewrites `contract-002`.
- Cases:
  - integration: adding `--nothing` to `ValidateArgs` and running
    `validate` fails naming `contract-002`'s include, and `--fix` then
    writes the flag into the contract — covers 2, 4, 23.
  - unit: `contract_exit_codes.rs` still exercises every code the
    contract states, read now from Behaviour's Exit codes subsection —
    covers 22.
  - e2e: no test under `crates/` reads a fenced block out of
    `contract-002` or `contract-003` to compare it to the binary —
    covers 21.

### Slice 7: The config and format contracts include their structs

- [x] Done — ticked by integrate at merge.
- Depends-on: 1, 4.
- Change: markers around the `serde` structs that read `config.toml`,
  `pack.toml` and `lock.toml`; `contract-004` becomes `kind: config`,
  `contract-005` and `contract-006` become `kind: format`, ids renamed
  to their kind token and refiled by `--fix`, each with a Definition
  of includes and sections regrouped. `contract_files.rs` is deleted;
  a unit test that the shipped `pack.toml` parses stays if one does not
  already exist elsewhere.
- Done-check: the three contracts validate; the Settings table of
  `contract-004` is replaced by the struct's doc-commented fields;
  `cargo nextest run` passes without `contract_files.rs`.
- Cases:
  - integration: renaming a field in the lock struct fails `validate`
    naming `contract-006`'s include — covers 4, 23.
  - e2e: no test compares a TOML block from a contract to the parser —
    covers 21.

### Slice 8: The interface contracts include their modules

- [x] Done — ticked by integrate at merge.
- Depends-on: 1, 4.
- Change: markers around the `pub` items `contract-007`, `contract-009`
  and `contract-010` declare, in the modules they describe; each
  becomes `kind: interface`, its Data model & API replaced by includes,
  Module boundaries, Key flows and Cross-cutting concerns as `###`
  under Behaviour, and a Stability section reading "Internal. Changes
  with the crate." `contract_interfaces.rs` is deleted. contract-010's
  own vocabulary block is included from `document.rs`, where the
  `SectionRule` and `DocSchema` structs with their doc comments are the
  declaration.
- Done-check: the three validate; `contract-010`'s Definition shows the
  `SectionRule` struct with `variants` and no `block_*`; `cargo nextest
  run` passes without `contract_interfaces.rs`.
- Cases:
  - integration: renaming a `pub fn` in `planner.rs` fails `validate`
    naming `contract-007`'s include — covers 4, 23.
  - unit: contract-010 carries no `PENDING` — covers 23.
  - e2e: no test matches signatures out of a contract against source —
    covers 21.

### Slice 9: The template contract includes its tree

- [x] Done — ticked by integrate at merge.
- Depends-on: 1, 4.
- Change: `contract-008` becomes `kind: format`. Its definition is the
  embedded template set and the substitution tokens, both declared in
  `crates/lib/superdev-core/src/templates.rs`; markers around the
  token list and the template registry make them the Definition. The
  tree listing that was the `text` block goes: the registry names
  every file, and a reader who wants the tree opens the directory the
  Definition names.
- Done-check: `contract-008` validates; its Definition shows the token
  and template registries from `templates.rs`; the test plan-021 slice
  12 wrote to compare the block to `templates.rs` — in
  `crates/lib/superdev-core/tests/contract_template.rs` since slice 7
  moved it out of `contract_files.rs` — is deleted with its file, the
  `every_drift_test_names_the_direction_it_failed_in` entry naming it
  goes, and the behaviour tests of substitution stay.
- Cases:
  - integration: adding a token to `templates.rs` fails `validate`
    naming `contract-008`'s include — covers 4, 23.
  - e2e: no test compares a block from `contract-008` to
    `templates.rs` — covers 23.

### Slice 10: Fifteen schemas go, and the records catch up

- [x] Done — ticked by integrate at merge. Sixteen kind schemas were on
  file, not fifteen: the count omitted `contract-interface`.
- Depends-on: 6, 7, 8, 9.
- Change: the fifteen `contract-*` schemas under `knowledge/schemas/`
  and `pack/knowledge/schemas/` are deleted with their `contract-style`
  include blocks; the schemas index lists one contract schema; the
  fragment's `description` says "included into the contract schema".
  `constraints-non-goals`, the glossary's `binding` and `drift test`,
  `development-commands`'s `--fix` sentence and the CHANGELOG say what
  is now true. `superdev validate` reports no contract type without a
  schema and no schema without a document type.
- Done-check: `ls knowledge/schemas/contract-*.md` lists nothing;
  `superdev validate` passes on this repository with every contract
  under `type: Contract`; `superdev status` reports no drift; the
  glossary's `drift test` entry says there are none.
- Cases:
  - e2e: `superdev validate` passes with fifteen schemas deleted and
    nine contracts of `type: Contract` — covers 8, 23.
  - e2e: `superdev status` reports no drift after the pack mirror loses
    fifteen files — covers 8.
  - unit: the `the_live_knowledge_conforms` snapshot passes — covers 23.
  - unit: the contract standard materialised into the contract schema
    states that a doc comment in a region is contract text, that
    unreachable behaviour is prose bound by test, and that `PENDING`
    applies to prose alone — covers 16, 17.

<!-- sokf:links -->
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]: /knowledge/issues/framed/issue-049-feature-request-a-contract-cannot-point-at-its-definition.md
