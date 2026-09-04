---
type: Plan
id: plan-004-workflow-autonomy
title: Workflow autonomy — branch, slice dependencies, unattended delivery
description: Give the workflow a branch at frame, model slice dependencies in the plan, run stages 4-7 unattended on a general superdev run facility with a Stop hook and a new execute-feature-plan skill, and commit at every successful integrate.
lifecycle: done
links:
  - rel: relates-to
    to: development-procedure
    note: Adds the branching convention and the commit points this plan introduces.
---

# Plan: workflow autonomy

## Goal

`/interface-design` ends by committing its documents on the user's go-ahead
and handing to `/execute-feature-plan`, which cuts the spec into slices and
takes each one through build, verify and integrate on the feature's own
branch, without stopping, until no slice is ready — then puts the questions
it could not answer to the user in sequence.

Four gaps in the process, raised together. The workflow never creates a
branch, so every feature runs wherever the user happened to be. The feature
plan encodes slice dependencies as list order and nothing else, so a
re-entering planner cannot say that a new slice belongs before an existing
one. Every phase boundary stops the agent and waits, including the seven
boundaries a thirteen-slice feature crosses. Integrate leaves the changelog,
knowledge and plan edits uncommitted. The four interlock: unattended
delivery needs slice dependencies to choose what to run next, and needs a
branch of its own before it may merge anything without a human watching.

What the work delivers:

- O1 — a feature runs on a branch of its own, created at frame, and nothing
  unattended reaches the default branch.
- O2 — a plan states each slice's dependencies, so a re-entering planner can
  place a new slice before an existing one.
- O3 — stages 4-7 run unattended over a thirteen-slice feature without a
  turn boundary stopping to ask.
- O4 — integrate commits the changelog, knowledge and plan edits it makes.

The facts the design rests on:

- `pack/knowledge/skills/frame/SKILL.md` ends at the frame and hands to
  `/spec`; no branch, no commit.
