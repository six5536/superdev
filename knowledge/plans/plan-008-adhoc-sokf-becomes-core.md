---
type: AdhocPlan
id: plan-008-adhoc-sokf-becomes-core
title: SOKF becomes a core part of superdev
description: AOKF is renamed SOKF and stops being a swappable capability, the two validators merge into one module behind one command, a document's type names the schema that governs it, and the schema layer is enforced for the first time.
status: done
---

# Plan: SOKF becomes a core part of superdev

## Context

The knowledge format is modelled as a replaceable component — a `Capability`
slot, a registry entry, a provider id, a `--no-knowledge` flag — and is not
replaceable in fact, since the workflow skills, the schemas, the validator and
the MCP server all assume it. Planning that removal surfaced a larger gap: the
schema layer that is supposed to check knowledge documents against their
templates has never run. Both are fixed here, because the naming, the dispatch
and the enforcement are one design.

## Facts

- `Capability::Knowledge` is a `Single` slot with one provider, `aokf`
  (`registry.rs:88-94`), reachable at 39 sites across 15 files. `--no-knowledge`
  is used by eight CLI tests to obtain a repo without the scaffold.
- `embeddings` is a field on `CapabilityConfig` (`manifest.rs:28`) that only
  the knowledge capability reads (`aokf_cli.rs:285-293`).
- The merge lives in `format::validate_repo` (`format/mod.rs:261`): the grammar
  half calls `aokf::validate` and wraps its `Report`. The coupling is not
  symmetric and not removable — `aokf/mcp.rs:23,627` calls `validate`, and the
  validator takes the `Bundle` the read side loads. Only the two halves of the
  check are mutually independent.
- `aokf` appears 1,023 times in 162 tracked files: 380 under `crates/`, 436
  under `knowledge/`, 95 under `.agents/` and `.claude/`, 65 under `pack/`.
  The four MCP tool names account for 147 of them.
- **`target-files` is declared required on every schema — "Glob selecting the
  documents this schema governs" (`grammar.yaml:464-467`) — and nothing reads
  it.** The key's only occurrence in the crate is its own declaration. There
  are four checks; `check_schema` validates a schema's own shape, not any
  document against it. No glob matching exists in `superdev-core` outside the
  pack resolver.
- 40 schemas claim to govern documents; 125 markdown files live under
  `knowledge/`; zero are checked against a schema. The plan filed in this
  directory passed validation with none of `schema-adhoc-plan`'s rules applied.
- The globs have rotted unnoticed, which is what a dead code path looks like:
  seven match nothing (`adr` wants `decisions/adr-*.md`, the files are
  `D017-…md`; likewise `bug-report`, `interface-contract`, `research`, `spec`,
  `templates-index`, `visual-system`), seven `**/*name*.md` patterns match the
  schema file itself, and `release-notes` matches
  `node_modules/diff/release-notes.md`.
- `type` carries 14 distinct values over 117 concepts against 40 schemas:
  `Reference` is claimed by 8 schemas, `Issue` by one schema for 17 documents.
  Twelve schemas declare no `type` const at all.
- **Nothing consumes `type`** — `aokf/mcp.rs`, `graph.rs` and `index.rs` never
  read it. SPEC line 94: "Type values are free-form and unregistered; consumers
  must tolerate unknown types."
- Of the 54 documents that dispatch to exactly one schema today, 14 carry 141
  approximate findings — and 13 of those 14 are specs failing identically.
  `schema-spec` requires a 16-section shape (`Summary`, `Behaviour`,
  `Acceptance criteria`, `Edge cases & errors`, `Test plan: …`, `Exit
  criteria`, …); every spec in the tree uses a 7-section shape (`Problem`,
  `Solution`, `Behaviour`, `Design decisions`, `Testing`, `Out of scope`,
  `Open questions`). The schema is the outlier.
- All fourteen agent-facing invocations of validation name the SOKF half and
  none names the schema half, though one command runs both: eleven skill gates
  read "knowledge validates to PASS", plus `.agents/core.md:32`,
  `.claude/settings.json:7` and `maintain/PROJECT.md:7`.
- Renames migrate themselves: "the lock is what superdev applied, the
  components' claims are what the blueprint wants now, and the difference is
  the migration" (`orphan.rs:3-5`); `apply.rs:1419-1466` proves a stale
  `.mcp.json` key is removed on the next sync.
- `superdev-core` is published to crates.io (`release.yml:278`), so the module
  rename is a public API break. The workspace is at 0.2.0 and the changelog
  permits breaking changes in a minor while pre-1.0.
- `knowledge/contracts/contract-001-content-packs.md` pins `[knowledge] provider =
  "aokf"` (`:54`), the manifest scaffold rule (`:212`) and the `agents/aokf.md`
  asset path (`:238`), so this plan is an interface change to a live contract.
