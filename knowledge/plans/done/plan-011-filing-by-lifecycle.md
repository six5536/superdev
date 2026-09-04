---
type: Plan
id: plan-011-filing-by-lifecycle
title: Documents are filed by lifecycle
description: One lifecycle field replaces two vocabularies, every document sits in a folder named for its state, and a document left in the base directory is unfiled — an error the fix pass repairs.
lifecycle: done
---

# Plan: Documents are filed by lifecycle

## Goal

A document's directory names its lifecycle, and one field says the same
thing inside the file. Every document in scope carries one lifecycle
value and no second answer to the question; every document sits in a
folder named exactly that value, and `superdev validate` refuses any
other arrangement; an agent asks for a lifecycle through `sokf_search`
rather than listing a directory; and no skill, template or process
document names a concept path.

`knowledge/issues/` holds twenty documents, nine of them closed, and a
reader listing the directory cannot tell which nine; the same is true of
plans, specs, decisions and contracts. Two vocabularies answer the
question today — a `done` tag on some documents, a non-SOKF `status:
done` on others — and neither is visible in the tree. Moving a settled
document was impossible while links named paths; plan-010-links-address-ids
removes that constraint, and this plan spends it.

The evidence the design rests on:

- Two lifecycle vocabularies are in use across the five directories: 14
  documents carry `tags: [done]`, 1 `tags: [wontfix]`, 9 `tags:
  [needs-triage]`, and 4 ad-hoc plans carry `status: done`.
- `status: done` is not a SOKF value. §4 admits `draft | stable |
  deprecated`, and
  `crates/lib/superdev-core/src/sokf/concept.rs:143` maps anything else
  to `Stable`, so plan-006 through plan-009 read as stable and `superdev
  validate` passes with 0 errors.
- The schemas describe `status` as the work lifecycle rather than as
  document maturity: `bug-report` says the resolution rides in tags, and
  `spec` says draft until accept tags it done. So `status` and the tags
  answer the same question in two ways.
- `sokf_search` already consumes both vocabularies. `SectionDoc::settled`
  (`crates/lib/superdev-core/src/sokf/index.rs:154`) is true for `status
  == "deprecated"` or a tag in `DOWNRANK_TAGS = ["done", "resolved",
  "wontfix"]` (`index.rs:58`), and a settled section's score is
  multiplied by 0.25 (`index.rs:62`). Deleting the tags without teaching
  the ranker would promote every closed document back into live results.
- `SearchArgs` filters on `types` and `tags` and on nothing else
  (`crates/lib/superdev-core/src/sokf/mcp.rs:78-83`). Nothing in
  `.agents/` or any skill tells an agent to use either filter.
- 68 documents are in scope, in 12 types: 20 issues (`BugReport`,
  `FeatureRequest`, `Chore`), 10 plans (`AdhocPlan`, `FeaturePlan`), 14
  `Spec`, 17 `Decision`, and 6 contracts across 5 contract types. They
  are served by 7 `index.md` files. The count is a snapshot that Block 4
  re-derives.
- `knowledge/contracts/` is partitioned by audience — `public/` and
  `private/` — and its one schema became 15 `contract-*` schemas, all in
  the uncommitted working tree as this is written.
- `lifecycle` needs no spec change: SOKF §4 permits producer-defined
  extension keys and defaults them to the `open` write class. §4 also
  defaults an absent `status` to `stable`, and `concept.rs` parses it
  that way, so dropping the key from a schema changes no behaviour.
- `index.md` is reserved by SOKF §2 and is not a concept, so it stays in
  the base directory of each kind whatever the documents do.
- 35 files in the live tree instruct the agent to write a concept path:
  28 under `knowledge/schemas/`, which say where each kind is filed, and
  7 under `.claude/skills/` and `.agents/`. `check_links` reads a
  `&Bundle` (`crates/lib/superdev-core/src/validate/sokf.rs:407`), so the
  skills are link-checked by nothing; the schemas are concepts and are.
- A further 16 files under `pack/` say the same thing: 8 skills and 8
  templates, which belong to
  [issue-021][sokf:issue-021-backport-the-knowledge-design-to-the-pack].
  Two templates say a new document is numbered after the highest in its
  directory (`pack/knowledge/templates/{adhoc-plan,feature-plan}.md`),
  and once documents split across folders the highest may sit in any of
  them.
