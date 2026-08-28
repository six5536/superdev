---
type: AdhocPlan
id: adhoc-plan-009-drop-the-bash-output-filter
title: Drop rtk and the bash-output-filter capability
description: The bash-output-filter slot, its rtk provider, the five things it owns and the flag that disabled it all leave, and a manifest still naming the table gets a guided error.
status: done
---

# Plan: Drop rtk and the bash-output-filter capability

## Context

The bash-output-filter capability rewrites every Bash command through rtk to
compact its output, and the compaction is not worth what it costs the agent
reading it. rtk is the slot's only provider and the slot exists for rtk, so
the two go together: a capability with no provider is a manifest key that can
only be switched off. The removal has to reach a managed repo that already
has the capability, which shapes the order below.

## Facts

- `Capability::BashOutputFilter` is a `Single` slot with one provider, `rtk`
  (`capability.rs:24`, `registry.rs:81-90`). `rtk`, `BashOutputFilter` and
  `bash-output-filter` appear 84 times across 12 Rust files under `crates/`.
- The component owns five things (`components/rtk.rs:67-100`): `.miserc.toml`,
  `mise.unix.toml`, `mise.windows-x64.toml`, `.agents/rtk.md`, and the
  `hooks.PreToolUse` element in `.claude/settings.json` keyed by
  `mise exec http:rtk -- rtk hook claude`. This repository carries all five,
  recorded at `.superdev/lock.toml:25,28,57-59`.
- The orphan pass removes every lock `files` entry no claim covers, whatever
  its shape, and releases rather than removes one whose content has changed
  (`orphan.rs:44-63`). So deleting the manifest table and running sync sweeps
  all five — while the binary still carries the component.
- A retired capability is refused by name: `Manifest::parse` rejects
  `[knowledge]` and `[workflows]` with a message naming the edit
  (`manifest.rs:217-235`), tested at `manifest.rs:443-451`.
- A retired spec keeps its document and takes `status: deprecated` —
  `S005-workflows-provider-default-design.md:6` and
  `S006-workflows-skill-overrides-design.md:6`, both left listed in
  `specs/index.md` and still cited in prose by `S008:25`.
- `--no-bash-output-filter` is declared at `manage.rs:36-39`, mapped at
  `manage.rs:53`, and used at 12 sites in `crates/app/superdev/tests/cli.rs`.
- The aggregator writes `@rtk.md` when the capability is enabled
  (`pipeline.rs:469-471`), asserted at `pipeline.rs:1037,1043`.
- `pack/rtk/rtk.md` reaches the binary through the `assets` symlink
  (`components/rtk.rs:30`). It is content in neither direction: `layout.rs:201`
  names it in the not-content list, and `snapshot.rs:156` counts it as one of
  the two binary-owned instruction files. `build.rs` generates the embedded
  file list from the pack tree, so deleting the file needs no second edit.
- `pack::manifest::REJECTED` refuses an external pack carrying
  `agents/rtk.md` (`pack/manifest.rs:20-26`), because the binary owns that
  file.
- `auto_env` has no second consumer: `rg auto_env` returns only rtk's own code,
  its spec, `.miserc.toml` and its tests. codegraph pins in `.mise.toml`
  directly (`.mise.toml:18`), so removing the platform config files leaves it
  installed.
- `MANAGED_MISE_TOOLS` holds codegraph's key alone (`components/enabled.rs:14`)
  — rtk's pin lives in the platform files, so that constant does not change.
- This repository pins rtk a second time for developers, unmanaged, at
  `.mise.toml:13`.
- Eleven knowledge documents name rtk or the capability: `architecture.md:71`,
  `glossary.md:13`, `api-contracts.md:15,84-86`, `configuration.md` (frontmatter
  and lines 21-22, 324-343), `index.md:19`, `software-components.md:45`,
  `directory-structure.md:41`, `development-procedure.md:39-43`. Two more
  state what is true now: `C001:232-241` quotes `pack::manifest::REJECTED`
  verbatim and is a live contract (`status: draft`), and `S014:94` lists the
  instruction files that exist. `S011:141`, `P003:80`, `S012` and the
  `specs/index.md` entry are records of their own moment.
