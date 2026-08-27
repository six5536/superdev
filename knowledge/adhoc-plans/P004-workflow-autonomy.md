---
type: Plan
id: adhoc-plan-workflow-autonomy
title: Workflow autonomy — branch, slice dependencies, unattended delivery
description: Give the workflow a branch at frame, model slice dependencies in the plan, run stages 4-7 unattended on a general superdev run facility with a Stop hook and a new execute-feature-plan skill, and commit at every successful integrate.
status: draft
links:
  - rel: relates-to
    to: development-procedure
    note: Adds the branching convention and the commit points this plan introduces.
---

# Plan: workflow autonomy

## Context

Four gaps in the process, raised together. The workflow never creates a
branch, so every feature runs wherever the user happened to be. The feature
plan encodes slice dependencies as list order and nothing else, so a
re-entering planner cannot say that a new slice belongs before an existing
one. Every phase boundary stops the agent and waits, including the seven
boundaries a thirteen-slice feature crosses. Integrate leaves the changelog,
knowledge and plan edits uncommitted.

The four interlock: unattended delivery needs slice dependencies to choose
what to run next, and needs a branch of its own before it may merge anything
without a human watching.

## Goal

`/interface-design` ends by committing its documents on the user's
go-ahead and handing to `/execute-feature-plan`, which cuts the spec into
slices and takes each one through build, verify and integrate on the
feature's own branch, without stopping, until no slice is ready — then puts
the questions it could not answer to the user in sequence.

Non-goals:

- No change to what the eight phases do. This adds a driver above stages
  4-7 and four steps inside existing skills; it does not rewrite them.
- No new planning logic. `/execute-feature-plan` invokes `/feature-plan`;
  the cutting rules stay in that skill.
- No autonomy for stages 1-3 or for accept. Framing, specifying and
  interface decisions stay interactive.
- No unattended merge to the default branch. `/execute-feature-plan` works
  on the branch `/frame` created, never on `main`; the user fast-forwards.
- No validator rule for `Depends-on`. The feature-plan GATE and
  `/double-check` cover it; `superdev aokf validate` stays knowledge
  validator.

## Current state

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
- [development-procedure](../development-procedure.md) names no branching
  convention, although `/integrate` and `/verify` both send the reader there
  for the merge target.

## Proposed approach

Two mechanisms for one behaviour. `/execute-feature-plan` is a
knowledge-carried skill and carries the whole loop in prose, so it works in
any harness. A managed Stop hook enforces it where Claude Code is present:
while a run is active and a slice is ready, the hook refuses to let the turn
end. A repo without the hook still gets the behaviour from the skill; a repo
with it gets the behaviour whether or not the model stays disciplined.

The hook is armed by a run-state file under `.superdev/cache/`. Absent that
file — every ordinary session — the hook exits 0 and is invisible. This is
what keeps a shipped Stop hook from holding users' sessions open.

The hook stays dumb. It reads the run state's owner, its `next` action and
its continue counter, and decides only whether to let the turn end; it never
parses the plan. The skill, which has read the plan anyway, writes the
verdict. So the slice format stays pure content, free to change in a pack
release without a binary release — the same argument S014 and ADR-006 make.

The counter is a watchdog, and the hook owns it: every continue costs a
tick, and the skill resets it to zero whenever it writes a new `next`. A run
whose model has stopped making progress therefore dies at the cap, which a
counter the skill incremented could never guarantee.

The run state is not the workflow's. `superdev run begin|advance|end` own
it, `superdev run hook` is the Stop hook that reads it, and any skill may
drive a no-stop run through those verbs — `/execute-feature-plan` is the
first consumer, not the only possible one. The verbs also give the exclusive
creation the lock rests on, which a model hand-writing TOML cannot do.

The run state names the session that owns it. `.claude/settings.json` is
repo-scoped, so the hook fires in every session in the repo, and a run
started in one must not hold the others open: the hook exits 0 unless the
payload's `session_id` matches the owner. `superdev run begin` creates the
file exclusively, so a second run in the same repo is refused rather than
racing the first on one working tree. Two git worktrees each have their own
`.superdev/cache/`, so genuinely parallel runs are already separate.

