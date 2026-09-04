---
type: Plan
id: plan-008-sokf-becomes-core
title: SOKF becomes a core part of superdev
description: AOKF is renamed SOKF and stops being a swappable capability, the two validators merge into one module behind one command, a document's type names the schema that governs it, and the schema layer is enforced for the first time.
lifecycle: done
---

# Plan: SOKF becomes a core part of superdev

## Goal

The knowledge format is SOKF, is part of superdev rather than a component it
loads, and every document it governs is checked against the schema its type
names.

The format is modelled as a replaceable component — a `Capability` slot, a
registry entry, a provider id, a `--no-knowledge` flag — and is not
replaceable in fact, since the workflow skills, the schemas, the validator
and the MCP server all assume it. Planning that removal surfaced a larger
gap: the schema layer that is supposed to check knowledge documents against
their templates has never run. Both are fixed here, because the naming, the
dispatch and the enforcement are one design.

What the work delivers:

- O1 — nothing in the manifest, the registry or the CLI can disable the SOKF
  knowledge or swap what provides it.
- O2 — one module owns both halves of the check, and neither half calls the
  other.
- O3 — `SOKF` names the format on every surface that names it, and no live
  file says AOKF.
- O4 — a document's `type` names exactly one schema, and every schema is
  named by exactly one type.
- O5 — every schema has been tested against the documents it governs, and
  the disagreements are resolved in favour of whichever side is right.
- O6 — a document that breaks its schema fails `superdev validate`.
- O7 — the skills, templates and concepts still ship from the content pack;
  only the specification and the instructions move with the binary.

The facts the design rests on:

- `Capability::Knowledge` is a `Single` slot with one provider, `aokf`
  (`registry.rs:88-94`), reachable at 39 sites across 15 files.
  `--no-knowledge` is used by eight CLI tests to obtain a repo without the
  scaffold.
- `embeddings` is a field on `CapabilityConfig` (`manifest.rs:28`) that only
  the knowledge capability reads (`aokf_cli.rs:285-293`).
- The merge lives in `format::validate_repo` (`format/mod.rs:261`): the
  grammar half calls `aokf::validate` and wraps its `Report`. The coupling
  is not symmetric and not removable — `aokf/mcp.rs:23,627` calls
  `validate`, and the validator takes the `Bundle` the read side loads. Only
  the two halves of the check are mutually independent.
- `aokf` appears 1,023 times in 162 tracked files: 380 under `crates/`, 436
  under `knowledge/`, 95 under `.agents/` and `.claude/`, 65 under `pack/`.
  The four MCP tool names account for 147 of them.
- `target-files` is declared required on every schema — "Glob selecting the
  documents this schema governs" (`grammar.yaml:464-467`) — and nothing
  reads it. The key's only occurrence in the crate is its own declaration.
  There are four checks; `check_schema` validates a schema's own shape, not
  any document against it. No glob matching exists in `superdev-core`
  outside the pack resolver.
- 40 schemas claim to govern documents; 125 markdown files live under
  `knowledge/`; zero are checked against a schema. The plan filed in this
  directory passed validation with none of `schema-adhoc-plan`'s rules
  applied.
- The globs have rotted unnoticed, which is what a dead code path looks
  like: seven match nothing (`adr` wants `decisions/adr-*.md`, the files are
  `D017-…md`; likewise `bug-report`, `interface-contract`, `research`,
  `spec`, `templates-index`, `visual-system`), seven `**/*name*.md` patterns
  match the schema file itself, and `release-notes` matches
  `node_modules/diff/release-notes.md`.
- `type` carries 14 distinct values over 117 concepts against 40 schemas:
  `Reference` is claimed by 8 schemas, `Issue` by one schema for 17
  documents. Twelve schemas declare no `type` const at all.
- Nothing consumes `type` — `aokf/mcp.rs`, `graph.rs` and `index.rs` never
  read it. SPEC line 94: "Type values are free-form and unregistered;
  consumers must tolerate unknown types."
