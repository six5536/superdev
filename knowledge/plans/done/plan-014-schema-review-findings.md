---
type: Plan
id: plan-014-schema-review-findings
title: Bring every schema in line with its own rules and the workflow
description: The schema review's findings land — worked examples satisfy their own schemas, the report schemas gain identity and filing, stale vocabulary leaves, the contract, ADR and idea shapes unify, and the pack mirror stays byte-identical.
lifecycle: done
links:
  - rel: references
    to: issue-022-a-schemas-worked-example-is-checked-by-nothing
    note: This plan hand-fixes the examples and appends the type-mismatch evidence.
  - rel: relates-to
    to: plan-013-workflow-autonomy
    note: W5 applies the feature-plan schema additions only where plan-013 has not.
  - rel: references
    to: adr-020-a-blocked-run-ends
    note: Names the Deferred decisions section W5 adds when plan-013 has not.
---

# Plan: Bring every schema in line with its own rules and the workflow

## Goal

Every schema states rules its own worked example satisfies, in the
workflow's current vocabulary, with one heading shape per document
family — in the live knowledge and the pack alike. The work is
documentation only: the validator and the rest of the code stay
untouched, and type-driven dispatch already handles a renamed `const`.

A full review of the 53 schemas in `knowledge/schemas/` found the
gaps this plan closes:

- 26 of the 53 `example:` blocks break their own schema's frontmatter
  constraints, checked mechanically by the conformance script below:
  18 carry a `type` the schema's `const` refuses — the pre-P008
  vocabulary `Reference`, `Convention`, `Policy`, `Procedure` — and 8
  omit frontmatter the schema constrains (`code-review`,
  `security-review`, `investigation`, `postmortem`, `status-update`,
  `release-notes`, `migration-guide`, `contract-interface`). The live
  documents all carry the `const` types — `knowledge/architecture.md`
  reads `type: Architecture`, and likewise for every named-file concept
  — so the schemas are right and the examples are stale.
- The seven report schemas constrain no `id` and say documents are
  "filed in the knowledge as a concept", while SOKF SPEC §11 requires
  every concept to carry a unique `id` (`.agents/sokf/SPEC.md:397-399`).
  Each also carries the unparseable sentence "matched by name
  (`**/*code-review*.md`); the source names no filing directory, and
  filed in the knowledge as a concept". No document of any of the seven
  kinds exists yet (`find knowledge` returns none), so no migration is
  needed.
- The spec document type was removed by P012, yet five schemas still
  route work to it: `testing-strategy.md:12`, `issue-tracker.md:32` and
  `:63`, `idea.md:18` and `:65`, `glossary.md:16` and `:34`,
  `contract-ui.md:15`.
- 15 contract schemas say the number series runs "public and private
  together" (`grep -rl 'private together' knowledge/schemas` = 15);
  the folders and the index say internal.
- `knowledge/schemas/index.md:50` omits "EARS acceptance criteria" from
  the feature-request summary; `schema-schemas-index` requires the
  summary to match the schema's description, and it is the only entry
  that drifts.
- The contract family splits by shape: the six public contracts open at
  a bare `# Commands`-style heading, the two interface contracts at a
  title heading with level-2 sections. Every other filed family opens
  with a title heading and level-2 sections, so the contracts adopt that
  shape rather than the cheaper direction, which would leave contracts
  the only headless documents in the tree.
- All 21 ADRs under `knowledge/adrs/active/` carry a `- Status:` bullet
  duplicating what `lifecycle` and the SOKF `supersedes` relationship
  already express (SPEC §8). ADR state becomes `lifecycle` plus a
  `supersedes` link, with `status: draft` for a proposed decision; the
  body keeps Date and Deciders.
- `idea-001` and its schema put `# Motivation` and four more level-1
  headings beside the `# Idea:` title heading.
- `schema-feature-request` says nothing about linking the contracts a
  feature touches, though the workflow requires it and
  `schema-bug-report` states its own link convention. Nothing names
  where `/accept` records its verdict. `schema-feature-plan` defines
  case coverage only against numbered acceptance criteria, while its
  title section admits a bug-framed plan.

Two constraints shape the approach. The seven report kinds file in one
`knowledge/reports/` directory with ids `^{kind}-\d{3}-[a-z0-9-]+$`
numbered per kind, because `plans/` already holds two kinds in one
directory and seven near-empty directories help nobody. And
[plan-013][sokf:plan-013-workflow-autonomy] owns the `Depends-on:` line
and the `## Deferred decisions` section of the feature-plan schema, and
is in flight on this branch: block 5 checks the schema at execution and
edits only what plan-013 has not landed, and runs last among the schema
edits, so one edit has one owner. The other risk is the rename window —
`Constraints` to `ConstraintsNonGoals` leaves the live document naming a
type no schema carries — closed by landing the schema rename and the
document rename in one commit.

