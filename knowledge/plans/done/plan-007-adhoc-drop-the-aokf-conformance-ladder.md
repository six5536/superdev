---
type: AdhocPlan
id: plan-007-adhoc-drop-the-aokf-conformance-ladder
title: Drop the AOKF conformance ladder
description: ADR-017 in code — the three-level ladder leaves the spec, the validator, the CLI and the parity goldens, knowledge passes or fails, and no file in the tree names a level but the ADR and this plan.
lifecycle: done
---

# Plan: Drop the AOKF conformance ladder

## Context

ADR-017 decided that AOKF conformance is pass or fail. The ladder exists so a
repository adopting AOKF over documentation it already has can be legal before
it is complete, and no such repository exists — every caller in this tree
grades at level 2. Meanwhile `--level 0` is a live flag that passes knowledge
with broken links and no manifest through both the edit-time hook and the
pre-PR check, neither of which anticipates anything but the default. This plan
removes the model from the spec, the validator and the CLI without changing a
single verdict.

## Facts

- The model lives in four files and 60 lines mention it:
  `crates/lib/superdev-core/src/aokf/validate.rs`, `aokf/mcp.rs`,
  `components/aokf.rs` and `crates/app/superdev/src/aokf_cli.rs`.
- `Finding { path, message, error_at }` is at `aokf/validate.rs:38`, where
  `error_at` is "the lowest conformance level at which this is an error;
  `None` for the spec's always-warn items". `Report` at line 63 carries
  `achieved_level` and `checked_level`; `achieved_level` is derived at line
  547.
- Every caller grades at 2 and nothing offers otherwise:
  `DEFAULT_LEVEL: u8 = 2` at `aokf_cli.rs:24`, used at lines 120 and 199;
  `CHECKED_LEVEL: u8 = 2` at `mcp.rs:33`, used at 630 and 640;
  `components/aokf.rs:359` asserts `report.achieved_level == 2`. Neither
  `package.json` nor CI passes `--level`.
- The ladder is populated rather than vestigial: 7 checks are fatal at level 0,
  5 at level 1, 14 at level 2, and 9 findings always warn.
- There are 11 goldens under `tests/fixtures/aokf/`. Their report keys are
  `achieved_level`, `checked_level`, `concepts`, `findings`, `passed`; their
  finding keys are `error_at_level`, `file`, `message`, `severity`.
- The goldens were captured at level 2 — `parity()` calls
  `validate(&bundle, &dir, 2)`. At level 2 a finding's `severity` is `error`
  exactly when `error_at` is set at all, so a pass/fail model reproduces every
  golden's `severity` and `message` unchanged. Only the three numeric fields
  differ.
- `validator_parity.rs` already normalises two things — the reference's
  `bundle` key, and the parse-error messages listed in `PARSE_ERRORS` — and
  its header records that the reference validator "is gone… there is nothing
  left to regenerate them from".
- Two finding messages name a level: `"no manifest (required at Level 1)"` at
  `validate.rs:254` and `"no `id` (required at Level 1)"` at line 293. One
  golden carries the first, `no-manifest.golden.json`, and one inline test
  asserts the second at `validate.rs:952`.
- SPEC §11 is the ladder, at `.agents/aokf/SPEC.md:366-378`. The version is
  declared at line 3 and again at line 387, and `knowledge/manifest.sokf.yaml`
  declares `aokf: "0.2"`.
- Five live files assert the model in prose: `.agents/aokf.md:28` and
  `.agents/core.md:31` ("PASS at level 2"),
  `.claude/skills/maintain/SKILL.md:23` (a loop ending on the same phrase),
  and `knowledge/api-contracts.md:24` (documents `--level 0..2`).
- `npm run coverage:check` fails under 90% lines for `crates/lib` and
  `crates/app` separately.

## Goal

The validator has one verdict, and nothing in the tree offers a level to grade
against.

## Outcomes

- O1 — knowledge passes or fails, and no report, finding or message names a
  conformance level.
- O2 — nothing in the tree names a conformance level: not the CLI, the spec,
  an instruction file, a skill, a test fixture, or a record of past work. The
  only mentions left are ADR-017 and this plan, which exist to record the
  removal.
