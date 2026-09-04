---
type: Issue
id: issue-024-the-workflow-cannot-run-unattended
title: The workflow cannot deliver a feature unattended
description: Every phase boundary stops and waits for the user, no feature gets a branch of its own, a plan models no slice dependencies, and integrate leaves its record edits uncommitted.
kind: feature
lifecycle: done
links:
  - rel: references
    to: plan-004-workflow-autonomy
    note: The adhoc plan that designed this work against the seven-phase workflow.
  - rel: references
    to: contract-002-cli-superdev
    note: Adds the superdev run verbs and the hook run Stop hook.
  - rel: references
    to: contract-009-interface-run-state
    note: New — the run-state seam between the driver skill and the Stop hook.
---

# Feature: the workflow cannot deliver a feature unattended

## Summary

The workflow stops at every phase boundary and waits for the user. It
creates no branch, so a feature runs wherever the user happened to be; a
feature plan encodes slice dependencies as list order and nothing else; and
integrate edits the changelog, the knowledge and the plan without
committing any of it. A feature is delivered at the pace of the user's
attention, one boundary at a time.

## Context

Four gaps, raised together in
[plan-004][sokf:plan-004-workflow-autonomy]:

- P003 ran eighteen slices, and every slice crossed build and integrate
  with the user present — over thirty attended boundaries for one feature.
- The `feature/content-packs` branch merged on 2026-08-31 carried 97
  commits across nine plans, because the workflow never cut a branch per
  feature.
- A re-entering planner cannot say that a new slice belongs before an
  existing one: order is the only dependency model, and renumbering
  strands the slice references already written into commits and issues.
- `/integrate` leaves the changelog, knowledge and plan edits uncommitted,
  so the records land in whatever commit comes next.

## Behaviour

The user frames a feature and settles its contracts interactively, as
today. From there the workflow delivers the plan on its own:

- `/frame` creates `feature/<slug>` off the default branch and commits the
  framed issue on it. Where the repo's development procedure names a
  branching convention, that convention wins; where it names none,
  `/frame` uses `feature/<slug>` and records the convention.
- `/adhoc-plan` creates `adhoc/<slug>` off the default branch when the
  planned work touches code, and no branch for a documentation-only plan.
- `/contract-design` ends by asking the user's go-ahead, committing the
  contract and decision-record edits, and handing to the unattended loop.
- Each slice in a feature plan states the slices it depends on; the slice
  list is ordered so every slice follows its dependencies, and a forward
  reference does not renumber the slices after it.
- The loop cuts the plan when none exists, then repeatedly takes a slice
  whose dependencies are all done through build and integrate, without a
  turn boundary stopping to ask. After each successful integrate, the
  changelog, knowledge and plan edits are committed.
- A question only the user can answer — a gate that returns to `/frame` or
  `/contract-design` — is written into the plan as a deferred decision;
  the run ends when no slice is ready and puts the deferred decisions to
  the user in sequence. Answering them and re-invoking the loop resumes
  the work from the plan.
- The run commits and merges only on the feature's branch. The user
  fast-forwards the default branch when they choose.

Failure behaviour:

- A slice that fails its checks returns to build at most twice; the third
  failure defers it and the loop continues with the next ready slice.
- A second run begun while one owns the working tree is refused, naming
  the owner and how to clear it.
- A run that continues without progressing ends at a fixed cap.
- A repo with no run in progress sees no behaviour change: no session is
  held open.

The feature is done when the workflow meets these expectations:

- When `/frame` files a feature's issue, superdev has created
  `feature/<slug>` off the default branch and commits the issue on it.
- When `/adhoc-plan` plans work that touches code, superdev creates
  `adhoc/<slug>` off the default branch.
- When `/contract-design` ends with the user's go-ahead, superdev commits
  the contract and decision-record edits before the unattended loop
  starts.
- Superdev records, for every slice in a feature plan, the slices it
  depends on, and refuses a plan whose dependencies form a cycle.
