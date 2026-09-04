---
type: Plan
id: plan-013-workflow-autonomy
title: Workflow autonomy
description: Eight blocks delivering the unattended workflow — the run state and its verbs, the Stop hook, the managed hook entry, the plan format's dependencies, the branching and commit conventions, the driver skill, and the records.
lifecycle: done
links:
  - rel: implements
    to: issue-024-the-workflow-cannot-run-unattended
    note: Delivers the thirteen acceptance criteria.
  - rel: supersedes
    to: plan-004-workflow-autonomy
    note: The adhoc plan that designed this work against the seven-phase workflow.
---

# Plan: Workflow autonomy

Request: [issue-024-the-workflow-cannot-run-unattended][sokf:issue-024-the-workflow-cannot-run-unattended]

## Goal

The workflow delivers a feature plan on its own: a run holds the
working tree, a Stop hook keeps the session going while a block is
ready, each feature and ad-hoc job runs on a branch of its own, and
every record edit is committed where it is made. The user stays in
frame and contract-design, and the default branch stays untouched
until the user fast-forwards it.

The validator seam first: the run state and its verbs, then the hook
that reads them, then the managed entry that arms the hook — each block
after the one whose vocabulary it needs. The plan format, the branching
and the commit points are independent of the hook and of each other, so
they run in parallel with it; the driver skill depends on all of them
and comes last but one.

This plan supersedes [plan-004][sokf:plan-004-workflow-autonomy], which
designed the same work against the seven-phase workflow. The decisions
are ADR-018 through ADR-021 and the seam is
[contract-009][sokf:contract-009-interface-run-state].

## Contract changes

- contract-009-interface-run-state: new — the run-state file at
  `.superdev/cache/run.toml`, the `run begin`, `run advance` and `run
  end` verbs that write it, the Stop hook's decision table with its
  hook-owned counter and cap, and the managed `hooks.Stop` entry that
  arms it.
- contract-002-cli-superdev: gains the `run begin`, `run advance` and
  `run end` verbs and the `hook run` subcommand beside `hook validate`.

## Work blocks

### Block 1: The run state and its verbs

- [x] Done — ticked at merge.
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
    checks that a run begun while another owns the working tree is
    refused, naming the owning session and how to clear it.
  - unit: `advance --next` rewrites next, zeroes the counter, refreshes
    the owner from `--session`/`CLAUDE_SESSION_ID` — checks the
    ownership the refusal reads.
  - unit: `end` removes the file; `end` with no file exits 0 and says so
    — checks that clearing the state is harmless without one.

### Block 2: The Stop hook

- [x] Done — ticked at merge.
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
    cap — each exits 0 — checks that a run without a step forward ends
    at a fixed cap, and that no active run leaves a session's turn
    boundaries untouched.
  - unit: an armed state exits 2 naming next; the counter is one higher
    afterwards; an empty owner is adopted from the payload — checks that
    a ready block crosses build and integrate with no turn boundary
    stopping to ask.
  - unit: a malformed payload exits 2 loudly; a malformed `run.toml` is
    reported and exits 0 — checks that an unreadable state leaves the
    session's turn boundaries untouched.

### Block 3: The managed Stop entry

- [x] Done — ticked at merge.
- Depends-on: 2.
- Change: `components/sokf.rs` declares the `hooks.Stop` JsonEntry with
  marker `superdev hook run`, claimed in the lock beside the PostToolUse
  entry.
- Done-check: `sync` in this repository writes the Stop entry and
  `status` exits 0 afterwards; a stale entry with the same marker is
  replanned.
- Cases:
  - unit: a fresh repo plans the Stop entry; a satisfied one plans
    nothing; a stale same-marker entry is replanned — checks that the
    hook is armed by the managed entry alone.
  - e2e: `sync` here writes `.claude/settings.json` and the lock claim;
    a repo with no run state sees every session end normally — checks
    that a repo with no run active keeps its turn boundaries untouched.

### Block 4: Dependencies in the plan format

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: `Depends-on:` per block and a `## Deferred decisions` section
  in the feature-plan template and `schema-feature-plan`, in `pack/` and
  the live copies; the feature-plan skill states dependencies, orders
  topologically, and GATEs on a cycle.