- Of the 54 documents that dispatch to exactly one schema today, 14 carry
  141 approximate findings — and 13 of those 14 are specs failing
  identically. `schema-spec` requires a 16-section shape (`Summary`,
  `Behaviour`, `Acceptance criteria`, `Edge cases & errors`, `Test plan: …`,
  `Exit criteria`, …); every spec in the tree uses a 7-section shape
  (`Problem`, `Solution`, `Behaviour`, `Design decisions`, `Testing`, `Out of
  scope`, `Open questions`). The schema is the outlier.
- All fourteen agent-facing invocations of validation name the SOKF half and
  none names the schema half, though one command runs both: eleven skill
  gates read "knowledge validates to PASS", plus `.agents/core.md:32`,
  `.claude/settings.json:7` and `maintain/PROJECT.md:7`.
- Renames migrate themselves: "the lock is what superdev applied, the
  components' claims are what the blueprint wants now, and the difference is
  the migration" (`orphan.rs:3-5`); `apply.rs:1419-1466` proves a stale
  `.mcp.json` key is removed on the next sync.
- `superdev-core` is published to crates.io (`release.yml:278`), so the
  module rename is a public API break. The workspace is at 0.2.0 and the
  changelog permits breaking changes in a minor while pre-1.0.
- `knowledge/contracts/contract-001-content-packs.md` pins `[knowledge]
  provider = "aokf"` (`:54`), the manifest scaffold rule (`:212`) and the
  `agents/aokf.md` asset path (`:238`), so this plan is an interface change
  to a live contract.
- Baseline: 534 tests passing, `superdev validate` clean over 61 files with 5
  warnings, `status --drift` reporting 65 pre-existing entries (I016).

Four constraints bound the work. `superdev validate` over this repository
stays usable as a hook, under 250 ms on a release build against 86 ms today
before schema enforcement (NFR-1). Per-crate line coverage holds at 90%
(NFR-2). No test is dropped to keep the suite green, so the count stays at
or above 534 (NFR-3). The work adds no blueprint drift: `status --drift`
names no path this plan touched (NFR-4).

Everything lands on `feature/content-packs` rather than splitting into
structural, content and feature branches (D-17). That is the author's call;
the block order is what keeps the tree green instead.

Out of scope:

- Standardising "bundle" and "knowledgebase" onto "knowledge". That is I013,
  landing before this plan starts.
- Rewriting the completed records. The decisions, specs, feature plans,
  closed issues and P006/P007 say AOKF because that is what it was called
  when they were written; P006 and P007 already carry the same exemption.
- Enforcing SPEC §9's index shape. That is I011, and it belongs to the SOKF
  half rather than the schema layer.
- The remaining validator gaps — index entries (I010) and warning severities
  (I012). I017 is answered incidentally: the schema layer becomes the
  agent-facing statement of what a document must contain.
- Reducing the 65 blueprint-drift entries. That is I016; this plan must not
  add to the set, and does not close it.

The risks and what answers them. I013 not finishing before this starts, so
the two sweeps collide across the same 162 files: this plan does not begin
until `git grep -Ii 'bundle'` is quiet outside the changelog and the closed
records, and that grep is the gate. Block 6 resolving a disagreement the
wrong way and rewriting thirteen specs into a shape nobody uses: D-15 states
the rule, and block 6 records the judgement per schema so a reviewer can
disagree with it. The backlog being larger than measured, since only 54 of
117 concepts dispatch 1:1 today: block 6's first run is the early signal,
and if the count is large the plan stalls there rather than in block 7. The
eight `--no-knowledge` tests being re-cut into weaker tests: the early
signal is the count falling below 534, or a re-cut test asserting less than
its original. Enforcement making the hook too slow to leave installed: NFR-1
is the signal, and dispatch is a map lookup with per-document rules, so the
cost should be linear in documents rather than in schemas. Retyping breaking
a link or an id: the SOKF half already validates ids and links on the same
command. `pack/` and `.claude/` drifting apart during the sweep and widening
I016's 65: block 8 changes both sides in one step, and NFR-4 is the check.
The crates.io API break reaching a downstream nobody knew about: no
mitigation is available, and the changelog entry and the pre-1.0 policy are
the whole answer.