`diff -rq knowledge/schemas pack/knowledge/schemas` is empty: the pack
mirrors the live schemas byte for byte, so every schema edit lands in
both trees, copied wholesale rather than edited twice. `superdev
validate` passes today: 0 errors, 5 warnings, all in `.claude/skills/`
frontmatter and unrelated to this plan.

Out of scope: the example checker itself, which is
[issue-022][sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]'s
feature — this plan fixes the instances by hand and appends the
type-mismatch evidence to that issue, because agents copy the examples
today and the checker has no delivery date. Also out of scope: index
entries and index shape (issues 010, 011 and 012); any code change;
plan-013's run facility, branching and skills; and backfilling
`Depends-on:` into settled plans.

Two skill bootstraps read `.agents/core.md`, a path the aggregator
rename removed; they were repointed at `.agents/superdev.md` in the
session that filed this plan, in `.claude/skills/` and the pack alike.
The schema reshape reaches managed repos only through a pack release,
which is release-procedure work outside this plan.

The example-conformance check, run from `knowledge/schemas/`, prints one
line per violation and nothing on a clean tree. It parses each schema's
fenced YAML for the frontmatter `type` const and `id` pattern or const,
then checks the `example:` block's frontmatter against them, including
that a block exists at all where the schema constrains one.

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

The index summary diff, run from the same directory, prints each index
entry whose summary differs from its schema's frontmatter description,
and nothing on a clean tree.

```python
import re, pathlib
idx = pathlib.Path('index.md').read_text()
for title, sid, summ in re.findall(r'\* \[([^\]]+)\]\[sokf:(schema-[a-z-]+)\] - (.+)', idx):
    fn = 'schemas-index.md' if sid == 'schema-schemas-index' else sid.removeprefix('schema-') + '.md'
    d = re.search(r'^description: (.+)$', pathlib.Path(fn).read_text(), re.M).group(1)
    if summ.strip().rstrip('.').lower() != d.strip().rstrip('.').lower():
        print(f'{fn}: index and description differ')
```

## Contract changes

- none.

## Work blocks