- O3 — the reference validator's behaviour is still pinned, by goldens that
  carry only what survives.

## Non-goals

- Changing which findings are fatal. Everything fatal at any level stays
  fatal, and the nine always-warn findings keep warning. The ladder's removal
  is a change to the model, not to the checks.
- Adding a way to opt out of a check. `--level` was not that, and nothing
  replaces it; a repository that cannot pass catches up.
- Merging the format checks in. That is the format-validator plan's business,
  and this lands first so the merged report has no level to reconcile.

## Requirements

### Functional

| ID | Requirement | Outcome |
|----|-------------|---------|
| FR-1 | `Report` carries no `achieved_level` or `checked_level`, and `Finding` carries a severity rather than a level | O1 |
| FR-2 | Every finding is an error or a warning, with the same 26 fatal and 9 warning classifications as at level 2 today | O1 |
| FR-3 | No finding message names a conformance level | O1 |
| FR-4 | `--level` no longer exists on the CLI, and no constant names a level | O2 |
| FR-5 | SPEC §11 states pass or fail, and the spec version is bumped | O2 |
| FR-6 | No instruction file or skill ends a loop on "PASS at level 2" | O2 |
| FR-7 | `validator_parity` passes with no level-aware normalisation, against goldens carrying no level fields | O3 |
| FR-8 | No file in the tree names a conformance level, except ADR-017 and this plan | O2 |

### Non-functional

| ID | Constraint | Budget |
|----|------------|--------|
| NFR-1 | No verdict changes | every fixture's `passed` and every finding's `severity` identical to its golden |
| NFR-2 | The change clears the repo's coverage gate | 90% lines in each of `crates/lib` and `crates/app` |

## Decisions

| ID | Decision | Alternative | Why |
|----|----------|-------------|-----|
| D-1 | The two level-naming messages are reworded, and the one golden carrying the first is rewritten with it | keep the text verbatim so no golden needs touching | a message reading "required at Level 1" in a tool with no levels is wrong where a reader can see it, and under D-2 the goldens are being edited anyway, so this costs one more line in the same reviewed diff |
| D-2 | The goldens are edited, by a transform recorded in the test header: drop three keys, rewrite one message | keep them untouched and add a level-aware normalisation to the comparison | leaving them alone keeps 51 references to a removed concept alive in the fixtures forever, plus the code that skips those fields; deleting keys is a mechanical projection that invents nothing, and every `severity`, `file` and `message` staying identical is provable from the diff |
| D-3 | The spec change lands before the code | code first, spec after | the spec is the statement the validator implements, and "the code is canonical" settles disagreements about behaviour, not about which document defines the format |
| D-4 | `passed()` becomes the whole verdict, replacing `achieved_level` | keep `achieved_level` as a two-valued field | a field that can hold only pass or fail is the boolean it already computes, and leaving it invites the ladder back |

## Workstreams

### W1: Amend the spec

Depends on: none.

1. Replace §11 — the ladder table goes; conformance becomes the document check
   passing, with the diff check still independent of it. The sentence "a
   knowledge's level is the highest it fully satisfies" goes with it.
2. Reword what leans on it — §10's items that exist to place a check on the
   ladder, and the two "required at Level 1" phrasings the validator echoes.
3. Bump the version — `.agents/aokf/SPEC.md` at both declaration sites, and
   `knowledge/manifest.sokf.yaml`, which names the version the canonical knowledge targets.

### W2: Collapse the model and hold parity

Depends on: W1.

1. Change the types — `Finding` carries a severity, `Report` drops both level
   fields, and `achieved_level` is deleted along with the function that
   derives it. `passed()` stays and becomes the verdict, per D-4.
2. Reword the two messages — drop "at Level 1" from the manifest and `id`
   findings, and fix the inline assertion that pins the second.
