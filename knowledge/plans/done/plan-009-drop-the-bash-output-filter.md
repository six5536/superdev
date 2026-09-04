---
type: Plan
id: plan-009-drop-the-bash-output-filter
title: Drop rtk and the bash-output-filter capability
description: The bash-output-filter slot, its rtk provider, the five things it owns and the flag that disabled it all leave, and a manifest still naming the table gets a guided error.
lifecycle: done
---

# Plan: Drop rtk and the bash-output-filter capability

## Goal

superdev manages three capabilities, no repository it manages carries an
rtk pin or a command-rewriting hook, and a manifest that still names the
fourth is told so. The bash-output-filter capability rewrites every Bash
command through rtk to compact its output, and the compaction does not
repay what it costs the agent reading it. rtk is the slot's only provider
and the slot exists for rtk, so the two go together: a capability with no
provider is a manifest key a user can only switch off. `Capability::ALL`
drives the flags, the registry, the manifest keys and the update targets,
so an empty slot leaves dead surface on each of them.

Once the work lands, no file under `crates/`, `pack/`, `README.md` or
`.mise.toml` names rtk, `BashOutputFilter` or `bash-output-filter`; the
binary offers three capability slots; this repository carries none of the
five things the component owns; a manifest carrying
`[bash-output-filter]` is refused with a message naming the table to
delete and the files sync then removes; and the knowledge describes three
capabilities, with `S012` reading `status: deprecated`.

The evidence the design rests on:

- `Capability::BashOutputFilter` is a `Single` slot with one provider,
  `rtk` (`capability.rs:24`, `registry.rs:81-90`). `rtk`,
  `BashOutputFilter` and `bash-output-filter` appear 84 times across 12
  Rust files under `crates/`.
- The component owns five things (`components/rtk.rs:67-100`):
  `.miserc.toml`, `mise.unix.toml`, `mise.windows-x64.toml`,
  `.agents/rtk.md`, and the `hooks.PreToolUse` element in
  `.claude/settings.json` keyed by `mise exec http:rtk -- rtk hook
  claude`. This repository carries all five, recorded at
  `.superdev/lock.toml:25,28,57-59`.
- The orphan pass removes every lock `files` entry no claim covers,
  whatever its shape, and releases rather than removes one whose content
  has changed (`orphan.rs:44-63`). Deleting the manifest table and
  running sync therefore sweeps all five, while the binary still carries
  the component.
- `Manifest::parse` refuses a retired capability by name, as it refuses
  `[knowledge]` and `[workflows]` with a message naming the edit
  (`manifest.rs:217-235`, tested at `manifest.rs:443-451`).
- A retired spec keeps its document and takes `status: deprecated`, as
  `spec-005-workflows-provider-default.md:6` and
  `spec-006-workflows-skill-overrides.md:6` do, both left listed in
  `specs/index.md` and still cited in prose by `S008:25`.
- `--no-bash-output-filter` is declared at `manage.rs:36-39`, mapped at
  `manage.rs:53`, and used at 12 sites in
  `crates/app/superdev/tests/cli.rs`.
- The aggregator writes `@rtk.md` when the capability is enabled
  (`pipeline.rs:469-471`), asserted at `pipeline.rs:1037,1043`.
- `pack/rtk/rtk.md` reaches the binary through the `assets` symlink
  (`components/rtk.rs:30`). It is content in neither direction:
  `layout.rs:201` names it in the not-content list, and
  `snapshot.rs:156` counts it as one of the two binary-owned instruction
  files. `build.rs` generates the embedded file list from the pack tree,
  so deleting the file needs no second edit.
- `pack::manifest::REJECTED` refuses an external pack carrying
  `agents/rtk.md` (`pack/manifest.rs:20-26`), because the binary owns
  that file.
- `auto_env` has no second consumer: `rg auto_env` returns only rtk's own
  code, its spec, `.miserc.toml` and its tests. codegraph pins in
  `.mise.toml:18` directly, so removing the platform config files leaves
  it installed, and `MANAGED_MISE_TOOLS` holds codegraph's key alone
  (`components/enabled.rs:14`) because rtk's pin lives in the platform
  files.
- This repository pins rtk a second time for developers, unmanaged, at
  `.mise.toml:13`. No command in the repository reaches rtk once the hook
  is gone.
- Eleven knowledge documents name rtk or the capability:
  `architecture.md:71`, `glossary.md:13`, `api-contracts.md:15,84-86`,
  `configuration.md` (frontmatter and lines 21-22, 324-343),
  `index.md:19`, `software-components.md:45`,
  `directory-structure.md:41`, `development-procedure.md:39-43`. Two more
  state what is true now: `C001:232-241` quotes
  `pack::manifest::REJECTED` verbatim and is a live contract, and
  `S014:94` lists the instruction files that exist.
