---
name: build
description: Use when the user asks to implement an approved plan or resume its implementation.
allowed-tools: read sokf_search sokf_graph superdev_run_phase superdev_ask
---

# BUILD

1. Call `superdev_run_phase` with `phase: "build", action: "inspect"`. Select the workflow by intent, not by matching numbers. Read its approved plan and linked issue, its saved checkpoint, unfinished work, consumed retries, and recovery notes. Missing local state is not prior approval.
2. Confirm the phase is `build`. For `build-not-started`, return to `/skill:scope` and request the handoff; plan approval alone does not start execution.
3. For saved work, use `resume`. In worker mode this reattaches the one persistent worker session. Inspect the actual worktree before continuing: an interrupted checkpoint is not a completed one. Do not repeat a block listed in `completed_blocks`.
4. Compare any appended user text with the approved plan. If it changes intent, explain the mismatch and recommend `/skill:scope <requested change>`. If implementation reveals that the intent itself is wrong, use `rescope`; do not widen the plan here.
5. Implement one work block at a time with `action: "run"` and `note` holding that block's instructions. State the block, its declared areas, and its acceptance criteria. Stay inside the approved plan: no new requirements, interfaces, contracts, or criteria.
6. Verify the block before committing it. Pass the commands you ran and their real results in `evidence`, including failures. Do not change a test and the code together to make a suite pass.
7. Commit the block with `commit-block`, its stable `block` number, a `note` commit message, and the `areas` the plan declares. The service refuses changes outside those areas rather than absorbing them. A block is committed once.
8. Reset context after every completed block, and again when context fills mid-block: call `run` with `reset: true`. Record partial work and any uncertain command outcome in `unfinished` first, because the next stage rebuilds from the approved plan and those saved facts rather than from this conversation.
9. For a failed verification or correction, use `consume-retry` with the budgeted activity in `note`. Budgets come from project policy and are durable. A reset, pause, or restart does not grant another attempt; when one is exhausted, pause and report it.
10. After the last block, run final verification with `stage: "verification"`, then review the candidate with `stage: "review"`. Both stages reset context first. Review inspects freely — diffs, searches, type-checks, the project's own commands — but leaves the checkout exactly as it found it: its Git state is compared before and after, and any change voids the assessment. Write scratch files outside the checkout.
11. Correct within-scope review findings and re-verify. The review's own findings are durable: `inspect` returns them as `lastAssessment`, so a compacted or restarted conversation recovers them rather than losing them. `stale: true` means the candidate has moved since they were made. Route intent changes to SCOPE instead. For `assessment-void`, inspect and restore the checkout, then review again: a voided assessment judged a candidate that no longer matches the one it started from.
12. Call `complete-build` only when the worktree is clean and verification passed. This moves the candidate to ACCEPT; continue with `/skill:accept`.

Report the block, its evidence, and its commit after each step. Report failures, skipped verification, and exhausted budgets plainly; never describe unverified work as verified. Use only `superdev_run_phase` for workflow and Git mutation. On Pause, the worker stops and progress, approvals, and consumed retries remain.
