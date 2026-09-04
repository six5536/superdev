---
type: AdhocPlan
id: plan-012-adhoc-contract-driven-workflow
title: The workflow becomes contract-driven
description: The seven-phase spec-driven workflow becomes five contract-driven phases — criteria move into the feature-request as EARS sentences, contracts become durable in public/ and internal/, the spec documents are migrated and deleted, and the skills merge to match.
lifecycle: done
links:
  - rel: depends-on
    to: plan-011-adhoc-filing-by-lifecycle
    note: Runs after the lifecycle filing lands; this plan writes in its vocabulary.
  - rel: references
    to: issue-021-backport-the-knowledge-design-to-the-pack
    note: The pack copies of the workflow skills this plan changes join that backport debt.
---

# Plan: The workflow becomes contract-driven

## Context

The workflow carries seven phases and two per-feature documents — the spec
and the private interface contract — whose durable content belongs in the
issue tracker and the contracts. The user directed a five-phase
contract-driven flow, and every design question was settled in
conversation on 2026-08-30. This plan runs after
[plan-011][sokf:plan-011-adhoc-filing-by-lifecycle] lands, as the user
decided, and writes in the lifecycle vocabulary that plan establishes.

## Facts

- `.agents/core.md:8-26` already states the five-phase flow (edited this
  session; the prior text is in the untracked `.agents/core.md.bak`).
  One edge is wrong: line 18 returns an ambiguous "criterion or
  test-plan case" to FRAME, but cases will live in the feature-plan.
- The old flow is named in `.agents/process.md:29,59`,
  `knowledge/development-procedure.md` ("Workflow" section), and six
  skill files: `.claude/skills/{frame,spec,interface-design,build,
  verify,how-do-i}/SKILL.md` (`grep -rln 'interface-design'`). Three
  more skills reference specs: `accept` reads and tags the spec,
  `maintain` audits spec/plan tag agreement, and `template-backport`
  step DOCUMENT names `knowledge/specs/spec-007-project-templates.md`.
- `knowledge/schemas/spec.md` holds the acceptance criteria, behaviour,
  edge cases and test plan; `knowledge/schemas/feature-request.md:37-99`
  has no criteria section, and its sections are all required, so adding
  a required section obliges a backfill of the nine FeatureRequest
  issues on file (Appendix).
- `knowledge/schemas/feature-plan.md:43-44,56-62` requires a "Spec:"
  line and assigns "test-plan case numbers" from the spec to slices.
  The line and the numbering are description-level; only headings are
  enforced (issue-018), so the schema edit breaks no on-file plan —
  but the body links to deleted specs would.
- Fourteen specs sit in `knowledge/specs/`, and 48 documents reference
  `sokf:spec-` ids (Appendix), including `architecture.md` (11),
  `glossary.md` (6), `backlog.md` (6) and `configuration.md` (4). An
  unresolvable `sokf:<id>` label is a validation error, not a warning
  (SOKF §10.5), so deletion and link re-pointing must land together.
- `knowledge/contracts/index.md` defines a private contract as
  per-feature and "discarded once the code is canonical"; one exists,
  `contract-001-interface-content-packs`, referenced from that index
  and from plans 003 and 005.
- The search down-rank already exists and needs no code:
  `crates/lib/superdev-core/src/sokf/index.rs:57` down-ranks settled
  sections by 0.25, and plan-011 re-keys it to the lifecycle field
  (its FR-7).
- `knowledge/issue-tracker.md` conventions name the spec path and
  declare an issue's feature by its link to the spec.
- The live workflow skills are pack-managed: their canonical copies sit
  under `pack/knowledge/skills/`, `/pack` is compiled into the binary
  through a symlink (ADR-006), and the dev shim rebuilds on `sync` —
  so a skill deleted from both live tree and pack stays deleted.
- The EARS reference is
  `__old/awa_experiment/.awa/.agent/schemas/REQ.schema.yaml`: the six
  pattern types, plus a requirements apparatus this plan leaves behind.
- `.agents/sokf/SPEC.md:281` glosses `implements` as "a plan or issue
  implementing a spec".

