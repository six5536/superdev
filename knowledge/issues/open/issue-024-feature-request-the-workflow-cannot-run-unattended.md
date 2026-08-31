---
type: FeatureRequest
id: issue-024-feature-request-the-workflow-cannot-run-unattended
title: The workflow cannot deliver a feature unattended
description: Every phase boundary stops and waits for the user, no feature gets a branch of its own, a plan models no slice dependencies, and integrate leaves its record edits uncommitted.
lifecycle: open
links:
  - rel: references
    to: plan-004-adhoc-workflow-autonomy
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

## Motivation

Four gaps, raised together in
[plan-004][sokf:plan-004-adhoc-workflow-autonomy]:

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

## Proposed behaviour

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

## Acceptance criteria

1. [event] WHEN `/frame` files a feature's issue THE SYSTEM SHALL have
   created `feature/<slug>` off the default branch and SHALL commit the
   issue on it.
2. [event] WHEN `/adhoc-plan` plans work that touches code THE SYSTEM
   SHALL create `adhoc/<slug>` off the default branch.
3. [event] WHEN `/contract-design` ends with the user's go-ahead THE
   SYSTEM SHALL commit the contract and decision-record edits before the
   unattended loop starts.
4. [ubiquitous] THE SYSTEM SHALL record, for every slice in a feature
   plan, the slices it depends on, and SHALL refuse a plan whose
   dependencies form a cycle.
5. [state] WHILE a run is active and a slice's dependencies are all done
   THE SYSTEM SHALL carry that slice through build and integrate without a
   turn boundary stopping to ask.
6. [event] WHEN integrate merges a slice THE SYSTEM SHALL commit the
   changelog, knowledge and plan edits it made.
7. [event] WHEN a gate returns to `/frame` or `/contract-design` THE
   SYSTEM SHALL write the question into the plan's deferred decisions and
   continue with the next ready slice.
8. [conditional] IF a slice fails its checks after two returns to build
   THE SYSTEM SHALL defer it and continue with the next ready slice.
9. [event] WHEN no slice is ready THE SYSTEM SHALL end the run and put the
   plan's deferred decisions to the user in sequence.
10. [ubiquitous] THE SYSTEM SHALL make no commit and no merge to the
    default branch during a run.
11. [event] WHEN a run is begun while another owns the working tree THE
    SYSTEM SHALL refuse it, naming the owning session and how to clear it.
12. [event] WHEN a run continues without a step forward THE SYSTEM SHALL
    end it at a fixed cap of continues.
13. [state] WHILE no run is active THE SYSTEM SHALL leave every session's
    turn boundaries untouched.

## Alternatives considered

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

## Scope

- In: the branching conventions at `/frame` and `/adhoc-plan`; slice
  dependencies in the feature-plan format; the unattended loop over
  feature-plan, build and integrate; the commit points at frame,
  contract-design and integrate; the failure behaviour above (refusal,
  retry bound, cap, deferral).
- Out: autonomy for `/frame`, `/contract-design` and `/accept`; unattended
  merge to the default branch; parallel runs in one working tree; resuming
  a blocked run in place rather than ending it; changes to what the phases
  themselves do.

## Comments

Framing interview, 2026-08-31: the go-ahead gate sits at the end of
`/contract-design`, and the unattended loop owns feature-plan, build and
integrate. [Plan-004][sokf:plan-004-adhoc-workflow-autonomy] is superseded
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
[sokf:plan-004-adhoc-workflow-autonomy]: /knowledge/plans/open/plan-004-adhoc-workflow-autonomy.md