- `parse_target` rejects an unknown update target with ``unknown capability
  `<name>` `` and no listing (`manage.rs:379-386`); `workflows` has been
  refused that way since it was dropped (`manage.rs:498`).
- `cargo check --workspace --all-targets` fails today in `pipeline.rs`: P008 is
  in flight in the working tree, uncommitted. Resolved before this plan ran:
  P008 landed, and the baseline was clean at 559 tests.

## Goal

superdev manages three capabilities, no repository it manages carries an rtk
pin or a command-rewriting hook, and a manifest that still names the fourth is
told so.

## Outcomes

- O1 — no file under `crates/`, `pack/`, `README.md` or `.mise.toml` names rtk
  or the capability, and the binary offers three capability slots.
- O2 — this repository carries none of the five owned things, and
  `superdev status --drift` exits 0.
- O3 — a manifest carrying `[bash-output-filter]` is refused with a message
  naming the deletion and what sync then removes.
- O4 — the knowledge describes three capabilities, and the spec that
  introduced the fourth reads deprecated rather than current.

## Non-goals

- Replacing the filtering with anything. The decision is that Bash output
  needs no rewriting, not that it needs a different rewriter.
- Rewriting the documents that mention the capability in their own tense —
  `S011`, `P003` and the `specs/index.md` entry. They record what was true
  when they were written, as `S008` still does for the dropped workflows
  capability. `C001` and `S014` are excluded: the first is a live contract
  quoting a constant this plan changes, and the second names the instruction
  files that exist, so both are corrected in W5.
- Automatic removal for a repository that has already run init. The guided
  error names the one-line edit, and sync does the rest; superdev rewrites no
  manifest it does not own.
- Anything in P008. The two plans touch the same five files, and this one
  starts from P008's merged tree.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | No file under `crates/`, `pack/`, `README.md` or `.mise.toml` names rtk, `BashOutputFilter` or `bash-output-filter` | O1 |
| FR-2 | `Manifest::parse` refuses a manifest carrying `[bash-output-filter]`, naming the table to delete and the files sync then removes | O3 |
| FR-3 | With the table deleted, sync removes the three mise files, the instruction file and the `PreToolUse` element, and prunes their lock entries | O2 |
| FR-4 | The `.agents/superdev.md` aggregator carries no `@rtk.md` import | O2 |

FR-4 holds for a managed repository and could not be checked here: this
repository has no `.agents/superdev.md`. Its `AGENTS.md` reads
`@.agents/core.md`, and `status --drift` has reported the aggregator missing
since before this plan. The import was removed at its source, `pipeline.rs`,
and `aggregator_imports_track_the_enabled_set` is what covers it.
| FR-5 | `superdev init` offers three capability-disable flags, and `superdev update` names three capabilities | O1 |
| FR-6 | The knowledge names three capabilities, and `S012` reads `status: deprecated` | O4 |

### Non-functional

| ID | Constraint | Budget |
|----|------------|--------|
| NFR-1 | A user-edited copy of any of the five owned things survives the sweep — released from the lock, left on disk | zero deletions where the on-disk content differs from the recorded hash |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | The slot goes with the provider | Keep `bash-output-filter` with no provider registered | `Capability::ALL` drives the flags, the registry, the manifest keys and the update targets, so an empty slot is dead surface on every one of them |
| D-2 | A manifest naming the table is refused with a guided message | Ignore it as an unknown capability | the `[workflows]` and `[knowledge]` precedent — a user who chose the capability is told it went, and told the edit that sweeps its files |
| D-3 | The sweep runs on this repository before the code is removed | Remove the code first and delete the five by hand | the orphan pass needs a binary that still knows the component; by hand, the lock and `.claude/settings.json` would be edited rather than planned |
| D-4 | `.miserc.toml` and both platform pin files go | Keep `.miserc.toml` for a later `auto_env` consumer | there is no other consumer, and an owned file with no owner is exactly what the orphan pass removes |
| D-5 | `agents/rtk.md` leaves `pack::manifest::REJECTED` | Leave the path refused | the entry exists because the binary owns that file; once nothing does, the refusal names a file with no meaning |
| D-6 | `S012` takes `status: deprecated` and stays | Delete the spec | `S005` and `S006` record the dropped workflows capability the same way; the spec is the record of why this one existed |
| D-7 | The unmanaged dev pin at `.mise.toml:13` goes too | Keep rtk available to developers | no command in the repository reaches rtk once the hook is gone |
| D-8 | This lands after P008 | Fold it into the in-flight work | both remove a capability and touch the same five files; landing them apart keeps a compiling baseline to verify each against |