## Goal

The workflow runs FRAME → CONTRACT-DESIGN → FEATURE-PLAN → BUILD →
INTEGRATE against durable contracts and EARS-stated acceptance criteria,
and no spec document exists.

## Outcomes

- O1 — the five-phase flow is stated identically in `.agents/core.md`,
  `.agents/process.md`, the development procedure and the skills; the
  spec and verify skills are gone and contract-design exists.
- O2 — the feature-request schema carries a required Acceptance criteria
  section in EARS form, and every FeatureRequest on file conforms.
- O3 — the feature-plan is self-contained: it links its feature-request,
  and each slice carries its cases inline, each naming the criteria it
  covers.
- O4 — contracts live in `contracts/public/` and `contracts/internal/`,
  all durable and keyed to an interface; no contract is per-feature.
- O5 — `knowledge/specs/` and `schema-spec` are gone, their durable
  content is held by contracts and the tracker, and nothing references
  a spec.
- O6 — a feature's contract changes are traceable through the
  feature-request's links to the contracts it touched.

## Non-goals

- The pack backport. This plan's share of the drift is recorded in
  [issue-021][sokf:issue-021-backport-the-knowledge-design-to-the-pack],
  which owns the whole debt — except the six workflow skills, whose pack
  copies are mirrored here so a rebuild cannot resurrect a deleted phase.
- The lifecycle machinery. plan-011 delivers it first; this plan uses it.
- Code. The ranking, the validator and the MCP server already behave as
  required.
- Rewriting settled plans and issues beyond what link integrity and the
  schema backfill require.
- Workflow autonomy (plan-004), which names the old phases and stays a
  record of its time.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | The five-phase flow is named identically in core.md, process.md and the development procedure, and the wrong-case edge returns to FEATURE-PLAN | O1 |
| FR-2 | `/contract-design` exists; `/spec` and `/verify` exist nowhere — not live, not in the pack | O1 |
| FR-3 | The feature-request schema requires an Acceptance criteria section of EARS sentences with type tags, TBD permitted only while the request is open, and every FeatureRequest on file carries one | O2 |
| FR-4 | The feature-plan schema links the feature-request in place of the spec, and each slice carries its cases inline, each naming the criteria it covers | O3 |
| FR-5 | `contracts/internal/` replaces `contracts/private/`, and the contracts index describes both tiers as durable and audience-split | O4 |
| FR-6 | The interface-contract schema is keyed to an interface slug and filed in `contracts/internal/` | O4 |
| FR-7 | contract-001 is dissolved: its durable content lives in durable contracts, and nothing live references its id | O4 |
| FR-8 | Each spec's durable content is confirmed held by a contract, concept or its feature's tracker record before the spec is deleted | O5 |
| FR-9 | `knowledge/specs/`, its index and `schema-spec` are deleted, and no document resolves or names a spec | O5 |
| FR-10 | The issue-tracker conventions declare a feature by its links to contracts and plans, and name no spec path | O6 |
| FR-11 | SOKF SPEC §8 glosses `implements` without reference to specs | O5 |

### Non-functional

