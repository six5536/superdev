---
type: FeaturePlan
id: plan-026-feature-filing-an-issue-without-framing-it
title: Filing an issue without framing it — feature plan
description: Slices delivering I030 — a heading declared per variant in the validator, the tracker schemas varying by a four-state lifecycle with the sweep of the issues on file, the /file skill and the workflow entry, /frame framing in place with the three phases' gates, the backlog's retirement, and the records.
lifecycle: open
links:
  - rel: implements
    to: issue-030-feature-request-filing-an-issue-requires-framing-it
    note: The framed feature whose seventeen criteria these slices deliver.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Carries the two per-variant heading promises PENDING (I030); slice 1 closes them, so it runs first.
  - rel: references
    to: adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed
    note: The four states, the forms per state, /file, the gates, the sweep and the migration every slice follows.
  - rel: references
    to: adr-049-a-heading-is-declared-per-variant
    note: The mechanism slice 1 builds and slice 2 uses.
---

# Feature plan: Filing an issue without framing it

Request: [issue-030-feature-request-filing-an-issue-requires-framing-it][sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]

The validator first: the slice that closes
[contract-010][sokf:contract-010-interface-document-schemas]'s two
PENDING promises with
[ADR-049][sokf:adr-049-a-heading-is-declared-per-variant]. Then the
schemas and the sweep in one slice, because the tree validates only
when the four-state enum, the per-state rules and the refiled issues
land together. Then the skills — `/file` with the workflow entry, and
`/frame` with the three gates — each a slice that reads
[ADR-048][sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]
for what it writes. The backlog's retirement follows the skill edits
it shares files with, and the records close.

Owned copies and their sources: every `knowledge/schemas/*.md` from
`pack/knowledge/schemas/`; every `.claude/skills/<name>/` that is
knowledge-carried from `pack/knowledge/skills/<name>/`;
`.agents/superdev.md` is the aggregator `pipeline.rs` renders from its
`AGGREGATOR_PREFIX` constant, so the workflow block is edited there.
A slice edits the source, runs `scripts/superdev sync`, and commits
the moved lock hashes; `superdev status` reporting no drift is part of
its done-check. A pack skeleton under `pack/knowledge/concepts/` is
write-once in a managed repository; the repository's own copy is
edited beside it.

## Slices