Three notes recorded out of band. `.superdev/cache/aokf-index/` is
gitignored machine state and is not in the lock, so nothing removes it; a
synced repo keeps an orphaned directory until its owner deletes it, and that
is not worth code. I013 is the precondition, I014 and I015 are closed by
this plan, I017 is answered incidentally and should be reviewed against this
plan's outcome rather than closed blind, and I010, I011 and I012 are
untouched. `templates/processes/` is 21 tracked Claude Code process
templates with no relation to the schema layer; the `**/*code-review*.md`
glob matches one of them by accident today, and under FR-11 it cannot.

## Contract changes

- contract-001-content-packs: the provider id `[knowledge] provider =
  "aokf"` is withdrawn — the knowledge is no longer a capability and nothing
  provides it; the manifest scaffold rule names `manifest.sokf.yaml`; and
  the binary-owned asset path moves from `agents/aokf.md` to
  `.agents/sokf.md` beside `.agents/sokf/SPEC.md`. An interface change, made
  deliberately.

## Work blocks

### Block 1: Knowledge stops being a capability

- [x] Done — ticked at merge.
- Depends-on: none.
- Change:
  1. Give the config a home — add a top-level `[knowledge]` table to
     `Manifest` carrying `custom` and `embeddings`, and take `embeddings`
     off `CapabilityConfig`. Nothing reads it yet, so the tree stays green.
  2. Loosen the component contract — `Component::capability` returns
     `Option<Capability>`, every implementation returning `Some`.
  3. Name the owner — `Owner::Capability(Capability::Knowledge)` becomes
     `Owner::Knowledge` in `content/item.rs` and `content/layout.rs`. Pack
     paths are unchanged.
  4. Make the component unconditional — `components/aokf.rs` becomes
     `components/sokf.rs`, returns `None` for its capability, and
     `components/enabled.rs` appends it to every resolution.
  5. Point the reads at the new table — the embeddings lookup and the skills
     adoption path read `[knowledge]`.
  6. Remove the slot — drop `Capability::Knowledge`, its registry entry and
     `--no-knowledge` from `manage.rs`. Hard to reverse: it deletes a
     user-facing flag and a manifest shape, so it lands with step 7.
  7. Re-cut the eight `--no-knowledge` tests — each used the flag for a
     light repo. Give them a repo with the scaffold and assert what they
     were actually asserting. Delete none.
  8. Refuse the old table by name — manifest load reports that `[knowledge]`
     is no longer a capability and names the edit.
  Decision: D-1 — knowledge leaves the capability system entirely, over
  keeping the slot with one provider; a slot states the thing is
  replaceable, which is the complaint.
  Decision: D-2 — `Component::capability` returns `Option<Capability>`, over
  adding a `Capability::Core` variant; a core component fills no slot, and
  `Planned.capability` is already optional.
  Decision: D-3 — `Owner::Capability(Knowledge)` becomes `Owner::Knowledge`,
  over folding it into `Owner::Repo`; `custom` lists are name-guarded per
  owner, and SOKF skills share `.claude/skills/` with pack skills.
  Decision: D-4 — no lever remains to run superdev without the SOKF
  knowledge, over keeping a flag or a manifest opt-out; a hatch would be the
  slot under another name.
  Decision: D-11 — an old `[knowledge]` capability table is a named error,
  over migrating it in place on load; an error naming the edit is honest,
  and migration code would outlive its purpose.
- Done-check: `Capability::ALL` has four entries, `init` writes the scaffold
  with no flag given, and an old capability table fails naming the edit.
