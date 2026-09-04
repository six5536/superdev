---
type: Plan
id: plan-012-contract-driven-workflow
title: The workflow becomes contract-driven
description: The seven-phase spec-driven workflow becomes five contract-driven phases — criteria move into the feature-request as EARS sentences, contracts become durable in public/ and internal/, the spec documents are migrated and deleted, and the skills merge to match.
lifecycle: done
links:
  - rel: depends-on
    to: plan-011-filing-by-lifecycle
    note: Runs after the lifecycle filing lands; this plan writes in its vocabulary.
  - rel: references
    to: issue-021-backport-the-knowledge-design-to-the-pack
    note: The pack copies of the workflow skills this plan changes join that backport debt.
---

# Plan: The workflow becomes contract-driven

## Goal

The workflow runs FRAME → CONTRACT-DESIGN → FEATURE-PLAN → BUILD →
INTEGRATE against durable contracts and EARS-stated acceptance criteria,
and no spec document exists. The five-phase flow is stated identically in
`.agents/core.md`, `.agents/process.md`, the development procedure and
the skills; the spec and verify skills are gone and contract-design
exists. The feature-request schema carries a required Acceptance criteria
section in EARS form and every FeatureRequest on file conforms. The
feature-plan is self-contained: it links its feature-request, and each
slice carries its cases inline, each naming the criteria it covers.
Contracts live in `contracts/public/` and `contracts/internal/`, all
durable and keyed to an interface, so no contract is per-feature;
`knowledge/specs/` and `schema-spec` are gone, their durable content held
by contracts and the tracker; and a feature's contract changes are
traceable through the feature-request's links to the contracts it
touched.

The workflow carries seven phases and two per-feature documents — the
spec and the private interface contract — whose durable content belongs
in the issue tracker and the contracts. The user directed a five-phase
contract-driven flow, and every design question was settled in
conversation on 2026-08-30. This plan runs after
[plan-011][sokf:plan-011-filing-by-lifecycle] lands, as the user decided,
and writes in the lifecycle vocabulary that plan establishes. Running
plan-011 first migrates fourteen specs into lifecycle folders that Block
3 then deletes; the ordering was chosen with that wasted motion accepted.

The evidence the design rests on:

- `.agents/core.md:8-26` already states the five-phase flow, edited in
  the drafting session, with the prior text in the untracked
  `.agents/core.md.bak`. One edge is wrong: line 18 returns an ambiguous
  "criterion or test-plan case" to FRAME, but cases live in the
  feature-plan.
- The old flow is named in `.agents/process.md:29,59`,
  `knowledge/development-procedure.md` (its Workflow section), and six
  skill files:
  `.claude/skills/{frame,spec,interface-design,build,verify,how-do-i}/SKILL.md`.
  Three more skills reference specs: `accept` reads and tags the spec,
  `maintain` audits spec and plan tag agreement, and
  `template-backport`'s DOCUMENT step names
  `knowledge/specs/spec-007-project-templates.md`.
- `knowledge/schemas/spec.md` holds the acceptance criteria, behaviour,
  edge cases and test plan. `knowledge/schemas/feature-request.md:37-99`
  has no criteria section, and its sections are all required, so adding a
  required section obliges a backfill of the nine FeatureRequest issues
  on file: issue-006, issue-010, issue-011, issue-012, issue-015,
  issue-017, issue-018, issue-022 and issue-023.
- `knowledge/schemas/feature-plan.md:43-44,56-62` requires a "Spec:" line
  and assigns test-plan case numbers from the spec to slices. The line
  and the numbering are description-level, and only headings are
  enforced, so the schema edit breaks no plan on file — but body links to
  deleted specs would.
- Fourteen specs sit in `knowledge/specs/`, and 48 documents reference
  `sokf:spec-` ids: 12 of the specs and their index; `architecture.md`
  (11 references), `glossary.md` (6), `backlog.md` (6),
  `configuration.md` (4), `contract-001` and `contract-002` (2 each), the
  16 ADRs (2 each), 9 issues (2 each), plans 002, 003 and 005 (2 each),
  and `schemas/feature-plan.md` (1). An unresolvable `sokf:<id>` label is
  a validation error rather than a warning (SOKF §10.5), so deletion and
  link re-pointing land together. The executing agent re-derives the
  list, since plan-011 moves the files.
- `knowledge/contracts/index.md` defines a private contract as
  per-feature and discarded once the code is canonical. One exists,
  `contract-001-interface-content-packs`, referenced from that index and
  from plans 003 and 005.
- The search down-rank already exists and needs no code:
  `crates/lib/superdev-core/src/sokf/index.rs:57` down-ranks settled
  sections by 0.25, and plan-011 re-keys it to the lifecycle field.
- `knowledge/issue-tracker.md` conventions name the spec path and declare
  an issue's feature by its link to the spec.
- The live workflow skills are pack-managed: their canonical copies sit
  under `pack/knowledge/skills/`, `/pack` compiles into the binary
  through a symlink (ADR-006), and the dev shim rebuilds on `sync`, so a
  skill deleted from both live tree and pack stays deleted.
