---
type: AdhocPlan
id: plan-011-adhoc-filing-by-lifecycle
title: Documents are filed by lifecycle
description: One lifecycle field replaces two vocabularies, every document sits in a folder named for its state, and a document left in the base directory is unfiled — an error the fix pass repairs.
lifecycle: done
---

# Plan: Documents are filed by lifecycle

## Context

`knowledge/issues/` holds twenty documents, nine of them closed, and a
reader listing the directory cannot tell which nine; the same is true of
plans, specs, decisions and contracts. Two vocabularies answer the
question today — a `done` tag on some documents, a non-SOKF `status:
done` on others — and neither is visible in the tree. Moving a settled
document was impossible while links named paths;
plan-010-adhoc-links-address-ids removes that constraint, and this plan
spends it.

## Facts

- Two lifecycle vocabularies are in use across the five directories: 14
  documents carry `tags: [done]`, 1 `tags: [wontfix]`, 9
  `tags: [needs-triage]`, and 4 ad-hoc plans carry `status: done`.
- `status: done` is not a SOKF value. §4 admits `draft | stable |
  deprecated`, and `crates/lib/superdev-core/src/sokf/concept.rs:143`
  maps anything else to `Stable`, so plan-006 through plan-009 read as
  stable and `superdev validate` passes with 0 errors.
- The schemas describe `status` as the work lifecycle, not as document
  maturity: `bug-report` says "draft while the bug is outstanding; the
  resolution rides in tags, not here", and `spec` says "draft until
  accept tags it done". So `status` and the tags answer the same question
  in two ways.
- `sokf_search` already consumes both vocabularies. `SectionDoc::settled`
  (`crates/lib/superdev-core/src/sokf/index.rs:154`) is true for
  `status == "deprecated"` or a tag in
  `DOWNRANK_TAGS = ["done", "resolved", "wontfix"]` (`index.rs:58`), and
  a settled section's score is multiplied by 0.25 (`index.rs:62`).
  Deleting the tags without teaching the ranker would promote every
  closed document back into live results.
- `SearchArgs` filters on `types` and `tags` and on nothing else
  (`crates/lib/superdev-core/src/sokf/mcp.rs:78-83`). Nothing in
  `.agents/` or any skill tells an agent to use either filter.
- 68 documents are in scope, in 12 types: 20 issues (`BugReport`,
  `FeatureRequest`, `Chore`), 10 plans (`AdhocPlan`, `FeaturePlan`), 14
  `Spec`, 17 `Decision`, and 6 contracts across 5 contract types. They
  are served by 7 `index.md` files. The count is a snapshot; W4
  re-derives it.
- `knowledge/contracts/` is partitioned by audience — `public/` and
  `private/` — and its one schema became 15 `contract-*` schemas, all in
  the uncommitted working tree.
- `lifecycle` needs no spec change. SOKF §4 permits producer-defined
  extension keys and defaults them to the `open` write class.
- SOKF §4 defaults an absent `status` to `stable`, and `concept.rs`
  parses it that way, so dropping the key from a schema changes no
  behaviour.
- `index.md` is reserved by SOKF §2 and is not a concept, so it stays in
  the base directory of each kind whatever the documents do.
- 35 files in the live tree instruct the agent to write a concept path:
  28 under `knowledge/schemas/`, which say where each kind is filed —
  `schema-adhoc-plan` opens "filed at
  `knowledge/plans/plan-{nnn}-adhoc-{slug}.md`" — and 7 under
  `.claude/skills/` and `.agents/`. `check_links` reads a `&Bundle`
  (`crates/lib/superdev-core/src/validate/sokf.rs:407`), so the skills
  are link-checked by nothing; the schemas are concepts and are.
- A further 16 files under `pack/` say the same thing: 8 skills and 8
  templates. They are pack content and belong to
  issue-021-backport-the-knowledge-design-to-the-pack.
- Two templates say a new document is "numbered after the highest" in its
  directory (`pack/knowledge/templates/{adhoc-plan,feature-plan}.md`).
  Once documents split across folders, the highest may sit in any of
  them.