- Cases:
  - unit: `git grep -n 'Capability::Knowledge\|no-knowledge\|no_knowledge'`
    returns nothing, and `Capability::ALL` has four entries — the registry
    carries four entries, none of them knowledge (FR-1).
  - e2e: `superdev init` in an empty scratch repo writes `knowledge/`, the
    hook and the `.mcp.json` entry with no flag given, and its `config.toml`
    holds a top-level `[knowledge]` table and no knowledge capability
    (FR-2, FR-3).
  - e2e: `superdev sync` against a config carrying `[knowledge] provider =
    "aokf"` fails naming the table and the edit (FR-3).

### Block 2: One validator module

- [x] Done — ticked at merge.
- Depends-on: none.
- Change:
  1. Create the parent — `src/validate/mod.rs` holding `validate_repo`,
     `RepoReport`, `Report` and `Finding`, moved from `format/mod.rs` and
     `aokf/validate.rs`. The merge point is now above both halves.
  2. Move the SOKF half — `aokf/validate.rs` becomes `validate/sokf.rs`.
  3. Move the schema half — `format/{check,grammar,read,re,doc}.rs` become
     `validate/schema/{check,grammar,read,re,doc}.rs`, with `grammar.yaml`.
  4. Rename the read side — `src/aokf/` becomes `src/sokf/`, and `lib.rs`
     declares `sokf` and `validate`. Hard to reverse: it breaks the
     published `superdev-core` API, so it lands in one commit with its
     changelog entry.
  5. Move the grammar file — `.agents/format/grammar.yaml` to
     `.agents/sokf/grammar.yaml`, beside the specification, and
     `load_grammar` with it. The embedded and on-disk copies stay
     byte-identical; a test already asserts it.
  6. Rename the suites and fixtures — `tests/fixtures/aokf/` to
     `tests/fixtures/sokf/`, `tests/fixtures/format/` to
     `tests/fixtures/schema/`, `format_snapshots.rs` to
     `schema_snapshots.rs`. Golden contents are relative to their case
     directory, so no golden changes; that is the check.
  Decision: D-8 — the grammar half is the schema-validator
  (`validate::schema`), over keeping "superdev-format" or coining a second
  acronym; I014 named it correctly, since its job is checking documents
  against schemas and skills are the ancillary case.
  Decision: D-9 — one `validate` module with `sokf` and `schema` beneath it,
  over one flat module; the D-18 boundary becomes structural instead of a
  doc comment.
- Done-check: neither half calls the other, and both golden trees move with
  no content change.
- Cases:
  - unit: `git grep -n 'schema' src/validate/sokf.rs` returns nothing, and
    the only `sokf` under `src/validate/schema/` is the grammar's own path
    and the four tool names it governs — no call in either direction, and
    `validate/mod.rs` names both (FR-4).
  - observation: after the move, `git diff --stat` over both golden trees
    shows no content change — only the directory renames (O2).

### Block 3: One command surface

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change:
  1. Split the CLI module — `aokf_cli.rs` becomes `validate_cli.rs` (the
     `validate` verb and the hook) and `sokf_cli.rs` (`index` and `mcp`).
  2. Set the verbs — `main.rs` gains `Hook`, `Sokf::Index` and `Mcp::Sokf`,
     and loses the `Aokf` group and `Mcp::Aokf`.
  3. Rename the hook marker — `superdev aokf hook validate` becomes
     `superdev hook validate`. Hard to reverse: the marker is the lock key in
     every managed repo, so the old element is removed by the orphan pass on
     the next sync and by nothing else.
  4. Rename the MCP registration — `mcpServers.superdev-sokf`, invoked as
     `superdev mcp sokf`.
  5. Rename the tools — `sokf_{search,read,graph,overview}`, `AokfServer` to
     `SokfServer`, and the server's instruction text. `tests/mcp_tools.rs`
     follows.
  6. Move the index cache — `.superdev/cache/aokf-index` to
     `.superdev/cache/sokf-index`.
  Decision: D-7 — `validate` and `hook` are bare while `sokf index` and `mcp
  sokf` carry the anchor, over putting all four at top level; the first two
  cover the whole repository and the last two do not.
  Decision: D-10 — no compatibility path for the old manifest, hook marker
  or MCP key, over accepting both for one release; the tree is pre-1.0, and
  the orphan pass is already the migration.