- Baseline: 534 tests passing, `superdev validate` clean over 61 files with 5
  warnings, `status --drift` reporting 65 pre-existing entries (I016).

## Goal

The knowledge format is SOKF, is part of superdev rather than a component it
loads, and every document it governs is checked against the schema its type
names.

## Outcomes

- O1 — nothing in the manifest, the registry or the CLI can disable the SOKF
  knowledge or swap what provides it.
- O2 — one module owns both halves of the check, and neither half calls the
  other.
- O3 — `SOKF` names the format on every surface that names it, and no live
  file says AOKF.
- O4 — a document's `type` names exactly one schema, and every schema is
  named by exactly one type.
- O5 — every schema has been tested against the documents it governs, and the
  disagreements are resolved in favour of whichever side is right.
- O6 — a document that breaks its schema fails `superdev validate`.
- O7 — the skills, templates and concepts still ship from the content pack;
  only the specification and the instructions move with the binary.

## Non-goals

- Standardising "bundle" and "knowledgebase" onto "knowledge". That is I013,
  landing before this plan starts.
- Rewriting the completed records. The decisions, specs, feature plans, closed
  issues and P006/P007 say AOKF because that is what it was called when they
  were written; P006 and P007 already carry the same exemption.
- Enforcing SPEC §9's index shape. That is I011, and it belongs to the SOKF
  half rather than the schema layer.
- The remaining validator gaps — index entries (I010) and warning severities
  (I012). I017 is answered incidentally: the schema layer becomes the
  agent-facing statement of what a document must contain.
- Reducing the 65 blueprint-drift entries. That is I016; this plan must not
  add to the set, and does not close it.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | `Capability` carries four variants and the registry four entries, none of them knowledge | O1 |
| FR-2 | `superdev init` always writes the SOKF scaffold, the hook and the MCP registration; `--no-knowledge` does not exist | O1 |
| FR-3 | `[knowledge]` is a top-level manifest table holding `custom` and `embeddings`, not a capability table | O1 |
| FR-4 | `superdev_core::validate` owns the whole check, and its `sokf` and `schema` halves import each other in neither direction | O2 |
| FR-5 | `validate`, `hook validate`, `sokf index` and `mcp sokf` are the verbs, and no `aokf` verb group exists | O2, O3 |
| FR-6 | The MCP server registers as `superdev-sokf` and serves `sokf_search`, `sokf_read`, `sokf_graph` and `sokf_overview` | O3 |
| FR-7 | The manifest is `knowledge/manifest.sokf.yaml`, declaring `sokf` and `name` | O3 |
| FR-8 | The specification is `.agents/sokf/SPEC.md`, titled SOKF, and §1 defines "SOKF knowledge"; no live file names AOKF | O3 |
| FR-9 | Every schema declares a `type` const, no two schemas declare the same one, and every concept's `type` names a schema that exists | O4 |
| FR-10 | A concept resolves to its schema by `type`; `target-files` resolves only for documents that carry no frontmatter | O4 |
| FR-11 | `target-files` resolution is confined to the SOKF knowledge root plus a named allowlist, never matches under `knowledge/schemas/`, and never leaves the repository | O4 |
| FR-12 | Every schema has been run against its documents and each disagreement resolved on the record | O5 |
| FR-13 | `superdev validate` reports a document that breaks its schema as an error: a missing required section, a section out of order, a prohibited section, a wrong table column, an over-limit line count | O6 |
| FR-14 | A concept whose type names no schema is reported, and so is a schema that governs nothing — one declaring neither a type const nor a glob | O4, O6 |
| FR-15 | One `sync` of a repo built by the previous release removes the old hook element, MCP key and `.agents/aokf*`, and writes their replacements | O1, O3 |
| FR-16 | The specification and the instructions are the only binary-owned files; skills, templates and concepts resolve from the pack | O7 |

### Non-functional

