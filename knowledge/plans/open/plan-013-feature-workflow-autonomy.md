---
type: FeaturePlan
id: plan-013-feature-workflow-autonomy
title: Workflow autonomy — feature plan
description: Eight slices delivering the unattended workflow — the run state and its verbs, the Stop hook, the managed hook entry, the plan format's dependencies, the branching and commit conventions, the driver skill, and the records.
lifecycle: open
links:
  - rel: implements
    to: issue-024-feature-request-the-workflow-cannot-run-unattended
    note: Delivers the thirteen acceptance criteria.
  - rel: supersedes
    to: plan-004-adhoc-workflow-autonomy
    note: The adhoc plan that designed this work against the seven-phase workflow.
---

# Feature plan: The workflow cannot deliver a feature unattended

Request:
[issue-024-feature-request-the-workflow-cannot-run-unattended][sokf:issue-024-feature-request-the-workflow-cannot-run-unattended].
Supersedes [plan-004][sokf:plan-004-adhoc-workflow-autonomy]; the
decisions are ADR-018 through ADR-021 and the seam is
[contract-009][sokf:contract-009-interface-run-state].

## Slices

### Slice 1: The run state and its verbs

- [x] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: a `run` module in `crates/app/superdev` owning
  `.superdev/cache/run.toml` per contract-009, and the `run begin`,
  `run advance` and `run end` verbs in `main.rs`.
- Done-check: `begin` creates the state exclusively and a second `begin`
  is refused naming the owner and `run end`; `advance` resets the counter
  and refreshes the owner; `end` removes the state and is harmless
  without one.
- Cases:
  - unit: `begin` writes session, next, a zero counter, started and pid;
    a second `begin` fails naming the owner and `superdev run end` —
    covers 11.
  - unit: `advance --next` rewrites next, zeroes the counter, refreshes
    the owner from `--session`/`CLAUDE_SESSION_ID` — covers 11.
  - unit: `end` removes the file; `end` with no file exits 0 and says so
    — covers 11.

### Slice 2: The Stop hook

- [x] Done — ticked by integrate at merge.
- Depends-on: 1.
- Change: `hook run` beside `hook validate` in the hook namespace: the
  decision table of contract-009, the hook-owned counter, the cap of
  ten, no gating on `stop_hook_active`
  ([research-001][sokf:research-001-claude-code-stop-hook-behaviour]),
  fail-open on an unreadable state, loud exit 2 on an unreadable
  payload, `CLAUDE_PROJECT_DIR` preference.
- Done-check: the hook exits 0 with no state, a foreign session, an
  empty next, or a spent counter, and otherwise exits 2 naming next with
  the counter one higher.
- Cases:
  - unit: absent state, foreign `session_id`, empty next, counter at
    cap — each exits 0 — covers 12, 13.
  - unit: an armed state exits 2 naming next; the counter is one higher
    afterwards; an empty owner is adopted from the payload — covers 5,
    12.
  - unit: a malformed payload exits 2 loudly; a malformed `run.toml` is
    reported and exits 0 — covers 13.

### Slice 3: The managed Stop entry

- [x] Done — ticked by integrate at merge.
- Depends-on: 2.
- Change: `components/sokf.rs` declares the `hooks.Stop` JsonEntry with
  marker `superdev hook run`, claimed in the lock beside the PostToolUse
  entry.
- Done-check: `sync` in this repository writes the Stop entry and
  `status` exits 0 afterwards; a stale entry with the same marker is
  replanned.
- Cases:
  - unit: a fresh repo plans the Stop entry; a satisfied one plans
    nothing; a stale same-marker entry is replanned — covers 13.
  - e2e: `sync` here writes `.claude/settings.json` and the lock claim;
    a repo with no run state sees every session end normally — covers
    13.

### Slice 4: Dependencies in the plan format

- [ ] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `Depends-on:` per slice and a `## Deferred decisions` section
  in the feature-plan template and `schema-feature-plan`, in `pack/` and
  the live copies; the feature-plan skill states dependencies, orders
  topologically, and GATEs on a cycle.
- Done-check: `superdev validate` passes a plan carrying `Depends-on`
  and deferred decisions; the skill's text carries the ordering rule and
  the cycle gate.
