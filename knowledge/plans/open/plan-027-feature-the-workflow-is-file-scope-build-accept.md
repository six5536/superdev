---
type: FeaturePlan
id: plan-027-feature-the-workflow-is-file-scope-build-accept
title: The workflow is file, scope, build, accept — feature plan
description: Slices delivering I052 — the validator's nested items and optional key closing contract-010's five PENDING promises, a contract's nested criteria, one issue schema with the sweep of the issues on file, one plan schema with the sweep of the plans, the scope and contract-design skills, the build, execute-plan and accept skills with the workflow text, and the concepts and records.
lifecycle: open
links:
  - rel: implements
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: The framed feature whose sixteen criteria these slices deliver.
  - rel: references
    to: contract-010-interface-document-schemas
    note: Carries five PENDING promises for `nested` and `item-key-optional`; slice 1 closes them, so it runs first.
  - rel: references
    to: adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept
    note: The decision every slice reads for what it writes.
  - rel: references
    to: adr-051-a-section-rule-declares-nested-items-and-an-optional-key
    note: The mechanism slice 1 builds and slice 2 uses.
---

# Feature plan: The workflow is file, scope, build, accept

Request: [issue-052-the-workflow-carries-more-process-than-it-needs][sokf:issue-052-the-workflow-carries-more-process-than-it-needs]

The validator first: the slice that closes
[contract-010][sokf:contract-010-interface-document-schemas]'s five
PENDING promises with
[ADR-051][sokf:adr-051-a-section-rule-declares-nested-items-and-an-optional-key].
The contract schema's nested criteria follow on it. The issue schema
and the plan schema are independent of each other and of the
validator work, each landing with its sweep, because the tree
validates only when the schema, the retired schemas, the refiled
documents and the lifecycle ranking change together. The skills come
after both schemas they name; the workflow text and the concepts
close.
[ADR-050][sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]
says what each writes.

Owned copies and their sources: every `knowledge/schemas/*.md` from
`pack/knowledge/schemas/`; every `.claude/skills/<name>/` that is
knowledge-carried from `pack/knowledge/skills/<name>/`;
`.agents/sokf/grammar.yaml` from
`crates/lib/superdev-core/src/validate/schema/grammar.yaml`;
`.agents/superdev.md` is the aggregator `pipeline.rs` renders from its
`AGGREGATOR_PREFIX` constant, so the workflow block is edited there.
A slice edits the source, runs `scripts/superdev sync`, and commits
the moved lock hashes; `superdev status` reporting no drift is part of
its done-check. A retired schema or skill is deleted from the pack and
from its owned copy, and the lock forgets it. The pack's skeleton
concepts under `pack/knowledge/concepts/` change beside the
repository's own copies. A normative test in
`crates/lib/superdev-core/tests/normative_shapes.rs` that pins a
retiring form is rewritten to pin the new one in the slice that
retires it, and the reason is stated in the commit. The changelog is
at its 800-line limit, so a slice adding a line folds the Unreleased
section as it goes.

## Slices

### Slice 1: The validator reads nested items and an optional key

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `validate::schema::document` — `Items::read` learns depth: a
  marker of the section's list kind indented past the item above opens
  a nested item, to the depth the rule declares, and a deeper marker
  or one of the other kind is text of the item it sits in; each level
  is checked by its rule's `item-key`, then `item-prohibited-pattern`,
  then `item-pattern`, one finding per item; a nested `required` with
  no nested item is an error naming the item above; nested keys join
  the document's key space; `item-key-optional` exempts an item that
  does not match `item-key` from everything but
  `item-prohibited-pattern`; the three mis-declarations are schema
  findings. `grammar.yaml` documents `nested` and `item-key-optional`;
  `.agents/sokf/grammar.yaml` synced. Contract-010's five `PENDING`
  markers removed.
- Done-check: `P_nested-binds`, `P_nested-required`,
  `P_key-optional-unkeyed`, `P_key-optional-keyed` and
  `P_misdeclared-nested` hold on the built binary through a scratch
  schema; contract-010 carries no `PENDING`; `superdev status` reports
  no drift.
- Cases: a schema with a two-level `nested` accepts a conforming
  document and reports a nested item missing its key, naming the item
  (covers AC_contract-criteria); a nested `required` reports a
  top-level item with no nested item, naming it (covers
  AC_contract-criteria); a key repeated between a top-level and a
  nested item is reported naming both (covers AC_contract-criteria); a
  third-level marker under a two-level rule is text of the nested
  item, and a bullet under a numbered rule likewise (covers
  AC_contract-criteria-optional); a rule with no `nested` treats a
  nested list as text, as today (covers
  AC_contract-criteria-optional); under `item-key-optional` an unkeyed
  item passes and a keyed item is held to `item-pattern` and its
  `nested` (covers AC_contract-criteria); `nested` on a prose section,
  a nested key with two captures, and the flag with no `item-key` are
  each a finding on the schema (covers AC_contract-criteria); the
  grammar's schema check accepts the two keys (covers
  AC_contract-criteria).