### Block 1: The contract family

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: the 16 contract schemas — the 14 public ones gain a required
  title heading-pattern (`^{Kind} contract: .+$`) and move every section
  to level 2, `contract-interface.md` already having the shape; "public
  and private together" becomes "public and internal together" in all
  15 that carry it; `contract-ui.md:15` loses its spec sentence for one
  contrasting the durable contract with the feature-request; every
  contract example gains or keeps conforming frontmatter
  (`contract-interface`'s lacks it entirely) and matches the unified
  shape.
- Done-check: the conformance script prints nothing for the contract
  schemas, and `grep -rl 'private together' knowledge/schemas` prints
  nothing.
- Cases:
  - manual: the conformance script reports no contract schema — checks
    that every contract example satisfies its own frontmatter
    constraints.
  - manual: no contract schema says "private together", and each names
    a title heading-pattern — checks the corrected vocabulary and the
    unified shape.

### Block 2: The report family

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: the seven report schemas — `code-review`, `security-review`,
  `investigation`, `postmortem`, `status-update`, `release-notes` and
  `migration-guide` — gain an `id` pattern
  (`^code-review-\d{3}-[a-z0-9-]+$` and likewise per kind) and the
  filing statement: filed at `knowledge/reports/{kind}-{nnn}-{slug}.md`,
  listed in that directory's index, selected by frontmatter `type`; the
  garbled filing sentence is deleted; each example gains conforming
  frontmatter with a pattern-satisfying id.
- Done-check: `grep -L "pattern:"` over the seven prints nothing, and
  each names `knowledge/reports/`.
- Cases:
  - manual: each of the seven declares an id pattern and names
    `knowledge/reports/`, in a sentence that parses — checks that a
    report document can conform to SOKF.
  - manual: the conformance script reports none of the seven — checks
    that each example carries frontmatter with a conforming id.

### Block 3: The remaining schemas and their index

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: 23 schemas — the 18 stale example types become the schema's
  `const` (`architecture`, `architectural-rules`, `software-components`,
  `configuration`, `directory-structure`, `technology-stack`,
  `dependency-policy`, `coding-standards`, `testing-strategy`,
  `error-handling`, `security-requirements`, `development-commands`,
  `development-procedure`, `release-procedure`, `definition-of-done`,
  `issue-tracker`, `constraints-non-goals` and `visual-system`); the
  spec vocabulary leaves (`testing-strategy.md:12` points cases at the
  feature-plan's slices; `issue-tracker.md:32` and `:63` route a
  behaviour decision to a feature-request and its contracts;
  `idea.md:18` and `:65` send a taken-up idea to a feature-request or
  contract; `glossary.md:16` and `:34` read "code, issues and plans");
  `schema-constraints-non-goals` moves to `const: ConstraintsNonGoals`;
  the ADR schema drops Status and states supersession as a `supersedes`
  link plus `lifecycle: deprecated`, with `status: draft` for a proposed
  decision; the idea schema's sections move to level 2 under the title
  heading; `feature-request.md` gains the contract-link sentence
  mirroring `bug-report.md`'s and a note that `/accept` records its
  verdict in the settled-issue section; `index.md`'s feature-request
  summary regains "EARS acceptance criteria".
- Done-check: the conformance script and the index summary diff both
  print nothing; no schema names a spec as a workflow document.
- Cases:
  - manual: the conformance script prints nothing across all 53 schemas
    — checks that every worked example satisfies its own frontmatter
    constraints.
  - manual: `grep -rilE '\ba spec\b|\bspecs\b' knowledge/schemas` prints
    nothing, references to the SOKF spec by that name aside — checks
    that no schema routes work to a removed document type.
  - manual: the index summary diff prints nothing — checks that each
    index summary matches its schema's description.
  - manual: `feature-request.md` names the contract links and the
    settled-issue home of `/accept`'s verdict — checks that the schema
    states the workflow's conventions.

### Block 4: The live documents

- [x] Done — ticked at merge.
- Depends-on: 1, 3.
- Change: the six public contracts gain their title heading and demote
  their sections to level 2, `contract-007` and `contract-009` being
  checked against the schema and left alone; the `- Status:` line leaves
  all 21 ADRs, every one accepted and active; `idea-001`'s five sibling
  level-1 headings become level 2; `knowledge/constraints-non-goals.md`
  reads `type: ConstraintsNonGoals`, in the same commit as block 3's
  rename so no state has a schema-less document.
- Done-check: every contract's first heading carries `contract: `, no
  ADR carries a Status bullet, `idea-001` has one level-1 heading, and
  `superdev validate` reports no ungoverned document.
- Cases:
  - manual: `grep -m1 '^#' knowledge/contracts/*/active/*.md` matches
    `contract: ` on all 8 — checks one heading shape for the family.
  - manual: `grep -rl '^- Status:' knowledge/adrs/active` prints nothing
    — checks that ADR state lives in `lifecycle` and links alone.
  - manual: `grep -c '^# ' knowledge/ideas/idea-001-*.md` prints 1 —
    checks that the idea's sections sit at level 2.
  - manual: `sokf_graph` resolves all eight contracts and `superdev
    validate` reports no link warning — checks that demoting the
    headings broke no section-addressed read.

### Block 5: The feature-plan schema

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: `knowledge/schemas/feature-plan.md` — the slice Cases
  description says what a case covers when the framed issue is a bug:
  the numbered repro steps and the expected behaviour stand in for
  criteria numbers. If the schema still lacks them, it gains
  `Depends-on:` beside Cases (slice numbers or `none`, dependencies
  binding where list order only reads) and `## Deferred decisions` (the
  questions a blocked run leaves, per ADR-018 and
  [ADR-020][sokf:adr-020-a-blocked-run-ends]); whichever plan-013
  landed is left exactly as landed.
- Done-check: the schema defines bug coverage, and carries `Depends-on`
  and `Deferred decisions` whichever plan landed them.
- Cases:
  - manual: the Cases description names the repro steps and the expected
    behaviour for a bug-framed plan — checks that coverage is defined
    for a bug.
  - manual: `grep -l 'Depends-on'` and `grep -l 'Deferred decisions'`
    both hit `feature-plan.md` — checks that the two sections stand,
    whichever plan landed them.

### Block 6: Mirror, evidence, verification

- [x] Done — ticked at merge.
- Depends-on: 1, 2, 3, 4, 5.
- Change: `knowledge/schemas/` is copied wholesale over
  `pack/knowledge/schemas/` (53 files); issue-022 gains a dated Comments
  entry recording that 26 examples broke their own frontmatter
  constraints (18 types, 8 missing blocks), found by hand review and
  fixed by this plan; `CHANGELOG.md` gains an Unreleased entry for the
  pack-visible schema reshape; `superdev validate --fix` places files
  and regenerates definition blocks.
- Done-check: `diff -rq knowledge/schemas pack/knowledge/schemas` prints
  nothing and `superdev validate` reports PASS with 0 errors.
- Cases:
  - manual: `diff -rq knowledge/schemas pack/knowledge/schemas` prints
    nothing — checks that the pack mirrors the live schemas byte for
    byte.
  - manual: `superdev validate` reports PASS with 0 errors on a clean
    checkout of the branch — checks that every schema edit conforms.
  - manual: issue-022's Comments carry the appended evidence,
    `CHANGELOG.md` names the reshape under Unreleased, and
    `knowledge/plans/index.md` lists this plan with `lifecycle: done` —
    checks that the records close.

<!-- sokf:links -->
[sokf:adr-020-a-blocked-run-ends]: /knowledge/adrs/active/adr-020-a-blocked-run-ends.md
[sokf:issue-022-a-schemas-worked-example-is-checked-by-nothing]: /knowledge/issues/done/issue-022-a-schemas-worked-example-is-checked-by-nothing.md
[sokf:plan-013-workflow-autonomy]: /knowledge/plans/done/plan-013-workflow-autonomy.md
