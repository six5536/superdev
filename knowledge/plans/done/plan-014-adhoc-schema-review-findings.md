---
type: AdhocPlan
id: plan-014-adhoc-schema-review-findings
title: Bring every schema in line with its own rules and the workflow
description: The schema review's findings land — worked examples satisfy their own schemas, the report schemas gain identity and filing, stale vocabulary leaves, the contract, ADR and idea shapes unify, and the pack mirror stays byte-identical.
lifecycle: done
links:
  - rel: references
    to: issue-022-a-schemas-worked-example-is-checked-by-nothing
    note: This plan hand-fixes the examples and appends the type-mismatch evidence.
  - rel: relates-to
    to: plan-013-feature-workflow-autonomy
    note: W5 applies the feature-plan schema additions only where plan-013 has not.
  - rel: references
    to: adr-020-a-blocked-run-ends
    note: Names the Deferred decisions section W5 adds when plan-013 has not.
---

# Plan: bring every schema in line with its own rules and the workflow

## Context

A full review of the 53 schemas in `knowledge/schemas/` found 26 worked
examples breaking their own schema's frontmatter rules, seven report schemas
whose documents cannot conform to SOKF, vocabulary from the removed spec
workflow, and three document shapes that differ inside their own family.
This plan lands every finding. It is documentation only; the validator and
the rest of the code are untouched.

## Facts

- 26 of the 53 `example:` blocks break their own schema's frontmatter
  constraints, checked mechanically this session (script in the Appendix):
  18 carry a `type` the schema's `const` refuses — the pre-P008 vocabulary
  `Reference`, `Convention`, `Policy`, `Procedure` — and 8 omit frontmatter
  the schema constrains (`code-review`, `security-review`, `investigation`,
  `postmortem`, `status-update`, `release-notes`, `migration-guide`,
  `contract-interface`).
- The live documents all carry the `const` types — `knowledge/architecture.md`
  reads `type: Architecture`, and likewise for every named-file concept — so
  the schemas are right and the examples are stale.
- [issue-022][sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]
  records the mechanism with id evidence only; the five ids it lists no
  longer break their patterns.
- The seven report schemas constrain no `id` and say documents are "filed in
  the knowledge as a concept", while SOKF SPEC §11 requires every concept to
  carry a unique `id` (`.agents/sokf/SPEC.md:397-399`). Each also carries the
  unparseable sentence "matched by name (`**/*code-review*.md`); the source
  names no filing directory, and filed in the knowledge as a concept". No
  document of any of the seven kinds exists yet (`find knowledge` returns
  none), so no migration is needed.
- The spec document type was removed by P012, yet five schemas still route
  work to it: `testing-strategy.md:12`, `issue-tracker.md:32` and `:63`,
  `idea.md:18` and `:65`, `glossary.md:16` and `:34`, `contract-ui.md:15`.
- 15 contract schemas say the number series runs "public and private
  together" (`grep -rl 'private together' knowledge/schemas` = 15); the
  folders and the index say internal.
- `knowledge/schemas/index.md:50` omits "EARS acceptance criteria" from the
  feature-request summary; `schema-schemas-index` requires the summary to
  match the schema's description, and it is the only entry that drifts.
- The contract family splits by shape: the six public contracts open at a
  bare `# Commands`-style heading (`contract-002-cli-superdev.md:` first
  heading `# Commands`), the two interface contracts at a title heading with
  level-2 sections (`contract-007-interface-pack-resolution.md`). Every
  other filed family opens with a title heading and level-2 sections.
- All 21 ADRs under `knowledge/adrs/active/` carry a `- Status:` bullet
  duplicating what `lifecycle` and the SOKF `supersedes` relationship
  already express (SPEC §8 defines `supersedes`/`superseded-by`).
- `idea-001` and its schema put `# Motivation` and four more level-1
  headings beside the `# Idea:` title heading.
- `schema-feature-request` says nothing about linking the contracts a
  feature touches, though the workflow requires it and `schema-bug-report`
  states its own link convention. Nothing names where `/accept` records its
  verdict. `schema-feature-plan` defines case coverage only against numbered
  acceptance criteria, while its title section admits a bug-framed plan.