`Stop` fires for the main agent only — `SubagentStop` is a separate event —
so subagent-per-slice and the hook compose: the hook sees the driver's turn
boundaries and never a slice's.

Alternatives: self-chaining the four skills was rejected — nowhere to hold
the decision queue, and every phase would carry the autonomy rules. The
`Workflow` tool was rejected — it cannot put a question to a human mid-run,
which is half of what this needs.

Autonomy rule, in one line: a gate that returns to `/build` or
`/feature-plan` is the agent's to answer; a gate that returns to `/spec` or
`/interface-design` is the user's, and becomes a deferred decision.

Each slice runs in its own subagent. The driver's context holds plan state
and the decision queue and nothing else, which is what makes thirteen slices
fit.

A blocked run ends rather than pauses: the questions are written into the
plan's Deferred decisions section, the run state is deleted and the lock
released, and the user is asked. The plan is the durable record, so resuming
is `/execute-feature-plan` again — it re-reads the plan and the answers. No
idle run holds the repo, and a stale run-state file stays exceptional.

The run never touches the default branch. It commits and merges on the
branch `/frame` created; the user fast-forwards `main` when they choose.
This changes today's practice, where each slice reaches `main` almost
immediately, and it is the trade the design rests on: unattended work must
not reach the default branch on its own.

## Steps

1. **Record the branching convention** — add a branching line to
   `pack/knowledge/templates/development-procedure.md`, and record this
   repo's own in `knowledge/development-procedure.md`: one branch per
   feature, `feature/<slug>` off the default branch, fast-forwarded to
   `main` by a human. Where a repo's concept names no convention, `/frame`
   uses `feature/<slug>` and writes the line into the concept, so the next
   feature reads one that exists. `/adhoc-plan` branches `chore/<slug>` when
   the work touches code, and not for a documentation-only plan. Nothing
   references any of this yet, so the step stands alone.

2. **Model dependencies in the plan** — add `Depends-on:` beside `Cases:` in
   `pack/knowledge/templates/feature-plan.md` (slice numbers in any
   direction, or `none`), and a `## Deferred decisions` section. Teach
   `pack/knowledge/skills/feature-plan/SKILL.md` to state each slice's
   dependencies, order the list topologically where it can, and GATE on a
   cycle. A forward reference is legal and load-bearing: integrate's replan
   edge adds a slice that must precede an undone one, and saying so must not
   renumber every slice after it and strand the references in commits and
   issues. List order is the reading and default order; dependencies bind.
   Existing plans keep their order-only form: P003 is complete before this
   work starts, and backfilling a finished plan buys nothing. The field
   applies to plans cut from here on.

3. **Branch and commit inside stages 1-3** — `/frame` creates the branch per
   step 1's convention and commits its knowledge edits; `/spec` commits the
   spec; `/interface-design` GATEs on the user's go-ahead, commits the
   contract and the ADRs, and hands to `/execute-feature-plan`.

4. **Commit at integrate** — add a final step to
   `pack/knowledge/skills/integrate/SKILL.md`: after a successful merge,
   commit the changelog, knowledge and plan edits per
   `template-commit-message`. A failed check returns to `/build` before the
   merge, so nothing is committed on failure.

5. **Add `/execute-feature-plan`** — a new skill at
   `pack/knowledge/skills/execute-feature-plan/SKILL.md` covering stages 4-7.
   It runs `/feature-plan` when no plan exists, then picks the next slice
   whose `Depends-on` are all Done and runs build to verify to integrate in a
   subagent, looping. Integrate's replan return re-enters `/feature-plan`
   inside the loop — the process diagram already carries that edge, and a
   driver that could not follow it would stop at every replan. A slice
   failing verify returns to build at most twice, then becomes a deferred
   decision. Stop when no slice is ready, then put the queue to the user in
   sequence. It drives the run through step 6's verbs: `superdev run begin`
   on entry, `superdev run advance --next` at every real step forward — which
   is also what resets the watchdog — and `superdev run end` when it stops.
   A blocked run **ends**: the questions go into the plan's Deferred
   decisions section, the run state is released, and resuming is a fresh
   `/execute-feature-plan` that re-reads the plan and the answers. The skill
   itself needs no Rust change; the layout rules pick the directory up.

