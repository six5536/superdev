---
type: BugReport
id: issue-016-sync-would-revert-the-schema-migration
title: Sync would revert the schema migration and the conformance decision, and the pre-PR check says so 65 times
description: The live tree carries the schema migration and ADR-017; /pack/ still carries what they replaced, so status --drift reports 65 changes and sync would restore 41 deleted templates, put the level ladder back into the AOKF spec, and overwrite 21 rewritten skills.
status: draft
tags: [needs-triage]
links:
  - rel: relates-to
    to: adhoc-plan-006-rust-format-validator
  - rel: relates-to
    to: adr-017-aokf-conformance-is-pass-or-fail
---

# Bug: sync would revert the schema migration and the conformance decision

## Summary

The schema migration and ADR-017 were made in the live tree. `/pack/` — which
is the same directory the binary embeds as `assets/` — still holds what they
replaced, so the blueprint and the working tree now disagree about 65 files.
`superdev sync` closes that gap in the wrong direction: it restores the 41
templates the schemas replaced, rewrites `.agents/aokf/SPEC.md` and
`.agents/aokf.md` back to the conformance ladder
[ADR-017](../decisions/D017-aokf-conformance-is-pass-or-fail.md) removed, recreates
`knowledge/plans/index.md` after the plans were split, puts the old
`.agents/superdev.md` aggregator back with its `AGENTS.md` line, and overwrites
21 rewritten skills. `npm run check:blueprint` is a pre-PR check, so a
contributor meets all 65 with the reflex to tidy them.

## Environment

- Version/commit: superdev 0.2.0, branch `feature/content-packs`
- Platform: any; the drift is content, not machine state

## Steps to reproduce

1. `cargo run --quiet -- status --drift`
2. Count the planned changes: `… | grep -c '^  - '` reports 65.
3. `npm run check:blueprint` exits 1.
4. `grep -c level pack/aokf/agents/aokf/SPEC.md .agents/aokf/SPEC.md` reports
   2 in the pack copy and 0 in the live one.

## Expected behaviour

A pre-PR check that exits 1 names work to do. Either the pack carries what the
live tree carries, so the check is green and `sync` is safe to run; or the
tree says plainly that these 65 are a known, deliberate divergence with an
owner and a date, so nobody spends a morning discovering it again.

## Actual behaviour

`status --drift` lists 65 changes and exits 1, and every one of them is a
revert:

```
- ensure AGENTS.md contains `@.agents/superdev.md`
- write .agents/superdev.md (superdev's agent instructions)
- write .agents/aokf/SPEC.md (AOKF specification)
- write .agents/aokf.md (knowledge instructions)
- write knowledge/plans/index.md (plans index)
- write knowledge/templates/*.md (41 of them)
- write .claude/skills/*/SKILL.md (21 of them)
```

Nothing warns that running the suggested remedy destroys the work. The two
files with the sharpest consequence are `.agents/aokf/SPEC.md` and
`.agents/aokf.md`: they are binary-owned rather than pack-owned, so `sync`
rewrites them from the copy compiled into the binary, and the AOKF spec goes
back to 0.2 with the level ladder in it.

## Root cause (if known)

The rule is `pack-backport`'s and it is the right one: an edit to a live copy
ships only when it is backported, and until then `sync` treats it as drift.
Both the [format validator plan](../adhoc-plans/P006-rust-format-validator.md)
and the schema migration list the backport as a non-goal and a follow-on, so the divergence is expected. What is missing is anything in the
tree that says so — the check reports 65 anonymous "write" lines, and the two
plans that own the debt say it in a Definition-of-done bullet nobody reads
while looking at a failing check.

## Proposed fix / workaround

- Backport the migration into `/pack/`: the 39 schemas replacing the 41
  templates, the split plan indexes, the rewritten skills, and the aggregator's
  removal. This is the plan-sized piece of work, and it is what closes the 65.
- Backport ADR-017's edits to `.agents/aokf/SPEC.md` and `.agents/aokf.md`
  separately and first. They are binary-owned, two files, and the consequence
  of leaving them is a spec version going backwards.
- Meanwhile, do not run `superdev sync` in this repository, and read
  `npm run check:blueprint`'s 65 as expected.
- `.agents/process.md` still tells the agent that document skeletons live in
  `knowledge/templates/`. It is pack-owned, so the live copy cannot be fixed
  without adding a sixty-sixth line of drift; it wants fixing in the pack, with
  the rest.

## Regression risk

The backport is the fix and it is also the risk: `sync` writes over live files,
so a backport that lands the pack content without first reconciling the lock
hashes reports every touched file as user-edited and backs it up — the failure
[I005](I005-a-backport-leaves-the-lock-stale.md) already recorded. The
`pack-backport` skill exists for this, and `superdev status --drift` returning
0 is what proves the job finished.