- [plan-013][sokf:plan-013-feature-workflow-autonomy], which supersedes
  plan-004, carries a slice for the `Depends-on:` line and the
  `## Deferred decisions` section. `schema-feature-plan` carries neither
  today, and the slice is unticked.
- `diff -rq knowledge/schemas pack/knowledge/schemas` is empty: the pack
  mirrors the live schemas byte for byte, so every schema edit must land in
  both trees.
- `superdev validate` passes today: 0 errors, 5 warnings, all in
  `.claude/skills/` frontmatter and unrelated to this plan.

## Goal

Every schema states rules its own worked example satisfies, in the
workflow's current vocabulary, with one heading shape per document family —
in the live knowledge and the pack alike.

## Outcomes

- O1 — the example-conformance check prints nothing: every worked example
  satisfies its own schema's frontmatter constraints.
- O2 — each report schema names an id pattern and a filing directory, and
  its filing sentence parses.
- O3 — no schema names the spec as a workflow document, calls internal
  contracts private, or drifts from the schemas index.
- O4 — contracts, ADRs and ideas each have one shape: a title heading and
  level-2 sections, and ADR state lives in `lifecycle` and links alone.
- O5 — the feature-request schema states the contract-link convention and
  the acceptance verdict's home; the feature-plan schema defines coverage
  for bug-framed plans and, where plan-013 has not landed them, carries
  `Depends-on:` and `## Deferred decisions`.
- O6 — `knowledge/schemas` and `pack/knowledge/schemas` are byte-identical
  and `superdev validate` passes.

## Non-goals

- The example checker itself. That is issue-022's feature; this plan fixes
  the instances by hand and strengthens the issue's evidence.
- Enforcing index entries or index shape — issues 010, 011 and 012.
- Any code change. Type-driven dispatch already handles a renamed `const`;
  the validator is untouched.
- Plan-013's run facility, branching and skills. Only the two schema
  sections it has not landed are covered, conditionally, by W5.
- Backfilling `Depends-on:` into settled plans — plan-013's inherited position,
  kept.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | Every schema's worked example satisfies that schema's frontmatter constraints | O1 |
| FR-2 | Each of the seven report schemas declares an id pattern `^{kind}-\d{3}-[a-z0-9-]+$` and files under `knowledge/reports/`, in a sentence that parses | O2 |
| FR-3 | No schema routes work to a spec document, says "public and private", or carries an index summary differing from its description; `schema-constraints-non-goals` uses `const: ConstraintsNonGoals` and the live concept matches | O3 |
| FR-4 | Every contract schema and every live contract opens with a title heading (`# {Kind} contract: …`) over level-2 sections | O4 |
| FR-5 | The ADR schema and all 21 live ADRs carry no body Status line; supersession is a `supersedes` link plus `lifecycle: deprecated`, and a proposed decision is `status: draft` | O4 |
| FR-6 | The idea schema and `idea-001` put their sections at level 2 under the title heading | O4 |
| FR-7 | `schema-feature-request` states the contract-link convention and that `/accept` records its verdict in the settled-issue section; `schema-feature-plan` defines what a case covers when the framed issue is a bug | O5 |
| FR-8 | If `schema-feature-plan` lacks them when W5 runs, it gains `Depends-on:` per slice and `## Deferred decisions` per ADR-018 and [ADR-020][sokf:adr-020-a-blocked-run-ends]; if plan-013 landed them, W5 changes nothing | O5 |
| FR-9 | After every change, `diff -rq knowledge/schemas pack/knowledge/schemas` is empty and `superdev validate` passes | O6 |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | Contracts adopt the title-heading + level-2-section shape | Drop the interface contracts' titles to match the public style, touching 3 files instead of ~20 | every other filed family opens with a title heading; the cheap direction leaves contracts the only headless documents in the tree |
| D-2 | ADR state: `lifecycle` + a `supersedes` link + `status: draft` for proposed; the body keeps Date and Deciders only | keep the Status bullet | P011 replaced dual state vocabularies with one; SOKF already defines the supersession edge |
| D-3 | All seven report kinds file in one `knowledge/reports/` directory, ids `^{kind}-\d{3}-[a-z0-9-]+$` numbered per kind | one directory per kind | `plans/` already holds two kinds in one directory with the id naming the kind; seven near-empty directories help nobody |
| D-4 | W5 checks `schema-feature-plan` at execution and edits only what plan-013 has not landed | fix unconditionally now | plan-013 owns the change and is in progress on this branch; two owners of one edit is a merge conflict by design |
| D-5 | Examples are fixed by hand now, and the type-mismatch evidence is appended to issue-022's Comments | wait for issue-022's checker | agents copy the examples today; the checker is a feature with no delivery date |

