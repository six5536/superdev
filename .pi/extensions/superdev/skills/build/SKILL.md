# BUILD — state machine

## 1. Definitions

```
machine: Build
initial: Inspecting

states:
  Inspecting
  Resuming
  PlanCheck
  Implementing
  Verifying
  Committing
  ContextReset
  FinalVerification
  Review
  RestoringCheckout
  Correcting
  Paused
  HandoffRequired (terminal)
  RescopeRequired (terminal)
  Accepted (terminal)

events:
  INSPECTED
  RESUME_OK
  RESUME_FAIL
  PLAN_OK
  INTENT_CHANGED
  BLOCK_DONE
  CONTEXT_FULL
  OUT_OF_SCOPE
  VERIFY_PASS
  VERIFY_FAIL
  COMMIT_OK
  AREAS_REJECTED
  NEXT_BLOCK
  ALL_BLOCKS_DONE
  FINDINGS
  REVIEW_CLEAN
  ASSESSMENT_VOID
  CHECKOUT_RESTORED
  CORRECTION_DONE
  PAUSE
  RESUME

guards:
  phaseIsBuild
  buildNotStarted
  hasSavedWork
  blocksRemaining
  evidenceComplete
  retryBudgetLeft
  findingsInScope
  worktreeClean
  verificationPassed

actions:
  inspectPhase
  requestHandoff
  reattachWorker
  inspectWorktree
  reportResumeFailure
  comparePlan
  explainMismatch
  recommendScopeSkill
  callRescope
  stateBlockContext
  runBlock
  runVerification
  attachEvidence
  requireEvidence
  commitBlock
  reportCommit
  reportRejectedAreas
  recordUnfinished
  resetContext
  consumeRetry
  reportExhaustedBudget
  pauseWorker
  runFinalVerification
  captureGitState
  compareGitState
  runReview
  recordAssessment
  reportVoid
  restoreCheckout
  reportDirtyWorktree
  completeBuild
  handoffToAccept

on-enter:
  Inspecting         / inspectPhase
  Resuming           / reattachWorker, inspectWorktree
  PlanCheck          / comparePlan
  Implementing       / stateBlockContext, runBlock
  Verifying          / runVerification
  Committing         / commitBlock
  ContextReset       / resetContext
  FinalVerification  / resetContext, runFinalVerification
  Review             / resetContext, captureGitState, runReview
  RestoringCheckout  / restoreCheckout

on-exit:
  Review             / compareGitState

invariants:
  reportAfterEachStep         # block, evidence, commit
  noUnverifiedAsVerified
  mutateOnlyViaRunPhase
  neverRepeatCompletedBlock
  commitBlockOnce

transitions:
# source                  + event             [guards]                            -> target             / actions

  [t01] Inspecting        + INSPECTED         [buildNotStarted]                   -> HandoffRequired    / requestHandoff
  [t02] Inspecting        + INSPECTED         [hasSavedWork, phaseIsBuild]        -> Resuming
  [t03] Inspecting        + INSPECTED         [phaseIsBuild]                      -> PlanCheck
  [t04] Inspecting        + INSPECTED                                             -> HandoffRequired    / requestHandoff

  [t05] Resuming          + RESUME_OK                                             -> PlanCheck
  [t06] Resuming          + RESUME_FAIL                                           -> Paused             / reportResumeFailure, pauseWorker

  [t07] PlanCheck         + PLAN_OK           [blocksRemaining]                   -> Implementing
  [t08] PlanCheck         + PLAN_OK                                               -> FinalVerification
  [t09] PlanCheck         + INTENT_CHANGED                                        -> RescopeRequired    / explainMismatch, recommendScopeSkill

  [t10] Implementing      + BLOCK_DONE                                            -> Verifying
  [t11] Implementing      + CONTEXT_FULL                                          -> ContextReset       / recordUnfinished
  [t12] Implementing      + OUT_OF_SCOPE                                          -> RescopeRequired    / callRescope

  [t13] Verifying         + VERIFY_PASS       [evidenceComplete]                  -> Committing         / attachEvidence
  [t14] Verifying         + VERIFY_PASS                                           -> .                  / requireEvidence
  [t15] Verifying         + VERIFY_FAIL       [retryBudgetLeft]                   -> Implementing       / attachEvidence, consumeRetry
  [t16] Verifying         + VERIFY_FAIL                                           -> Paused             / attachEvidence, reportExhaustedBudget, pauseWorker
  [t17] Verifying         + CONTEXT_FULL                                          -> ContextReset       / recordUnfinished

  [t18] Committing        + COMMIT_OK                                             -> ContextReset       / reportCommit
  [t19] Committing        + AREAS_REJECTED    [retryBudgetLeft]                   -> Implementing       / reportRejectedAreas, consumeRetry
  [t20] Committing        + AREAS_REJECTED                                        -> Paused             / reportRejectedAreas, reportExhaustedBudget, pauseWorker

  [t21] ContextReset      + NEXT_BLOCK                                            -> Implementing
  [t22] ContextReset      + ALL_BLOCKS_DONE                                       -> FinalVerification

  [t23] FinalVerification + VERIFY_PASS                                           -> Review
  [t24] FinalVerification + VERIFY_FAIL       [retryBudgetLeft]                   -> Implementing       / attachEvidence, consumeRetry
  [t25] FinalVerification + VERIFY_FAIL                                           -> Paused             / attachEvidence, reportExhaustedBudget, pauseWorker

  [t26] Review            + ASSESSMENT_VOID                                       -> RestoringCheckout  / reportVoid
  [t27] Review            + FINDINGS          [findingsInScope, retryBudgetLeft]  -> Correcting         / recordAssessment, consumeRetry
  [t28] Review            + FINDINGS          [findingsInScope]                   -> Paused             / recordAssessment, reportExhaustedBudget, pauseWorker
  [t29] Review            + FINDINGS                                              -> RescopeRequired    / recordAssessment, callRescope
  [t30] Review            + REVIEW_CLEAN      [worktreeClean, verificationPassed] -> Accepted           / completeBuild, handoffToAccept
  [t31] Review            + REVIEW_CLEAN                                          -> .                  / reportDirtyWorktree

  [t32] RestoringCheckout + CHECKOUT_RESTORED                                     -> Review

  [t33] Correcting        + CORRECTION_DONE                                       -> FinalVerification
  [t34] Correcting        + CONTEXT_FULL                                          -> ContextReset       / recordUnfinished
  [t35] Correcting        + OUT_OF_SCOPE                                          -> RescopeRequired    / callRescope

  [t36] Paused            + RESUME                                                -> Inspecting
  [t37] Paused            + PAUSE                                                 -> .
  [t38] *                 + PAUSE                                                 -> Paused             / pauseWorker
```

