---
type: AdhocPlan
id: plan-010-adhoc-links-address-ids
title: Links address ids
description: SOKF 0.4 gives a body link an id-addressed form, superdev validate --fix converts the tree to it, and a renamed or moved concept stops breaking the documents that cite it.
status: draft
---

# Plan: Links address ids

## Context

A markdown link in the knowledge tree names a path, so renaming a
concept breaks every document citing it —
`knowledge/schemas/backlog.md:64` names an ADR path that has never
existed, and nothing reports it. SOKF already carries the remedy: an
`id` survives file moves (§5) and is the preferred target for a typed
link (§8), but body mirroring forces a path back into the body. superdev
owns the specification, so the fix is to give a body link an id-addressed
form and convert the tree to it.

## Facts

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
  `adr-012-pack-source-schemes-are-allowlisted.md`. `superdev validate`
  passes with 0 errors.
- Concept renames are in flight as this is written: `api-contracts.md`
  became `contracts.md`, `interface-contract` became 15 `contract-*`
  schemas, and `knowledge/contracts/` gained `public/` and `private/` —
  all in the uncommitted working tree.
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
  `pack/sokf/agents/sokf/SPEC.md`. §12: "Before 1.0 a minor bump may
  break." Its opening paragraph declares SOKF a superset of OKF v0.2,
  which an added link form preserves.
- `.agents/sokf.md` is the agent-facing instruction file for the format,
  imported by `.agents/core.md`. It tells the agent to run `superdev
  validate` after edits and says nothing about how to write a link.

## Goal

A link between concepts names an id, so renaming or moving a document
breaks nothing.

## Outcomes

- O1 — a body link addresses a concept by id, resolves wherever the file
  sits, and still navigates for a reader on GitHub.
- O2 — a path-form or unresolvable concept link is an error `superdev
  validate` names and `superdev validate --fix` repairs.
- O3 — the knowledge tree carries no path link to a concept.
- O4 — the agent is told to write the id form, and the pack drift this
  leaves is recorded rather than accumulated silently.

## Non-goals

- Links to files that are not concepts. `/CONTRIBUTING.md` and
  `/README.md` have no id, so they stay paths; that is the one exception
  to the rule.
- Filing documents by lifecycle. That is
  plan-011-adhoc-filing-by-lifecycle, which depends on this plan and
  supplies its first real test.
- Link-checking outside the SOKF knowledge. `check_links` reads a
  `&Bundle`; the skills and templates that name concept paths are
  plan-011's W6.
- The pack. Nothing under `pack/` changes: it is four migrations behind
  already, and backporting each one separately means rewriting the same
  files while the design is still moving.
  issue-021-chore-backport-the-knowledge-design-to-the-pack owns the
  whole debt, and W6 adds this plan's share to it.
- Generating the indexes. Their entries change form, not grouping.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | A body link addresses a concept as `[text][sokf:<id>]` and resolves through the knowledge's id map | O1 |
| FR-2 | A `links` entry is mirrored by an id-form body link, with no path required | O1 |
| FR-3 | A document carries a generated `<!-- sokf:links -->` block giving each cited id its current repo-root path | O1 |
| FR-4 | Link resolution succeeds when that block is absent or stale | O1 |
| FR-5 | A link to a concept written as a path — in a concept body or an `index.md` — is reported, naming the id it should carry | O2 |
| FR-6 | A `sokf:` label no concept answers to is reported, naming the label | O2 |
| FR-7 | `superdev validate --fix` converts a path link to the id form and regenerates every definition block | O2 |
| FR-8 | `superdev hook validate` writes no file | O2 |
| FR-9 | Every concept link in `knowledge/` is in the id form, and the 9 links to non-concept files are unchanged | O3 |
| FR-10 | Two concepts claiming the same id are reported, naming both paths | O3 |
| FR-11 | `.agents/sokf.md` tells the agent to write the id form and to run `--fix` before committing | O4 |
| FR-12 | issue-021-chore-backport-the-knowledge-design-to-the-pack names the pack files this plan leaves stale | O4 |

### Non-functional

