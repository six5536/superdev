---
type: Plan
id: plan-010-links-address-ids
title: Links address ids
description: SOKF 0.4 gives a body link an id-addressed form, superdev validate --fix converts the tree to it, and a renamed or moved concept stops breaking the documents that cite it.
lifecycle: done
---

# Plan: Links address ids

## Goal

A link between concepts names an id, so renaming or moving a document
breaks nothing. A body link addresses a concept by id, resolves wherever
the file sits, and still navigates for a reader on GitHub; a path-form or
unresolvable concept link is an error `superdev validate` names and
`superdev validate --fix` repairs; the knowledge tree carries no path
link to a concept; and the agent is told to write the id form.

A markdown link in the knowledge tree names a path today, so renaming a
concept breaks every document citing it. SOKF already carries the
remedy: an `id` survives file moves (§5) and is the preferred target for
a typed link (§8), but body mirroring forces a path back into the body.
superdev owns the specification, so the fix is to give a body link an
id-addressed form and convert the tree to it.

The evidence the design rests on:

- All 83 `links` `to:` values under `knowledge/` name an id; none names a
  path (`grep -c "to: /" knowledge/` returns 0). No frontmatter edge
  breaks when a file moves.
- 502 markdown links in `knowledge/` point at a `.md` file. 9 of them
  name a file outside the knowledge tree — `/CONTRIBUTING.md`,
  `/README.md` — which is not a concept and has no id. The remaining 493
  are concept links, and 2 of those are root-absolute.
- SOKF §8 requires every `links` entry be mirrored by "a plain markdown
  link" in the body. That sentence is the only rule forcing a path into a
  document.
- `knowledge/schemas/backlog.md:64` cites
  `knowledge/decisions/adr-012-pack-transport-allowlist.md`. The file is
  `adr-012-pack-source-schemes-are-allowlisted.md`, and `superdev
  validate` passes with 0 errors.
- Concept renames are in flight as this is written: `api-contracts.md`
  became `contracts.md`, `interface-contract` became 15 `contract-*`
  schemas, and `knowledge/contracts/` gained `public/` and `private/` —
  all in the uncommitted working tree. `git status` grew from 80 to 97
  uncommitted paths under `knowledge/` during planning. Blocks 1 to 3
  touch no document, and Block 4 is a single mechanical pass that runs
  whenever the tree is clean, so this plan makes those renames cheaper
  rather than competing with them.
- For every issue, plan, spec, ADR and contract the `id` equals the
  filename stem. Of 130 concepts the 42 where they differ are the
  `schema-`-prefixed schemas, whose files carry the id in frontmatter.
- `markdown_links_and_footnotes`
  (`crates/lib/superdev-core/src/validate/sokf.rs:514`) walks
  pulldown-cmark events. A reference link whose definition is missing is
  emitted as literal text, not as a link, unless the parser is built with
  a broken-link callback. The crate at 0.13.4 offers one, and
  `LinkType::{ReferenceUnknown, CollapsedUnknown, ShortcutUnknown}` exist
  for exactly this.
- `superdev validate` has no write mode: `ValidateArgs` carries `--json`,
  `--doc`, `--knowledge` and free paths
  (`crates/app/superdev/src/validate_cli.rs:23-32`).
- No concept carries a `verified` block, so no verification lapses when
  this plan rewrites a document (SOKF §7).
- The largest concept is 569 lines against a schema limit of 800, so a
  definition block of one line per cited id fits everywhere.
- SOKF is superdev's own specification — `.agents/sokf/SPEC.md`, version
  0.3, Draft — shipped to managed repositories at
  `pack/sokf/agents/sokf/SPEC.md`. §12 states that a minor bump may break
  before 1.0. Its opening paragraph declares SOKF a superset of OKF v0.2,
  which an added link form preserves.
- `.agents/sokf.md` is the agent-facing instruction file for the format,
  imported by `.agents/core.md`. It tells the agent to run `superdev
  validate` after edits and says nothing about how to write a link.

A concept link is a reference-style link labelled `sokf:<id>` — for
example `[the schemas do not ship][sokf:issue-020-the-schemas-do-not-ship]`
— and anything else keeps an ordinary path, for example
`[contributing](/CONTRIBUTING.md)`. A generated `<!-- sokf:links -->`
block at the foot of the document gives each cited id its current
repo-root path, so a renderer navigates; superdev resolves the id and
never the block.

Out of scope: links to files that are not concepts, which have no id and
stay paths; filing documents by lifecycle, which plan-011 delivers on top
of this plan; link-checking outside the SOKF knowledge, since
`check_links` reads a `&Bundle` and the skills and templates naming
concept paths are plan-011's work; the pack, which is four migrations
behind and whose whole debt
[issue-021][sokf:issue-021-backport-the-knowledge-design-to-the-pack]
owns; and generating the indexes, whose entries change form rather than
grouping.