### Slice 2: A contract's promise carries its criteria

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `pack/knowledge/schemas/contract.md` — the Behaviour and
  Stability rules gain a `nested` rule: `item-key` `AC_`, the EARS
  tag pattern, `required: false`; the section descriptions say a
  promise MAY carry the criteria that check it and how a plan case
  and a test cite one; the contract-style fragment and the worked
  example show a promise with two nested criteria; the glossary's
  Promise key entry names `AC_` as the contract's criterion prefix
  and drops the tracker prefixes it lists. Synced to
  `knowledge/schemas/contract.md`.
- Done-check: every contract on file passes unchanged; a scratch
  contract with nested criteria passes, and one with a nested item
  lacking its key or tag, or a criterion key equal to a promise key,
  fails naming the item; `superdev status` reports no drift.
- Cases: a contract whose promise nests two keyed tagged criteria
  passes (covers AC_contract-criteria); a nested item without a key
  fails naming it, and a criterion key repeated across the contract
  fails naming both items (covers AC_contract-criteria); the nine contracts on
  file, none nesting a criterion, pass (covers
  AC_contract-criteria-optional); a normative test reads the contract
  schema's Behaviour rule and finds the `nested` `AC_` key with
  `required: false` (covers AC_contract-criteria-optional).

### Slice 3: One issue template, and the issues on file rewritten

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `pack/knowledge/schemas/issue.md` — `type: Issue`, id
  `issue-\d{3}-[a-z0-9-]+`, `kind` enum `bug|feature|chore`
  required, `lifecycle` enum `open|done|wontfix` as `variant-key`;
  headings Summary, Context, Behaviour, Scope, Resolution, Comments,
  each `content: prose`, Resolution required under `[done, wontfix]`
  and prohibited under `[open]`, Scope and Comments optional; no
  item declaration anywhere; one example per state. `bug-report.md`,
  `feature-request.md` and `chore.md` deleted from the pack and the
  owned copies, and from the schemas index. `LIVE_LIFECYCLES` is
  `["open", "active"]`. The 51 issues rewritten by judgement into the
  template — the settled verdict under Resolution, criteria and
  expected behaviour as bullets under Behaviour, keys dropped —
  `framed` and `unframed` becoming `open`, refiled by `validate
  --fix`; every id drops its kind segment (`issue-030-filing-…`),
  and every reference to an old id across the knowledge, the tests
  and the changelog is rewritten with it; `issues/framed/` and `issues/unframed/` removed; the issues
  index and the `issue-tracker` concept and its pack skeleton
  describe the template and the three states; `/file` writes it.
  The normative tests that pin the three kinds, the framed state and
  `LIVE_LIFECYCLES` are rewritten to pin the template.
- Done-check: `superdev validate` passes with 51 issues under `open/`,
  `done/` and `wontfix/` and no other issue folder; no `BugReport`,
  `FeatureRequest` or `Chore` type in the tree; `superdev status`
  reports no drift.
- Cases: an issue with `kind: feature`, `lifecycle: open` and the six
  headings passes, with prose, bullets or both under Behaviour
  (covers AC_issue-schema, AC_issue-plain); an open issue carrying
  Resolution fails, and a done issue without it fails (covers
  AC_issue-resolution); a document typed `BugReport` is reported as
  naming no schema (covers AC_old-kinds-gone); the issue schema's
  rules carry no `item-key`, `item-pattern` or `nested` (covers
  AC_issue-schema); a normative sweep test finds every issue on file
  typed `Issue` in its lifecycle folder and no `framed` or `unframed`
  folder (covers AC_issue-sweep); `sokf_search` ranks an `open` issue
  live and a `done` one settled, and no `framed` value ranks live
  (covers AC_live-lifecycles); `/file`'s text names the template's
  headings and `lifecycle: open` (covers AC_issue-schema).

### Slice 4: One plan template, and the plans on file rewritten