| ID | Constraint | Budget |
|----|------------|--------|
| NFR-1 | `--fix` writes only inside the SOKF knowledge | 0 files written outside `<knowledge>/` |
| NFR-2 | Conversion preserves prose | every converted document byte-identical but for its links and its definition block |
| NFR-3 | Resolution is independent of the definition block | a tree with every block deleted resolves every link |
| NFR-4 | `--fix` is idempotent | a second run reports and writes nothing |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | A body link addresses a concept by id, written as a reference-style link labelled `sokf:<id>` | an inline `[text](sokf:<id>)` URI | an inline `sokf:` URI renders as a dead link in GitHub and every markdown viewer; a reference label resolves through a definition block, so the same link still navigates |
| D-2 | The label carries the `sokf:` prefix | a bare id as the label | a bare label cannot be told from an ordinary reference link, so a mistyped id would read as an unresolved reference rather than a broken edge |
| D-3 | The definition block is generated and owned by `--fix` | authors maintain it | the block holds every path in the tree, and the reason the id form exists is that a writer cannot be relied on to keep a path current |
| D-4 | Resolution reads the id and never the block | resolve through the definition | a stale block would then break the link, which is the failure this plan removes; the block is a rendering aid |
| D-5 | A stale or missing block is an error, not a warning | leave it a warning, since it breaks nothing | the remedy is one command, which is what makes `cargo fmt --check` tolerable; a warning is the state P008 found, where 39 sat unread |
| D-6 | `--fix` is a flag on `superdev validate` | a `superdev sokf fix` verb | the findings and their repairs are one thing; a second verb needs the same walk, the same grammar and a second report, which is how the two halves drifted before P008 merged them |
| D-7 | The hook never fixes | wire `--fix` into `superdev hook validate` | the hook fires PostToolUse, so it would rewrite the file the agent has just written and may still be working on |
| D-8 | `--fix` converts a path link by reading the target document's `id`, falling back to the filename stem only when the path resolves to nothing | always derive the id from the stem | 42 schemas have an id that is not their stem, so the stem is a guess where the file itself carries the answer |
| D-9 | Path links become an error in W5, after the conversion | switch the error on with the check in W2 | the tree carries 493 of them, so the gate would fail every run between W2 and W4 |
| D-10 | Conversion covers every concept link in the tree | convert only links between documents that move | a rule scoped to certain directories is a rule about where a document sits, which is the thing this plan is removing; and root documents are being renamed right now |
| D-11 | Duplicate ids are reported by number as well as by id | rely on the existing uniqueness check | a reused number is what an author actually does wrong, and naming it that way is what plan-011 needs once documents split across folders |

## Workstreams

### W1: SOKF 0.4 — a link may address an id

Depends on: none.

1. Amend §8 — body mirroring is satisfied by a reference-style link
   labelled `sokf:<id>`; the id form is what a producer SHOULD write for
   a concept, and a path stays legal for anything that is not one.
   `.agents/sokf/SPEC.md`.
2. Amend §9 — specify the `<!-- sokf:links -->` block: generated, one
   definition per cited id, each carrying that concept's current
   repo-root path, and state that a consumer resolves the id and never
   the block (D-4).
3. Bump the version — the header, §12, and `sokf: "0.4"` in
   `knowledge/manifest.sokf.yaml`. Hard to reverse: every managed
   repository's manifest declares a version, so an undone bump leaves
   manifests naming a version no binary supports. It lands in the same
   commit as W2, which is what makes 0.4 true of the code.

### W2: Resolve and check a link by id

Depends on: W1.

1. Read the new form — `markdown_links_and_footnotes` in
   `crates/lib/superdev-core/src/validate/sokf.rs` recognises a reference
   link labelled `sokf:<id>` and records the id as a body target, so §8
   mirroring holds without a path (FR-1, FR-2). It builds the parser with
   a broken-link callback, so a label with no definition is still read as
   a link; without that the check would see literal text and resolution
   would depend on the block (D-4, NFR-3).
2. Report an unresolvable label — a `sokf:` label matching no concept id
   names the label and the document (D-2, FR-6).
3. Check the definition block — a cited id with no definition, or one
   whose path is stale, is reported with the command that repairs it
   (D-5, FR-3).
4. Report a path link to a concept — in a body or an `index.md`, with the
   id it should carry. A link to a file that is no concept is left alone
   (FR-5, FR-9).
5. Report a duplicate number — two ids sharing a kind and a number,
   naming both paths (D-11, FR-10).