- Done-check: the help lists the new verbs and no `aokf` verb group, and the
  MCP server answers under its new name.
- Cases:
  - e2e: `superdev --help` lists `validate`, `hook`, `sokf` and `mcp`, and
    no `aokf`; `superdev sokf --help` lists `index` (FR-5).
  - integration: `superdev mcp sokf` starts, and `tests/mcp_tools.rs` drives
    `sokf_search`, `sokf_read`, `sokf_graph` and `sokf_overview` (FR-6).

### Block 4: SOKF, the format

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change:
  1. Rewrite the specification — `.agents/aokf/SPEC.md` to
     `.agents/sokf/SPEC.md`, titled "SOKF — Superdev Open Knowledge Format",
     every AOKF replaced, and §1 defining **SOKF knowledge** as the term for
     the document tree.
  2. Rewrite the instructions — `.agents/aokf.md` to `.agents/sokf.md`, the
     `<aokf-system>` element to `<sokf-system>`, the import to
     `@sokf/SPEC.md`, and the commands to block 3's verbs.
  3. Change the manifest in code — `MANIFEST` becomes `manifest.sokf.yaml`,
     `BundleManifest::aokf` becomes `sokf`, and the required-key check
     follows.
  4. Move this repository's manifest — `knowledge/manifest.aokf.yaml` to
     `knowledge/manifest.sokf.yaml`, `aokf: "0.3"` to `sokf: "0.3"`. The
     version stays 0.3: the rename changes no rule, the manifest key is the
     one on-disk change, and §2 records it.
  5. Update the eleven fixtures and regenerate — `UPDATE_GOLDENS=1` over
     both suites, then read the diff: only the manifest filename and key may
     move. Any other change is a behaviour change and stops the step.
  6. Repoint the binary-owned paths — `BINARY_OWNED` and the assets beneath
     it.
  Decision: D-5 — "SOKF knowledge" is the standing term and `sokf` anchors
  every identifier, over "knowledge" alone; `frame/SKILL.md` uses
  "knowledge" in three senses in 43 lines, and `sokf_search` on the same
  line cannot be mistaken for anything.
  Decision: D-6 — the directory stays `knowledge/`, over renaming it
  `sokf/`; it is a path in a repo whose manifest declares the format, and it
  is the most visible thing superdev writes into a user's tree.
  Decision: D-16 — the specification and the instructions stay binary-owned,
  over shipping them from the pack; unchanged rule, since they describe a
  version the binary pins and a format its compiled validator enforces.
- Done-check: the manifest, the specification and the instructions all name
  SOKF, and the regenerated goldens differ only in the manifest filename and
  key.
- Cases:
  - unit: `knowledge/manifest.sokf.yaml` holds `sokf: "0.3"`, and `git grep
    -l 'manifest.aokf.yaml'` returns nothing outside the changelog (FR-7).
  - observation: `head -1 .agents/sokf/SPEC.md` reads `# SOKF — Superdev
    Open Knowledge Format`, and §1 defines "SOKF knowledge" (FR-8).

### Block 5: A type names a schema