### Slice 1: A heading is declared per variant

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `validate::schema::document` — where a heading (or heading
  pattern) is named by more than one section rule, the rules must all
  carry `variants` with pairwise-disjoint sets; a document is checked
  against the one its discriminator value selects, at the heading's
  one place in the declared order (presence, order, prohibition,
  content kind, columns, the five item declarations, all the selected
  rule's own). Two such rules whose sets share a value, or of which
  one is untagged, are a finding on the schema naming the heading and
  the overlap, and both bind nothing. A heading declared once is
  unchanged. The example check runs each variant's example against
  the rules its value selects, as today. The grammar's `variants` doc
  says a heading may recur with disjoint sets. Contract-010's two
  PENDING markers go.
- Done-check: a probe schema declaring `Acceptance criteria` twice —
  once `[unframed]` with no key, once `[framed, done, wontfix]` keyed
  — passes an unframed document with plain items and fails a framed
  one with a keyless item; the same schema with overlapping sets, or
  one rule untagged, reports on the schema and binds nothing;
  `superdev validate` passes the live tree; no drift.
- Cases:
  - unit: two rules for one heading with disjoint variants — the
    document sees the rule its value selects and none of the other's
    findings — covers AC_one-schema-per-kind.
  - unit: overlapping sets are a schema finding naming the heading
    and the shared value; an untagged twin is a schema finding; both
    rules bind nothing — covers AC_one-schema-per-kind.
  - unit: `sections-ordered` holds with the recurring heading at one
    position — covers AC_one-schema-per-kind.
  - unit: a keyed example map with one example per value passes when
    each example matches its own variant's rule for the shared
    heading — covers AC_one-schema-per-kind.
  - integration: contract-010 carries no PENDING for the per-variant
    heading and the live tree validates — covers AC_one-schema-per-kind.

### Slice 2: The tracker schemas vary by lifecycle, and the issues on file are swept

- [ ] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `pack/knowledge/schemas/feature-request.md`, `bug-report.md`
  and `chore.md`, synced — `lifecycle` enum `[unframed, framed, done,
  wontfix]` with the description saying what each state means;
  `variant-key: lifecycle`; each cited list declared twice: the
  `[unframed]` rule with its list kind alone, the `[framed, done,
  wontfix]` rule with the key, the tag where the item is a requirement
  (`AC_` and `EX_` items tagged; `RS_` and `DD_` items keyed alone with
  a tag prohibited), and no `TBD` admitted; a bug's Expected behaviour
  becomes `numbered-list` in every state; the prose of each schema
  says the two states and what framing adds; one example per state,
  four per schema. The sweep: every `knowledge/issues/open/*.md` takes
  `lifecycle: framed`, or `unframed` where a `TBD` item remains (I030,
  I042), and `superdev validate --fix` refiles them into
  `issues/framed/` and `issues/unframed/`; every bug report whose
  Expected behaviour is prose has each paragraph rewritten as one
  numbered item `` `EX_c<n>` [ubiquitous] <the paragraph> `` by a
  one-off script, words unchanged; the tracker's `index.md` and the
  pack skeleton `pack/knowledge/concepts/issues/index.md` say the four
  folders; the filing check's own schema fixture in `lifecycle.rs`
  reads the four values. `feature-plan.md`'s case description names
  `EX_` beside `RS_` for a bug.
- Done-check: `superdev validate` passes the live tree; `ls
  knowledge/issues/` is `unframed framed done wontfix`; every bug
  report's Expected behaviour is a numbered keyed list; a scratch
  unframed issue with plain criteria passes and the same issue set
  `framed` fails naming each keyless item and each `TBD`; the twelve
  examples pass; no drift.
- Cases:
  - integration: the three schemas declare the four values and
    `variant-key: lifecycle`, and `--fix` files an issue by its value
    — covers AC_lifecycle-values.
  - integration: an unframed feature request whose criteria are plain
    sentences, a `TBD` and one keyed item passes — covers
    AC_unframed-form.
  - integration: a framed feature request with a keyless criterion, a
    `TBD` and a tagless criterion fails naming each; a framed bug with
    a prose Expected behaviour, a keyless step and an untagged `EX_`
    item fails naming each; a framed chore with a keyless done item
    fails — covers AC_framed-form.
  - integration: a done and a wontfix issue are held to the framed
    rules — covers AC_settled-form.
  - integration: each schema's four examples pass its own check —
    covers AC_one-schema-per-kind.
  - integration: every issue on file sits in a folder named by its
    `lifecycle`, I030 and I042 under `unframed/`, and every bug
    report's Expected behaviour is a keyed tagged numbered list whose
    words equal the paragraphs at `6bee067` — covers AC_sweep.

### Slice 3: `/file` and the workflow entry

- [ ] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: `pack/knowledge/skills/file/SKILL.md`, synced into
  `.claude/skills/file/` and claimed in the lock — the skill per
  ADR-048: reads the tracker concept, the three schemas and the idea
  schema; asks for the kind when none is given or it is unknown;
  writes an unframed issue with the minimum record, numbered after
  the highest, `superdev validate --fix` filing it; writes an idea per
  `schema-idea`; promotes a named idea into an unframed issue with a
  `references` link; never interviews, branches, or invents a
  criterion. The aggregator's workflow block in `pipeline.rs` gains
  `<outside skill="/file" when="…" />`, synced into
  `.agents/superdev.md`; the how-do-i skill's map names `/file`. The
  normative tests that enumerate the knowledge-carried skills and the
  claimed files follow.
- Done-check: `.claude/skills/file/SKILL.md` equals the pack copy and
  the lock claims it; `.agents/superdev.md` lists `/file`; `superdev
  status` no drift; the skill text states each of the four behaviours
  and the refusal.
- Cases:
  - unit: the skill text names the four kinds, the minimum record, the
    `unframed` lifecycle, the idea path, the promotion link, and says
    it does not interview, branch or invent criteria — covers
    AC_file-issue, AC_file-idea, AC_promote-idea.
  - unit: the skill text says it asks for a missing or unknown kind
    and files nothing — covers AC_file-asks.
  - unit: the aggregator prefix and `.agents/superdev.md` list `/file`
    outside the phases, and how-do-i's map names it — covers
    AC_workflow-lists-file.
  - unit: the pack skill and the synced copy match, and the lock
    claims the copy — covers AC_skill-ships.
  - manual: `/file` invoked on a probe bug in a scratch repository
    files an unframed issue that validates — covers AC_file-issue.

### Slice 4: `/frame` frames in place, and the later phases refuse an unframed issue

- [ ] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: `pack/knowledge/skills/frame/SKILL.md`, synced — the "File
  or fetch" step fetches an unframed issue and frames it in place; the
  close-out sets `lifecycle: framed` and lets `--fix` refile; run with
  no issue it files and frames in one pass; the criteria step writes
  keys and tags; the branch step names `feature/<nnn>-<slug>`.
  `contract-design`, `feature-plan` and `execute-feature-plan` gain a
  gate: the framed issue's `lifecycle` is `framed`, on-fail `/frame`.
- Done-check: the four skills' pack and synced copies match; no
  drift; the normative skill tests pass.
- Cases:
  - unit: frame's text says it frames an unframed issue in place and
    sets `framed` — covers AC_frame-in-place.
  - unit: frame's text says a run with no issue files and frames in
    one pass — covers AC_frame-files.
  - unit: each of the three phase skills carries a gate on `framed`
    returning to `/frame` — covers AC_phases-refuse.

### Slice 5: The backlog retires

- [ ] Done — ticked by integrate at merge.
- Depends-on: 4.
- Change: three ideas — `idea-006` the knowledge-capture skill,
  `idea-007` template pre-filled skeletons, `idea-008`
  comment-preserving manifest stamping — written from the backlog's
  entries per `schema-idea` and listed in the ideas index;
  `issue-051-chore-pin-node-in-the-managed-repo`, `wontfix`, carrying
  the decided-against reasoning; `knowledge/backlog.md`,
  `pack/knowledge/concepts/backlog.md`, `pack/knowledge/schemas/backlog.md`
  and the synced `knowledge/schemas/backlog.md` deleted; the root
  index, the schemas index, the ideas index's prose, `schema-idea`'s
  prose, frame's bootstrap and "Record the decisions" step, and
  contract-design's backlog rule lose their references; the tests that
  count schemas and claimed files follow.
- Done-check: `git grep -i backlog` finds only ADRs, the changelog and
  settled records; `superdev validate` passes; no drift.
- Cases:
  - unit: no schema, skill, index or live concept names the backlog —
    covers AC_backlog-retired.
  - integration: the three ideas and the wontfix chore validate and
    are listed — covers AC_backlog-retired.

### Slice 6: The records close

- [ ] Done — ticked by integrate at merge.
- Depends-on: 3, 5.
- Change: the issue-tracker concept says the four states, the folders,
  `/file` and `/frame`'s roles; the glossary's Lifecycle entry names
  the four values, its EARS entry the framed state, its Promise key
  entry `EX_` as declared; the changelog's Unreleased carries the
  states, `/file`, the per-variant heading, the sweep and the backlog's
  retirement with the migration note for a managed repository carrying
  a `Backlog` document; I030's Comments record the sweep counts.
- Done-check: `superdev validate` passes; the glossary's Promise key
  entry no longer says `EX_` is reserved.
- Cases:
  - unit: the tracker concept and the glossary name the four states
    and `/file` — covers AC_records.
  - unit: the changelog's Unreleased names the states and `/file` —
    covers AC_records.

## Deferred decisions

- None yet.

<!-- sokf:links -->
[sokf:adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed]: /knowledge/adrs/active/adr-048-an-issues-lifecycle-distinguishes-framed-from-unframed.md
[sokf:adr-049-a-heading-is-declared-per-variant]: /knowledge/adrs/active/adr-049-a-heading-is-declared-per-variant.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-030-feature-request-filing-an-issue-requires-framing-it]: /knowledge/issues/framed/issue-030-feature-request-filing-an-issue-requires-framing-it.md