Two faults found while planning are filed rather than fixed here.
`check_links` reaching the grammar's roots, so a concept path written in
a skill is checked, is
[I023][sokf:issue-023-a-concept-path-written-outside-the-knowledge-is-checked-by-nothing]:
plan-011 removes today's paths, and nothing stops tomorrow's.
`knowledge/schemas/adhoc-plan.md`'s worked example carries an `id` the
same schema's own pattern refuses, and sits inside a fenced block that no
check reads; four more schemas carried the same fault from the same
filename-convention migration, so the five ids were corrected by hand and
the missing check is
[I022][sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing].
[I012][sokf:issue-012-five-decidable-findings-only-warn] asks for the
same promotion Block 5 performs, and Block 5 does not close it: the five
findings it names are a different set, and all five still warn.

## Contract changes

- none.

## Work blocks

### Block 1: SOKF 0.4 — a link may address an id

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: amend `.agents/sokf/SPEC.md` §8 — body mirroring is satisfied
  by a reference-style link labelled `sokf:<id>`, the id form is what a
  producer should write for a concept, and a path stays legal for
  anything that is not one. An inline `[text](sokf:<id>)` URI renders as
  a dead link in GitHub and every markdown viewer, and a bare id as the
  label cannot be told from an ordinary reference link, so a mistyped id
  would read as an unresolved reference rather than a broken edge. Amend
  §9 — specify the `<!-- sokf:links -->` block: generated, one definition
  per cited id, each carrying that concept's current repo-root path, and
  state that a consumer resolves the id and never the block. Bump the
  version in the header, in §12 and as `sokf: "0.4"` in
  `knowledge/manifest.sokf.yaml`. The bump is hard to reverse, because
  every managed repository's manifest declares a version and an undone
  bump leaves manifests naming a version no binary supports, so it lands
  in the same commit as Block 2, which is what makes 0.4 true of the
  code.
- Done-check: `.agents/sokf/SPEC.md` reads version 0.4, and the pack copy
  is left at 0.3 with that recorded in issue-021 rather than silently
  diverging.
- Cases:
  - checks that a markdown renderer follows `[text][sokf:<id>]` to the
    linked file.

### Block 2: Resolve and check a link by id

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: `markdown_links_and_footnotes` in
  `crates/lib/superdev-core/src/validate/sokf.rs` recognises a reference
  link labelled `sokf:<id>` and records the id as a body target, so §8
  mirroring holds without a path. It builds the parser with a
  broken-link callback, so a label with no definition is still read as a
  link; without that the check would see literal text and resolution
  would depend on the block. Five findings follow: a `sokf:` label
  matching no concept id, naming the label and the document; a cited id
  with no definition or with a stale path, reported with the command that
  repairs it; a path link to a concept, in a body or an `index.md`, with
  the id it should carry, leaving a link to a file that is no concept
  alone; and two ids sharing a kind and a number, naming both paths,
  because a reused number is what an author actually does wrong and is
  what plan-011 needs once documents split across folders. All five emit
  as warnings until Block 5, since the tree carries 493 path links until
  Block 4 and the gate would otherwise fail every run in between.
  Fixtures land under
  `crates/lib/superdev-core/tests/fixtures/sokf/`, exercised by
  `sokf_snapshots.rs`.
- Done-check: `cargo nextest run --workspace` passes.
- Cases:
  - unit: a body link written `[text][sokf:<id>]` resolves through the
    knowledge's id map, and a `links` entry mirrored only by that form
    passes §8 — no criterion.
  - unit: a `sokf:` label no concept answers to is reported, naming the
    label — no criterion.
  - unit: a cited id with no definition, or one whose definition names a
    stale path, is reported with the repairing command — no criterion.
  - unit: a link to a concept written as a path is reported with the id
    it should carry, and a link to a non-concept file is left alone — no
    criterion.
  - unit: two concepts claiming the same kind and number are reported,
    naming both paths — no criterion.

### Block 3: superdev validate --fix

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: add `fix: bool` to `ValidateArgs`
  (`crates/app/superdev/src/validate_cli.rs`) and a repair outcome on
  each repairable finding, so the report says what changed. The flag
  rides on `superdev validate` rather than a `superdev sokf fix` verb,
  because the findings and their repairs are one thing and a second verb
  needs the same walk, the same grammar and a second report. The repair
  pass rewrites each concept path link to the id form, reading the target
  document's `id` and falling back to the filename stem only when the
  path resolves to nothing, since 42 schemas have an id that is not their
  stem. It regenerates one definition block per document and index, at
  the foot, listing every cited id and its current path. `--fix` refuses
  to write outside the resolved knowledge directory, and a second run
  reports and writes nothing. The hook never fixes: it fires PostToolUse,
  so it would rewrite the file the agent has just written and may still
  be working on. Tests use a fixture tree carrying each repairable fault,
  fixed and compared against the expected tree.