## Workstreams

### W1: The contract family

Depends on: none.

1. Unify the shape — in the 14 public contract schemas, add a required
   title heading-pattern (`^{Kind} contract: .+$`) and move every section to
   level 2; `contract-interface.md` already has the shape.
2. Fix the wording — "public and private together" becomes "public and
   internal together" in all 15 contract schemas; `contract-ui.md:15` loses
   its spec sentence in favour of one contrasting the durable contract with
   the feature-request.
3. Fix the examples — every contract example gains or keeps conforming
   frontmatter (`contract-interface`'s lacks it entirely) and matches the
   unified shape.

### W2: The report family

Depends on: none.

1. Give the seven schemas identity — an `id` pattern
   (`^code-review-\d{3}-[a-z0-9-]+$` and likewise per kind) and the filing
   statement: filed at `knowledge/reports/{kind}-{nnn}-{slug}.md`, listed in
   that directory's index, selected by frontmatter `type`.
2. Replace the garbled sentence — delete "matched by name …; the source
   names no filing directory, and filed in the knowledge as a concept" from
   all seven.
3. Fix the examples — each gains conforming frontmatter with a
   pattern-satisfying id.

### W3: The remaining schemas and their index

Depends on: none.

1. Fix the 18 stale example types — the example's `type` becomes the
   schema's `const` in `architecture`, `architectural-rules`,
   `software-components`, `configuration`, `directory-structure`,
   `technology-stack`, `dependency-policy`, `coding-standards`,
   `testing-strategy`, `error-handling`, `security-requirements`,
   `development-commands`, `development-procedure`, `release-procedure`,
   `definition-of-done`, `issue-tracker`, `constraints-non-goals` and
   `visual-system`.
2. Remove the spec vocabulary — `testing-strategy.md:12` points cases at
   the feature-plan's slices; `issue-tracker.md:32` and `:63` route a
   behaviour decision to a feature-request and its contracts; `idea.md:18`
   and `:65` send a taken-up idea to a feature-request or contract;
   `glossary.md:16` and `:34` read "code, issues and plans".
3. Rename the asymmetric const — `schema-constraints-non-goals` moves to
   `const: ConstraintsNonGoals`; the live concept's `type` moves with it in
   the same commit.
4. Restructure the ADR schema — the title section keeps Date and Deciders;
   Status leaves; the prose states supersession as a `supersedes` link from
   the new ADR plus `lifecycle: deprecated` on the old, and `status: draft`
   as the proposed state; the example follows.
5. Restructure the idea schema — sections move to level 2 under the title
   heading; the example follows.
6. State the workflow conventions — `feature-request.md` gains the
   contract-link sentence (mirroring `bug-report.md`'s) and a note in the
   settled-issue section that `/accept` records its verdict there;
   `index.md`'s feature-request summary regains "EARS acceptance criteria".

### W4: The live documents

Depends on: W1, W3.

1. Restyle the contracts — the six public contracts gain their title
   heading and demote their sections to level 2; `contract-007` and
   `contract-009` are checked against the schema and left alone.
2. Strip the ADR Status bullets — delete the `- Status:` line from all 21
   ADRs; every one is accepted and active, so nothing else changes.
3. Restyle `idea-001` — its five sibling level-1 headings become level 2.
4. Move the renamed type — `knowledge/constraints-non-goals.md` reads
   `type: ConstraintsNonGoals` (paired with W3 step 3; both land in one
   commit so no state has a schema-less document).

### W5: The feature-plan schema

Depends on: none.

1. Define bug coverage — the slice Cases description says what a case
   covers when the framed issue is a bug: the numbered repro steps and the
   expected behaviour stand in for criteria numbers.
2. Conditionally add plan-013's sections — if the schema still lacks them:
   `Depends-on:` beside Cases (slice numbers or `none`, dependencies bind
   where list order only reads) and `## Deferred decisions` (the questions a
   blocked run leaves, per ADR-018 and ADR-020). If plan-013 landed either,
   leave it exactly as landed.

### W6: Mirror, evidence, verification

Depends on: W1, W2, W3, W4, W5.

1. Mirror the schemas — copy `knowledge/schemas/` over
   `pack/knowledge/schemas/` and confirm the diff is empty.
2. Append the evidence — a dated Comments entry on issue-022: 26 examples
   broke their own frontmatter constraints (18 types, 8 missing blocks),
   found by hand review and fixed by this plan; the count strengthens the
   case for the checker.
3. Record the change — a CHANGELOG.md Unreleased entry for the pack-visible
   schema reshape.
4. Verify — run every Acceptance row; `superdev validate --fix` places
   files and regenerates definition blocks along the way.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `knowledge/schemas/contract-*.md` (16) | modified — shape, wording, examples | W1 |
| `knowledge/schemas/{code,security}-review.md`, `investigation.md`, `postmortem.md`, `status-update.md`, `release-notes.md`, `migration-guide.md` (7) | modified — id, filing, examples | W2 |
| `knowledge/schemas/` the 18 stale-type schemas, `adr.md`, `idea.md`, `feature-request.md`, `glossary.md`, `index.md` (23) | modified — examples, vocabulary, structure, conventions | W3 |
| `knowledge/schemas/feature-plan.md` | modified — bug coverage; conditionally Depends-on and Deferred decisions | W5 |
| `knowledge/contracts/public/active/*.md` (6) | modified — title heading, sections to level 2 | W4 |
| `knowledge/adrs/active/*.md` (21) | modified — Status bullet removed | W4 |
| `knowledge/ideas/idea-001-schemas-carry-a-reading-reminder.md` | modified — sections to level 2 | W4 |
| `knowledge/constraints-non-goals.md` | modified — `type: ConstraintsNonGoals` | W4 |
| `pack/knowledge/schemas/**` (53) | modified — mirror of every schema change | W6 |
| `knowledge/issues/open/issue-022-a-schemas-worked-example-is-checked-by-nothing.md` | modified — evidence appended to Comments | W6 |
| `CHANGELOG.md` | modified — Unreleased entry for the schema reshape | W6 |

## Acceptance

| Check | Verifies |
|-------|----------|
| The Appendix conformance script prints nothing | FR-1 |
| `grep -L "pattern:" knowledge/schemas/{code-review,security-review,investigation,postmortem,status-update,release-notes,migration-guide}.md` prints nothing, and each names `knowledge/reports/` | FR-2 |
| `grep -rilE '\ba spec\b|\bspecs\b' knowledge/schemas` prints nothing (references to the SOKF spec by that name do not match); `grep -rl 'private together' knowledge/schemas` prints nothing | FR-3 |
| The schemas-index summary diff (Appendix) prints nothing | FR-3 |
| `grep -m1 '^#' knowledge/contracts/*/active/*.md` matches `contract: ` on all 8 | FR-4 |
| `grep -rl '^- Status:' knowledge/adrs/active` prints nothing | FR-5 |
| `grep -c '^# ' knowledge/ideas/idea-001-*.md` prints 1 | FR-6 |
| `grep -c 'implements' knowledge/schemas/feature-request.md` is at least 1, and the settled-issue section names accept | FR-7 |
| `grep -l 'Depends-on' knowledge/schemas/feature-plan.md` and `grep -l 'Deferred decisions' knowledge/schemas/feature-plan.md` both hit, whichever plan landed them | FR-8 |
| `diff -rq knowledge/schemas pack/knowledge/schemas` prints nothing | FR-9 |
| `superdev validate` reports PASS with 0 errors | FR-9 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- `knowledge/plans/index.md` lists this plan, and its `lifecycle` reads done.
- Issue-022's Comments section carries the appended evidence.
- `CHANGELOG.md` names the schema reshape under Unreleased.

## Risks

- Risk: collision with plan-013's in-flight work on this branch, which owns
  the feature-plan sections and is adding ADRs. Mitigation: D-4's
  execution-time check, and landing W5 last among the schema edits; early
  signal: a merge conflict in `feature-plan.md` or a 22nd ADR appearing.
- Risk: renaming `Constraints` to `ConstraintsNonGoals` leaves a window
  where the document names a type no schema carries. Mitigation: W3 step 3
  and W4 step 4 land in one commit; early signal: `superdev validate`
  reporting an ungoverned document.
- Risk: demoting live contract headings breaks inbound section-addressed
  reads (`sokf_read` heading paths) somewhere unseen. Mitigation: run
  `superdev validate` after W4 and check `sokf_graph` for the eight
  contracts; early signal: a link warning naming a contract.
- Risk: the pack mirror is edited directly and drifts from the live tree on
  a later tweak. Mitigation: W6 copies wholesale rather than editing twice,
  and the diff check is an Acceptance row.

## Out-of-band notes

- The skill bootstraps that read `.agents/core.md` — a path the aggregator
  rename removed — were repointed at `.agents/superdev.md` in the session
  that filed this plan, in `.claude/skills/` and the pack alike.
- The schema reshape reaches managed repos only through a pack release;
  cutting one is release-procedure work outside this plan.

## Appendix

### Findings-to-workstream map

| Finding | Workstream |
|---------|------------|
| 26 examples break their own frontmatter rules | W1-W3, verified in W6 |
| Report schemas: no id, garbled filing sentence | W2 |
| Spec vocabulary in five schemas | W1 (contract-ui), W3 |
| "Public and private" in 15 contract schemas | W1 |
| Schemas-index summary drift | W3 |
| Feature-request: contract links, accept's verdict home | W3 |
| Feature-plan: bug coverage; Depends-on; Deferred decisions | W5 |
| Contract shape split; idea sibling headings; ADR dual state | W1, W3, W4 |
| `Constraints` const asymmetry | W3, W4 |

### The example-conformance check

Run from `knowledge/schemas/`; prints one line per violation, nothing on a
clean tree. It parses each schema's fenced YAML for the frontmatter `type`
const and `id` pattern or const, then checks the `example:` block's
frontmatter against them, including that a block exists at all where the
schema constrains one.

```python
import re, pathlib
for p in sorted(pathlib.Path('.').glob('*.md')):
    if p.name == 'index.md': continue
    t = p.read_text()
    m = re.search(r'^````yaml\n(.*?)^````', t, re.S | re.M)
    if not m: continue
    y = m.group(1)
    tc = re.search(r'^frontmatter:.*?^  type:\n    const: (\S+)', y, re.S | re.M)
    ip = re.search(r"^  id:\n    (pattern|const): '?([^'\n]+)'?", y, re.M)
    ex = re.search(r'^example: \|\n(.*)', y, re.S | re.M)
    if not ex: continue
    exb = ex.group(1)
    has_fm = re.match(r'\s*---\n', exb)
    et = re.search(r'^  type: (\S+)', exb, re.M)
    ei = re.search(r'^  id: (\S+)', exb, re.M)
    if tc and not has_fm: print(f'{p.name}: example lacks frontmatter')
    if tc and et and et.group(1) != tc.group(1):
        print(f'{p.name}: example type {et.group(1)} != {tc.group(1)}')
    if ip and ei:
        kind, val = ip.groups()
        ok = ei.group(1) == val if kind == 'const' else re.match(val, ei.group(1))
        if not ok: print(f'{p.name}: example id {ei.group(1)} violates {val}')
```

### The index summary diff

Run from `knowledge/schemas/`; prints each index entry whose summary
differs from its schema's frontmatter description, nothing on a clean tree.

```python
import re, pathlib
idx = pathlib.Path('index.md').read_text()
for title, sid, summ in re.findall(r'\* \[([^\]]+)\]\[sokf:(schema-[a-z-]+)\] - (.+)', idx):
    fn = 'schemas-index.md' if sid == 'schema-schemas-index' else sid.removeprefix('schema-') + '.md'
    d = re.search(r'^description: (.+)$', pathlib.Path(fn).read_text(), re.M).group(1)
    if summ.strip().rstrip('.').lower() != d.strip().rstrip('.').lower():
        print(f'{fn}: index and description differ')
```

<!-- sokf:links -->
[sokf:adr-020-a-blocked-run-ends]: /knowledge/adrs/active/adr-020-a-blocked-run-ends.md
[sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]: /knowledge/issues/done/issue-022-a-schemas-worked-example-is-checked-by-nothing.md
[sokf:plan-013-feature-workflow-autonomy]: /knowledge/plans/done/plan-013-feature-workflow-autonomy.md