| ID | Constraint | Budget |
|----|------------|--------|
| NFR-1 | `superdev validate` over this repository stays usable as a hook | ≤ 250 ms release build (86 ms today, before schema enforcement) |
| NFR-2 | Per-crate line coverage holds | ≥ 90% per crate |
| NFR-3 | No test is dropped to keep the suite green | ≥ 534 tests |
| NFR-4 | The plan adds no blueprint drift | `status --drift` names no path this plan touched |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | Knowledge leaves the capability system entirely | keep the slot with one provider | a slot states the thing is replaceable, which is the complaint |
| D-2 | `Component::capability` returns `Option<Capability>` | add a `Capability::Core` variant | a core component fills no slot, and `Planned.capability` is already optional |
| D-3 | `Owner::Capability(Knowledge)` becomes `Owner::Knowledge` | fold it into `Owner::Repo` | `custom` lists are name-guarded per owner, and SOKF skills share `.claude/skills/` with pack skills |
| D-4 | No lever remains to run superdev without the SOKF knowledge | keep a flag or a manifest opt-out | that is what D-1 means; a hatch would be the slot under another name |
| D-5 | "SOKF knowledge" is the standing term, and `sokf` anchors every identifier | "knowledge" alone | `frame/SKILL.md` uses "knowledge" in three senses in 43 lines; `sokf_search` on the same line cannot be mistaken for anything |
| D-6 | The directory stays `knowledge/` | rename it `sokf/` | it is a path in a repo whose manifest declares the format, and it is the most visible thing superdev writes into a user's tree |
| D-7 | `validate` and `hook` are bare; `sokf index` and `mcp sokf` carry the anchor | all four top-level | the first two cover the whole repository, the last two do not |
| D-8 | The grammar half is the schema-validator (`validate::schema`) | keep "superdev-format"; coin a second acronym | I014 named it correctly: its job is checking documents against schemas, and skills are the ancillary case |
| D-9 | One `validate` module with `sokf` and `schema` beneath it | one flat module | the D-18 boundary becomes structural instead of a doc comment |
| D-10 | No compatibility path for the old manifest, hook marker or MCP key | accept both for one release | pre-1.0, and the orphan pass is already the migration |
| D-11 | An old `[knowledge]` capability table is a named error, not a silent lift | migrate it in place on load | D-10; an error naming the edit is honest, and migration code would outlive its purpose |
| D-12 | `type` names exactly one schema; `Reference` splits into eight and `Issue` into its kinds | keep `type` coarse and add a `schema` or `kind` field | SPEC line 94 makes type free-form, nothing consumes it today, and a second field would say the same thing twice on every concept |
| D-13 | `target-files` survives, scoped, for documents with no frontmatter | delete it and use exact paths plus an `index.md` convention | one dispatch mechanism rather than two, and it stays available for a schema that wants a pattern |
| D-14 | Schemas are reconciled against practice before enforcement is switched on | ship enforcement as warnings and promote later | I012 is this repository's own evidence that a warning backlog goes unread |
| D-15 | Where documents agree with each other and not the schema, the schema is wrong | bring every document to its schema | 13 of 14 specs already agree on a shape the schema does not describe; the practice is the evidence |
| D-16 | The specification and the instructions stay binary-owned | ship them from the pack | unchanged rule: they describe a version the binary pins and a format its compiled validator enforces |
| D-17 | Everything lands on `feature/content-packs` | split into structural, content and feature branches | the author's call; the workstream order is what keeps the tree green instead |

## Workstreams

### W1: Knowledge stops being a capability

Depends on: none.

1. Give the config a home — add a top-level `[knowledge]` table to `Manifest`
   carrying `custom` and `embeddings`, and take `embeddings` off
   `CapabilityConfig`. Nothing reads it yet, so the tree stays green.
2. Loosen the component contract — `Component::capability` returns
   `Option<Capability>`, every implementation returning `Some`.
3. Name the owner — `Owner::Capability(Capability::Knowledge)` becomes
   `Owner::Knowledge` in `content/item.rs` and `content/layout.rs`. Pack paths
   are unchanged.
4. Make the component unconditional — `components/aokf.rs` becomes
   `components/sokf.rs`, returns `None` for its capability, and
   `components/enabled.rs` appends it to every resolution.
5. Point the reads at the new table — the embeddings lookup and the skills
   adoption path read `[knowledge]`.
6. Remove the slot — drop `Capability::Knowledge`, its registry entry and
   `--no-knowledge`. Hard to reverse: it deletes a user-facing flag and a
   manifest shape, so it lands with step 7.
7. Re-cut the eight `--no-knowledge` tests — each used the flag for a light
   repo. Give them a repo with the scaffold and assert what they were actually
   asserting. Delete none.
8. Refuse the old table by name — manifest load reports that `[knowledge]` is
   no longer a capability and names the edit (D-11).

### W2: One validator module

Depends on: none.

1. Create the parent — `src/validate/mod.rs` holding `validate_repo`,
   `RepoReport`, `Report` and `Finding`, moved from `format/mod.rs` and
   `aokf/validate.rs`. The merge point is now above both halves.
2. Move the SOKF half — `aokf/validate.rs` becomes `validate/sokf.rs`.
3. Move the schema half — `format/{check,grammar,read,re,doc}.rs` become
   `validate/schema/{check,grammar,read,re,doc}.rs`, with `grammar.yaml`.
4. Rename the read side — `src/aokf/` becomes `src/sokf/`, and `lib.rs`
   declares `sokf` and `validate`. Hard to reverse: it breaks the published
   `superdev-core` API, so it lands in one commit with its changelog entry.