- The pack scaffolds `knowledge/plans/index.md` and
  `knowledge/specs/index.md` from `pack/knowledge/concepts/`
  (`crates/lib/superdev-core/src/content/layout.rs:78-83`), and ships no
  `knowledge/issues/` at all.

## Goal

A document's directory names its lifecycle, and one field says the same
thing inside the file.

## Outcomes

- O1 — every document in scope carries one lifecycle field, and no
  document carries a second answer to the question.
- O2 — every document sits in a folder named exactly its lifecycle value,
  and `superdev validate` refuses any other arrangement.
- O3 — an agent asks for a lifecycle through `sokf_search` and never
  needs to list a directory to find live work.
- O4 — no skill, template or process document names a concept path.
- O5 — the pack drift this leaves is recorded rather than accumulated
  silently.

## Non-goals

- The link form. plan-010-adhoc-links-address-ids delivers it; this plan
  depends on it and is the first thing to exercise it.
- Reading the schema's `frontmatter` block in general. The new check
  reads one key; the general frontmatter contract is
  issue-018-the-schema-layer-checks-sections-and-nothing-else.
- `knowledge/schemas/`. A schema is a standard, not a work item: the 42
  schemas gain no lifecycle and do not move.
- Generating the seven `index.md` files. They keep the grouping they
  have, and they stay in each kind's base directory.
- The pack. Nothing under `pack/` changes — not the scaffold, not the 8
  skills, not the 8 templates. It is four migrations behind already, and
  backporting each separately means rewriting the same files while the
  design is still moving.
  issue-021-backport-the-knowledge-design-to-the-pack owns the
  whole debt, and W7 adds this plan's share to it.
- A triage state of its own. `needs-triage` becomes `lifecycle: open`
  like any other live issue; whether triage deserves its own marker is
  in Out-of-band notes.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | Every document in scope carries a `lifecycle` value drawn from its schema's enum | O1 |
| FR-2 | No document in scope carries `tags: [done]`, `[wontfix]` or `[needs-triage]`, or a `status` key | O1 |
| FR-3 | Every document sits in a folder named exactly its `lifecycle` value | O2 |
| FR-4 | A document whose folder disagrees with its `lifecycle` is an error, and `--fix` moves it | O2 |
| FR-5 | A document directly in a kind's base directory is an error — unfiled — and `--fix` files it by its `lifecycle` | O2 |
| FR-6 | A `lifecycle` value outside the schema's enum is an error naming the value and the enum | O1 |
| FR-7 | `sokf_search` treats any value but the live one as settled, and down-ranks it as it down-ranks a `done` tag today | O3 |
| FR-8 | `sokf_search` takes a `lifecycle` filter, and `sokf_overview` and `sokf_read` report each concept's value | O3 |
| FR-9 | `.agents/sokf.md` tells the agent to ask for a lifecycle through that filter rather than by listing a directory | O3 |
| FR-10 | No schema, skill or process document in the live tree names a path under the five directories | O4 |
| FR-11 | Each schema says a document is written under its kind's directory and numbered after the highest across all of that kind's folders | O4 |
| FR-12 | issue-021-backport-the-knowledge-design-to-the-pack names the pack files this plan leaves stale | O5 |

### Non-functional