- Cases:
  - unit: this plan and the schema's example validate with `Depends-on`
    lines — covers 4.
  - manual: a plan with a dependency cycle is refused by the
    feature-plan skill's gate — covers 4.

### Slice 5: Branching conventions

- [ ] Done — ticked by integrate at merge.
- Depends-on: none.
- Change: `/frame` creates `feature/<slug>` off the default branch and
  commits the framed issue; `/adhoc-plan` creates `adhoc/<slug>` when
  its work touches code; the development-procedure template gains the
  branching line and this repo's concept records its convention — in
  `pack/` and the live copies.
- Done-check: both skills carry the branch step and the repo-convention
  precedence; the development-procedure documents name the convention.
- Cases:
  - manual: `/frame` in a scratch repo leaves the tree on
    `feature/<slug>` with the issue committed — covers 1.
  - manual: `/adhoc-plan` planning code work leaves the tree on
    `adhoc/<slug>`; a documentation-only plan branches nothing — covers
    2.

### Slice 6: Commit points

- [ ] Done — ticked by integrate at merge.
- Depends-on: 5.
- Change: `/contract-design` ends with the go-ahead gate and commits the
  contract and ADR edits; `/integrate` commits the changelog, knowledge
  and plan edits after a successful merge — in `pack/` and the live
  copies.
- Done-check: both skills carry the commit step, and integrate's sits
  after the merge so a failed check commits nothing.
- Cases:
  - manual: `/contract-design` in a scratch repo ends by committing the
    records on the feature branch — covers 3.
  - manual: `/integrate` leaves no uncommitted changelog, knowledge or
    plan edit — covers 6.

### Slice 7: The driver skill

- [ ] Done — ticked by integrate at merge.
- Depends-on: 2, 3, 4, 5, 6.
- Change: a new `execute-feature-plan` skill in
  `pack/knowledge/skills/` carrying the loop — cut the plan when none
  exists, pick a ready slice, build and integrate it in a subagent,
  drive `run begin`/`advance`/`end`, retry a failing slice at most
  twice then defer, write user-gates into deferred decisions, end when
  no slice is ready and put the queue to the user; the how-do-i map and
  `pack/agents/process.md` name it.
- Done-check: the skill's loop covers every edge the process diagram
  carries, the autonomy rule names its gates, and the process documents
  name the skill.
- Cases:
  - manual: a multi-slice plan in a scratch repo runs feature-plan,
    build and integrate to completion with no turn boundary stopping to
    ask — covers 5.
  - manual: a slice failing its checks twice is deferred and the loop
    continues; the run ends putting the deferred decisions in sequence
    — covers 7, 8, 9.
  - manual: `git log` on the scratch repo's default branch shows
    nothing from the run — covers 10.

### Slice 8: Records and rehearsal

- [ ] Done — ticked by integrate at merge.
- Depends-on: 7.
- Change: the changelog entry for the verbs, the hook and the skill;
  the glossary's run term; the development-procedure commit points;
  plan-004 refiled done with its supersession note.
- Done-check: `superdev validate` passes, `npm run check:blueprint`
  is green, and a rehearsal of the full loop on a scratch feature
  confirms the acceptance criteria end to end.
- Cases:
  - e2e: the pre-PR check list passes on a clean checkout — covers 4,
    11, 12, 13.
  - manual: the rehearsal walks criteria 1, 3, 5, 6, 9 and 10 in one
    scratch-repo run — covers 1, 3, 5, 6, 9, 10.

<!-- sokf:links -->
[sokf:contract-009-interface-run-state]: /knowledge/contracts/internal/active/contract-009-interface-run-state.md
[sokf:issue-024-feature-request-the-workflow-cannot-run-unattended]: /knowledge/issues/open/issue-024-feature-request-the-workflow-cannot-run-unattended.md
[sokf:plan-004-adhoc-workflow-autonomy]: /knowledge/plans/open/plan-004-adhoc-workflow-autonomy.md
[sokf:research-001-claude-code-stop-hook-behaviour]: /knowledge/research/research-001-claude-code-stop-hook-behaviour.md