6. Emit all five as warnings — the tree carries 493 path links until W4,
   so the gate stays open (D-9).
7. Fixtures and goldens — cases under
   `crates/lib/superdev-core/tests/fixtures/sokf/`, exercised by
   `sokf_snapshots.rs`.

### W3: superdev validate --fix

Depends on: W2.

1. Add the flag — `fix: bool` on `ValidateArgs`, and a repair outcome on
   each repairable finding so the report says what changed
   (`crates/app/superdev/src/validate_cli.rs`).
2. Convert path links — rewrite each concept path link to the id form,
   reading the target document's `id` and falling back to the stem only
   when the path resolves to nothing (D-8, FR-7).
3. Regenerate the definition block — one block per document and index, at
   the foot, listing every cited id and its current path (FR-3).
4. Bound the writes — `--fix` refuses to write outside the resolved
   knowledge directory (NFR-1), and a second run reports nothing
   (NFR-4).
5. Keep the hook read-only — a test asserting `superdev hook validate`
   passes no `--fix` and writes no file (D-7, FR-8).
6. Tests — a fixture tree carrying each repairable fault, fixed and
   compared against the expected tree.

### W4: Convert the knowledge tree

Depends on: W3.

1. Run `--fix` over `knowledge/` on a clean working tree — 493 concept
   links convert and every document gains its definition block (FR-9).
   Hard to reverse: it rewrites most of the tree in one pass. `git diff`
   is the whole record and `git checkout` the whole undo.
2. Read the diff — every document is byte-identical but for its links and
   its block (NFR-2).
3. Correct `knowledge/schemas/backlog.md:64` by hand — its path resolves
   to nothing and its stem is not an id, so `--fix` cannot repair it
   (D-8).
4. Run `--fix` again — it reports and writes nothing (NFR-4).

### W5: Close the gate

Depends on: W4.

1. Promote the five findings from warning to error (D-9, D-5).
2. Prove each — one test per finding, plus positive controls: break a
   label, delete a block, write a path link, and confirm each fails.

### W6: Tell the agent, and ship

Depends on: W5.

1. Instruct the agent — `.agents/sokf.md` gains the link form, the one
   exception for non-concept files, and the instruction to run
   `superdev validate --fix` before committing (FR-11).
2. Record the break — `CHANGELOG.md` carries the SOKF bump and the new
   flag.
3. Add the drift to the backport's record — this plan leaves
   `pack/sokf/agents/` at 0.3 and `pack/knowledge/concepts/index.md`
   carrying path links, which
   issue-021-chore-backport-the-knowledge-design-to-the-pack owns
   alongside the four migrations already waiting there.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `.agents/sokf/SPEC.md` | modified — §8, §9 and §12: the id link form, the definition block, version 0.4 | W1 |
| `knowledge/manifest.sokf.yaml` | modified — `sokf: "0.4"` | W1 |
| `crates/lib/superdev-core/src/validate/sokf.rs` | modified — the scanner reads the id form; five new findings | W2 |
| `crates/lib/superdev-core/src/validate/mod.rs` | modified — carry the fix outcome and count the new findings | W2, W3 |
| `crates/lib/superdev-core/src/validate/fix.rs` | new — the repair pass, its idempotence and its write bound | W3 |
| `crates/app/superdev/src/validate_cli.rs` | modified — the `--fix` flag and what the report says it changed | W3 |
| `crates/lib/superdev-core/tests/fixtures/sokf/**` | new — link-form cases | W2 |
| `crates/lib/superdev-core/tests/fix.rs` | new — a fault tree repaired and compared | W3 |
| `crates/app/superdev/tests/*` | modified — the hook writes no file | W3 |
| `knowledge/**/*.md` (about 100 concepts and 7 indexes) | modified — 493 links converted, definition blocks written | W4 |
| `knowledge/schemas/backlog.md` | modified — the unresolvable ADR path corrected by hand | W4 |
| `.agents/sokf.md` | modified — how to write a link, and when to run `--fix` | W6 |
| `CHANGELOG.md` | modified — the SOKF bump and the new flag | W6 |
| `knowledge/issues/issue-021-chore-backport-the-knowledge-design-to-the-pack.md` | modified — this plan's pack drift added to its surfaces | W6 |

## Acceptance