| ID | Constraint | Budget |
|----|------------|--------|
| NFR-1 | Migration preserves content | every moved document byte-identical but for its frontmatter |
| NFR-2 | No window where settled work ranks as live | the ranker reads `lifecycle` in a commit at or before the one that deletes the tags |
| NFR-3 | `--fix` moves only inside the SOKF knowledge | 0 files written outside `<knowledge>/` |
| NFR-4 | A committed tree has no unfiled document | `find knowledge/{issues,plans,specs,decisions} -maxdepth 1 -name '*.md' ! -name index.md` returns nothing |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | Every document sits in a folder named its `lifecycle` value, the live value included | live work in the base directory, subfolders for settled states | a glob an agent reaches for — `knowledge/issues/*.md` — would then return 11 documents of 20 with no signal it missed nine, which is a wrong answer with no error attached; with the base directory empty it returns `index.md` alone, which is obviously not an issue and is the listing of every one |
| D-2 | The rule has no exception: folder name equals field value | a live value that names no folder | one sentence with no conditional, so an agent that knows the field knows the path and a reader that sees the path knows the field |
| D-3 | Each kind gets the vocabulary that reads correctly for it | one vocabulary across all kinds | a contract is never "closed" — it is active or deprecated — and a folder named for a word that does not fit its documents teaches the reader the wrong noun |
| D-4 | A kind's base directory is a staging area: a document there is unfiled, and `--fix` files it | the creating skill names the live folder | no skill then names a path at all, which is what FR-10 asks; the base directory is empty in every committed tree (NFR-4), and the error names the folder the document belongs in |
| D-5 | `lifecycle` is a new extension key | overload SOKF `status` | `status` is a SOKF field with a defined meaning, and §4 already permits extension keys |
| D-6 | `status` is dropped from the schemas in scope | keep it, rewritten as document maturity | it is described as the lifecycle on those schemas today, so keeping it keeps two answers; §4 defaults an absent status to `stable`, so nothing breaks |
| D-7 | Contracts nest lifecycle inside audience — `contracts/public/active/` | lifecycle outermost | audience is the coarser and more stable partition, and it is the one already in the tree; the fix pass reads and writes only the last segment, so a further audience change does not disturb this one |
| D-8 | The migration runs `--fix` rather than a one-off script | a scripted transform over the tree | P008's scripted transforms dropped content from four issues, one of them 42% of the document; the fix pass is code this plan ships and tests |
| D-9 | A duplicate number is an error, and the templates say to search the whole kind | a `superdev sokf next` verb allocating numbers | the check plan-010 adds already names the collision, so an agent that guesses wrong is told immediately; a verb an agent can forget to call is a weaker guarantee for a new command's cost |
| D-10 | The ranker change lands before the tags are deleted | one commit for both | a commit that deletes the tags first leaves every settled document ranking as live until the next one (NFR-2) |
| D-11 | The folders are a human affordance; an agent asks `sokf_search` for a lifecycle | let the folder be how an agent finds live work too | an agent's retrieval is id-addressed and path-blind, so the folder buys it nothing that a filter does not buy better — and D-1 exists to make the folder's absence loud rather than to make the folder useful |
| D-12 | The folder findings become errors in W5, after the migration | switch them on with the check in W3 | all 68 documents are unfiled until W4, so the gate would fail every run in between |

## Workstreams

### W1: One lifecycle field

Depends on: plan-010-adhoc-links-address-ids.

1. Add the key — `lifecycle` on every schema governing a document in the
   five directories: `bug-report`, `feature-request`, `chore`, `spec`,
   `feature-plan`, `adhoc-plan`, `adr` and the 15 `contract-*` schemas,
   each with the enum its kind admits (Appendix). The set is defined by
   which documents live in those directories, not by a list that goes
   stale — one contract schema became fifteen while this plan was being
   written.
2. Drop `status` — from those same schemas, and with it the `adhoc-plan`
   schema's non-SOKF `draft | active | done | abandoned` enum (D-6,
   FR-2).
3. Document the field — `knowledge/glossary.md` states what `lifecycle`
   means, that the folder is its value, and why SOKF `status` no longer
   appears on these kinds.

### W2: Search reads the field

Depends on: W1.

1. Teach `settled()` — `SectionDoc::settled`
   (`crates/lib/superdev-core/src/sokf/index.rs:154`) is true for any
   `lifecycle` value but the kind's live one. `DOWNRANK_TAGS` stays until
   W4 deletes the tags it names; `status == "deprecated"` stays, since
   documents outside these directories still use it (FR-7, D-10).
2. Add the filter — `SearchArgs` and `SearchOpts` take `lifecycle`
   alongside `types` and `tags`, filtering both lists before fusion as
   those do. Without it, deleting the `done` tag in W4 leaves an agent no
   way to ask for live work at all, and the folder becomes the only
   answer to a question the API should answer (D-11, FR-8).