- The pack scaffolds `knowledge/plans/index.md` and
  `knowledge/specs/index.md` from `pack/knowledge/concepts/`
  (`crates/lib/superdev-core/src/content/layout.rs:78-83`), and ships no
  `knowledge/issues/` at all.

The rule in one sentence: a document sits in a folder named exactly its
`lifecycle` value. A kind's base directory holds `index.md` and nothing
else; a document found there is unfiled. The live value names a folder
like any other, because a glob an agent reaches for —
`knowledge/issues/*.md` — would otherwise return 11 documents of 20 with
no signal that it missed nine, which is a wrong answer with no error
attached; with the base directory empty it returns `index.md` alone,
which is obviously not an issue. The folders are a human affordance: an
agent's retrieval is id-addressed and path-blind, so it asks
`sokf_search` for a lifecycle instead.

| Kinds | Live value | Settled values |
|-------|------------|----------------|
| `BugReport`, `FeatureRequest`, `Chore` | `open` | `done`, `wontfix` |
| `FeaturePlan`, `AdhocPlan` | `open` | `done`, `abandoned` |
| `Spec`, `Decision`, the contract types | `active` | `deprecated` |

```
knowledge/issues/index.md  open/  done/  wontfix/
knowledge/plans/index.md   open/  done/  abandoned/
knowledge/specs/index.md   active/  deprecated/
knowledge/decisions/index.md  active/  deprecated/
knowledge/contracts/public/index.md   active/  deprecated/
knowledge/contracts/private/index.md  active/  deprecated/
```

Each kind gets the vocabulary that reads correctly for it, because a
contract is never closed — it is active or deprecated — and a folder
named for a word that does not fit its documents teaches the reader the
wrong noun. Block 4 derives each document's value from what the tree
carries today, applying the rules in order and taking the first match:

| Today | Count | `lifecycle` | Filed under |
|-------|-------|-------------|-------------|
| `tags: [done]` | 14 | `done` | `done/` |
| `tags: [wontfix]` | 1 | `wontfix` | `wontfix/` |
| `status: done` | 4 | `done` | `done/` |
| `status: deprecated` | 3 | `deprecated` | `deprecated/` |
| anything else, on an issue or a plan | 13 | `open` | `open/` |
| anything else, on a spec, ADR or contract | 33 | `active` | `active/` |

The counts are the snapshot taken while planning: 68 documents, 22 of
them settled, every one of which moves since none sits in a state folder
today. The 9 issues tagged `needs-triage` fall into the last-but-one row
and become `open`.

Out of scope: the link form, which plan-010-links-address-ids delivers
and this plan is the first thing to exercise; reading the schema's
`frontmatter` block in general, which is
[issue-018][sokf:issue-018-the-schema-layer-checks-sections-and-nothing-else],
since the new check reads one key; `knowledge/schemas/`, because a schema
is a standard rather than a work item, so the 42 schemas gain no
lifecycle and do not move; generating the seven `index.md` files, which
keep the grouping they have and stay in each kind's base directory; the
pack, whose whole debt issue-021 owns; and a triage state of its own,
since `needs-triage` becomes `lifecycle: open` like any other live issue.
`knowledge/issue-tracker.md` and `pack/knowledge/concepts/issue-tracker.md`
describe the triage label vocabulary, and Blocks 1 and 6 leave them
consistent with `lifecycle` replacing those labels; whether an untriaged
issue deserves a marker distinct from `open` belongs in the issue-tracker
document rather than in this plan.

One residual risk stands: an agent that globs `knowledge/issues/**/*.md`
still reads settled work as live. The base directory is empty, so the
shallow glob fails loudly, but a deep glob still returns everything, and
the value is in the frontmatter of every document the agent opens.
`superdev init` also keeps writing the old flat layout until the backport
lands, so a managed repository starts life in a shape this plan's gate
refuses; that is one item in issue-021, alongside a scaffold that already
fails validation for four other reasons.

## Contract changes

- none.

## Work blocks

### Block 1: One lifecycle field

- [x] Done — ticked at merge.
- Depends-on: none. The plan as a whole runs after
  plan-010-links-address-ids, which is what makes a document movable.