5. Move the grammar file — `.agents/format/grammar.yaml` to
   `.agents/sokf/grammar.yaml`, beside the specification, and `load_grammar`
   with it. The embedded and on-disk copies stay byte-identical; a test
   already asserts it.
6. Rename the suites and fixtures — `tests/fixtures/aokf/` to
   `tests/fixtures/sokf/`, `tests/fixtures/format/` to
   `tests/fixtures/schema/`, `format_snapshots.rs` to `schema_snapshots.rs`.
   Golden contents are relative to their case directory, so no golden changes;
   that is the check.

### W3: One command surface

Depends on: W2.

1. Split the CLI module — `aokf_cli.rs` becomes `validate_cli.rs` (the
   `validate` verb and the hook) and `sokf_cli.rs` (`index` and `mcp`).
2. Set the verbs — `main.rs` gains `Hook`, `Sokf::Index` and `Mcp::Sokf`, and
   loses the `Aokf` group and `Mcp::Aokf`.
3. Rename the hook marker — `superdev aokf hook validate` becomes `superdev
   hook validate`. Hard to reverse: the marker is the lock key in every
   managed repo, so the old element is removed by the orphan pass on the next
   sync and by nothing else.
4. Rename the MCP registration — `mcpServers.superdev-sokf`, invoked as
   `superdev mcp sokf`.
5. Rename the tools — `sokf_{search,read,graph,overview}`, `AokfServer` to
   `SokfServer`, and the server's instruction text. `tests/mcp_tools.rs`
   follows.
6. Move the index cache — `.superdev/cache/aokf-index` to
   `.superdev/cache/sokf-index`.

### W4: SOKF, the format

Depends on: W2.

1. Rewrite the specification — `.agents/aokf/SPEC.md` to
   `.agents/sokf/SPEC.md`, titled "SOKF — Superdev Open Knowledge Format",
   every AOKF replaced, and §1 defining **SOKF knowledge** as the term for the
   document tree.
2. Rewrite the instructions — `.agents/aokf.md` to `.agents/sokf.md`, the
   `<aokf-system>` element to `<sokf-system>`, the import to `@sokf/SPEC.md`,
   and the commands to the W3 verbs.
3. Change the manifest in code — `MANIFEST` becomes `manifest.sokf.yaml`,
   `BundleManifest::aokf` becomes `sokf`, and the required-key check follows.
4. Move this repository's manifest — `knowledge/manifest.aokf.yaml` to
   `knowledge/manifest.sokf.yaml`, `aokf: "0.3"` to `sokf: "0.3"`.
5. Update the eleven fixtures and regenerate — `UPDATE_GOLDENS=1` over both
   suites, then read the diff: only the manifest filename and key may move.
   Any other change is a behaviour change and stops the step.
6. Repoint the binary-owned paths — `BINARY_OWNED` and the assets beneath it.

### W5: A type names a schema

Depends on: none.

1. Give every schema a type const — the twelve without one get theirs;
   `interface-contract` gets `Contract`, which its documents already carry.
2. Split the collisions — `Reference` becomes eight distinct types across
   `api-contracts`, `architecture`, `configuration`, `constraints-non-goals`,
   `development-commands`, `directory-structure`, `software-components` and
   `technology-stack`; `Issue` splits into `BugReport`, `FeatureRequest` and
   whatever else the seventeen filed issues need; `Convention`, `Procedure`
   and `Policy` likewise.
3. Retype the documents — every concept's frontmatter `type` follows its
   schema. Roughly 40 documents. The SOKF half already requires `type`, so a
   missed one is caught by the existing checks.
4. Delete the dead globs — `target-files` comes off every schema that now
   dispatches by type, which is all but the frontmatter-less handful.
5. Rewrite the issue tracker's shape — I015 is answered by step 2; the
   tracker's convention document and the issues index follow the new types.

### W6: Reconcile the schemas to practice

Depends on: W5.

1. Build the reconciliation harness — a test that runs every schema against
   the documents its type names and prints the disagreements. Temporary
   scaffolding for step 2, kept afterwards as the enforcement suite's fixture.
2. Judge each disagreement — where documents agree with each other and not the
   schema, correct the schema (D-15); where a document is genuinely malformed,
   correct the document. Record which, per schema, in the Appendix.
3. Correct `schema-spec` first — 13 of 14 specs fail it identically, which is
   the clearest case in the tree and the one that sets the precedent.
4. Reach zero — the harness reports no disagreement before W7 begins. This is
   the gate; enforcement cannot land green without it.

### W7: Enforcement

Depends on: W2, W6.