3. Project the goldens — one recorded pass over
   `crates/lib/superdev-core/tests/fixtures/aokf/` deleting `checked_level`
   and `achieved_level` from each report and `error_at_level` from each
   finding, and rewriting the one message that names a level. The transform
   goes in the test header beside the two normalisations already there, so a
   reader can see exactly what was done and that `severity`, `file` and every
   other `message` are untouched. Hard to reverse: the reference that produced
   these is gone, so the diff is the only evidence the edit was a projection
   and not an invention — it lands on its own, reviewed on its own.

### W3: Remove the flag and its callers

Depends on: W2.

1. Drop the flag — `--level` and `DEFAULT_LEVEL` leave
   `crates/app/superdev/src/aokf_cli.rs`, from the subcommand definition and
   from both call sites.
2. Drop the other constants — `CHECKED_LEVEL` leaves
   `crates/lib/superdev-core/src/aokf/mcp.rs` and its two uses, and the
   `achieved_level` assertion leaves
   `crates/lib/superdev-core/src/components/aokf.rs`.

### W4: Correct the prose

Depends on: W3.

1. Fix the instruction files — `.agents/aokf.md` no longer says the canonical knowledge
   "must PASS at level 2".
2. Fix the skill — the loop in `.claude/skills/maintain/SKILL.md` ends on
   the validator passing rather than on a level.
3. Fix the documented surface — `knowledge/api-contracts.md` drops `--level`
   from the knowledge verbs, and
   `knowledge/plans/plan-006-adhoc-rust-format-validator.md` records D-19 as moot.
4. Fix the records that describe the old behaviour — the two open issues
   `knowledge/issues/issue-010-index-entries-are-never-checked-against-their-concept.md`
   and `knowledge/issues/issue-011-index-shape-is-described-but-not-enforced.md`
   assert what validate does today, and
   `knowledge/specs/spec-008-knowledge-owned-skills.md` and
   `knowledge/plans/plan-002-feature-agent-instructions-layer.md` each name a
   level in a verification step. Dropping the phrase leaves what happened
   intact; keeping it leaves an instruction nobody can follow.

## Files affected

| File | Change | Workstream |
|------|--------|------------|
| `.agents/aokf/SPEC.md` | modified — §11 replaced, §10 reworded, version bumped at both sites | W1 |
| `knowledge/manifest.sokf.yaml` | modified — the version the canonical knowledge targets | W1 |
| `crates/lib/superdev-core/src/aokf/validate.rs` | modified — `Finding`, `Report`, `achieved_level` deleted, two messages reworded, one inline test | W2 |
| `crates/lib/superdev-core/tests/validator_parity.rs` | modified — a third normalisation and its header note | W2 |
| `crates/app/superdev/src/aokf_cli.rs` | modified — `--level` and `DEFAULT_LEVEL` removed | W3 |
| `crates/lib/superdev-core/src/aokf/mcp.rs` | modified — `CHECKED_LEVEL` removed | W3 |
| `crates/lib/superdev-core/src/components/aokf.rs` | modified — the `achieved_level` assertion | W3 |
| `.agents/aokf.md` | modified — drops "must PASS at level 2" | W4 |
| `.claude/skills/maintain/SKILL.md` | modified — the loop's `until` | W4 |
| `knowledge/api-contracts.md` | modified — `--level` leaves the documented surface | W4 |
| `knowledge/plans/plan-006-adhoc-rust-format-validator.md` | modified — D-19 recorded moot | W4 |
| `crates/lib/superdev-core/tests/fixtures/aokf/` | modified — 11 goldens projected: three keys dropped, one message reworded | W2 |
| `knowledge/issues/issue-010-index-entries-are-never-checked-against-their-concept.md` | modified — drops the level from what it observes | W4 |
| `knowledge/issues/issue-011-index-shape-is-described-but-not-enforced.md` | modified — drops the level from what it observes | W4 |
| `knowledge/specs/spec-008-knowledge-owned-skills.md` | modified — a verification step names a level | W4 |
| `knowledge/plans/plan-002-feature-agent-instructions-layer.md` | modified — a verification step names a level | W4 |

## Acceptance