- Change: add the `lifecycle` key to every schema governing a document in
  the five directories — `bug-report`, `feature-request`, `chore`,
  `spec`, `feature-plan`, `adhoc-plan`, `adr` and the 15 `contract-*`
  schemas — each with the enum its kind admits. The set is defined by
  which documents live in those directories rather than by a list that
  goes stale, because one contract schema became fifteen while this plan
  was written. `lifecycle` is a new extension key rather than an overload
  of SOKF `status`, which has a defined meaning of its own. Drop `status`
  from those same schemas, and with it the `adhoc-plan` schema's non-SOKF
  `draft | active | done | abandoned` enum: `status` is described as the
  lifecycle on those schemas today, so keeping it keeps two answers.
  `knowledge/glossary.md` states what `lifecycle` means, that the folder
  is its value, and why SOKF `status` no longer appears on these kinds.
- Done-check: every schema in scope declares a `lifecycle` enum and no
  `status` key.
- Cases:
  - checks that each schema in scope admits exactly the lifecycle values
    its kind uses, per the table under Goal.

### Block 2: Search reads the field

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: teach `SectionDoc::settled`
  (`crates/lib/superdev-core/src/sokf/index.rs:154`) to return true for
  any `lifecycle` value but the kind's live one. `DOWNRANK_TAGS` stays
  until Block 4 deletes the tags it names, and `status == "deprecated"`
  stays, since documents outside these directories still use it; the
  ranker therefore reads `lifecycle` in a commit at or before the one
  that deletes the tags, leaving no window in which settled work ranks as
  live. Add the filter — `SearchArgs` and `SearchOpts` take `lifecycle`
  alongside `types` and `tags`, filtering both lists before fusion as
  those do. Without it, deleting the `done` tag leaves an agent no way to
  ask for live work at all, and the folder becomes the only answer to a
  question the API should answer. `sokf_overview` and `sokf_read` render
  each concept's `lifecycle`, as `mcp.rs:425` renders `status` today.
  Re-point `index.rs`'s ranking test so it gains a `lifecycle`-valued
  case, covering both paths while both are live.
- Done-check: `cargo nextest run --workspace` passes.
- Cases:
  - unit: `sokf_search` for a term in a `done` issue ranks it below the
    same term in an open one — checks that any value but the live one is
    down-ranked as a `done` tag is today.
  - integration: `sokf_search` with `lifecycle: ["open"]` returns open
    issues and no settled ones, and `sokf_overview` reports each
    concept's value.
  - checks that `git log -S DOWNRANK_TAGS` shows the ranker reading
    `lifecycle` no later than the commit deleting the tags.

### Block 3: The check and the filing repair

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: a new `crates/lib/superdev-core/src/validate/lifecycle.rs`
  reads `lifecycle` and reports a value outside the schema's enum, naming
  the value and the enum. It compares the document's last path segment
  before the filename with its `lifecycle` value, and reports a document
  whose parent is the kind's own directory as unfiled, naming the folder
  it belongs in: a kind's base directory is a staging area, so no skill
  names a path at all. The fix pass writes the last segment, replacing it
  when it names another value and appending it when the document is
  unfiled; every segment above is untouched, which is what lets contracts
  nest inside audience and what keeps a further audience change from
  disturbing this one. Moves run before plan-010's link conversion, so a
  definition block is written against the path the document ends at and
  one run leaves a clean tree. Both findings emit as warnings until Block
  5, since all 68 documents are unfiled until Block 4. The migration runs
  `--fix` rather than a one-off script, because P008's scripted
  transforms dropped content from four issues, one of them 42% of the
  document, and the fix pass is code this plan ships and tests.
- Done-check: the fixture cases under
  `crates/lib/superdev-core/tests/fixtures/lifecycle/` pass, and `--fix`
  writes zero files outside `<knowledge>/`.
- Cases:
  - unit: a `lifecycle` value outside the schema's enum raises one error
    naming the value and the enum.
  - unit: a document whose folder disagrees with its `lifecycle` raises
    one error naming the folder it belongs in, and `--fix` moves it.
  - unit: a document directly in a kind's base directory is reported
    unfiled, and `--fix` files it by its `lifecycle`.
  - unit: `--fix` moves only inside the SOKF knowledge — no criterion.

### Block 4: Migrate the tree

- [x] Done — ticked at merge.
- Depends-on: 3.
- Change: set `lifecycle` on all 68 documents by the derivation table
  under Goal, and delete the `tags` and `status` values it replaces.
  Delete `DOWNRANK_TAGS` and its branch, now that no tag it names
  survives. Run `--fix` on a clean working tree, so every document moves
  into the folder its value names and the definition blocks naming it
  follow. The pass is hard to reverse, because it renames all 68 files at
  once: `git diff` is the whole record and `git checkout` the whole undo.
  Read the diff, then confirm the indexes: their entries need no edit,
  since plan-010 made them id links, and their definition blocks are
  regenerated by the same pass.