6. **Add the run facility and its Stop hook** — a new top-level `run` verb
   in `crates/app/superdev/src/main.rs` with `begin`, `advance`, `end` and
   `hook`, and a module beside `aokf_cli.rs` holding the run state. `begin`
   creates `.superdev/cache/run.toml` exclusively — recording the owning
   `session_id`, `next`, the continue counter, a start timestamp and the pid
   — and refuses when one exists, naming the owner and how to clear it.
   `advance --next` rewrites `next`, resets the counter to zero, and
   refreshes the owning session id so a resumed session does not orphan its
   own run. `end` removes the file.

   `run hook` is the Stop hook body, modelled on `hook_validate` including
   its `CLAUDE_PROJECT_DIR` preference and its loud exit 2 on an unreadable
   payload. It exits 0 when the run state is absent, when the payload's
   `session_id` is not the owner, when `next` is empty, or when the counter
   has reached its cap; otherwise it increments the counter and exits 2
   naming `next`, which is documented to prevent Claude from stopping and
   continue the conversation. Confirm whether the Stop payload carries a
   `stop_hook_active` flag and honour it as well if it does, but do not rest
   the guard on it — the counter is the guarantee.

   Register it as a second `ManagedItem::JsonEntry` in `components/aokf.rs`
   at `hooks.Stop`, marker `superdev run hook`. It ships with the canonical knowledge
   capability, like the validation hook, so `--no-knowledge` never gets it.
   Document the four new verbs in `knowledge/api-contracts.md`: they join a
   surface that carries a stability promise.

7. **Update the process documents** — `pack/agents/process.md`: the branch at
   frame, the go-ahead gate after stage 3, the loop over stages 4-7, and the
   commit points, in the diagram and the phase list. Add
   `/execute-feature-plan` to `pack/knowledge/skills/how-do-i/SKILL.md`'s map.

8. **Sync and verify** — `cargo run -- sync` to materialise the pack edits
   into `.claude/skills/`, `.agents/` and `knowledge/templates/`, then the
   checks below. Changelog entry for the new skill and the new hook.

## Files affected

| File | Change |
|------|--------|
| `pack/knowledge/templates/development-procedure.md` | modified — a branching line in the skeleton |
| `knowledge/development-procedure.md` | modified — this repo's branch convention and commit points |
| `pack/knowledge/templates/feature-plan.md` | modified — `Depends-on:` per slice, `## Deferred decisions` |
| `pack/knowledge/skills/feature-plan/SKILL.md` | modified — state dependencies, order topologically, forward-reference GATE |
| `pack/knowledge/skills/frame/SKILL.md` | modified — create the branch, commit the canonical knowledge edits |
| `pack/knowledge/skills/spec/SKILL.md` | modified — commit the spec |
| `pack/knowledge/skills/interface-design/SKILL.md` | modified — go-ahead GATE, commit the contract and ADRs, hand off |
| `pack/knowledge/skills/integrate/SKILL.md` | modified — commit the records after a successful merge |
| `pack/knowledge/skills/execute-feature-plan/SKILL.md` | new — the unattended loop over stages 4-7 |
| `pack/knowledge/skills/adhoc-plan/SKILL.md` | modified — branch `chore/<slug>` when the work touches code |
| `pack/knowledge/skills/how-do-i/SKILL.md` | modified — the new skill in the map |
| `pack/agents/process.md` | modified — diagram, phase list, commit points |
| `crates/app/superdev/src/run.rs` | new — the run state and the `begin`/`advance`/`end`/`hook` bodies |
| `crates/app/superdev/src/main.rs` | modified — the top-level `run` verb and its subcommands |
| `crates/lib/superdev-core/src/components/aokf.rs` | modified — the `hooks.Stop` JsonEntry and its constants |
| `knowledge/api-contracts.md` | modified — the four `superdev run` verbs on the promised CLI surface |
| `CHANGELOG.md` | modified — the new skill and the new hook |
| `.superdev/lock.toml` | modified — the new Stop-hook claim, written by sync |