| Check | Verifies |
|-------|----------|
| `cargo nextest run --workspace` passes | FR-1 … FR-10 |
| `cargo run -- validate` exits 0 on the converted tree | FR-5, FR-6, FR-9 |
| `rg -o '\]\([^)]*\.md\)' knowledge/` returns 9 hits, all naming files outside the knowledge tree, against 502 today | FR-9, O3 |
| Delete every `<!-- sokf:links -->` block, then `cargo run -- validate`: findings name the blocks and no link is unresolved | FR-4, NFR-3 |
| `cargo run -- validate --fix` twice: the second run reports and writes nothing | NFR-4 |
| Rename a concept file without touching its `id`, then `cargo run -- validate`: no link error, only stale-block findings; `--fix` clears them | FR-1, O1 |
| Mistype a `sokf:` label, then `cargo run -- validate`: one error naming the label | FR-6 |
| Give two documents the same kind and number: one error naming both paths | FR-10 |
| `cargo run -- validate --fix --knowledge <tmp>` writes no file outside `<tmp>` | NFR-1 |
| `git diff` after W4 shows link and block changes only | NFR-2 |
| A markdown renderer follows `[text][sokf:<id>]` to the linked file | O1 |
| issue-021's Surfaces name `pack/sokf/agents/` and `pack/knowledge/concepts/index.md` with the counts this plan leaves | FR-12 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- `knowledge/plans/index.md` lists this plan, and its status reads done.
- `.agents/sokf/SPEC.md` reads version 0.4, and the pack copy is left at
  0.3 with that recorded in issue-021 rather than silently diverging.
- `CHANGELOG.md` records the SOKF bump as a breaking change to the
  knowledge format.
- plan-011-adhoc-filing-by-lifecycle is unblocked: a document can be
  moved without touching a link.

## Risks

- Risk: another agent is restructuring `knowledge/` as this is written —
  `api-contracts.md` became `contracts.md`, one contract schema became
  fifteen, and `git status` grew from 80 to 97 uncommitted paths under
  `knowledge/` during planning. Mitigation: W1 to W3 touch no document at
  all, and W4 is a single mechanical pass that can run whenever the tree
  is clean. This plan makes those renames cheaper rather than competing
  with them.
- Risk: `--fix` is the first write path in a command every hook and merge
  gate runs. Mitigation: NFR-1 bounds the writes, D-7 keeps the hook
  read-only, and the flag is opt-in. Early signal: the hook test in W3
  fails.
- Risk: the reference-label form collides with an ordinary markdown
  reference link an author wrote. Mitigation: the `sokf:` prefix (D-2)
  separates the two, and W2 step 2 reports a label matching no id rather
  than ignoring it.
- Risk: a block regenerated on every run churns the diff of documents
  nobody edited. Mitigation: NFR-4's idempotence test. Early signal: W4
  step 4's second run writes something.
- Risk: SOKF 0.4 refuses a 0.3 manifest in a managed repository.
  Mitigation: the bump lands with the code that reads it, and the
  guidance goes in `CHANGELOG.md` as a breaking change.

## Out-of-band notes

Filed as follow-ups rather than done here:

- `check_links` reaching the grammar's roots, so a concept path written
  in a skill is checked. plan-011's W6 removes today's; nothing stops
  tomorrow's.
- `knowledge/schemas/adhoc-plan.md`'s worked example carries
  `id: adhoc-plan-002-scheme-match-cleanup`, which the same schema's own
  `id` pattern refuses. It sits inside a fenced block, so no check reads
  it.
- issue-012-feature-request-five-decidable-findings-only-warn asks for
  the same promotion W5 performs. W5 does not close it; the five findings
  it names are a different set.

## Appendix

### The two link forms

| Target | Form | Example |
|--------|------|---------|
| A concept | reference link labelled `sokf:<id>` | `[the schemas do not ship][sokf:issue-020-bug-the-schemas-do-not-ship]` |
| Anything else | an ordinary path | `[contributing](/CONTRIBUTING.md)` |

The definition block sits at the foot of the document:

```markdown
<!-- sokf:links -->
[sokf:issue-020-bug-the-schemas-do-not-ship]: /knowledge/issues/issue-020-bug-the-schemas-do-not-ship.md
```

It is regenerated by `--fix` and read by no consumer. A renderer uses it
to navigate; superdev resolves the id.