- The EARS reference is
  `__old/awa_experiment/.awa/.agent/schemas/REQ.schema.yaml`: the six
  pattern types, plus a requirements apparatus this plan leaves behind.
  It is read-only input and is not touched.
- `.agents/sokf/SPEC.md:281` glosses `implements` as "a plan or issue
  implementing a spec".

A criterion is an EARS sentence opening with its type tag:

```
1. [event] WHEN a git:// source is given THE SYSTEM SHALL refuse it, naming the source.
2. [ubiquitous] THE SYSTEM SHALL resolve https sources as before.
3. [state] WHILE an apply is running THE SYSTEM SHALL hold the lock.
```

The type tags are `ubiquitous`, `event`, `state`, `conditional`,
`optional` and `complex` — the EARS/INCOSE set from the REQ reference.
The criteria live in the feature-request, with TBD permitted only while
the request is open, rather than in a new framed-feature issue type,
which would recreate the spec under another name. Bugs and chores enter
the same flow with their repro steps or done-definition as implicit
criteria, so a one-line repro carries no ceremony. Cases sit inline per
slice in the feature-plan, because every case belongs to exactly one
slice and a second document keeps the cross-document numbering;
integration and end-to-end cases sit with the slice that completes their
boundary, and ACCEPT walks criteria coverage on the merged code. Both
contract tiers are durable and keyed to an interface, since an ephemeral
contract contradicts the premise that contracts describe the app, and
they are named `public/` and `internal/` because an "external contract"
reads as one a third party owns. Contract-change traceability is the
feature-request's links plus each public contract's stability section,
rather than a Changes section that duplicates git history. Finished
feature-plans are kept and settled by lifecycle, as the user chose, since
search already down-ranks settled work.

Out of scope: the pack backport, whose whole debt
[issue-021][sokf:issue-021-backport-the-knowledge-design-to-the-pack]
owns, except the six workflow skills, whose pack copies are mirrored here
so a rebuild cannot resurrect a deleted phase; the lifecycle machinery,
which plan-011 delivers and this plan uses; code, since the ranking, the
validator and the MCP server already behave as required; rewriting
settled plans and issues beyond what link integrity and the schema
backfill require; and workflow autonomy (plan-004), which names the old
phases and stays a record of its time.

Three risks shaped the order. plan-011 may land in a shape its draft does
not predict, moving the paths this plan names, so Blocks 2 and 3 address
documents by id and take paths from the tree as found. Backfilled
criteria could assert behaviour a settled feature never shipped, so each
is derived from the issue's Resolved section and the contract that
shipped it. A spec may hold the only statement of some behaviour, so
Block 3 dispositions every spec before it deletes any. Block 4 lands as
one change, and no feature enters FRAME between its first commit and its
merge, so no feature follows two workflows.

## Contract changes

- contract-001-interface-content-packs: dissolved and deleted. Its
  manifest, lock and pack-format material is confirmed held by the public
  contracts 004, 005 and 006; its resolver, content-set and `Ctx`
  interfaces move into durable internal contracts; every inbound link is
  re-pointed in the commit that deletes it.
- contract-007-interface-pack-resolution: receives contract-001's
  internal interfaces — pack source identity, the item model, the
  resolved content set, the resolution phase and the `Ctx` — as a durable
  contract keyed to an interface rather than to a feature.

## Work blocks

### Block 1: The issue holds the criteria

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: rewrite `knowledge/schemas/feature-request.md` — a required
  numbered-list Acceptance criteria section of EARS sentences, each
  opening with its type tag, TBD permitted only while the request is
  open; update the worked example. Backfill the nine FeatureRequest
  issues on file in the same change, deriving a settled request's
  criteria from its Resolved section and leaving TBD entries only in open
  ones, so the tree validates in the commit that tightens the schema. An
  optional section cannot make FRAME's exit condition checkable, and the
  resolutions on file state what shipped. Rewrite
  `knowledge/schemas/feature-plan.md` — the title section links the
  feature-request in place of the spec, and each slice carries a Cases
  list inline, each case naming the criteria it covers; update the worked
  example.
- Done-check: `superdev validate` exits 0 at this block's final commit.
- Cases:
  - checks that `rg 'THE SYSTEM SHALL' knowledge/schemas/feature-request.md`
    matches.
  - checks that every file `rg -l 'type: FeatureRequest' knowledge` names
    contains `## Acceptance criteria`.
  - checks that `rg 'Spec:' knowledge/schemas/feature-plan.md` returns
    nothing and that the schema's example links a feature-request.