3. Report it — `sokf_overview` and `sokf_read` render each concept's
   `lifecycle`, as `mcp.rs:425` renders `status` today (FR-8).
4. Re-point the ranking test — `index.rs`'s test sorting a live concept
   above a done-tagged plan and a deprecated spec gains a
   `lifecycle`-valued case, so both paths are covered while both are
   live.

### W3: The check and the filing repair

Depends on: W2.

1. Check the value — a new
   `crates/lib/superdev-core/src/validate/lifecycle.rs` reads
   `lifecycle` and reports a value outside the schema's enum, naming the
   value and the enum (FR-6).
2. Compare with the folder — the document's last path segment before the
   filename must equal its `lifecycle` value (FR-3, FR-4). A document
   whose parent is the kind's own directory is unfiled, and is reported
   with the folder it belongs in (FR-5).
3. Repair it — the fix pass writes the last segment: replacing it when it
   names another value, appending it when the document is unfiled. Every
   segment above is untouched, which is what lets contracts nest inside
   audience (D-7, NFR-3).
4. Order the pass — moves run before plan-010's link conversion, so a
   definition block is written against the path the document ends at and
   one run leaves a clean tree.
5. Emit both findings as warnings — all 68 documents are unfiled until
   W4 (D-12).

### W4: Migrate the tree

Depends on: W3.

1. Set `lifecycle` on all 68 documents by the rule in the Appendix, and
   delete the `tags` and `status` values it replaces (FR-1, FR-2).
2. Delete `DOWNRANK_TAGS` and its branch, now that no tag it names
   survives (D-10).
3. Run `--fix` on a clean working tree — every document moves into the
   folder its value names, and the definition blocks naming it follow
   (FR-3, FR-4). Hard to reverse: it renames all 68 files in one pass.
   `git diff` is the whole record and `git checkout` the whole undo.
4. Read the diff — every moved document is byte-identical but for its
   frontmatter (NFR-1), and each kind's base directory holds `index.md`
   and nothing else (NFR-4).
5. Confirm the indexes — their entries need no edit, since plan-010 made
   them id links; their definition blocks are regenerated by the same
   pass.

### W5: Close the gate

Depends on: W4.

1. Promote the enum, folder and unfiled findings from warning to error
   (D-12).
2. Prove each — a value outside the enum, a document in the wrong state
   folder, a document left in the base directory, and one positive
   control: move a document by hand and confirm the run fails and `--fix`
   restores it.

### W6: The live tree addresses ids

Depends on: W1.

1. Rewrite the schemas' filing lines — the 28 under `knowledge/schemas/`
   that name a path say instead that a document is written under its
   kind's directory with the live `lifecycle` value, and that
   `superdev validate --fix` files it. No schema names a state folder
   (D-4, FR-10).
2. Rewrite numbering — the same schemas say to number after the highest
   across all of a kind's folders, and note that a duplicate is an error
   (D-9, FR-11).
3. Rewrite retrieval — the 6 skills under `.claude/skills/` and
   `.agents/process.md` name the id and read it with `sokf_read`
   (FR-10).
4. Point the agent at the filter — `.agents/sokf.md` says to ask
   `sokf_search` for a lifecycle and not to glob the knowledge tree,
   since a kind's base directory holds no documents at all (D-11, FR-9).

### W7: Record what the pack still owes

Depends on: W5, W6.

1. Add this plan's share to
   issue-021-backport-the-knowledge-design-to-the-pack: the folder
   scaffold `init` does not yet write, the 8 pack skills and 8 pack
   templates left naming paths, and the frontmatter change (FR-12).
2. Record the break — `CHANGELOG.md` carries the frontmatter change and
   the new layout, so a managed repo meets it in the release notes rather
   than in a failing check.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `knowledge/schemas/{bug-report,feature-request,chore,spec,feature-plan,adhoc-plan,adr}.md` and `knowledge/schemas/contract-*.md` (15) | modified — the `lifecycle` key added, `status` dropped | W1 |