## 2. Semantics

**Matching.** Guards evaluate top to bottom; the first fully-satisfied line wins.
An unguarded line following a guarded one for the same `(source, event)` pair is
the else-branch. `[a, b]` is conjunction. `*` matches any non-terminal source and
is checked after all explicit lines. An event with no matching line is a logged
no-op, never an error. Terminal states accept no events.

**Ordering.** On a taken transition, in order:

1. `on-exit` of the source state
2. transition actions, left to right
3. `on-enter` of the target state

This ordering is load-bearing. `recordUnfinished` sits on the transition and
`resetContext` on the entry, so partial work is always persisted before the
context that holds it is discarded.

**Transition kinds.** `-> .` is internal: transition actions run, state does not
change, exit and entry do not fire. `-> SameState` is a real self-transition and
runs all three phases.

## 3. States

- `Inspecting` — `superdev_run_phase` with `action: "inspect"`. Select the workflow by intent, not by matching numbers. Reads the approved plan, linked issue, saved checkpoint, unfinished work, consumed retries, recovery notes. Absent local state is absence of information, never prior approval.
- `Resuming` — `action: "resume"`. In worker mode this reattaches the one persistent worker session. The worktree is inspected directly before proceeding — an interrupted checkpoint is not a completed one, and its own record of itself is not trustworthy.
- `PlanCheck` — Any user text appended since approval is compared against the approved plan.
- `Implementing` — One work block, `action: "run"`, `note` carrying that block's instructions. The block, its declared areas, and its acceptance criteria are stated before work starts. No new requirements, interfaces, contracts, or criteria.
- `Verifying` — The block's verification commands run here. Their real results, failures included, become the evidence. A test and the code under it are never changed together to make a suite pass.
- `Committing` — `commit-block` with the stable `block` number, a `note` commit message, and the plan's declared `areas`.
- `ContextReset` — `run` with `reset: true`. Reached after every committed block and whenever context fills mid-block.
- `FinalVerification` — `stage: "verification"`, after the last block and after every correction.
- `Review` — `stage: "review"`. Inspects freely — diffs, searches, type-checks, project commands — and leaves the checkout exactly as found. Scratch files go outside the checkout.
- `RestoringCheckout` — The checkout is inspected and returned to its pre-review state so a valid assessment can be made.
- `Correcting` — Within-scope review findings are fixed. Findings survive as `lastAssessment`, so a compacted or restarted conversation recovers them.
- `Paused` — The worker stops. Progress, approvals, and consumed retries all remain.
- `HandoffRequired` — Terminal. Return to `/skill:scope` and request the handoff.
- `RescopeRequired` — Terminal. The intent is wrong and must change in SCOPE; the plan is not widened here.
- `Accepted` — Terminal. The candidate has moved to ACCEPT; continue with `/skill:accept`.

## 4. Guards

- `phaseIsBuild` — Does `inspect` report phase `build`?
- `buildNotStarted` — Does it report `build-not-started`? Plan approval alone does not start execution.
- `hasSavedWork` — Is there a saved checkpoint to reattach to?
- `blocksRemaining` — Are there blocks in the plan not listed in `completed_blocks`?
- `evidenceComplete` — Does the capture hold the commands actually run and their actual results?
- `retryBudgetLeft` — Is budget left under project policy? Durable across reset, pause, and restart — none of those grants another attempt.
- `findingsInScope` — Are the findings fixable inside the approved plan?
- `worktreeClean` — Is the worktree clean?
- `verificationPassed` — Did final verification pass on the _current_ candidate?