### Block 2: Contracts become the description of the app

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: move `knowledge/contracts/private/` to
  `knowledge/contracts/internal/`, within whatever lifecycle folders
  plan-011 left, and rewrite `knowledge/contracts/index.md`: two durable
  tiers split by audience, updated by CONTRACT-DESIGN as features change
  them. Re-key `knowledge/schemas/contract-interface.md` so the id names
  the interface rather than a feature, and file the document in
  `internal/`. Dissolve `contract-001-interface-content-packs`: confirm
  the manifest, lock and pack-format material is held by the public
  contracts 004 to 006, move the resolver, content-set and `Ctx`
  interfaces into one or more durable internal contracts, re-point every
  inbound link, then delete it. The deletion is hard to reverse, because
  it discards the only assembled view of those interfaces, so the new
  contracts land in the same commit.
- Done-check: `superdev validate` exits 0 at this block's final commit,
  and `ls knowledge/contracts` prints `index.md`, `internal` and
  `public`.
- Cases:
  - checks that `rg 'interface-\{feature-slug\}'
    knowledge/schemas/contract-interface.md` returns nothing.
  - checks that `rg -l 'contract-001-interface-content-packs' knowledge
    --glob '!knowledge/plans/*'` returns nothing, and that the plans'
    mentions resolve to nothing typed.

### Block 3: The specs leave

- [x] Done — ticked at merge.
- Depends-on: 1, 2. The destinations for durable content must exist
  first.
- Change: disposition each of the fourteen specs — confirm its durable
  content is held by a contract, a concept or its feature's tracker
  record, and fold in what is not. Re-point or unlink the `sokf:spec-`
  references in the 48 documents holding them, each to the contract or
  concept that now holds the content, in the same change that deletes
  `knowledge/specs/` and `knowledge/schemas/spec.md`: an unresolvable id
  is a validation error, so this is one commit. Rewrite the
  `knowledge/issue-tracker.md` conventions so a feature is declared by
  links to its contracts and plan and no spec path exists. Amend
  `.agents/sokf/SPEC.md` §8 to gloss `implements` as "a plan or issue
  implementing a contract or feature", and record the change in the SOKF
  changelog.
- Done-check: `superdev validate` exits 0 at this block's final commit,
  and `test ! -d knowledge/specs` succeeds.
- Cases:
  - checks that `rg -l 'sokf:spec-|knowledge/specs|schema-spec' knowledge
    .agents .claude` names only this plan and settled plans' historical
    prose.
  - checks that `rg 'knowledge/specs' knowledge/issue-tracker.md` returns
    nothing.
  - checks that `rg 'implementing a spec' .agents/sokf/SPEC.md` returns
    nothing.

### Block 4: The flow says what the system does

- [x] Done — ticked at merge.
- Depends-on: 1, 2, 3. The skills must direct agents at documents that
  exist.
- Change: merge `.claude/skills/spec/` into `.claude/skills/frame/` —
  framing ends when the issue's criteria are concrete EARS sentences,
  with a bug's repro steps or a chore's done-definition serving as the
  implicit criteria — and delete the spec skill. Rename
  `.claude/skills/interface-design/` to `.claude/skills/contract-design/`,
  widen it to every contract kind, public and internal, and have it end
  by recording the feature-request's links to each contract the feature
  changes. Merge `.claude/skills/verify/` into
  `.claude/skills/integrate/`, so the phase verifies the slice against
  its cases and then merges, and delete the verify skill. Update
  `.claude/skills/{feature-plan,build,accept,maintain,template-backport,how-do-i}/SKILL.md`
  to the new phase names and documents: accept walks criteria coverage
  against the feature-request, maintain audits issue and plan agreement
  with no spec, and template-backport documents into the templates
  contract or concept that inherits spec-007's content. Mirror all four
  edits into `pack/knowledge/skills/` in the same change, so the compiled
  snapshot cannot resurrect a deleted phase on the next sync. Restate the
  flow in `.agents/process.md` and the development procedure's Workflow
  section, and split `.agents/core.md`'s ambiguous edge in two: an
  ambiguous criterion returns to FRAME, a wrong test case to
  FEATURE-PLAN. Record the pack files this plan changed or left stale in
  issue-021's surface list, and remove the `.agents/core.md.bak` working
  copy from the drafting session.
- Done-check: `superdev validate` exits 0 on the finished branch, and
  `knowledge/plans/index.md` lists this plan, settled under the lifecycle
  convention plan-011 left in force.
- Cases:
  - checks that `rg -l 'FRAME → CONTRACT-DESIGN → FEATURE-PLAN → BUILD →
    INTEGRATE' .agents/core.md .agents/process.md` names both, and that
    the development procedure names the same five skills in order.
  - checks that `rg 'to="FEATURE-PLAN"' .agents/core.md` includes the
    wrong-case edge.
  - checks that `ls .claude/skills pack/knowledge/skills` lists
    `contract-design` and neither `spec` nor `verify` in either tree.
  - checks that issue-021's surface list names this plan's pack drift, or
    states it left none.

<!-- sokf:links -->
[sokf:issue-021-backport-the-knowledge-design-to-the-pack]: /knowledge/issues/done/issue-021-backport-the-knowledge-design-to-the-pack.md
[sokf:plan-011-filing-by-lifecycle]: /knowledge/plans/done/plan-011-filing-by-lifecycle.md