1. Dispatch by type — `validate::schema` resolves a concept to its schema
   through the frontmatter `type`, and reports a type naming no schema and a
   schema that governs nothing (FR-14).

   Corrected during execution. This step and FR-14 first said "a schema no
   type names", which is a different claim: a schema whose type no document
   *yet* carries. Nine of the forty-one are exactly that — `code-review`,
   `investigation`, `migration-guide`, `postmortem`, `release-notes`,
   `research`, `security-review`, `status-update`, `visual-system` — kinds
   nobody has written here. Reporting them would have made the first run
   fail on nine contracts doing their job, and the only way to silence it
   would be deleting the contract or writing a document to satisfy it. What
   ships instead is the decidable case: a schema declaring neither a type
   const nor a `target-files` glob can reach no document ever, whatever
   anyone writes later. That is reported, and the `governs-nothing` fixture
   is its test.
2. Scope the glob — `target-files` resolves only for frontmatter-less
   documents, confined to the SOKF knowledge root plus a named allowlist,
   refusing anything under `knowledge/schemas/` or outside the repository
   (FR-11). The seven self-matching patterns and the `node_modules` reach are
   the tests.
3. Apply the section rules — required sections, `sections-ordered`,
   `sections-prohibited`, and `heading-pattern`, reported per document.
4. Apply the content rules — declared table columns, `content` kinds, and
   `line-limit`.
5. Add the fixtures — one case per rule under `tests/fixtures/schema/`, each
   with its golden, in the shape the existing snapshot suites use.
6. Correct the fourteen invocations — the eleven skill gates and the three
   command sites say what they now check, rather than naming only the SOKF
   half.

### W8: The sweep

Depends on: W1, W2, W3, W4, W5, W7.

1. Move the pack's instruction files — `pack/aokf/agents/**` to
   `pack/sokf/agents/**`. `classify` matches neither, which is what keeps
   them binary-owned; `paths_matching_no_rule_are_not_items` gains both new
   paths.

   Corrected during execution. This step first said `pack/knowledge/agents/`,
   which `classify` does indeed ignore — but `pack::manifest::REJECTED`
   guards the paths a *fetched* pack may not supply, and superdev's own pack
   then matched its own guard, so the pack stopped resolving. The old layout
   used a top-level directory for exactly this separation; `pack/sokf/`
   keeps it under the new name.
2. Sweep the skills — the 21 under `.claude/skills/` and their
   `pack/knowledge/skills/` mirrors, both sides in one step so the drift set
   does not grow.
3. Change the contract — C001 pins the provider id, the manifest scaffold name
   and the asset path, all three of which move. An interface change, made
   deliberately.
4. Sweep the concepts — the 18 live concepts and the schemas that name a tool,
   a verb or the manifest. The completed records are left alone.
5. Sweep the entry documents and scripts — `.agents/core.md`,
   `pack/agents/*.md`, `README.md`, `CONTRIBUTING.md`, `.gitattributes`,
   `scripts/superdev`, `scripts/manage-smoke.sh`.
6. Re-sync this repository — `.superdev/config.toml` loses its `[knowledge]`
   capability table and gains the top-level one; the lock and `.mcp.json`
   follow from `superdev sync`, never hand-edited.
7. Record the breaks — one changelog entry per surface: the capability, the
   crate API, the verbs, the MCP server, the manifest, and enforcement.
