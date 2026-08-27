---
type: Decision
id: adr-017-aokf-conformance-is-pass-or-fail
title: AOKF Conformance Is Pass or Fail
description: The conformance ladder goes; knowledge passes or fails, because superdev grades at level 2 everywhere and the flag that reaches the other levels can only weaken a gate.
status: stable
links:
  - rel: relates-to
    to: adhoc-plan-006-rust-format-validator
---

# ADR-017: AOKF conformance is pass or fail

- Status: accepted
- Date: 2026-08-27
- Deciders: project owner

## Context

SPEC §11 makes knowledge conformance a ladder of three levels, and the validator
implements it: `Finding` carries `error_at`, the lowest level at which a
finding is fatal; `validate` takes a `checked_level` that "decides which
findings count as errors, and nothing else"; `Report` carries both
`checked_level` and the derived `achieved_level`; and the CLI exposes
`--level 0..2`.

The ladder is populated — 7 checks are fatal at level 0, 5 at level 1, 14 at
level 2, and 9 findings are always warnings. It is also never used. Every
caller grades at 2: `DEFAULT_LEVEL` is 2, the PostToolUse hook passes it,
`components/aokf.rs` asserts `achieved_level == 2`, and neither `package.json`
nor CI ever passes `--level`.

It exists so a repository adopting AOKF over documentation it already has can
be a legal knowledge before it is a complete one — `type` on everything is level
0, ids and a manifest reach level 1, typed links with body mirroring reach 2.
That path costs a field in every finding, two in every report, and a flag that
can quietly weaken the gates: `--level 0` passes knowledge with broken links
and no manifest through a hook and a CI check that both assume the default.

## Decision

We will drop the ladder. Knowledge passes or it fails. `Finding` carries a
severity rather than a level, `Report` drops `checked_level` and
`achieved_level`, `achieved_level` goes entirely, and `--level` is removed. A
repository adopting AOKF catches up rather than declaring a level it has
reached.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Drop the ladder; pass or fail | One model rather than four; a field leaves every finding and two leave every report; the gates can no longer be weakened by a flag | A repository mid-adoption has no legal intermediate state, so `validate` is red until it has caught up |
| Keep the ladder as it is | Graduated adoption for repositories with existing documentation, and it is what the spec as published says | A dimension nothing exercises, and `--level 0` silently passes knowledge with broken links through gates written for the default |
| Keep the ladder, remove `--level` | Keeps the model, closes the foot-gun | Three levels with one reachable value is worse than either — the machinery stays and the reason for it does not |

## Consequences

- Positive: the model collapses to error and warning. Warnings are unchanged;
  the nine always-warn findings keep warning, and everything that was fatal at
  any level is now simply fatal.
- Positive: the hook and the pre-PR check can no longer be weakened by a flag,
  because there is no flag.
- Negative: SPEC §11 changes and the spec version bumps, so knowledge written
  against the 0.2 ladder no longer has a level to claim.
- Negative: `validator_parity` needs a third documented normalisation. Its
  goldens carry `checked_level`, `achieved_level` and a per-finding
  `error_at_level`, and they cannot be regenerated — the reference validator is
  gone. They were captured at level 2, where `severity` is `error` exactly
  when `error_at` is set at all, so every golden's `severity` and `message`
  still compares verbatim and only the three numeric fields are dropped. No
  verdict changes.
- Follow-ups: SPEC §11 and the §10 wording that leans on levels;
  `.agents/aokf.md` and `.agents/core.md`, which both say "PASS at level 2";
  `maintain`, whose loop ends on the same phrase; `api-contracts.md`, which
  documents `--level 0..2`; and D-19 in the
  [format validator plan](../adhoc-plans/P006-rust-format-validator.md), which
  keeps format findings out of `achieved_level` and is moot once there is no
  `achieved_level` to keep them out of.