| `knowledge/glossary.md` | modified — what `lifecycle` means and how the folder follows it | W1 |
| `crates/lib/superdev-core/src/sokf/index.rs` | modified — `settled()` reads `lifecycle`, `SearchOpts` gains the filter; `DOWNRANK_TAGS` goes in W4 | W2, W4 |
| `crates/lib/superdev-core/src/sokf/mcp.rs` | modified — the `lifecycle` search argument, and the value rendered by overview and read | W2 |
| `crates/lib/superdev-core/src/validate/lifecycle.rs` | new — the enum check, the folder comparison and the unfiled finding | W3 |
| `crates/lib/superdev-core/src/validate/fix.rs` | modified — writing the state segment, and its ordering before link conversion | W3 |
| `crates/lib/superdev-core/src/validate/mod.rs` | modified — declare `lifecycle` and count its findings | W3 |
| `crates/lib/superdev-core/tests/fixtures/lifecycle/**` | new — enum, wrong-folder and unfiled cases | W3 |
| `knowledge/{issues,plans,specs,decisions}/**` (61 files) | moved and modified — `lifecycle` set, `tags` and `status` dropped | W4 |
| `knowledge/contracts/{public,private}/**` (6 files) | moved and modified — as above, under the audience folder | W4 |
| the 7 `index.md` files under those directories | modified — definition blocks regenerated | W4 |
| `knowledge/schemas/*.md` (28) | modified — the filing and numbering lines; no state folder named | W6 |
| `.claude/skills/*/SKILL.md` (6 files) | modified — ids, not paths | W6 |
| `.agents/process.md` | modified — ids, not paths | W6 |
| `.agents/sokf.md` | modified — ask the filter for a lifecycle; do not glob the knowledge tree | W6 |
| `knowledge/issues/issue-021-backport-the-knowledge-design-to-the-pack.md` | modified — this plan's pack drift added to its surfaces | W7 |
| `CHANGELOG.md` | modified — the frontmatter change and the layout | W7 |

## Acceptance

| Check | Verifies |
|-------|----------|
| `cargo nextest run --workspace` passes | FR-1 … FR-8 |
| `cargo run -- validate` exits 0 on the migrated tree | FR-1, FR-3, FR-4, FR-6 |
| `rg '^(tags\|status):' knowledge/{issues,plans,specs,decisions,contracts}` returns nothing, against 24 tag lines and 68 status lines today | FR-2 |
| `find knowledge/{issues,plans,specs,decisions} -maxdepth 1 -name '*.md' ! -name index.md` returns nothing | FR-5, NFR-4 |
| `ls knowledge/issues/` shows `done/`, `index.md`, `open/` and `wontfix/` and no document | FR-3, D-1 |
| Set a document's `lifecycle` to `done` without moving it: one error naming the folder it belongs in; `--fix` moves it and the run exits 0 | FR-4 |
| Drop a new document in `knowledge/issues/` with `lifecycle: open`: one unfiled error; `--fix` files it into `open/` | FR-5 |
| Set a `lifecycle` value outside the enum: one error naming the value and the enum | FR-6 |
| `sokf_search` for a term in a `done` issue ranks it below the same term in an open one | FR-7 |
| `sokf_search` with `lifecycle: ["open"]` returns open issues and no settled ones, and `sokf_overview` reports each concept's value | FR-8, O3 |
| `git log -S DOWNRANK_TAGS` shows the ranker reading `lifecycle` no later than the commit deleting the tags | NFR-2 |
| `rg 'knowledge/(issues\|plans\|specs\|decisions\|contracts)/' knowledge/schemas .agents .claude/skills` returns nothing, against 35 files today | FR-10 |
| Every schema in scope tells the agent to number after the highest across a kind's folders | FR-11 |
| `git diff` after W4 shows renames and frontmatter changes only | NFR-1 |
| issue-021's Surfaces name the folder scaffold, the 8 pack skills and the 8 pack templates this plan leaves | FR-12 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- `knowledge/plans/index.md` lists this plan, and its status reads done.
- The schemas and the live skills agree on where a document is filed and
  how it is addressed; the pack's copies of both are recorded as owing
  the same change.