- While a run is active and a slice's dependencies are all done, superdev
  carries that slice through build and integrate without a turn boundary
  stopping to ask.
- When integrate merges a slice, superdev commits the changelog, knowledge
  and plan edits it made.
- When a gate returns to `/frame` or `/contract-design`, superdev writes
  the question into the plan's deferred decisions and continues with the
  next ready slice.
- If a slice fails its checks after two returns to build, superdev defers
  it and continues with the next ready slice.
- When no slice is ready, superdev ends the run and puts the plan's
  deferred decisions to the user in sequence.
- Superdev makes no commit and no merge to the default branch during a
  run.
- When a run is begun while another owns the working tree, superdev
  refuses it, naming the owning session and how to clear it.
- When a run continues without a step forward, superdev ends it at a
  fixed cap of continues.
- While no run is active, superdev leaves every session's turn boundaries
  untouched.

## Scope

The work covers the branching, dependency and loop machinery and stops
short of the attended phases.

- In: the branching conventions at `/frame` and `/adhoc-plan`; slice
  dependencies in the feature-plan format; the unattended loop over
  feature-plan, build and integrate; the commit points at frame,
  contract-design and integrate; the failure behaviour above (refusal,
  retry bound, cap, deferral).
- Out: autonomy for `/frame`, `/contract-design` and `/accept`; unattended
  merge to the default branch; parallel runs in one working tree; resuming
  a blocked run in place rather than ending it; changes to what the phases
  themselves do.

Alternatives considered:

- Keeping the attended workflow and driving it faster by hand — the
  boundary count grows with the slice count, and P003's thirty stops is
  the measured cost.
- Autonomy over framing and contract design as well — those phases are
  where the user's intent enters the work; automating them removes the
  user's control over what gets built.
- Merging each slice to the default branch as it passes, as today —
  unattended work must not reach the default branch on its own.
- An external orchestrator carrying the loop — it cannot put a question to
  the user mid-run and answer-and-resume from a durable record.

## Resolution

Delivered by [plan-013][sokf:plan-013-workflow-autonomy] in eight
slices on 2026-08-31: the `superdev run` verbs and the `hook run` Stop
hook (contract-009, ADR-018/019), the managed `hooks.Stop` entry, the
feature-plan format's `Depends-on` and deferred decisions, the branching
and commit conventions in the workflow skills (ADR-021), and the
`/execute-feature-plan` driver (ADR-020). The loop's machinery is
unit- and e2e-tested and was rehearsed live: a headless session under the
armed hook was held at its turn boundary, acted on the named next step,
and released the run. Full-workflow rehearsal in anger is the next
feature's delivery; `/accept` runs at the user's request.

## Comments

Framing interview, 2026-08-31: the go-ahead gate sits at the end of
`/contract-design`, and the unattended loop owns feature-plan, build and
integrate. [Plan-004][sokf:plan-004-workflow-autonomy] is superseded
by this request and the records contract-design and feature-plan produce
from it; it is refiled `done` with a note once those records exist. The
retry bound is two returns to build. `/adhoc-plan` branches
`adhoc/<slug>`.

Contract design, 2026-08-31: the
[CLI contract][sokf:contract-002-cli-superdev] gains the `superdev run`
verbs and the `hook run` Stop hook; the run-state seam between the driver
skill and that hook is the new
[run-state interface contract][sokf:contract-009-interface-run-state].
The decisions are ADR-018 through ADR-021, and the Stop hook body is
`superdev hook run` with a watchdog cap of ten.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:contract-009-interface-run-state]: /knowledge/contracts/internal/active/contract-009-interface-run-state.md
[sokf:plan-004-workflow-autonomy]: /knowledge/plans/done/plan-004-workflow-autonomy.md
[sokf:plan-013-workflow-autonomy]: /knowledge/plans/done/plan-013-workflow-autonomy.md