| ID | Constraint | Budget |
|----|------------|--------|
| NFR-1 | The tree validates after each workstream lands | `superdev validate` exits 0 at every workstream's final commit |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | Acceptance criteria live in the feature-request, TBD only while open | a new framed-feature issue type | it recreates the spec under another name |
| D-2 | EARS patterns plus the type tag, nothing more | the full REQ apparatus — stories, MoSCoW, AC ids | each element already has a home: Summary, Scope, the flat numbered list |
| D-3 | Cases inline per slice in the feature-plan | a separate test-plan document | every case belongs to exactly one slice, and a second document keeps the cross-document numbering |
| D-4 | Integration and e2e cases sit with the slice that completes their boundary; ACCEPT walks criteria coverage on the merged code | a dedicated acceptance slice | the boundary rule exists today and generalises |
| D-5 | Both contract tiers are durable, keyed to an interface | keep the ephemeral private tier | an ephemeral contract contradicts the premise that contracts describe the app |
| D-6 | The tiers are named `public/` and `internal/` | `external/` and `internal/` | an "external contract" reads as one a third party owns |
| D-7 | Contract-change traceability is links from the feature-request, plus each public contract's stability section | a Changes section per contract | it duplicates git history, against SOKF's git-is-history rule |
| D-8 | Specs are migrated and deleted | kept and marked historical | the user chose the clean end state; the code and contracts already hold the content |
| D-9 | Finished feature-plans are kept and settled by lifecycle | deleted at DONE | the user chose keeping them; search already down-ranks settled work |
| D-10 | This plan runs after plan-011 | independent execution | the user chose the ordering; the lifecycle vocabulary is plan-011's to define |
| D-11 | Bugs and chores enter the same flow with their repro steps or done-definition as implicit criteria | EARS sections on all three issue schemas | it forces ceremony onto a one-line repro |
| D-12 | Settled feature-requests are backfilled with criteria derived from their Resolved sections | leaving the new section optional | an optional section cannot make FRAME's exit condition checkable, and the resolutions on file state what shipped |

## Workstreams

### W1: The issue holds the criteria

Depends on: none.

1. Rewrite the Acceptance criteria contract into
   `knowledge/schemas/feature-request.md` — a required numbered-list
   section of EARS sentences, each opening with its type tag
   (Appendix: the criterion form), TBD permitted only while the request
   is open; update the worked example.
2. Backfill the nine FeatureRequest issues on file (Appendix) in the
   same change — criteria for a settled request derived from its
   Resolved section, TBD entries only in open ones — so the tree
   validates in the commit that tightens the schema.
3. Rewrite `knowledge/schemas/feature-plan.md` — the title section
   links the feature-request in place of the spec, and each slice
   carries a Cases list inline, each case naming the criteria it
   covers; update the worked example.

### W2: Contracts become the description of the app

Depends on: none.

1. Move `knowledge/contracts/private/` to `knowledge/contracts/internal/`
   (within whatever lifecycle folders plan-011 left) and rewrite
   `knowledge/contracts/index.md`: two durable tiers split by audience,
   updated by CONTRACT-DESIGN as features change them.
2. Re-key `knowledge/schemas/contract-interface.md` — the id names the
   interface, not a feature, and the document is filed in `internal/`.
3. Dissolve `contract-001-interface-content-packs` — confirm the
   manifest, lock and pack-format material is held by the public
   contracts 004-006, move the resolver, content-set and Ctx interfaces
   into one or more durable internal contracts, re-point every inbound
   link, then delete it. Hard to reverse: the deletion discards the
   only assembled view of those interfaces, so the new contracts land
   in the same commit.

### W3: The specs leave

Depends on: W1, W2 — the destinations for durable content must exist.

1. Disposition each of the fourteen specs: confirm its durable content
   is held by a contract, a concept or its feature's tracker record,
   and fold in what is not.
2. Re-point or unlink the `sokf:spec-` references in the 48 documents
   holding them (Appendix), each to the contract or concept that now
   holds the content, in the same change that deletes
   `knowledge/specs/` and `knowledge/schemas/spec.md` — an
   unresolvable id is a validation error, so this is one commit.
3. Rewrite the `knowledge/issue-tracker.md` conventions: a feature is
   declared by links to its contracts and plan, and no spec path
   exists.
4. Amend `.agents/sokf/SPEC.md` §8: gloss `implements` as "a plan or
   issue implementing a contract or feature". Record the change in the
   SOKF changelog.

### W4: The flow says what the system does

Depends on: W1, W2, W3 — the skills must direct agents at documents that
exist.

1. Merge `.claude/skills/spec/` into `.claude/skills/frame/` — framing
   ends when the issue's criteria are concrete EARS sentences, with a
   bug's repro steps or a chore's done-definition serving as the
   implicit criteria (D-11) — and delete the spec skill.
2. Rename `.claude/skills/interface-design/` to
   `.claude/skills/contract-design/`, widen it to every contract kind,
   public and internal, and have it end by recording the
   feature-request's links to each contract the feature changes (D-7).