- [x] Done — ticked at merge.
- Depends-on: none.
- Change:
  1. Give every schema a type const — the twelve without one get theirs;
     `interface-contract` gets `Contract`, which its documents already
     carry.
  2. Split the collisions — `Reference` becomes eight distinct types across
     `api-contracts`, `architecture`, `configuration`,
     `constraints-non-goals`, `development-commands`, `directory-structure`,
     `software-components` and `technology-stack`; `Issue` splits into
     `BugReport`, `FeatureRequest` and whatever else the seventeen filed
     issues need; `Convention`, `Procedure` and `Policy` likewise.
  3. Retype the documents — every concept's frontmatter `type` follows its
     schema. Roughly 40 documents. The SOKF half already requires `type`, so
     a missed one is caught by the existing checks.
  4. Delete the dead globs — `target-files` comes off every schema that now
     dispatches by type, which is all but the frontmatter-less handful.
  5. Rewrite the issue tracker's shape — I015 is answered by step 2;
     `knowledge/issue-tracker.md` and `knowledge/issues/index.md` follow the
     new types.
  Decision: D-12 — `type` names exactly one schema, and `Reference` splits
  into eight while `Issue` splits into its kinds, over keeping `type` coarse
  and adding a `schema` or `kind` field; SPEC line 94 makes type free-form,
  nothing consumes it today, and a second field would say the same thing
  twice on every concept.
  Decision: D-13 — `target-files` survives, scoped, for documents with no
  frontmatter, over deleting it for exact paths plus an `index.md`
  convention; one dispatch mechanism rather than two, and it stays available
  for a schema that wants a pattern.
- Done-check: every schema declares a distinct type const, every concept's
  type names a schema, and `target-files` remains only where dispatch by
  type cannot reach.
- Cases:
  - unit: a test asserts every schema declares a `type` const, that no two
    are equal, and that every concept's type names a schema (FR-9).
  - unit: a test asserts `target-files` appears only on schemas whose
    documents carry no frontmatter (FR-10).

### Block 6: Reconcile the schemas to practice

- [x] Done — ticked at merge.
- Depends-on: 5.
- Change:
  1. Build the reconciliation harness — a test that runs every schema
     against the documents its type names and prints the disagreements.
     Temporary scaffolding for step 2, kept afterwards as the enforcement
     suite's fixture.
  2. Judge each disagreement — where documents agree with each other and not
     the schema, correct the schema; where a document is genuinely
     malformed, correct the document. Record which, per schema.
  3. Correct `schema-spec` first — 13 of 14 specs fail it identically, which
     is the clearest case in the tree and the one that sets the precedent.
  4. Reach zero — the harness reports no disagreement before block 7 begins.
     This is the gate; enforcement cannot land green without it.
  Decision: D-14 — schemas are reconciled against practice before
  enforcement is switched on, over shipping enforcement as warnings and
  promoting later; I012 is this repository's own evidence that a warning
  backlog goes unread.
  Decision: D-15 — where documents agree with each other and not the schema,
  the schema is wrong, over bringing every document to its schema; 13 of 14
  specs already agree on a shape the schema does not describe, and the
  practice is the evidence.
- Done-check: the harness reports zero disagreements across all 40 schemas,
  and each judgement is recorded with the side it went against.
- Cases:
  - integration: the reconciliation harness reports zero disagreements
    across all 40 schemas (FR-12).
- Record — 218 findings on the first run, zero on the last, judged as
  follows:
  - `spec`, 13 of 14 documents: the **documents** were wrong. First judged
    the schema wrong and relaxed it to what the corpus shared. That
    misapplied D-15: the thirteen did not agree with each other either — 12
    of 14 had `Out of scope`, 10 had `Testing`, nothing else above 6 — so
    the precondition was never met, and relaxing ratified the drift the
    validator exists to catch. The schema is restored in full and all
    thirteen conformed.
  - `feature-request`, 6 documents: the documents. Retyped in block 5,
    still carrying bug-report bodies. Reshaped into motivation, proposed
    behaviour, alternatives and scope.
  - `chore`, 2 documents: the documents, reshaped into surfaces and a
    definition of done.
  - `bug-report`, `feature-request` and `chore`, 9 documents: the schema.
    The resolution rule sat last; every settled issue puts it directly under
    the title, so it moved. `Comments` split off and stays last, per the
    tracker's own convention.
  - `adhoc-plan`, P004: the document. It predates the schema and used the
    four headings it prohibits. Reorganised — its "Current state" was
    already Facts with evidence attached — and its Outcomes, Non-goals,
    Requirements and Definition of done written from its own content.
  - `feature-plan`, P001 and P002: the documents. They predate the schema,
    and their numbered task lists were slice lists without the headings.
  - `readme`, README.md: the **document**, for the same reason as `spec`.
    One document is not a corpus, so "the documents agree" was vacuous. The
    schema is restored and the README gained `Quick start`, `Usage` and
    `Configuration` — which it needed anyway, and which found a stale
    `--no-knowledge` in the prose.
  - `architecture`, `coding-standards`, `constraints-non-goals` and
    `software-components`, 4 documents: the documents. Each qualified a
    required heading — "CI/CD (`.github/workflows`)"; the qualifier moved
    into the prose.
  - `issue-tracker`, 1 document: the document. A table column read "Tag in
    this repo" where the shipped schema declares "Label".
  - `bug-report`, I009: the document. No regression risk section; a settled
    report should still say what would catch a recurrence.