8. Close what this supersedes — I014 and I015 become done, naming this plan.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `crates/lib/superdev-core/src/manifest.rs` | modified — top-level `[knowledge]`; `embeddings` off `CapabilityConfig`; the named refusal | W1 |
| `crates/lib/superdev-core/src/capability.rs` | modified — `Knowledge` removed, `ALL` becomes four | W1 |
| `crates/lib/superdev-core/src/registry.rs` | modified — the `aokf` entry removed | W1 |
| `crates/lib/superdev-core/src/component.rs` | modified — `capability` returns `Option<Capability>` | W1 |
| `crates/lib/superdev-core/src/content/{item,layout}.rs` | modified — `Owner::Knowledge`; the instruction paths stay unclassified | W1, W8 |
| `crates/lib/superdev-core/src/components/aokf.rs` | deleted — succeeded by `sokf.rs` | W1 |
| `crates/lib/superdev-core/src/components/sokf.rs` | new — the unconditional component, its hook marker and MCP registration | W1, W3, W4 |
| `crates/lib/superdev-core/src/components/enabled.rs` | modified — appended, not resolved | W1 |
| `crates/lib/superdev-core/src/validate/mod.rs` | new — `validate_repo`, `RepoReport`, `Report`, `Finding` | W2 |
| `crates/lib/superdev-core/src/validate/sokf.rs` | new — moved from `aokf/validate.rs` | W2, W4 |
| `crates/lib/superdev-core/src/validate/schema/*.rs` | new — moved from `format/` | W2 |
| `crates/lib/superdev-core/src/validate/schema/document.rs` | new — dispatch by type, the glob for frontmatter-less documents, and the section, column and line-limit rules | W7 |
| `crates/lib/superdev-core/tests/document_snapshots.rs` | new — one case per rule | W7 |
| `crates/lib/superdev-core/tests/fixtures/documents/**` | new — eight cases and their goldens | W7 |
| `crates/lib/superdev-core/src/pack/resolve.rs` | modified — the rejected-path test follows `REJECTED` | W8 |
| `crates/lib/superdev-core/src/{aokf,format}/` | deleted — both directories | W2 |
| `crates/lib/superdev-core/src/sokf/*.rs` | new — moved from `aokf/`; `mcp.rs` renames the tools and the server | W2, W3 |
| `crates/lib/superdev-core/src/lib.rs` | modified — `sokf` and `validate` replace `aokf` and `format` | W2 |
| `assets/aokf/**` | deleted — succeeded by `assets/sokf/agents/**` | W8 |
| `assets/sokf/agents/**` | new — the instructions and the SOKF specification (`assets/` is a symlink to `pack/`) | W4, W8 |
| `crates/lib/superdev-core/tests/fixtures/sokf/**` | modified — moved; eleven manifests renamed and re-keyed | W2, W4 |
| `crates/lib/superdev-core/tests/fixtures/schema/**` | modified — moved; one new case per enforcement rule | W2, W7 |
| `crates/lib/superdev-core/tests/{validate,schema}_snapshots.rs` | modified — new fixture roots; `format_snapshots.rs` deleted | W2, W7 |
| `crates/lib/superdev-core/tests/mcp_tools.rs` | modified — the four renamed tools | W3 |
| `crates/app/superdev/src/main.rs` | modified — the new verbs; the `Aokf` group removed | W3 |
| `crates/app/superdev/src/aokf_cli.rs` | deleted — split into the two below | W3 |
| `crates/app/superdev/src/{validate_cli,sokf_cli}.rs` | new — the verb and the hook; index and mcp | W1, W3 |
| `crates/app/superdev/src/manage.rs` | modified — `--no-knowledge` removed | W1 |
| `crates/app/superdev/tests/{cli,manage}.rs` | modified — the eight re-cut tests, the new verbs and keys | W1, W3 |
| `.agents/aokf.md`, `.agents/aokf/SPEC.md`, `.agents/format/grammar.yaml` | deleted — succeeded under `.agents/sokf/` | W2, W4 |
| `.agents/sokf.md`, `.agents/sokf/SPEC.md`, `.agents/sokf/grammar.yaml` | new — instructions, specification, grammar | W2, W4 |
| `.agents/core.md` | modified — the tool names, the verbs, the gate wording | W7, W8 |
| `knowledge/manifest.aokf.yaml` | deleted — succeeded by `manifest.sokf.yaml` | W4 |
| `knowledge/manifest.sokf.yaml` | new — `sokf` and `name` | W4 |
| `knowledge/schemas/*.md` | modified — all 40: a type const each, dead globs removed, and reconciled to practice | W5, W6 |
| `knowledge/**/*.md` | modified — roughly 40 concepts retyped; those the reconciliation finds genuinely malformed corrected | W5, W6 |
| `knowledge/issue-tracker.md`, `knowledge/issues/index.md` | modified — the issue types I015 asked for | W5 |
| `knowledge/contracts/contract-001-content-packs.md` | modified — the provider id, the manifest name, the asset path | W8 |
| `.claude/skills/*/SKILL.md` | modified — 21 skills: tool names, verbs, and the eleven gates | W7, W8 |
| `pack/knowledge/skills/*/SKILL.md` | modified — the same 21, mirrored | W8 |
| `pack/aokf/**` | deleted — succeeded by `pack/sokf/agents/**` | W8 |
| `pack/{sokf/agents,agents}/*.md` | new and modified — the binary-owned pair and the general rules | W8 |
| `.gitattributes`, `scripts/superdev`, `scripts/manage-smoke.sh` | modified — the fixture comment, the hook command, the two verbs | W8 |
| `.superdev/config.toml` | modified — the capability table becomes the top-level one | W8 |
| `.superdev/lock.toml`, `.mcp.json` | modified — written by `superdev sync` | W8 |
| `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | modified — the verbs, the commands, one entry per break | W8 |

## Acceptance

| Check | Verifies |
|-------|----------|
| `git grep -Ii 'aokf' -- . ':!CHANGELOG.md' ':!knowledge/plans' ':!knowledge/decisions' ':!knowledge/specs' ':!knowledge/issues'` returns only deliberate references: the tests asserting the old verbs are gone, the tripwire refusing a pre-SOKF manifest, the snapshot header's note on the Python reference, and two links to S002, whose id is historical | FR-8 |
| `git grep -n 'Capability::Knowledge\|no-knowledge\|no_knowledge'` returns nothing, and `Capability::ALL` has four entries | FR-1, FR-2 |
| `superdev init` in an empty scratch repo writes `knowledge/`, the hook and the `.mcp.json` entry with no flag given, and its `config.toml` holds a top-level `[knowledge]` table and no knowledge capability | FR-2, FR-3 |
| `superdev sync` against a config carrying `[knowledge] provider = "aokf"` fails naming the table and the edit | FR-3 |
| `git grep -n 'schema' src/validate/sokf.rs` returns nothing, and the only `sokf` under `src/validate/schema/` is the grammar's own path and the four tool names it governs — no call in either direction; `validate/mod.rs` names both | FR-4 |
| `superdev --help` lists `validate`, `hook`, `sokf`, `mcp`, and no `aokf`; `superdev sokf --help` lists `index` | FR-5 |
| `superdev mcp sokf` starts, and `tests/mcp_tools.rs` drives `sokf_search`, `sokf_read`, `sokf_graph`, `sokf_overview` | FR-6 |
| `knowledge/manifest.sokf.yaml` holds `sokf: "0.3"`, and `git grep -l 'manifest.aokf.yaml'` returns nothing outside the changelog | FR-7 |
| `head -1 .agents/sokf/SPEC.md` reads `# SOKF — Superdev Open Knowledge Format`, and §1 defines "SOKF knowledge" | FR-8 |
| A test asserts every schema declares a `type` const, that no two are equal, and that every concept's type names a schema | FR-9 |
| A test asserts `target-files` appears only on schemas whose documents carry no frontmatter | FR-10 |
| A test feeds `**/*release-notes*.md` and asserts the resolver refuses `node_modules/`, refuses `knowledge/schemas/`, and stays inside the repository | FR-11 |
| The reconciliation harness reports zero disagreements across all 40 schemas | FR-12 |
| One fixture case per rule — missing section, misordered section, prohibited section, wrong table column, over-limit — each failing with its own message and its own golden | FR-13 |
| A fixture with a type naming no schema, and one schema declaring neither a type const nor a glob, both reported — `unknown-type` and `governs-nothing` | FR-14 |
| A fixture repo carrying the previous release's lock, `.mcp.json`, `.claude/settings.json` and `.agents/aokf*` syncs to the new state, and a second `status --drift` exits 0 | FR-15 |
| `binary_owned_count()` is 2 and names the two `.agents/sokf/` paths; every skill, template and concept traces to a pack item | FR-16 |
| `read_pack("superdev", &root)` resolves — the binary-owned pair sits outside the `agents/` position `REJECTED` guards | FR-16 |
| `superdev validate` over this repository reports PASS, with the 5 known warnings and no errors | FR-12, FR-13 |
| `cargo nextest run --workspace` passes with at least 534 tests | NFR-3 |
| `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --doc` and `RUSTDOCFLAGS="-D warnings" cargo doc` are clean | NFR-3 |
| `npm run coverage:check` passes | NFR-2 |
| `time ./target/release/superdev validate` over this repository is under 250 ms | NFR-1 |
| `superdev status --drift` names no path this plan touched | NFR-4 |
| After W2, `git diff --stat` over both golden trees shows no content change — only the directory renames | O2 |