## Workstreams

### W1: Sweep this repository

Depends on: none. Runs against a binary built from a tree that still carries
the component, so it must complete before W2.

1. Delete the table — `[bash-output-filter]` leaves `.superdev/config.toml`,
   so `enabled` stops resolving the component and its five claims lose their
   owner.
2. Sync — `cargo run -- sync` removes `.miserc.toml`, `mise.unix.toml`,
   `mise.windows-x64.toml` and `.agents/rtk.md`, drops the `PreToolUse`
   element from `.claude/settings.json`, rewrites `.agents/superdev.md`
   without `@rtk.md`, and prunes the five lock entries.

   Corrected during execution, on the author's instruction. A `sync` of this
   repository would also overwrite the nineteen skills the schema migration
   rewrote with the pre-grammar copies the pack still carries — I016, the same
   reason P008's own re-sync step was never completed. The sweep was therefore
   applied by hand to exactly what the orphan pass would have planned, after
   checking all four files against their recorded hashes so NFR-1's release
   case was known not to apply. `.agents/superdev.md` does not exist here, so
   there was no aggregator to rewrite.
3. Commit the sweep on its own. Hard to reverse: after W2 no binary can plan
   these files again, so a revert that reaches them has to reach this commit
   alone, and the commit is also the evidence FR-3 is checked against.

### W2: Remove the capability from the core

Depends on: W1.

1. Delete the component — `components/rtk.rs`, its `pub mod rtk;`
   (`components/mod.rs:8`), and its import and dispatch arm
   (`components/enabled.rs:7,58`).
2. Delete the slot — `Capability::BashOutputFilter` with its `ALL` entry and
   `as_str` arm (`capability.rs:24,35,44`), and the registry entry with
   `RTK_VERSION`, `RTK_PLATFORMS` and their version test
   (`registry.rs:81-136,259-263`). `ALL` and `ENTRIES` become three.
3. Drop the aggregator import — `pipeline.rs:469-471` and its two assertions
   (`pipeline.rs:1037,1043`); `enabled_skips_disabled_capabilities` expects
   three components rather than four (`components/enabled.rs:156-165`).
4. Refuse the retired table — a `bash-output-filter` arm beside `workflows` in
   `Manifest::parse` (`manifest.rs:230`), with a test beside
   `a_workflows_table_gets_the_guided_error`. After step 2, so that arm is the
   only path a manifest naming the table can take.
5. Delete the instruction file — `pack/rtk/rtk.md` and the `include_str!` that
   reads it. `build.rs` regenerates the embedded list, so the only edits are
   `snapshot.rs:156`, which now counts one binary-owned instruction file, and
   `layout.rs:201`, which drops the path from the not-content list.
6. Stop rejecting the path — `agents/rtk.md` leaves
   `pack::manifest::REJECTED` (`pack/manifest.rs:24`), per D-5.

### W3: Remove the CLI surface

Depends on: W2.

1. Delete the flag — `no_bash_output_filter` and its mapping
   (`manage.rs:36-39,53,450`).
2. Update the tests — the 12 `--no-bash-output-filter` sites in `tests/cli.rs`,
   the init-journey assertions (`tests/manage.rs:182-201`), and the disable
   journey (`tests/manage.rs:533-565`), which has no capability left to
   disable.
3. Correct the README — the flag list at `README.md:18`, and the manifest
   table's capability row at `README.md:132`, which this step first missed.

### W4: Remove what sits outside the blueprint

Depends on: W2.

1. Drop the dev pin — `"github:rtk-ai/rtk"` leaves `.mise.toml:13`, per D-7.
2. Correct the project template's prose — the Dockerfile comment
   (`pack/projects/rust-npm/devcontainer/Dockerfile:4`) and post-create's note
   about the platform config files (`.../scripts/post-create.sh:10-12`) name
   codegraph alone.