- Done-check: `superdev validate` passes a plan carrying `Depends-on`
  and deferred decisions; the skill's text carries the ordering rule and
  the cycle gate.
- Cases:
  - unit: this plan and the schema's example validate with `Depends-on`
    lines — checks that a plan records, for every block, the blocks it
    depends on.
  - manual: a plan with a dependency cycle is refused by the
    feature-plan skill's gate — checks that a cyclic plan is refused.

### Block 5: Branching conventions

- [x] Done — ticked at merge.
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
    `feature/<slug>` with the issue committed — checks that framing a
    feature cuts its branch off the default branch and commits the issue
    on it.
  - manual: `/adhoc-plan` planning code work leaves the tree on
    `adhoc/<slug>`; a documentation-only plan branches nothing — checks
    that ad-hoc work touching code cuts `adhoc/<slug>`.

### Block 6: Commit points

- [x] Done — ticked at merge.
- Depends-on: 5.
- Change: `/contract-design` ends with the go-ahead gate and commits the
  contract and ADR edits; `/integrate` commits the changelog, knowledge
  and plan edits after a successful merge — in `pack/` and the live
  copies.
- Done-check: both skills carry the commit step, and integrate's sits
  after the merge so a failed check commits nothing.
- Cases:
  - manual: `/contract-design` in a scratch repo ends by committing the
    records on the feature branch — checks that the contract and
    decision-record edits are committed before the unattended loop
    starts.
  - manual: `/integrate` leaves no uncommitted changelog, knowledge or
    plan edit — checks that merging a block commits the record edits it
    made.

### Block 7: The driver skill

- [x] Done — ticked at merge.
- Depends-on: 2, 3, 4, 5, 6.
- Change: a new `execute-feature-plan` skill in
  `pack/knowledge/skills/` carrying the loop — cut the plan when none
  exists, pick a ready block, build and integrate it in a subagent,
  drive `run begin`/`advance`/`end`, retry a failing block at most
  twice then defer, write user-gates into deferred decisions, end when
  no block is ready and put the queue to the user; the how-do-i map and
  `pack/agents/process.md` name it.
- Done-check: the skill's loop covers every edge the process diagram
  carries, the autonomy rule names its gates, and the process documents
  name the skill.
- Cases:
  - manual: a multi-block plan in a scratch repo runs feature-plan,
    build and integrate to completion with no turn boundary stopping to
    ask — checks the unattended loop over ready blocks.
  - manual: a block failing its checks twice is deferred and the loop
    continues; the run ends putting the deferred decisions in sequence
    — checks the deferral after two returns to build, the gate written
    into the plan's deferred decisions, and the run's end when no block
    is ready.
  - manual: `git log` on the scratch repo's default branch shows
    nothing from the run — checks that a run makes no commit and no
    merge to the default branch.

### Block 8: Records and rehearsal

- [x] Done — ticked at merge.
- Depends-on: 7.
- Change: the changelog entry for the verbs, the hook and the skill;
  the glossary's run term; the development-procedure commit points;
  plan-004 refiled done with its supersession note.
- Done-check: `superdev validate` passes, `npm run check:blueprint`
  is green, and a rehearsal of the full loop on a scratch feature
  confirms the acceptance criteria end to end.
- Cases:
  - e2e: the pre-PR check list passes on a clean checkout — checks the
    plan format's dependency lines, the refusal of a second run, the
    continue cap and the untouched turn boundaries.
  - manual: the rehearsal walks branching, the go-ahead commit, the
    unattended loop, the commit at merge, the run's end and the
    untouched default branch in one scratch-repo run.

<!-- sokf:links -->
[sokf:contract-009-interface-run-state]: /knowledge/contracts/internal/active/contract-009-interface-run-state.md
[sokf:issue-024-the-workflow-cannot-run-unattended]: /knowledge/issues/done/issue-024-the-workflow-cannot-run-unattended.md
[sokf:plan-004-workflow-autonomy]: /knowledge/plans/done/plan-004-workflow-autonomy.md
[sokf:research-001-claude-code-stop-hook-behaviour]: /knowledge/research/research-001-claude-code-stop-hook-behaviour.md