## Testing & verification

- `run.rs` hook tests: exit 0 with no run-state file; exit 0 when the
  payload's `session_id` is not the owner; exit 0 when `next` is empty; exit
  0 at the continue cap; exit 2 naming `next` otherwise, and the counter is
  one higher afterwards; exit 2 on an unreadable payload, matching
  `hook_validate`.
- `run.rs` state tests: `begin` refuses when a run exists and names its
  owner; `advance` resets the counter and refreshes the owning session;
  `end` removes the file and is harmless when none exists.
- `components/aokf.rs` test: the Stop entry is planned into an empty repo and
  claimed as
  `.claude/settings.json:hooks.Stop[superdev run hook]`; a
  settings file that already carries it plans nothing.
- `cargo nextest run --workspace`.
- `npm run check:blueprint` — the live copies byte-match what the rebuilt
  binary ships, including the new skill directory.
- After sync: `.claude/skills/execute-feature-plan/SKILL.md` exists and is
  hashed into `.superdev/lock.toml`, and the Stop-hook claim is listed
  beside it.
- `cargo run --quiet -- aokf validate knowledge` — PASS.
- Manual: a scratch repo with a two-slice plan, slice 2 depending on slice 1.
  Run `/execute-feature-plan` and confirm it cuts the plan, lands both
  slices without a prompt, commits the records at each integrate, and clears
  the run-state file at the end.
- Manual: with no run active, confirm an ordinary session ends normally —
  the Stop hook must be invisible.
- Manual: with a run active in one session, confirm a second session in the
  same repo ends its turns normally and is never told to continue a slice.
- Manual: a wedged run — set `next` and never advance — stops at the cap
  rather than looping.

## Risks & open questions

- Risk: a shipped Stop hook that holds users' sessions open. This is the
  worst failure this plan can cause, and it lands in every managed repo.
  Mitigation: the hook exits 0 unless the run-state file exists, and the
  hook itself increments the cap counter, so a model that has stopped
  behaving cannot keep the loop alive; the tests above cover each path. It ships with the knowledge capability, so `--no-knowledge`
  never gets it.
- Risk: a run in one session holding every other session in the repo open,
  or two runs racing on one working tree. Mitigation: the run state names its
  owning session and the hook compares `session_id`; `superdev run begin`
  creates the file exclusively so the second run is refused. Both paths are
  tested above.
- Risk: unattended merging. Mitigation: step 3 gives the feature a branch
  before any slice runs, so `/execute-feature-plan` merges only into that
  branch. Ordering matters — step 3 must land before step 5.
- Risk: a subagent per slice loses the interview context. Accepted: by stage
  5 the spec, the contract and the plan are the primary sources and all are
  on disk.
- Risk: the run stalls on a bad autonomy call — the agent answers a question
  that was the user's. Mitigation: the rule is drawn on the existing gates,
  which already name their return phase, rather than on judgement.
- Risk: the run state is written by the model, so the hook trusts it. This
  is the accepted cost of keeping the plan's format out of the binary. The
  watchdog bounds the damage: a wrong `next` continues at most to the cap.
- Risk: `superdev run` adds four verbs to a CLI surface carrying a stability
  promise. Accepted deliberately — the alternative, skills hand-writing the
  state file, forfeits the exclusive creation the lock rests on.
- Open question: whether `session_id` survives a `--resume`. Not established
  from the documentation; `run advance` refreshes the owner to cover it.

## Out-of-band notes

- Sequenced behind P003: start once its last slice is merged and P003 is
  tagged `done`. The pack layout this plan edits is on `main` already, but its
  closing slices cut the pack release and dogfood the pin, and the new skills
  should ship in a release after that.