- `pack/knowledge/templates/feature-plan.md` gives each slice Done, Change,
  Done-check and Cases. Order is the only dependency model; P003's preamble
  states the constraint in prose ("both sit as early as their dependencies
  allow"), which nothing can check.
- `pack/knowledge/skills/integrate/SKILL.md:31-42` edits the changelog, the
  knowledge and the plan, and commits none of it.
- Hooks are `ManagedItem::JsonEntry` against `.claude/settings.json`;
  `crates/lib/superdev-core/src/components/aokf.rs:83-89` declares the
  PostToolUse validation hook that way, and
  `crates/app/superdev/src/aokf_cli.rs:60` already says "one verb per hook,
  so future hooks slot in beside `validate`".
- Since P003 slice 3, `build.rs` derives the embedded file table from
  `/pack` and items are read by layout rules, so a new skill directory ships
  with no Rust edit. No test asserts an exhaustive skill set.
- [development-procedure][sokf:development-procedure] names no branching
  convention, although `/integrate` and `/verify` both send the reader there
  for the merge target.

The design is two mechanisms for one behaviour (D-1).
`/execute-feature-plan` is a knowledge-carried skill and holds the whole loop
in prose, so it works in any harness. A managed Stop hook enforces it where
Claude Code is present: while a run is active and a slice is ready, the hook
refuses to let the turn end. A repo without the hook still gets the
behaviour from the skill; a repo with it gets the behaviour whether or not
the model stays disciplined.

A run-state file under `.superdev/cache/` arms the hook. Absent that file —
every ordinary session — the hook exits 0 and is invisible, which is what
keeps a shipped Stop hook from holding users' sessions open. The hook stays
dumb (D-2): it reads the run state's owner, its `next` action and its
continue counter, and decides only whether to let the turn end; it never
parses the plan. The skill, which has read the plan anyway, writes the
verdict, so the slice format stays pack content and changes in a pack
release rather than a binary release — the argument S014 and ADR-006 make.
The counter is a watchdog the hook owns (D-3): every continue costs a tick,
and the skill resets it to zero whenever it writes a new `next`, so a run
whose model has stopped making progress dies at the cap. The run state names
the session that owns it (D-4), because `.claude/settings.json` is
repo-scoped and the hook fires in every session in the repo.

The run state is not the workflow's. `superdev run begin|advance|end` own
it, `superdev run hook` reads it, and any skill may drive a no-stop run
through those verbs (D-6); `/execute-feature-plan` is the first consumer,
not the only possible one. The verbs also give the exclusive creation the
lock rests on, which a model hand-writing TOML cannot do. `Stop` fires for
the main agent only — `SubagentStop` is a separate event — so
subagent-per-slice and the hook compose. A blocked run ends rather than
pauses (D-5): the questions go into the plan's Deferred decisions section,
the run state is released, and resuming is `/execute-feature-plan` again.
The run never touches the default branch (D-7); the user fast-forwards
`main` when they choose, which is the trade the design rests on.

The autonomy rule, in one line: a gate that returns to `/build` or
`/feature-plan` is the agent's to answer; a gate that returns to `/spec` or
`/interface-design` is the user's, and becomes a deferred decision.
Self-chaining the four skills was rejected — nowhere to hold the decision
queue, and every phase would carry the autonomy rules. The `Workflow` tool
was rejected — it cannot put a question to a human mid-run, which is half of
what this needs.

Out of scope:

- Changing what the eight phases do. This adds a driver above stages 4-7 and
  four steps inside existing skills.
- New planning logic. `/execute-feature-plan` invokes `/feature-plan`; the
  cutting rules stay in that skill.
- Autonomy for stages 1-3 or for accept. Framing, specifying and interface
  decisions stay interactive.
- An unattended merge to the default branch.
- A validator rule for `Depends-on`. The feature-plan GATE and
  `/double-check` cover it.
- Parallel runs in one working tree. Two git worktrees already have separate
  `.superdev/cache/`, so genuinely parallel runs are already separate; a
  second run in one tree is refused rather than raced.
- Teaching the binary the plan format.
- Resuming a blocked run in place.

The risks and what answers them. A shipped Stop hook that holds users'
sessions open is the worst failure this plan can cause, and it lands in
every managed repo: the hook exits 0 unless the run-state file exists, the
hook itself increments the cap counter, and it ships with the canonical
knowledge capability, so `--no-knowledge` never gets it. A run in one
session holding every other session open, or two runs racing on one working
tree: the run state names its owning session and `begin` creates the file
exclusively. Unattended merging: block 2 gives the feature a branch before
any slice runs, so block 2 must land before block 3. A subagent per slice
loses the interview context — accepted, because by stage 5 the spec, the
contract and the plan are the primary sources and all are on disk. A stalled
run on a bad autonomy call: the rule is drawn on the existing gates, which
already name their return phase, rather than on judgement. The model writes
the run state and the hook trusts it — the accepted cost of keeping the
plan's format out of the binary, bounded by the watchdog. `superdev run`
adds four verbs to a CLI surface carrying a stability promise — taken
deliberately, because skills hand-writing the state file forfeit the
exclusive creation the lock rests on.

This plan is sequenced behind P003 and starts once its last slice is merged.
[Issue-024][sokf:issue-024-the-workflow-cannot-run-unattended] supersedes
it, delivered by [plan-013][sokf:plan-013-workflow-autonomy] against the
five-phase workflow; the decisions there are ADR-018 to ADR-021 and the seam
is contract-009.

## Contract changes

- none.

## Work blocks

### Block 1: Conventions and the plan format

- [x] Done — ticked at merge.
- Depends-on: none.
- Change: add a branching line to
  `pack/knowledge/templates/development-procedure.md` and record this repo's
  own in `knowledge/development-procedure.md` — one branch per feature,
  `feature/<slug>` off the default branch, fast-forwarded to `main` by a
  human. Where a repo's concept names no convention, `/frame` uses
  `feature/<slug>` and writes the line into the concept. `/adhoc-plan`
  branches `chore/<slug>` when the work touches code, and not for a
  documentation-only plan. Add `Depends-on:` beside `Cases:` in
  `pack/knowledge/templates/feature-plan.md` (slice numbers in any
  direction, or `none`) and a `## Deferred decisions` section, and teach
  `pack/knowledge/skills/feature-plan/SKILL.md` to state each slice's
  dependencies, order the list topologically where it can, and GATE on a
  cycle. A forward reference is legal and load-bearing: integrate's replan
  edge adds a slice that must precede an undone one, and saying so must not
  renumber every slice after it and strand the references in commits and
  issues. List order is the reading and default order; dependencies bind.
  Existing plans keep their order-only form, since P003 is complete before
  this work starts.
- Done-check: nothing yet references the convention or the field, so the
  block stands alone and the tree is unchanged in behaviour.
- Cases:
  - observation: every slice in a plan cut from here on carries
    `Depends-on:`, and the planner's output is topologically ordered —
    checks that the plan format models dependencies rather than list order.

### Block 2: Branching and committing inside the workflow

- [x] Done — ticked at merge.
- Depends-on: 1.
- Change: `/frame` creates the branch per block 1's convention and commits
  its knowledge edits; `/spec` commits the spec; `/interface-design` GATEs on
  the user's go-ahead, commits the contract and the ADRs, and hands to
  `/execute-feature-plan`. Add a final step to
  `pack/knowledge/skills/integrate/SKILL.md`: after a successful merge,
  commit the changelog, knowledge and plan edits per
  `template-commit-message`. A failed check returns to `/build` before the
  merge, so nothing is committed on failure.
- Done-check: a scratch repo run through `/frame` sits on `feature/<slug>`
  with its knowledge edits committed, and `/integrate` leaves no
  uncommitted record edit.
- Cases:
  - e2e: `/frame` in a scratch repo leaves the working tree on
    `feature/<slug>` with the knowledge edits committed — checks that a
    feature runs on a branch of its own.
  - e2e: `/integrate` leaves no uncommitted changelog, knowledge or plan
    edit — checks that integrate commits the records it writes.

### Block 3: The run facility and the unattended loop

- [x] Done — ticked at merge.
- Depends-on: 1, 2.
- Change: add `pack/knowledge/skills/execute-feature-plan/SKILL.md` covering
  stages 4-7. It runs `/feature-plan` when no plan exists, then picks the
  next slice whose `Depends-on` are all Done and runs build to verify to
  integrate in a subagent, looping; integrate's replan return re-enters
  `/feature-plan` inside the loop. A slice failing verify returns to build at
  most twice, then becomes a deferred decision. It drives the run through
  `superdev run begin` on entry, `superdev run advance --next` at every real
  step forward — which is also what resets the watchdog — and `superdev run
  end` when it stops. The layout rules pick the directory up, so the skill
  needs no Rust change. Add a top-level `run` verb in
  `crates/app/superdev/src/main.rs` with `begin`, `advance`, `end` and
  `hook`, and `crates/app/superdev/src/run.rs` beside `aokf_cli.rs` holding
  the run state. `begin` creates `.superdev/cache/run.toml` exclusively —
  recording the owning `session_id`, `next`, the continue counter, a start
  timestamp and the pid — and refuses when one exists, naming the owner and
  how to clear it. `advance --next` rewrites `next`, resets the counter to
  zero, and refreshes the owning session id. `end` removes the file. `run
  hook` is the Stop hook body, modelled on `hook_validate` including its
  `CLAUDE_PROJECT_DIR` preference and its loud exit 2 on an unreadable
  payload; honour a `stop_hook_active` flag as well if the Stop payload
  carries one, without resting the guard on it. Register it as a second
  `ManagedItem::JsonEntry` in `components/aokf.rs` at `hooks.Stop`, marker
  `superdev run hook`, shipping with the canonical knowledge capability, and
  document the four verbs in `knowledge/api-contracts.md`.
- Done-check: the hook exits 0 on every disarmed path and exits 2 naming
  `next` otherwise; `begin` refuses a second run; a repo with no run in
  progress sees no behaviour change.
- Cases:
  - unit: `superdev run hook` exits 0 with no run-state file, with a foreign
    `session_id`, with an empty `next`, and at the continue cap — checks
    that the hook is disarmed unless a run it owns is live.
  - unit: `superdev run hook` exits 2 naming `next` otherwise, and the
    counter is one higher afterwards — checks that the hook owns the
    watchdog.
  - unit: `superdev run hook` exits 2 on an unreadable payload, matching
    `hook_validate` — checks that a malformed payload fails loudly.
  - unit: `superdev run begin` refuses when a run exists and names its owner
    — checks the exclusive creation the lock rests on.
  - unit: `superdev run advance` resets the counter and refreshes the owning
    session, and `end` removes the file and is harmless when none exists —
    checks the run state's lifecycle.

### Block 4: Documents and dogfooding

- [x] Done — ticked at merge.
- Depends-on: 3.
- Change: update `pack/agents/process.md` — the branch at frame, the
  go-ahead gate after stage 3, the loop over stages 4-7, and the commit
  points, in the diagram and the phase list — and add
  `/execute-feature-plan` to `pack/knowledge/skills/how-do-i/SKILL.md`'s map.
  Run `cargo run -- sync` to materialise the pack edits into
  `.claude/skills/`, `.agents/` and `knowledge/templates/`, and add a
  changelog entry for the new skill and the new hook.
- Done-check: `superdev sync` writes the Stop-hook claim into
  `.superdev/lock.toml`, `superdev status` exits 0 afterwards, and the
  process documents name the branch, the commit points and the new skill.
- Cases:
  - e2e: a thirteen-slice plan runs stages 4-7 to completion with no turn
    boundary stopping to ask — checks that the loop is unattended end to
    end.
  - e2e: `git log` on the default branch shows nothing from an unattended
    run — checks that unattended work stays on the feature branch.

## Deferred decisions

- Block 3: does `session_id` survive a `--resume`? Not established from the
  documentation; `run advance` refreshes the owner to cover it. Blocks
  nothing.

<!-- sokf:links -->
[sokf:development-procedure]: /knowledge/development-procedure.md
[sokf:issue-024-the-workflow-cannot-run-unattended]: /knowledge/issues/done/issue-024-the-workflow-cannot-run-unattended.md
[sokf:plan-013-workflow-autonomy]: /knowledge/plans/done/plan-013-workflow-autonomy.md