- Note: the pre-reconciliation backlog was measured with a probe applying
  required sections, `heading-pattern`, `sections-prohibited` and
  `line-limit` to the 54 documents that dispatched to exactly one schema —
  14 documents, 141 findings, 13 of the 14 specs failing `schema-spec`
  identically. The probe understated, as it said it would: it checked
  neither content kinds, table columns nor section order, and it predated
  block 5, so the real first run reported 218. Of those 218, 143 were first
  resolved by relaxing a schema and then re-resolved by conforming the
  documents — 140 for `schema-spec` and 3 for `schema-readme`. The lesson is
  D-15's, sharpened: the test is whether the documents agree **with each
  other**. Inconsistent documents are evidence of inconsistency, not of a
  wrong standard, and a validator that bends to them ratifies drift. Every
  other judgement above went to the documents from the start. The twelve
  older specs are conformed in shape with thin content where the record does
  not carry more: no test plan was written for them at the time, so their
  plans name the automated cases that exist and say plainly that no manual
  step was recorded, rather than inventing one.

### Block 7: Enforcement

- [x] Done — ticked at merge.
- Depends-on: 2, 6.
- Change:
  1. Dispatch by type — `validate::schema` resolves a concept to its schema
     through the frontmatter `type`, and reports a type naming no schema and
     a schema that governs nothing.

     Corrected during execution. This step and FR-14 first said "a schema no
     type names", which is a different claim: a schema whose type no
     document *yet* carries. Nine of the forty-one are exactly that —
     `code-review`, `investigation`, `migration-guide`, `postmortem`,
     `release-notes`, `research`, `security-review`, `status-update`,
     `visual-system` — kinds nobody has written here. Reporting them would
     have made the first run fail on nine contracts doing their job, and the
     only way to silence it would be deleting the contract or writing a
     document to satisfy it. What ships instead is the decidable case: a
     schema declaring neither a type const nor a `target-files` glob can
     reach no document ever, whatever anyone writes later. That is reported,
     and the `governs-nothing` fixture is its test.
  2. Scope the glob — `target-files` resolves only for frontmatter-less
     documents, confined to the SOKF knowledge root plus a named allowlist,
     refusing anything under `knowledge/schemas/` or outside the repository.
     The seven self-matching patterns and the `node_modules` reach are the
     tests.
  3. Apply the section rules — required sections, `sections-ordered`,
     `sections-prohibited` and `heading-pattern`, reported per document.
  4. Apply the content rules — declared table columns, `content` kinds and
     `line-limit`.
  5. Add the fixtures — one case per rule under `tests/fixtures/schema/`,
     each with its golden, in the shape the existing snapshot suites use,
     driven by `tests/document_snapshots.rs`.
  6. Correct the fourteen invocations — the eleven skill gates and the three
     command sites say what they now check, rather than naming only the SOKF
     half.
- Done-check: a document that breaks its schema fails `superdev validate`
  with a message naming the rule, and the glob reaches nothing outside the
  knowledge root.
