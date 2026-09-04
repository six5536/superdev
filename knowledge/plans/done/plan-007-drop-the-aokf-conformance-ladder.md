---
type: Plan
id: plan-007-drop-the-aokf-conformance-ladder
title: Drop the AOKF conformance ladder
description: ADR-017 in code — the three-level ladder leaves the spec, the validator, the CLI and the parity goldens, knowledge passes or fails, and no file in the tree names a level but the ADR and this plan.
lifecycle: done
---

# Plan: Drop the AOKF conformance ladder

## Goal

The validator has one verdict, and nothing in the tree offers a level to
grade against.

ADR-017 decided that AOKF conformance is pass or fail. The ladder exists so
a repository adopting AOKF over documentation it already has can be legal
before it is complete, and no such repository exists — every caller in this
tree grades at level 2. Meanwhile `--level 0` is a live flag that passes
knowledge with broken links and no manifest through both the edit-time hook
and the pre-PR check, neither of which anticipates anything but the default.
This plan removes the model from the spec, the validator and the CLI without
changing a single verdict.

What the work delivers:

- O1 — knowledge passes or fails, and no report, finding or message names a
  conformance level.
- O2 — nothing in the tree names a conformance level: not the CLI, the spec,
  an instruction file, a skill, a test fixture, or a record of past work. The
  only mentions left are ADR-017 and this plan, which exist to record the
  removal.
- O3 — the reference validator's behaviour is still pinned, by goldens that
  carry only what survives.

The facts the design rests on:

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
- The ladder is populated rather than vestigial: 7 checks are fatal at level
  0, 5 at level 1, 14 at level 2, and 9 findings always warn.
- There are 11 goldens under `tests/fixtures/aokf/`. Their report keys are
  `achieved_level`, `checked_level`, `concepts`, `findings`, `passed`; their
  finding keys are `error_at_level`, `file`, `message`, `severity`.
- The goldens were captured at level 2 — `parity()` calls
  `validate(&bundle, &dir, 2)`. At level 2 a finding's `severity` is `error`
  exactly when `error_at` is set at all, so a pass/fail model reproduces
  every golden's `severity` and `message` unchanged. Only the three numeric
  fields differ.
- `validator_parity.rs` already normalises two things — the reference's
  `bundle` key, and the parse-error messages listed in `PARSE_ERRORS` — and
  its header records that the reference validator "is gone… there is nothing
  left to regenerate them from".
- Two finding messages name a level: `"no manifest (required at Level 1)"`
  at `validate.rs:254` and `"no `id` (required at Level 1)"` at line 293. One
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

Two constraints bound the work. No verdict changes: every fixture's `passed`
and every finding's `severity` stays identical to its golden. The change
clears the repo's coverage gate of 90% lines in each of `crates/lib` and
`crates/app`.

Four decisions shape it. The two level-naming messages are reworded and the
one golden carrying the first is rewritten with it, rather than kept
verbatim: a message reading "required at Level 1" in a tool with no levels
is wrong where a reader can see it, and the goldens are being edited anyway
(D-1). The goldens are edited by a transform recorded in the test header —
drop three keys, rewrite one message — rather than left alone behind a
level-aware normalisation, because leaving them keeps 51 references to a
removed concept alive in the fixtures forever, and deleting keys is a
mechanical projection that invents nothing (D-2). The spec change lands
before the code, because the spec is the statement the validator implements
and "the code is canonical" settles disagreements about behaviour, not about
which document defines the format (D-3). `passed()` becomes the whole
verdict, replacing `achieved_level`, because a field that can hold only pass
or fail is the boolean it already computes, and leaving it invites the
ladder back (D-4).

Out of scope:

- Changing which findings are fatal. Everything fatal at any level stays
  fatal, and the nine always-warn findings keep warning.
- Adding a way to opt out of a check. `--level` was not that, and nothing
  replaces it.
- Merging the format checks in. That is the format-validator plan's
  business, and this lands first so the merged report has no level to
  reconcile.

The risks and what answers them. Reaching further into `validate` than the
types while changing them, and altering a finding the golden edit then
hides, because both land in block 2: the golden projection is its own
commit, made before the types change and reviewed as a diff of deletions, so
the types then have to satisfy goldens nobody touched again. The reworded
messages being pinned in more places than the two found: `cargo test
--workspace` runs in block 2 rather than later, since an inline assertion
fails in the same crate. A repository somewhere passing `--level` and
breaking on its next `sync`: nothing in this tree does, and the changelog
entry announces the removal of a documented surface. `.agents/core.md` is
user-owned and says `until="PASS at level 2"`, which the tool stops
emitting; the deferred decision below settles it before block 4.

Two notes recorded out of band. The nine always-warn findings are
unaffected: `error_at: None` becomes a warning severity and nothing about
them changes. They are worth watching all the same, because the canonical
knowledge sits at zero warnings and a model with one fatal tier makes the
warning tier the only place a soft finding can live. ADR-017 records the
option rejected rather than deferred — keeping the ladder but removing
`--level`. Three levels with one reachable value leaves the machinery in
place and the reason for it gone, and that is the shape this plan drifts
into if block 3 is dropped from it.

## Contract changes

- none.

## Work blocks

### Block 1: Amend the spec

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: replace `.agents/aokf/SPEC.md` §11 — the ladder table goes, and
  conformance becomes the document check passing, with the diff check still
  independent of it; the sentence "a knowledge's level is the highest it
  fully satisfies" goes with it. Reword what leans on it: §10's items that
  exist to place a check on the ladder, and the two "required at Level 1"
  phrasings the validator echoes. Bump the version at both declaration sites
  and in `knowledge/manifest.sokf.yaml`, which names the version the
  canonical knowledge targets.