- Done-check: `cargo run -- validate --fix --knowledge <tmp>` writes no
  file outside `<tmp>`, and `cargo run -- validate --fix` twice leaves
  the second run reporting and writing nothing.
- Cases:
  - unit: a fault tree is repaired and compares equal to the expected
    tree — no criterion.
  - unit: `--fix` writes zero files outside `<knowledge>/` — no
    criterion.
  - unit: a second `--fix` run reports and writes nothing — no criterion.
  - integration: `superdev hook validate` passes no `--fix` and writes no
    file — no criterion.
  - integration: a tree with every `<!-- sokf:links -->` block deleted
    resolves every link, and the findings name the blocks — checks that
    resolution is independent of the block.

### Block 4: Convert the knowledge tree

- [x] Done — ticked at merge.
- Depends-on: 3.
- Change: run `--fix` over `knowledge/` on a clean working tree, so 493
  concept links convert and every document gains its definition block.
  The pass is hard to reverse, because it rewrites most of the tree at
  once: `git diff` is the whole record and `git checkout` the whole undo.
  Read the diff, then correct `knowledge/schemas/backlog.md:64` by hand,
  since its path resolves to nothing and its stem is not an id, so
  `--fix` cannot repair it. Run `--fix` again to confirm it reports and
  writes nothing.
- Done-check: `rg -o '\]\([^)]*\.md\)' knowledge/` returns 9 hits, all
  naming files outside the knowledge tree, against 502 today; `cargo run
  -- validate` exits 0 on the converted tree.
- Cases:
  - checks that every concept link in `knowledge/` is in the id form and
    the 9 links to non-concept files are unchanged.
  - checks that `git diff` after the pass shows link and definition-block
    changes only, so every converted document is byte-identical but for
    its links and its block.

### Block 5: Close the gate

- [x] Done — ticked at merge.
- Depends-on: 4.
- Change: promote the five findings from warning to error, now that the
  tree carries none of them. A warning is the state P008 found, where 39
  sat unread; the remedy here is one command, which is what makes `cargo
  fmt --check` tolerable.
- Done-check: `cargo run -- validate` exits 0 on the converted tree, and
  each positive control fails.
- Cases:
  - integration: renaming a concept file without touching its `id` raises
    no link error, only stale-block findings, which `--fix` clears.
  - integration: a mistyped `sokf:` label raises one error naming the
    label.
  - integration: two documents sharing a kind and a number raise one
    error naming both paths.
  - integration: a deleted definition block and a hand-written path link
    each fail the run — checks that the promoted findings are errors.

### Block 6: Tell the agent, and ship

- [x] Done — ticked at merge.
- Depends-on: 5.
- Change: `.agents/sokf.md` gains the link form, the one exception for
  non-concept files, and the instruction to run `superdev validate --fix`
  before committing. `CHANGELOG.md` records the SOKF bump as a breaking
  change to the knowledge format, and the new flag. This plan leaves
  `pack/sokf/agents/` at 0.3 and `pack/knowledge/concepts/index.md`
  carrying path links, which
  [issue-021][sokf:issue-021-backport-the-knowledge-design-to-the-pack]
  owns alongside the four migrations already waiting there.
- Done-check: `knowledge/plans/index.md` lists this plan, the plan reads
  `done`, and plan-011 is unblocked: a document moves without touching a
  link.
- Cases:
  - checks that issue-021's Surfaces name `pack/sokf/agents/` and
    `pack/knowledge/concepts/index.md` with the counts this plan leaves.

<!-- sokf:links -->
[sokf:issue-012-five-decidable-findings-only-warn]: /knowledge/issues/done/issue-012-five-decidable-findings-only-warn.md
[sokf:issue-021-backport-the-knowledge-design-to-the-pack]: /knowledge/issues/done/issue-021-backport-the-knowledge-design-to-the-pack.md
[sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]: /knowledge/issues/done/issue-022-a-schemas-worked-example-is-checked-by-nothing.md
[sokf:issue-023-a-concept-path-written-outside-the-knowledge-is-checked-by-nothing]: /knowledge/issues/open/issue-023-a-concept-path-written-outside-the-knowledge-is-checked-by-nothing.md