### W5: Update the knowledge

Depends on: W2, W3.

1. The capability set — `architecture.md:71` loses its table row,
   `glossary.md:13` the name, `api-contracts.md:15,84-86` the flag and the
   update targets, and `software-components.md:45` and
   `directory-structure.md:41` their rtk references.
2. The configuration concept — `configuration.md`: the manifest example
   (21-22), the `.claude/settings.json` merge notes (324-332), the owned-files
   section (333-343) and the frontmatter description. `index.md:19` carries
   that description too, so it changes in the same step.
3. The development procedure — `development-procedure.md:39-43` names the
   capabilities this repository enables on itself.
4. The content-pack documents that state what is true now — `C001:232-241`
   quotes `REJECTED` verbatim and binds, so it drops `agents/rtk.md` with the
   code; `S014:94` lists the instruction files that exist and drops
   `.agents/rtk.md` from that list.
5. Deprecate the spec — `S012-bash-output-filter-design.md` frontmatter reads
   `status: deprecated`, following `S005` and `S006`.
6. Record the change — a `Removed` entry under `## [Unreleased]` in
   `CHANGELOG.md` naming the guided error and the files sync removes.
7. Close the plan — this document's status becomes `done`.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `.superdev/config.toml` | modified — the `[bash-output-filter]` table deleted | W1 |
| `.superdev/lock.toml` | modified — the five claims pruned | W1 |
| `.miserc.toml` | deleted — swept | W1 |
| `mise.unix.toml` | deleted — swept | W1 |
| `mise.windows-x64.toml` | deleted — swept | W1 |
| `.agents/rtk.md` | deleted — swept | W1 |
| `.claude/settings.json` | modified — the `PreToolUse` element removed | W1 |
| `.agents/superdev.md` | modified — the `@rtk.md` import dropped | W1 |
| `crates/lib/superdev-core/src/components/rtk.rs` | deleted — the component and its tests | W2 |
| `crates/lib/superdev-core/src/components/mod.rs` | modified — the module declaration dropped | W2 |
| `crates/lib/superdev-core/src/components/enabled.rs` | modified — the import, the dispatch arm and the component-count test | W2 |
| `crates/lib/superdev-core/src/capability.rs` | modified — the variant, `ALL` and `as_str` | W2 |
| `crates/lib/superdev-core/src/registry.rs` | modified — the entry, the version, the platform table and its test | W2 |
| `crates/lib/superdev-core/src/pipeline.rs` | modified — the aggregator import and its assertions | W2 |
| `crates/lib/superdev-core/src/manifest.rs` | modified — the guided error and its test | W2 |
| `crates/lib/superdev-core/src/content/snapshot.rs` | modified — one binary-owned instruction file, not two | W2 |
| `crates/lib/superdev-core/src/content/layout.rs` | modified — `rtk/rtk.md` leaves the not-content list | W2 |
| `crates/lib/superdev-core/src/pack/manifest.rs` | modified — `agents/rtk.md` leaves `REJECTED` | W2 |
| `pack/rtk/rtk.md` | deleted — the instruction file the binary embedded | W2 |
| `crates/app/superdev/src/manage.rs` | modified — the flag and its mapping | W3 |
| `crates/app/superdev/tests/cli.rs` | modified — the 12 flag sites | W3 |
| `crates/app/superdev/tests/manage.rs` | modified — the init assertions, and the disable journey deleted | W3 |
| `README.md` | modified — the flag list | W3 |
| `.mise.toml` | modified — the unmanaged rtk dev pin removed | W4 |
| `pack/projects/rust-npm/devcontainer/Dockerfile` | modified — the comment names codegraph alone | W4 |
| `pack/projects/rust-npm/devcontainer/scripts/post-create.sh` | modified — the platform-config-files note | W4 |
| `knowledge/architecture.md` | modified — the capability table row | W5 |
| `knowledge/glossary.md` | modified — the capability list | W5 |
| `knowledge/api-contracts.md` | modified — the init flags and the update targets | W5 |
| `knowledge/software-components.md` | modified — the pin reference | W5 |
| `knowledge/directory-structure.md` | modified — the instruction-file list | W5 |
| `knowledge/configuration.md` | modified — the example, the merge notes, the owned-files section, the description | W5 |
| `knowledge/index.md` | modified — the configuration entry's description | W5 |
| `knowledge/development-procedure.md` | modified — the capabilities this repo enables | W5 |
| `knowledge/contracts/C001-content-packs.md` | modified — the quoted `REJECTED` constant | W5 |
| `knowledge/specs/S014-content-packs-design.md` | modified — the instruction-file list | W5 |
| `knowledge/specs/S012-bash-output-filter-design.md` | modified — `status: deprecated` | W5 |
| `CHANGELOG.md` | modified — a `Removed` entry under `[Unreleased]` | W5 |
| `knowledge/adhoc-plans/P009-drop-the-bash-output-filter.md` | new — this plan | W5 |
| `knowledge/adhoc-plans/index.md` | modified — this plan listed | W5 |