- `parse_target` refuses an unknown update target with ``unknown
  capability `<name>` `` and no listing (`manage.rs:379-386`), as it has
  refused `workflows` since that capability went (`manage.rs:498`).

Out of scope: replacing the filtering, since the decision is that Bash
output needs no rewriting rather than a different rewriter; rewriting the
documents that mention the capability in their own tense, namely `S011`,
`P003` and the `specs/index.md` entry, which record what was true when
they were written as `S008` does for the dropped workflows capability;
and automatic removal for a repository that has already run init, since
the guided error names the one-line edit and sync does the rest.
`C001` and `S014` are the two exceptions, corrected in Block 5: the first
is a live contract quoting a constant this plan changes, and the second
names the instruction files that exist.

P008 is also out of scope, and this plan starts from its merged tree.
Both remove a capability and touch the same five files, so landing them
apart keeps a compiling baseline to verify each against. `cargo check
--workspace --all-targets` failed while this plan was written, with P008
uncommitted in the working tree; P008 then landed and the baseline was
clean at 559 tests.

The mise version floor goes with the capability: `MISE_FLOOR` was the
only thing in superdev requiring mise 2026.8 or newer, so a managed
repository works on an older mise again once this lands.
`configuration.md` states the floor and loses it in Block 5.

All five blocks land in one pull request, Block 1 first. A binary that
plans rtk against a repository whose manifest no longer names it, or the
reverse, is the state the ordering prevents.

## Contract changes

- contract-001 (content packs): the quoted `pack::manifest::REJECTED`
  constant drops `agents/rtk.md`, so the contract no longer promises that
  an external pack carrying that path is refused.

## Work blocks

### Block 1: Sweep this repository

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: delete the `[bash-output-filter]` table from
  `.superdev/config.toml`, so `enabled` stops resolving the component and
  its five claims lose their owner; then run `cargo run -- sync`, which
  removes `.miserc.toml`, `mise.unix.toml`, `mise.windows-x64.toml` and
  `.agents/rtk.md`, drops the `PreToolUse` element from
  `.claude/settings.json`, rewrites `.agents/superdev.md` without
  `@rtk.md`, and prunes the five lock entries. The block runs against a
  binary built from a tree that still carries the component, so it
  completes before Block 2, and it commits on its own: after Block 2 no
  binary can plan these files again, so a revert that reaches them has to
  reach this commit alone. Corrected during execution, on the author's
  instruction: a `sync` of this repository would also overwrite the
  nineteen skills the schema migration rewrote with the pre-grammar
  copies the pack still carries — I016, the same reason P008's own
  re-sync step was never completed. The sweep was therefore applied by
  hand to exactly what the orphan pass would have planned, after checking
  all four files against their recorded hashes. `.agents/superdev.md`
  does not exist here, so there was no aggregator to rewrite.
- Done-check: `git show <this block's commit> --stat` lists
  `.miserc.toml`, `mise.unix.toml`, `mise.windows-x64.toml` and
  `.agents/rtk.md` deleted, with `.claude/settings.json`,
  `.agents/superdev.md` and `.superdev/lock.toml` modified.
- Cases:
  - e2e: with the table deleted, sync removes the three mise files, the
    instruction file and the `PreToolUse` element, and prunes their lock
    entries — no criterion.
  - e2e: `superdev status --drift` names no path this plan touched, and
    its count is unchanged at 65. It exits 1, not 0, on I016's
    pre-existing entries, a precondition this plan did not set and does
    not clear.
  - unit: `cargo test -p superdev-core orphan::` passes,
    `each_shape_classifies_by_disk_state` included — checks that a
    user-edited copy of an owned file is released from the lock and left
    on disk, with zero deletions where the content differs from the
    recorded hash.

### Block 2: Remove the capability from the core

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: delete the component — `components/rtk.rs`, its `pub mod rtk;`
  (`components/mod.rs:8`), and its import and dispatch arm
  (`components/enabled.rs:7,58`). Delete the slot —
  `Capability::BashOutputFilter` with its `ALL` entry and `as_str` arm
  (`capability.rs:24,35,44`), and the registry entry with `RTK_VERSION`,
  `RTK_PLATFORMS` and their version test (`registry.rs:81-136,259-263`),
  leaving `ALL` and `ENTRIES` at three. Drop the aggregator import
  (`pipeline.rs:469-471`) and its two assertions
  (`pipeline.rs:1037,1043`), and expect three components in
  `enabled_skips_disabled_capabilities` (`components/enabled.rs:156-165`).
  Refuse the retired table with a `bash-output-filter` arm beside
  `workflows` in `Manifest::parse` (`manifest.rs:230`), naming the table
  and the files sync removes, with a test beside
  `a_workflows_table_gets_the_guided_error`; the arm lands after the slot
  goes, so it is the only path a manifest naming the table can take.
  Delete `pack/rtk/rtk.md` and the `include_str!` that reads it, which
  leaves `snapshot.rs:156` counting one binary-owned instruction file and
  drops the path from `layout.rs:201`'s not-content list. Drop
  `agents/rtk.md` from `pack::manifest::REJECTED`
  (`pack/manifest.rs:24`): the entry exists because the binary owns that
  file, and once nothing does, the refusal names a file with no meaning.