3. Merge `.claude/skills/verify/` into `.claude/skills/integrate/` —
   the phase verifies the slice against its cases, then merges — and
   delete the verify skill.
4. Update
   `.claude/skills/{feature-plan,build,accept,maintain,template-backport,how-do-i}/SKILL.md`
   to the new phase names and documents: accept walks criteria coverage
   against the feature-request, maintain audits issue/plan agreement
   with no spec, and template-backport documents into the templates
   contract or concept that inherits spec-007's content.
5. Mirror steps 1-4 into `pack/knowledge/skills/` in the same change,
   so the compiled snapshot cannot resurrect a deleted phase on the
   next sync.
6. Restate the flow in `.agents/process.md` and the development
   procedure's Workflow section, and split `.agents/core.md`'s
   ambiguous edge in two: an ambiguous criterion returns to FRAME, a
   wrong test case to FEATURE-PLAN.
7. Record the pack files this plan changed or left stale in
   issue-021's surface list.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `knowledge/schemas/feature-request.md` | modified — required EARS Acceptance criteria section | W1 |
| the nine FeatureRequest issues (Appendix) | modified — criteria backfilled | W1 |
| `knowledge/schemas/feature-plan.md` | modified — feature-request link, inline cases | W1 |
| `knowledge/contracts/private/` | moved — becomes `knowledge/contracts/internal/` | W2 |
| `knowledge/contracts/index.md` | modified — two durable tiers | W2 |
| `knowledge/schemas/contract-interface.md` | modified — keyed to an interface slug | W2 |
| `knowledge/contracts/private/contract-001-interface-content-packs.md` | deleted — dissolved | W2 |
| `knowledge/contracts/internal/` (new interface contracts) | new — the resolver, content-set and Ctx interfaces | W2 |
| `knowledge/specs/` (fourteen specs and the index) | deleted — content dispositioned first | W3 |
| `knowledge/schemas/spec.md` | deleted | W3 |
| the 48 documents with inbound spec references (Appendix) | modified — links re-pointed or unlinked | W3 |
| `knowledge/issue-tracker.md` | modified — conventions name no spec | W3 |
| `.agents/sokf/SPEC.md` | modified — §8 `implements` gloss | W3 |
| `.agents/sokf/changelog.md` | modified — records the gloss change | W3 |
| `.claude/skills/frame/SKILL.md` | modified — absorbs the spec skill | W4 |
| `.claude/skills/spec/` | deleted | W4 |
| `.claude/skills/interface-design/` | moved — becomes `.claude/skills/contract-design/`, widened | W4 |
| `.claude/skills/verify/` | deleted — absorbed by integrate | W4 |
| `.claude/skills/{integrate,feature-plan,build,accept,maintain,template-backport,how-do-i}/SKILL.md` | modified — new phases and documents | W4 |
| `pack/knowledge/skills/` (the same six skills) | modified/moved/deleted — mirror of the live edits | W4 |
| `.agents/process.md` | modified — five phases | W4 |
| `.agents/core.md` | modified — wrong-case edge to FEATURE-PLAN | W4 |
| `knowledge/development-procedure.md` | modified — Workflow section | W4 |
| `knowledge/issues/issue-021-backport-the-knowledge-design-to-the-pack.md` | modified — this plan's pack share recorded | W4 |

## Acceptance