## 5. Actions

- `inspectPhase` — Calls `action: "inspect"` and loads durable workflow state. Read-only.
- `requestHandoff` — Returns to `/skill:scope` and asks for the handoff. Does not begin work.
- `reattachWorker` — Reattaches the single persistent worker session. Not idempotent across sessions — one worker only.
- `inspectWorktree` — Reads the actual worktree. Never trusts the checkpoint's self-report.
- `reportResumeFailure` — States what could not be reattached or read, before pausing.
- `comparePlan` — Diffs appended user text against approved intent. Read-only.
- `explainMismatch` — States what diverged, in terms of intent rather than wording.
- `recommendScopeSkill` — Recommends `/skill:scope <requested change>`. Advisory only; mutates nothing.
- `callRescope` — Calls `action: "rescope"`, for when implementation reveals the approved intent is itself wrong. Mutates workflow state — distinct from `recommendScopeSkill`.
- `stateBlockContext` — Declares block, areas, and acceptance criteria before work begins.
- `runBlock` — Calls `action: "run"` with the block's instructions in `note`.
- `runVerification` — Runs the block's verification commands and captures real results, failures included. Must not summarize a failure into a pass.
- `attachEvidence` — Passes the captured commands and results in `evidence`.
- `requireEvidence` — Blocks the commit and asks for the missing commands and results.
- `commitBlock` — Calls `commit-block` within the plan's declared areas. The service refuses changes outside them rather than absorbing them. Runs exactly once per block; `AREAS_REJECTED` means nothing was committed.
- `reportCommit` — Reports the block and its commit.
- `reportRejectedAreas` — Names the files that fell outside the plan's declared areas.
- `recordUnfinished` — Writes partial work and any uncertain command outcome to `unfinished`. Ordered before `resetContext`: the next stage rebuilds from the approved plan and these saved facts, not from the conversation.
- `resetContext` — Calls `run` with `reset: true`. Discards conversation, keeps durable state.
- `consumeRetry` — Calls `consume-retry` with the budgeted activity in `note`. Durable. One per failed verification or correction.
- `reportExhaustedBudget` — Reports the exhausted budget plainly, before pausing.
- `pauseWorker` — Stops the worker. Progress, approvals, and consumed retries persist.
- `runFinalVerification` — Calls `stage: "verification"` across the whole candidate.
- `captureGitState` — Snapshots Git state on review entry.
- `compareGitState` — Compares against the snapshot on review exit. Any change voids the assessment.
- `runReview` — Calls `stage: "review"`. Inspection only; must leave the checkout byte-identical.
- `recordAssessment` — Persists findings as `lastAssessment`, recoverable via `inspect`. `stale: true` means the candidate has moved since they were made.
- `reportVoid` — Reports that the assessment judged a candidate that no longer matches the one it started from.
- `restoreCheckout` — Returns the checkout to its snapshot.
- `reportDirtyWorktree` — Names what is uncommitted, blocking completion without voiding the review.
- `completeBuild` — Calls `complete-build`. Moves the candidate to ACCEPT.
- `handoffToAccept` — Hands off to `/skill:accept`.

## 6. Notes

**t04** — Any phase that is neither `build` nor `build-not-started` also routes to
handoff, so the guard set is total.

**t11, t17, t34** — A reset triggered mid-block returns to `Implementing` on the
_same_ block. `NEXT_BLOCK` names the event, not necessarily a different block;
`recordUnfinished` is what carries position across the reset.

**t14** — `VERIFY_PASS` without complete evidence reports and waits rather than
failing the block or spending budget.

**t19** — A refused commit is treated as a budgeted activity. Nothing was
committed, so the block is retried in full.

**t27 → t33 → t23** — Corrections re-enter final verification, which re-enters
review. This is what prevents a `stale: true` assessment from being acted on as
current.

**t31** — `REVIEW_CLEAN` with a dirty worktree reports and waits. It does not
void the assessment or consume budget.

**t38** — The wildcard is checked after every explicit line, so `Paused + PAUSE`
is caught by t37 and no-ops instead of re-running `pauseWorker`.

## 7. Positions to confirm against project policy

The machine takes a position on each of these. None is forced by the workflow
itself, and each is cheap to change.

1. **Retry budget scope.** One shared budget for the whole build, not one per
   block. Failed verification, refused commit, and review correction all draw
   from the same pool.

2. **Where a failed final verification resumes.** t24 returns to `Implementing`
   on the block the failure implicates.

3. **Refused commits are budgeted.** t19 consumes; flip to a free retry if
   area violations should not count against verification attempts.

4. **Voided assessments are not budgeted.** t26 restores and re-reviews at no
   cost, which permits an unbounded loop if the checkout keeps drifting.

5. **Resume re-enters through `Inspecting`.** A paused build re-reads durable
   state rather than trusting anything held locally.

6. **`PlanCheck` runs once, before the first block.** User text appended during
   implementation does not get the same intent comparison.