- Cases:
  - unit: a test feeds `**/*release-notes*.md` and asserts the resolver
    refuses `node_modules/`, refuses `knowledge/schemas/`, and stays inside
    the repository (FR-11).
  - integration: one fixture case per rule — missing section, misordered
    section, prohibited section, wrong table column, over-limit — each
    failing with its own message and its own golden (FR-13).
  - integration: a fixture with a type naming no schema, and one schema
    declaring neither a type const nor a glob, are both reported —
    `unknown-type` and `governs-nothing` (FR-14).

### Block 8: The sweep

- [x] Done — ticked at merge.
- Depends-on: 1, 2, 3, 4, 5, 7.
- Change:
  1. Move the pack's instruction files — `pack/aokf/agents/**` to
     `pack/sokf/agents/**`. `classify` matches neither, which is what keeps
     them binary-owned; `paths_matching_no_rule_are_not_items` gains both
     new paths.

     Corrected during execution. This step first said
     `pack/knowledge/agents/`, which `classify` does indeed ignore — but
     `pack::manifest::REJECTED` guards the paths a *fetched* pack may not
     supply, and superdev's own pack then matched its own guard, so the pack
     stopped resolving. The old layout used a top-level directory for
     exactly this separation; `pack/sokf/` keeps it under the new name.
  2. Sweep the skills — the 21 under `.claude/skills/` and their
     `pack/knowledge/skills/` mirrors, both sides in one step so the drift
     set does not grow.
  3. Change the contract — C001 pins the provider id, the manifest scaffold
     name and the asset path, all three of which move.
  4. Sweep the concepts — the 18 live concepts and the schemas that name a
     tool, a verb or the manifest. The completed records are left alone.
  5. Sweep the entry documents and scripts — `.agents/core.md`,
     `pack/agents/*.md`, `README.md`, `CONTRIBUTING.md`, `.gitattributes`,
     `scripts/superdev`, `scripts/manage-smoke.sh`.
  6. Re-sync this repository — `.superdev/config.toml` loses its
     `[knowledge]` capability table and gains the top-level one; the lock
     and `.mcp.json` follow from `superdev sync`, never hand-edited.
  7. Record the breaks — one changelog entry per surface: the capability,
     the crate API, the verbs, the MCP server, the manifest, and
     enforcement.
  8. Close what this supersedes — I014 and I015 become done, naming this
     plan.
- Done-check: no live file says AOKF, a repo built by the previous release
  migrates on one `sync`, the old grammar path, fixture roots, cache
  directory and asset directories are absent, and the full check set is
  green.
- Cases:
  - observation: `git grep -Ii 'aokf' -- . ':!CHANGELOG.md'
    ':!knowledge/plans' ':!knowledge/decisions' ':!knowledge/specs'
    ':!knowledge/issues'` returns only deliberate references: the tests
    asserting the old verbs are gone, the tripwire refusing a pre-SOKF
    manifest, the snapshot header's note on the Python reference, and two
    links to S002, whose id is historical (FR-8).
  - e2e: a fixture repo carrying the previous release's lock, `.mcp.json`,
    `.claude/settings.json` and `.agents/aokf*` syncs to the new state, and
    a second `status --drift` exits 0 (FR-15).
  - unit: `binary_owned_count()` is 2 and names the two `.agents/sokf/`
    paths; every skill, template and concept traces to a pack item (FR-16).
  - unit: `read_pack("superdev", &root)` resolves — the binary-owned pair
    sits outside the `agents/` position `REJECTED` guards (FR-16).
  - e2e: `superdev validate` over this repository reports PASS, with the 5
    known warnings and no errors (FR-12, FR-13).
  - e2e: `cargo nextest run --workspace` passes with at least 534 tests, and
    `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --
    -D warnings`, `cargo test --doc` and `RUSTDOCFLAGS="-D warnings" cargo
    doc` are clean (NFR-3).
  - e2e: `npm run coverage:check` passes (NFR-2), `time ./target/release/superdev
    validate` over this repository is under 250 ms (NFR-1), and `superdev
    status --drift` names no path this plan touched (NFR-4).