- Done-check: `cargo run -- validate` exits 0 on the migrated tree, and
  `rg '^(tags|status):' knowledge/{issues,plans,specs,decisions,contracts}`
  returns nothing, against 24 tag lines and 68 status lines today.
- Cases:
  - checks that every document in scope carries a `lifecycle` value from
    its schema's enum and no `tags: [done]`, `[wontfix]` or
    `[needs-triage]` and no `status` key.
  - checks that `find knowledge/{issues,plans,specs,decisions} -maxdepth
    1 -name '*.md' ! -name index.md` returns nothing, and that `ls
    knowledge/issues/` shows `done/`, `index.md`, `open/` and `wontfix/`
    and no document.
  - checks that `git diff` shows renames and frontmatter changes only, so
    every moved document is byte-identical but for its frontmatter.

### Block 5: Close the gate

- [x] Done — ticked at merge.
- Depends-on: 4.
- Change: promote the enum, folder and unfiled findings from warning to
  error, now that the tree carries none of them. A document committed
  unfiled then fails the merge gate with a message naming the folder it
  belongs in.
- Done-check: each positive control fails the run, and `--fix` clears it.
- Cases:
  - integration: setting a document's `lifecycle` to `done` without
    moving it raises one error naming the folder it belongs in; `--fix`
    moves it and the run exits 0.
  - integration: a new document dropped in `knowledge/issues/` with
    `lifecycle: open` raises one unfiled error; `--fix` files it into
    `open/`.
  - integration: a `lifecycle` value outside the enum raises one error
    naming the value and the enum.
  - integration: moving a document by hand fails the run, and `--fix`
    restores it — checks the promoted findings are errors.

### Block 6: The live tree addresses ids

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: rewrite the filing lines in the 28 schemas under
  `knowledge/schemas/` that name a path, so each says a document is
  written under its kind's directory with the live `lifecycle` value and
  that `superdev validate --fix` files it; no schema names a state
  folder. The same schemas say to number after the highest across all of
  a kind's folders and note that a duplicate is an error, which the check
  plan-010 adds already names, so an agent that guesses wrong is told
  immediately. Rewrite retrieval in the 6 skills under `.claude/skills/`
  and in `.agents/process.md`: they name the id and read it with
  `sokf_read`. `.agents/sokf.md` says to ask `sokf_search` for a
  lifecycle and not to glob the knowledge tree, since a kind's base
  directory holds no documents at all.
- Done-check: `rg
  'knowledge/(issues|plans|specs|decisions|contracts)/' knowledge/schemas
  .agents .claude/skills` returns nothing, against 35 files today.
- Cases:
  - checks that no schema, skill or process document in the live tree
    names a path under the five directories.
  - checks that every schema in scope tells the agent to number after the
    highest across a kind's folders.
  - checks that `.agents/sokf.md` sends the agent to the `lifecycle`
    filter rather than to a directory listing.

### Block 7: Record what the pack still owes

- [x] Done — ticked at merge.
- Depends-on: 5, 6.
- Change: add this plan's share to
  [issue-021][sokf:issue-021-backport-the-knowledge-design-to-the-pack] —
  the folder scaffold `init` does not yet write, the 8 pack skills and 8
  pack templates left naming paths, and the frontmatter change. Record
  the break in `CHANGELOG.md`, so a managed repository meets the
  frontmatter change and the new layout in the release notes rather than
  in a failing check. `knowledge/plans/index.md` lists this plan, and the
  plan reads `done`.
- Done-check: issue-021's Surfaces name the folder scaffold, the 8 pack
  skills and the 8 pack templates this plan leaves.
- Cases:
  - checks that no document in the five directories carries a `status`
    key, and that `sokf_read` still reports a status for every concept,
    since SOKF defaults it.
  - checks that the schemas and the live skills agree on where a document
    is filed and how it is addressed, and that the pack's copies of both
    are recorded as owing the same change.

<!-- sokf:links -->
[sokf:issue-018-the-schema-layer-checks-sections-and-nothing-else]: /knowledge/issues/done/issue-018-the-schema-layer-checks-sections-and-nothing-else.md
[sokf:issue-021-backport-the-knowledge-design-to-the-pack]: /knowledge/issues/done/issue-021-backport-the-knowledge-design-to-the-pack.md