## Acceptance

| Check | Verifies |
|-------|----------|
| `rg 'rtk\|bash-output-filter\|BashOutputFilter' crates pack README.md .mise.toml` returns only `manifest.rs`'s guided error and its test — FR-2 requires that message to name the table and the five files, so this row cannot be literally empty | FR-1 |
| `cargo check --workspace --all-targets` is clean and `cargo test --workspace` passes | FR-1 |
| `superdev init --help` lists `--no-frontend`, `--no-skills` and `--no-code-index`, and no fourth flag | FR-5 |
| `superdev update bash-output-filter` fails with ``unknown capability `bash-output-filter` ``, as `workflows` does today | FR-5 |
| `cargo test -p superdev-core manifest::` passes, including the new test: a config carrying `[bash-output-filter]` is refused with a message naming the table | FR-2 |
| `git show <W1 commit> --stat` lists `.miserc.toml`, `mise.unix.toml`, `mise.windows-x64.toml` and `.agents/rtk.md` deleted, with `.claude/settings.json`, `.agents/superdev.md` and `.superdev/lock.toml` modified | FR-3, FR-4 |
| `superdev status --drift` names no path this plan touched, and its count is unchanged at 65. It exits 1, not 0, on I016's pre-existing entries — a precondition this plan did not set and does not clear | O2 |
| `cargo test -p superdev-core orphan::` passes, `each_shape_classifies_by_disk_state` included | NFR-1 |
| `rg 'rtk\|bash-output-filter' knowledge/` returns hits only in `S011`, `S012`, `P003`, `specs/index.md`, `adhoc-plans/index.md` and this plan | FR-6 |
| `superdev validate` reports PASS over `knowledge/` | FR-6 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- `npm run check:validate` and `npm run check:blueprint` pass, and clippy
  `--all-targets -- -D warnings` is clean.
- Line coverage stays ≥ 90% per crate.
- `CHANGELOG.md` carries the removal under `[Unreleased]`, naming the manifest
  edit a user has to make.
- `knowledge/adhoc-plans/index.md` lists this plan, and its status reads
  `done`.
- P008 has landed, per D-8.

## Risks

- Risk: W1's sweep commit and W2's removal land out of order, leaving a
  binary that plans rtk against a repository whose manifest no longer names it
  — or the reverse — mitigation: all five workstreams land in one pull
  request, W1 first.
- Risk: a managed repository has edited one of the five owned things, so the
  sweep releases rather than removes it and an rtk pin survives — early
  signal: `superdev status` reports it released from the lock; the guided
  error names every file so the reader can finish by hand.
- Risk: removing `.miserc.toml` turns off `auto_env` for a developer who had
  come to rely on the platform config files — mitigation: codegraph, the only
  other pinned binary, is pinned in `.mise.toml` and is unaffected.
- Risk: the knowledge sweep misses a mention, leaving the documents claiming
  four capabilities — early signal: the final `rg` over `knowledge/` returns a
  file outside the historical list.

## Out-of-band notes

The mise version floor goes with the capability: `MISE_FLOOR` was the only
thing in superdev requiring mise 2026.8 or newer, so a managed repository
works on an older mise again once this lands. `configuration.md` states the
floor and loses it in W5.