- [ ] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `pack/knowledge/schemas/plan.md` — `type: Plan`, id
  `plan-\d{3}-[a-z0-9-]+`, `lifecycle` enum `open|done|abandoned`;
  title heading `Plan: …` with an optional Request line; headings
  Goal (prose), Contract changes (bullet-list — one bullet per
  contract touched naming the promises and criteria added, changed or
  withdrawn, or the single bullet "none"), Work blocks with `Block
  n: …` subsections (bullet-list: Done checkbox, Depends-on, Change,
  Done-check, Cases — a case citing the contract criteria it covers
  by key where one exists), Deferred decisions (bullet-list,
  optional); one example. `feature-plan.md` and `adhoc-plan.md`
  deleted from the pack, the owned copies and the schemas index. The
  36 plans on file rewritten — Slices become Work blocks, an ad-hoc
  plan's sections fold into Goal and Work blocks, a feature plan's
  Contract changes recovered from its issue's contract links — and
  this plan with them; the plans index follows. The normative tests
  that pin the two kinds are rewritten.
- Done-check: `superdev validate` passes with every plan typed `Plan`;
  no `FeaturePlan` or `AdhocPlan` type in the tree; `superdev status`
  reports no drift.
- Cases: a plan with Goal, Contract changes, two blocks and no
  Deferred decisions passes (covers AC_plan-schema); a plan without
  Contract changes fails naming the heading (covers AC_plan-schema);
  a document typed `FeaturePlan` is reported as naming no schema
  (covers AC_old-kinds-gone); a normative sweep test finds every plan
  on file typed `Plan` (covers AC_plan-schema).

### Slice 5: Scope, and contract-design as its sub-skill

- [ ] Done — ticked by integrate at merge.
- Depends-on: 3, 4.
- Change: `pack/knowledge/skills/scope/SKILL.md` — persona, bootstrap
  reads (the issue or the request, the contracts, the plan schema),
  steps: branch (`feature/<nnn>-<slug>` after the issue,
  `adhoc/<nnn>-<slug>` after the plan), `/grill-me` where the design
  is open, decide the contract changes and call `/contract-design`
  when any, `/research`, `/design`, `/prototype` where they apply,
  write the plan, `/double-check`, validate, commit, hand to
  `/build`. `contract-design/SKILL.md` rewritten as the sub-skill:
  input one plan's Contract changes, output the contract edits and
  ADRs, hands back to `/scope`. `frame/`, `feature-plan/` and
  `adhoc-plan/` deleted from the pack and the owned copies, and their
  `PROJECT.md` files with them; `pack/manifest.rs` and
  `content/layout.rs` fixtures that name `frame` renamed; `how-do-i`
  and `maintain` point at `/scope`; `/file` says `/scope` takes an
  issue up. The `development-procedure` concept's branch convention
  names the plan for ad-hoc work.
- Done-check: `.claude/skills/scope/` and
  `.claude/skills/contract-design/` exist, `frame`, `feature-plan`
  and `adhoc-plan` do not, in the pack and the owned copies; `superdev
  status` reports no drift; the pack's manifest test passes.
- Cases: a normative test reads `scope/SKILL.md` and finds the branch
  step, the `/grill-me`, `/contract-design`, `/double-check` calls and
  the hand-off to `/build` (covers AC_scope-skill); a normative test
  finds no `frame`, `feature-plan` or `adhoc-plan` skill directory
  in the pack or under `.claude/skills/` (covers AC_scope-skill); a
  normative test reads `contract-design/SKILL.md` and finds its
  input is a plan's contract changes and its hand-off is `/scope`
  (covers AC_contract-design-sub); the pack manifest lists `scope`
  and not `frame` (covers AC_scope-skill).

### Slice 6: Build, execute-plan, accept, and the workflow text

- [ ] Done — ticked by integrate at merge.
- Depends-on: 5.
- Change: `build/SKILL.md` — reads the plan, works the blocks in
  order: tests, code, the block's own tests and the tests it touches,
  commit; after the last block the full build, tests, lint and
  `superdev validate` once, the changelog and the knowledge, the merge
  on the branch, the plan set `done`; no review step; the
  contract-change gate returns to `/scope`. `execute-plan/SKILL.md`
  replaces `execute-feature-plan/` — drives `/build` over the blocks
  with `superdev run`, deferred decisions as today, the gate that
  returned to frame or contract-design returns to `/scope`.
  `accept/SKILL.md` — invoked by the user; `/code-review` first, a
  finding the user wants fixed returning to `/build`; the contract
  criteria walked on the merged code; the documentation checked; the
  issue set `done`. `integrate/` deleted. `AGGREGATOR_PREFIX` in
  `pipeline.rs` — the flow `FILE → SCOPE → BUILD → ACCEPT`, accept
  optional, the sub-skills listed under scope, the edges rewritten;
  `.agents/superdev.md` synced. Contract-009's run-state prose, where
  it names the driver, follows.
- Done-check: `.claude/skills/execute-plan/` exists,
  `execute-feature-plan` and `integrate` do not; `.agents/superdev.md`
  carries the four-phase flow; `superdev status` reports no drift.