| Check | Verifies |
|-------|----------|
| `superdev validate` exits 0 at each workstream's final commit and on the finished branch | NFR-1 |
| `rg -l 'sokf:spec-\|knowledge/specs\|schema-spec' knowledge .agents .claude` names only this plan and settled plans' historical prose | FR-8, FR-9 |
| `test ! -d knowledge/specs` succeeds | FR-9 |
| `rg -l 'FRAME → CONTRACT-DESIGN → FEATURE-PLAN → BUILD → INTEGRATE' .agents/core.md .agents/process.md` names both, and the development procedure names the same five skills in order | FR-1 |
| `rg 'to="FEATURE-PLAN"' .agents/core.md` includes the wrong-case edge | FR-1 |
| `ls .claude/skills pack/knowledge/skills` lists `contract-design` and neither `spec` nor `verify` in either tree | FR-2 |
| `rg 'THE SYSTEM SHALL' knowledge/schemas/feature-request.md` matches | FR-3 |
| every file `rg -l 'type: FeatureRequest' knowledge` names contains `## Acceptance criteria` | FR-3 |
| `rg 'Spec:' knowledge/schemas/feature-plan.md` returns nothing; the schema's example links a feature-request | FR-4 |
| `ls knowledge/contracts` prints `index.md`, `internal`, `public` | FR-5 |
| `rg 'interface-\{feature-slug\}' knowledge/schemas/contract-interface.md` returns nothing | FR-6 |
| `rg -l 'contract-001-interface-content-packs' knowledge --glob '!knowledge/plans/*'` returns nothing, and the plans' mentions resolve to nothing typed | FR-7 |
| `rg 'knowledge/specs' knowledge/issue-tracker.md` returns nothing | FR-10 |
| `rg 'implementing a spec' .agents/sokf/SPEC.md` returns nothing | FR-11 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- `knowledge/plans/index.md` lists this plan, and the plan is settled
  under the lifecycle convention plan-011 left in force.
- issue-021's surface list names this plan's pack drift, or states it
  left none.
- The `.agents/core.md.bak` working copy from the drafting session is
  removed.

## Risks

- Risk: plan-011 lands in a shape its draft does not predict, moving the
  paths this plan names — mitigation: W2 and W3 address documents by id
  and take paths from the tree as found; early signal: `validate --fix`
  relocating files this plan just wrote.
- Risk: backfilled criteria assert behaviour a settled feature never
  shipped — mitigation: D-12 derives each from the issue's Resolved
  section and the contract that shipped it; early signal: a criterion
  citing no resolution or contract.
- Risk: a spec holds the only statement of some behaviour and deletion
  loses it — mitigation: W3 step 1 dispositions every spec before step
  2 deletes any; early signal: a re-pointed link with no target that
  holds the content.
- Risk: the flow is stated inconsistently mid-migration and a feature
  started then follows two workflows — mitigation: W4 lands as one
  change, and no feature enters FRAME between W4's first commit and its
  merge.

## Out-of-band notes

Running plan-011 first migrates fourteen specs into lifecycle folders
that W3 then deletes; the ordering was chosen with that wasted motion
accepted. The EARS reference under `__old/` is read-only input and is
not touched. The broader pack backport remains issue-021's, with W4
step 5 the one exception mirrored immediately.

## Appendix

### The criterion form

```
1. [event] WHEN a git:// source is given THE SYSTEM SHALL refuse it, naming the source.
2. [ubiquitous] THE SYSTEM SHALL resolve https sources as before.
3. [state] WHILE an apply is running THE SYSTEM SHALL hold the lock.
```

Type tags: `ubiquitous`, `event`, `state`, `conditional`, `optional`,
`complex` — the EARS/INCOSE set from the REQ reference.

### The nine FeatureRequest issues on file

issue-006, issue-010, issue-011, issue-012, issue-015, issue-017,
issue-018, issue-022, issue-023.

### Documents with inbound spec references

As counted at planning (`grep -rl 'sokf:spec-' knowledge`): 48 files —
12 of the specs and their index; `architecture.md` (11 references),
`glossary.md` (6), `backlog.md` (6), `configuration.md` (4),
`contract-001` and `contract-002` (2 each), the 16 ADRs (2 each),
9 issues (2 each), plans 002, 003 and 005 (2 each), and
`schemas/feature-plan.md` (1). The executing agent re-derives the list;
plan-011 will have moved the files.

<!-- sokf:links -->
[sokf:issue-021-backport-the-knowledge-design-to-the-pack]: /knowledge/issues/done/issue-021-backport-the-knowledge-design-to-the-pack.md
[sokf:plan-011-adhoc-filing-by-lifecycle]: /knowledge/plans/done/plan-011-adhoc-filing-by-lifecycle.md