## Definition of done

- Every Acceptance row passes on a clean checkout of `feature/content-packs`.
- `knowledge/plans/index.md` lists this plan, and its status reads done.
- The Appendix records, per schema, which side the reconciliation found wrong.
- The changelog carries one entry per break — the capability, the
  `superdev-core` module rename, the verbs, the MCP server and tools, the
  manifest — and one for enforcement.
- I014 and I015 are done and name this plan as what closed them.
- The old grammar path, fixture roots, cache directory and asset directories
  are absent from the tree.

## Risks

- Risk: I013 has not finished when this starts, and the two sweeps collide
  across the same 162 files — mitigation: this plan does not begin until
  `git grep -Ii 'bundle'` is quiet outside the changelog and the closed
  records; that grep is the gate.
- Risk: W6 resolves a disagreement the wrong way, and thirteen specs are
  rewritten into a shape nobody uses — mitigation: D-15 states the rule, and
  W6 step 2 records the judgement per schema so a reviewer can disagree with
  it.
- Risk: the backlog is larger than measured. Only 54 of 117 concepts dispatch
  1:1 today; the rest arrive with W5 — early signal: W6 step 1's first run.
  If the count is large, W6 is where the plan stalls, not W7.
- Risk: the eight `--no-knowledge` tests are re-cut into weaker tests to keep
  the suite green — early signal: the count falls below 534, or a re-cut test
  asserts less than its original.