| Check | Verifies |
|-------|----------|
| `cargo test -p superdev-core --test validator_parity` passes, and its comparison carries no level-aware step | FR-7, O3 |
| The goldens diff shows only deleted `checked_level`, `achieved_level` and `error_at_level` keys plus one reworded message; every `severity`, `file` and other `message` is unchanged | D-2, NFR-1 |
| `rg -n -e achieved_level -e checked_level -e error_at -e 'at level' -e 'Level [0-9]' -e conformance.level .agents .claude knowledge crates` returns hits only where the removal is recorded: ADR-017, this plan, P006's reversed D-19, and the parity header's provenance note | FR-8, O2 |
| Each fixture's `passed` and each finding's `severity` equal its golden's, compared field by field | NFR-1 |
| `rg -n -e achieved_level -e checked_level -e error_at -e DEFAULT_LEVEL -e CHECKED_LEVEL crates/` returns nothing outside the parity header's provenance note | FR-1, FR-4 |
| `superdev aokf validate --level 2` fails as an unknown argument | FR-4 |
| `rg -n 'Level [0-9]' crates/lib/superdev-core/src/` returns nothing | FR-3 |
| `superdev aokf validate knowledge` on this knowledge still reports no findings and exits 0 | FR-2 |
| Deleting `knowledge/manifest.sokf.yaml` makes it fail, where `--level 0` would once have passed it | FR-2, FR-4 |
| `rg -n 'level 2' .agents .claude knowledge/api-contracts.md` returns nothing | FR-6 |
| SPEC §11 names no level, and the version differs from `0.2` at both declaration sites | FR-5 |
| `npm run coverage:check` passes | NFR-2 |

## Definition of done

- Every Acceptance row passes on a clean checkout of the branch.
- The goldens' diff is reviewable as a projection: only the three level keys
  and the one reworded message change, and `validator_parity.rs` records the
  transform beside the two normalisations already documented there.
- `knowledge/plans/index.md` lists this plan and its status reads done.
- The changelog has an Unreleased entry naming the removed flag, since it is a
  breaking change to a documented surface.
- ADR-017's follow-up list is empty or each remaining item has an issue.

## Risks

- Risk: reaching further into `validate` than the types while changing them,
  and altering a finding that the golden edit then hides, because both land in
  W2. Mitigation: the golden projection is its own commit, made before the
  types change and reviewed as a diff of deletions; the types then have to
  satisfy goldens nobody touched again.
- Risk: the reworded messages are pinned in more places than the two found —
  one golden and one inline test. Early signal: `cargo test --workspace` in W2
  rather than a later run, since an inline assertion fails in the same crate.
- Risk: a repository somewhere passes `--level` and its next `sync` breaks on
  an unknown argument. Mitigation: nothing in this tree does, and the changelog
  entry names it; the flag is a documented part of the CLI surface, so its
  removal is announced rather than silent.
- Risk: `.agents/core.md` is user-owned and says `until="PASS at level 2"`,
  which the tool will stop emitting. Early signal: the open question below is
  answered before W4 rather than after, or the core's validation loop ends on
  a phrase that never appears.

## Open questions

- What version does the spec become? Recommended default: 0.3, with §12's rule
  reworded to ordinary pre-1.0 semantics, because §12 currently says minor
  bumps are backward-compatible additions and this removes a conformance
  model. Declaring 1.0 instead would assert a stability the format has not
  earned. Blocks W1 step 3 only.
  - Answer - 0.3
- `.agents/core.md:31` reads `until="PASS at level 2"` and is yours; I have not
  touched it. Recommended default: `until="the validator passes"`, which is
  what the loop actually waits for and survives this change. Blocks W4.
  - Answer - this is yours too

## Out-of-band notes

The nine always-warn findings are unaffected: `error_at: None` becomes a
warning severity and nothing about them changes. They are worth watching all
the same, because the canonical knowledge currently sits at zero warnings, and a model with
one fatal tier makes the warning tier the only place a soft finding can live.

ADR-017 records the option that was rejected rather than deferred — keeping the
ladder but removing `--level`. Three levels with one reachable value leaves the
machinery in place and the reason for it gone, which is the worst of both, and
it is the shape this plan would drift into if W3 were dropped from it.