- Cases: a normative test reads `build/SKILL.md` and finds the
  per-block test step, the single full-suite step after the last
  block, and no review step (covers AC_build-verifies-once); a
  normative test finds no `integrate` skill (covers
  AC_build-verifies-once); a normative test reads
  `execute-plan/SKILL.md` and finds `superdev run begin`, `advance`,
  `end` and the `/build` loop, and finds no `execute-feature-plan`
  skill (covers AC_execute-plan); a normative test reads
  `accept/SKILL.md` and finds the `/code-review` step before the
  criteria walk and the return to `/build` (covers AC_accept-reviews);
  a normative test reads `.agents/superdev.md` and finds the flow
  `FILE → SCOPE → BUILD → ACCEPT`, an `optional` mark on accept, and
  no `CONTRACT-DESIGN`, `FEATURE-PLAN` or `INTEGRATE` phase (covers
  AC_workflow-text).

### Slice 7: The concepts and the records

- [ ] Done — ticked by integrate at merge.
- Depends-on: 6.
- Change: `definition-of-done`, `development-procedure`,
  `issue-tracker`, `glossary` (the phases, `Scope`, `Work block`,
  `Plan`, the retired terms `Frame`, `Slice`, `Framed`, `Unframed`,
  `Integrate`), `constraints-non-goals` where it names a phase,
  `project-overview`, the README's workflow section and their pack
  skeletons say the four-phase workflow and the issue and plan
  templates; the changelog's Unreleased section carries the change
  and a migration note for a managed repository — the three issue
  kinds, the two plan kinds and the five skills retired, the folders
  moved; ADR-046's index entry unchanged, ADR-050 and ADR-051
  listed; a `migration-guide` if the managed-repository migration
  needs steps beyond `validate --fix`.
- Done-check: `superdev validate` passes; no concept, skill or README
  section names `/frame`, `/feature-plan`, `/adhoc-plan`,
  `/integrate`, `/execute-feature-plan`, `framed` or `unframed` except
  as history in an ADR, an issue or the changelog; `superdev status`
  reports no drift.
- Cases: a normative test greps the concepts, the skills and the
  README for the retired names and finds none (covers
  AC_workflow-text); a normative test finds ADR-050 active with
  `supersedes` links to ADR-031 and ADR-048, both under
  `adrs/deprecated/`, and a `references` link to ADR-046 (covers
  AC_adrs); the glossary defines Scope, Work block and Plan and no
  longer defines Frame, Slice, Framed or Unframed (covers
  AC_workflow-text).

## Deferred decisions

- Slice 1 (built; the question is wording): contract-010's "Nested
  items" paragraph says a marker beyond the declared depth "is text of
  the item it sits in", and its "What an item is" paragraph says a
  nested item's lines are dropped from the item above. The validator
  follows the latter — the undeclared marker's lines are excluded from
  the item's text, so a promise's pattern never reads a deeper note.
  Should the "Nested items" paragraph say "is dropped from the item it
  sits in, as an undeclared nested item's lines are"? Blocks nothing.
- Slice 3 (built; the question is sequencing): the tracker holds 52
  issues, not 51 — I052 itself is the 52nd — and from this slice the
  `/frame` and `/accept` skills write `lifecycle: framed` and
  `lifecycle: unframed`, values `schema-issue` refuses, until slices 5
  and 6 rewrite them; plan-027's own cases cite I052's `AC_` keys,
  which the issue no longer carries, until slice 4 rewrites the plans
  to cite contract criteria. Should slices 5 and 6 land before the
  branch merges, or should `/frame` and `/accept` take a one-line fix
  now? Blocks nothing on the branch.
- Slice 2 (built; the question is scope): the nested rule carries
  `item-key`, `item-pattern` and `required`, as the plan lists, and no
  `item-prohibited-pattern`, so a criterion carrying `MUST` beside its
  `SHALL` passes while a promise's does not. Should the criterion be
  held to the retired-verb and one-verb rule as a promise is? Blocks
  nothing.

<!-- sokf:links -->
[sokf:adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept]: /knowledge/adrs/active/adr-050-keys-and-ears-live-in-the-contracts-and-the-workflow-is-file-scope-build-accept.md
[sokf:adr-051-a-section-rule-declares-nested-items-and-an-optional-key]: /knowledge/adrs/active/adr-051-a-section-rule-declares-nested-items-and-an-optional-key.md
[sokf:contract-010-interface-document-schemas]: /knowledge/contracts/internal/active/contract-010-interface-document-schemas.md
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/open/issue-052-the-workflow-carries-more-process-than-it-needs.md