- No document in the five directories carries a `status` key, and
  `sokf_read` still reports a status for every concept, since SOKF
  defaults it.
- `CHANGELOG.md` records the frontmatter change and the layout for
  managed repositories.

## Risks

- Risk: the ranker silently promotes settled work. Mitigation: D-10
  orders the two changes and NFR-2 pins the order in git. Early signal:
  W2's test fails, or a `sokf_search` for a settled issue's terms puts it
  first.
- Risk: an agent globs `knowledge/issues/**/*.md` and reads settled work
  as live. Mitigation: none available in the tree — D-11 and W6 step 5
  send it to the filter instead, and the value is in the frontmatter of
  every document it opens. This is the residue of D-1: the base directory
  is empty so the shallow glob fails loudly, but a deep glob still
  returns everything.
- Risk: another agent is restructuring `knowledge/contracts/` — its
  schema became fifteen and its documents gained an audience partition
  during planning. Mitigation: D-7 nests inside audience and the fix pass
  writes only the last segment, so a further audience change does not
  disturb this one. Early signal: the counts in Facts no longer
  reproduce.
- Risk: a document is committed unfiled, because `--fix` was not run.
  Mitigation: W5 makes it an error, so the merge gate refuses it, and the
  message names the folder.
- Risk: `superdev init` keeps writing the old flat layout until the
  backport lands, so a managed repo starts life in a shape this plan's
  gate refuses. Mitigation: none here — it is one item in
  issue-021-backport-the-knowledge-design-to-the-pack, alongside a
  scaffold that already fails validation for four other reasons. Early
  signal: `superdev init` into a temporary repo, then
  `superdev validate --knowledge` at what it produced.

## Out-of-band notes

`knowledge/issue-tracker.md` and `pack/knowledge/concepts/issue-tracker.md`
describe the triage label vocabulary, including `needs-triage`. W1 and W6
leave them consistent with `lifecycle` replacing those labels. Whether an
untriaged issue deserves a marker distinct from `open` is a separate
question and belongs in the issue tracker document rather than in this
plan.

## Appendix

### The lifecycle enum, and the folder each value names

| Kinds | Live value | Settled values |
|-------|------------|----------------|
| `BugReport`, `FeatureRequest`, `Chore` | `open` | `done`, `wontfix` |
| `FeaturePlan`, `AdhocPlan` | `open` | `done`, `abandoned` |
| `Spec`, `Decision`, the contract types | `active` | `deprecated` |

The rule in one sentence: a document sits in a folder named exactly its
`lifecycle` value. A kind's base directory holds `index.md` and nothing
else; a document found there is unfiled.

```
knowledge/issues/index.md  open/  done/  wontfix/
knowledge/plans/index.md   open/  done/  abandoned/
knowledge/specs/index.md   active/  deprecated/
knowledge/decisions/index.md  active/  deprecated/
knowledge/contracts/public/index.md   active/  deprecated/
knowledge/contracts/private/index.md  active/  deprecated/
```

### Deriving lifecycle from what the tree carries today

Applied in order; the first match wins.

| Today | Count | `lifecycle` | Filed under |
|-------|-------|-------------|-------------|
| `tags: [done]` | 14 | `done` | `done/` |
| `tags: [wontfix]` | 1 | `wontfix` | `wontfix/` |
| `status: done` | 4 | `done` | `done/` |
| `status: deprecated` | 3 | `deprecated` | `deprecated/` |
| anything else, on an issue or a plan | 13 | `open` | `open/` |
| anything else, on a spec, ADR or contract | 33 | `active` | `active/` |

Counts are the snapshot taken while planning: 68 documents, 22 of them
settled, every one of which moves since none sits in a state folder
today. The 9 issues tagged `needs-triage` fall into the last-but-one row
and become `open`. W4 re-derives the table against the tree as it stands
then.