- Done-check: `rg 'rtk|bash-output-filter|BashOutputFilter' crates pack
  README.md .mise.toml` returns only `manifest.rs`'s guided error and its
  test, and the message must name the table and the five files, so the
  check cannot be literally empty; `cargo check --workspace
  --all-targets` is clean and `cargo test --workspace` passes.
- Cases:
  - unit: a config carrying `[bash-output-filter]` is refused with a
    message naming the table to delete and the files sync then removes —
    `cargo test -p superdev-core manifest::` runs it.
  - integration: `aggregator_imports_track_the_enabled_set` shows the
    `.agents/superdev.md` aggregator carrying no `@rtk.md` import. The
    case covers the removal at its source, `pipeline.rs`, because this
    repository has no `.agents/superdev.md`: its `AGENTS.md` reads
    `@.agents/core.md`, and `status --drift` has reported the aggregator
    missing since before this plan.

### Block 3: Remove the CLI surface

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: delete the flag — `no_bash_output_filter` and its mapping
  (`manage.rs:36-39,53,450`). Update the tests — the 12
  `--no-bash-output-filter` sites in `tests/cli.rs`, the init-journey
  assertions (`tests/manage.rs:182-201`), and the disable journey
  (`tests/manage.rs:533-565`), which has no capability left to disable.
  Correct the README — the flag list at `README.md:18` and the manifest
  table's capability row at `README.md:132`, which this step first
  missed.
- Done-check: `superdev init --help` lists `--no-frontend`, `--no-skills`
  and `--no-code-index`, and no fourth flag.
- Cases:
  - e2e: `superdev init --help` offers three capability-disable flags —
    no criterion.
  - e2e: `superdev update bash-output-filter` fails with ``unknown
    capability `bash-output-filter` ``, as `workflows` does today — no
    criterion.

### Block 4: Remove what sits outside the blueprint

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: drop the dev pin — `"github:rtk-ai/rtk"` leaves
  `.mise.toml:13`, since no command in the repository reaches rtk once
  the hook is gone. Correct the project template's prose — the Dockerfile
  comment (`pack/projects/rust-npm/devcontainer/Dockerfile:4`) and
  post-create's note about the platform config files
  (`.../scripts/post-create.sh:10-12`) name codegraph alone. Removing
  `.miserc.toml` turns `auto_env` off, and codegraph, the only other
  pinned binary, is unaffected because it pins in `.mise.toml`.
- Done-check: `rg 'rtk' .mise.toml pack/projects` returns nothing.
- Cases:
  - checks that no file under `.mise.toml` or `pack/projects` names rtk —
    no criterion.

### Block 5: Update the knowledge

- [x] Done — ticked at merge.
- Depends-on: 2, 3.
- Change: the capability set — `architecture.md:71` loses its table row,
  `glossary.md:13` the name, `api-contracts.md:15,84-86` the flag and the
  update targets, and `software-components.md:45` and
  `directory-structure.md:41` their rtk references. The configuration
  concept — `configuration.md`: the manifest example (21-22), the
  `.claude/settings.json` merge notes (324-332), the owned-files section
  (333-343) and the frontmatter description, which `index.md:19` carries
  too and which changes in the same step. The development procedure —
  `development-procedure.md:39-43` names the capabilities this repository
  enables on itself. The two content-pack documents that state what is
  true now — `C001:232-241` drops `agents/rtk.md` with the code it
  quotes, and `S014:94` drops `.agents/rtk.md` from the instruction-file
  list. `spec-012-bash-output-filter.md` takes `status: deprecated`,
  following `S005` and `S006`, rather than being deleted: the spec is the
  record of why the capability existed. `CHANGELOG.md` gains a `Removed`
  entry under `## [Unreleased]` naming the guided error and the files
  sync removes, so a user meets the manifest edit in the release notes.
  `knowledge/plans/index.md` lists this plan, and the plan reads `done`.
- Done-check: `rg 'rtk|bash-output-filter' knowledge/` returns hits only
  in `S011`, `S012`, `plan-003-content-packs`, `specs/index.md`,
  `plans/index.md` and this plan; `superdev validate` reports PASS over
  `knowledge/`.
- Cases:
  - checks that the knowledge names three capabilities and that `S012`
    reads `status: deprecated` — the `rg` sweep returns a file outside
    the historical list when a mention survives.
  - checks that `superdev validate` reports PASS over `knowledge/` — no
    criterion.
  - checks the merge gate on a clean checkout of the branch: `npm run
    check:validate` and `npm run check:blueprint` pass, clippy
    `--all-targets -- -D warnings` is clean, and line coverage stays at
    or above 90% per crate.