- Done-check: SPEC §11 names no level, and the version differs from `0.2` at
  both declaration sites and in the manifest.
- Cases:
  - observation: SPEC §11 states pass or fail and names no level, and the
    declared version is no longer `0.2` — checks that the format's statement
    of conformance drops the ladder.

### Block 2: Collapse the model and hold parity

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: `Finding` carries a severity, `Report` drops both level fields,
  and `achieved_level` is deleted along with the function that derives it;
  `passed()` stays and becomes the verdict (D-4). Drop "at Level 1" from the
  manifest and `id` findings in
  `crates/lib/superdev-core/src/aokf/validate.rs`, and fix the inline
  assertion that pins the second. Project the goldens under
  `crates/lib/superdev-core/tests/fixtures/aokf/` in one recorded pass,
  deleting `checked_level` and `achieved_level` from each report and
  `error_at_level` from each finding, and rewriting the one message that
  names a level. The transform goes in `validator_parity.rs`'s header beside
  the two normalisations already there, so a reader can see what was done
  and that `severity`, `file` and every other `message` are untouched. Hard
  to reverse: the reference that produced these is gone, so the diff is the
  only evidence the edit was a projection and not an invention — it lands on
  its own, reviewed on its own.
- Done-check: `cargo test -p superdev-core --test validator_parity` passes
  with no level-aware step in its comparison, and every fixture's verdict is
  unchanged.
- Cases:
  - integration: `cargo test -p superdev-core --test validator_parity`
    passes, and its comparison carries no level-aware step — checks that the
    goldens still pin the reference's behaviour.
  - observation: the goldens diff shows only deleted `checked_level`,
    `achieved_level` and `error_at_level` keys plus one reworded message,
    with every `severity`, `file` and other `message` unchanged — checks
    that the edit is a projection.
  - integration: each fixture's `passed` and each finding's `severity`
    equals its golden's, compared field by field — checks that no verdict
    changes.
  - e2e: `superdev aokf validate knowledge` on this knowledge reports no
    findings and exits 0 — checks that the same 26 fatal and 9 warning
    classifications survive.
  - observation: `rg -n 'Level [0-9]' crates/lib/superdev-core/src/` returns
    nothing — checks that no finding message names a level.

### Block 3: Remove the flag and its callers

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: `--level` and `DEFAULT_LEVEL` leave
  `crates/app/superdev/src/aokf_cli.rs`, from the subcommand definition and
  from both call sites; `CHECKED_LEVEL` leaves
  `crates/lib/superdev-core/src/aokf/mcp.rs` and its two uses; the
  `achieved_level` assertion leaves
  `crates/lib/superdev-core/src/components/aokf.rs`.
- Done-check: `superdev aokf validate --level 2` fails as an unknown
  argument, and no constant in `crates/` names a level.
- Cases:
  - e2e: `superdev aokf validate --level 2` fails as an unknown argument —
    checks that the flag is gone from the CLI.
  - observation: `rg -n -e achieved_level -e checked_level -e error_at -e
    DEFAULT_LEVEL -e CHECKED_LEVEL crates/` returns nothing outside the
    parity header's provenance note — checks that no constant or field names
    a level.
  - e2e: deleting `knowledge/manifest.sokf.yaml` makes the run fail, where
    `--level 0` would once have passed it — checks that the escape hatch is
    gone.

### Block 4: Correct the prose

- [x] Done — ticked at merge.
- Depends-on: 3.
- Change: `.agents/aokf.md` no longer says the canonical knowledge "must
  PASS at level 2"; the loop in `.claude/skills/maintain/SKILL.md` ends on
  the validator passing rather than on a level; `knowledge/api-contracts.md`
  drops `--level` from the knowledge verbs, and
  `knowledge/plans/plan-006-rust-format-validator.md` records D-19 as moot.
  Correct the records that describe the old behaviour — the two open issues
  `knowledge/issues/issue-010-index-entries-are-never-checked-against-their-concept.md`
  and `knowledge/issues/issue-011-index-shape-is-described-but-not-enforced.md`
  assert what validate does today, and
  `knowledge/specs/spec-008-knowledge-owned-skills.md` and
  `knowledge/plans/plan-002-agent-instructions-layer.md` each name a level in
  a verification step. Dropping the phrase leaves what happened intact;
  keeping it leaves an instruction nobody can follow. Add the changelog
  entry for the removed flag, since it is a breaking change to a documented
  surface, and leave ADR-017's follow-up list empty or each remaining item
  filed as an issue.
- Done-check: no file in the tree names a conformance level except ADR-017
  and this plan, `knowledge/plans/index.md` lists this plan with status
  done, and `npm run coverage:check` passes.
- Cases:
  - observation: `rg -n -e achieved_level -e checked_level -e error_at -e
    'at level' -e 'Level [0-9]' -e conformance.level .agents .claude
    knowledge crates` returns hits only where the removal is recorded —
    ADR-017, this plan, P006's reversed D-19, and the parity header's
    provenance note.
  - observation: `rg -n 'level 2' .agents .claude knowledge/api-contracts.md`
    returns nothing — checks that no instruction file or skill ends a loop on
    "PASS at level 2".
  - e2e: `npm run coverage:check` passes — checks the 90% per-crate line
    gate.

## Deferred decisions

- Block 1: what version does the spec become? Recommended default: 0.3, with
  §12's rule reworded to ordinary pre-1.0 semantics, because §12 says minor
  bumps are backward-compatible additions and this removes a conformance
  model; declaring 1.0 would assert a stability the format has not earned.
  Answer: 0.3.
- Block 4: `.agents/core.md:31` reads `until="PASS at level 2"` and is
  yours; I have not touched it. Recommended default: `until="the validator
  passes"`, which is what the loop waits for and survives this change.
  Answer: this is yours too.