- Risk: enforcement makes the hook too slow to leave installed — early signal:
  NFR-1. Dispatch is a map lookup and the rules are per-document, so the cost
  should be linear in documents rather than in schemas.
- Risk: retyping breaks a link or an id that referenced a type — mitigation:
  the SOKF half already validates ids and links, and it runs on the same
  command.
- Risk: `pack/` and `.claude/` drift apart during the sweep, widening I016's
  65 — mitigation: W8 step 2 changes both sides in one step; NFR-4 is the
  check.
- Risk: the crates.io API break reaches a downstream nobody knew about —
  mitigation: none available; the changelog entry and the pre-1.0 policy are
  the whole answer.

## Out-of-band notes

`.superdev/cache/aokf-index/` is gitignored machine state and is not in the
lock, so nothing removes it; a synced repo keeps an orphaned directory until
its owner deletes it. Not worth code.

I013 is the precondition. I014 and I015 are closed by this plan. I017 is
answered incidentally — the schema layer becomes the statement of what a
document must contain, which is what it asked for — and should be reviewed
against this plan's outcome rather than closed blind. I010, I011 and I012 are
untouched.

The specification's version stays 0.3: the rename changes no rule. The
manifest key rename is the one on-disk change, and §2 records it.

`templates/processes/` is 21 tracked Claude Code process templates with no
relation to the schema layer. The `**/*code-review*.md` glob matches one of
them by accident today; under FR-11 it cannot.

## Appendix

### Reconciliation record

One row per disagreement W6 judged, naming which side was wrong. 218
findings on the first run, zero on the last.

| Schema | Documents | Wrong side | Change |
|--------|-----------|------------|--------|
| `spec` | 13 of 14 | **documents** | Corrected. First judged the schema wrong and relaxed it to what the corpus shared. That misapplied D-15: the thirteen did not agree with each other either — 12 of 14 had `Out of scope`, 10 had `Testing`, nothing else above 6 — so the precondition was never met, and relaxing ratified the drift the validator exists to catch. The schema is restored in full and all thirteen conformed. |
| `feature-request` | 6 | documents | Retyped in W5, still carrying bug-report bodies. Reshaped: motivation, proposed behaviour, alternatives, scope. |
| `chore` | 2 | documents | The same, into surfaces and a definition of done. |
| `bug-report`, `feature-request`, `chore` | 9 | schema | The resolution rule sat last; every settled issue puts it directly under the title. Moved. `Comments` split off and stays last, per the tracker's own convention. |
| `adhoc-plan` | P004 | document | Predates the schema and used the four headings it prohibits. Reorganised — its "Current state" was already Facts with evidence attached. Outcomes, Non-goals, Requirements and Definition of done written from its own content. |
| `feature-plan` | P001, P002 | documents | Predate the schema; their numbered task lists were slice lists without the headings. |
| `readme` | README.md | **document** | Corrected, for the same reason: one document is not a corpus, so "the documents agree" was vacuous. The schema is restored and the README gained `Quick start`, `Usage` and `Configuration` — which it needed anyway, and which found a stale `--no-knowledge` in the prose. |
| `architecture`, `coding-standards`, `constraints-non-goals`, `software-components` | 4 | documents | Each qualified a required heading (`CI/CD (\`.github/workflows\`)`). The qualifier moved into the prose. |
| `issue-tracker` | 1 | document | A table column read "Tag in this repo" where the shipped schema declares "Label". |
| `bug-report` | I009 | document | No regression risk section; a settled report should still say what would catch a recurrence. |

### Approximate backlog before reconciliation

Measured with a probe that applies required sections, `heading-pattern`,
`sections-prohibited` and `line-limit` to the 54 documents that dispatch to
exactly one schema today. It does not check content kinds, table columns or
section order, so it understates; it also predates W5, so it covers fewer than
half the concepts.

| Documents dispatched | With findings | Findings | Concentration |
|----------------------|---------------|----------|---------------|
| 54 | 14 | 141 | 13 of the 14 are specs, all failing `schema-spec` identically |

The probe understated, as it said it would. The real first run, after W5
brought every concept into dispatch, reported 218.

Of those 218, 143 were first resolved by relaxing a schema and then
re-resolved by conforming the documents — 140 for `schema-spec` and 3 for
`schema-readme`. The lesson is D-15's, sharpened: the test is whether the
documents agree **with each other**. Inconsistent documents are evidence of
inconsistency, not of a wrong standard, and a validator that bends to them
is one that ratifies drift. Every other judgement in the table above went to
the documents from the start.

The twelve older specs are conformed in shape with thin content where the
record does not carry more: no test plan was written for them at the time,
so their plans name the automated cases that exist and say plainly that no
manual step was recorded, rather than inventing one.
