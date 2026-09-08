---
type: Idea
id: idea-015-scoped-workflows-feed-independent-build-batches
title: Scoped workflows feed independent build batches
description: Let users scope several issues independently, then build the ready workflows sequentially or in configurable parallel batches without one stopped workflow stopping the others.
status: draft
---

# Idea: scoped workflows feed independent build batches

Decouple SCOPE from immediate BUILD. A user can scope issue X, leave its
approved plan ready for BUILD, then scope Y and Z before building any of them.
The user can later build all ready workflows sequentially or in parallel batches
with a configurable concurrency limit.

Each build worker continues independently until its workflow completes or
returns to SCOPE. One worker returning to SCOPE, blocking, failing, awaiting a
human decision, or being cancelled must not stop unrelated workers.

## Motivation

The current single-owner flow makes workflow preparation and implementation
strictly serial. A user cannot prepare a queue of reviewed work while deciding
when or how much implementation capacity to use. A long or blocked build also
prevents independent ready work from progressing.

## Sketch

Store transient ownership and live activity per workflow rather than once per
repository. Completing SCOPE places that plan in a durable ready-for-BUILD
queue and releases the user to select or scope another issue without cancelling,
resuming, or changing the completed scope.

Provide one build scheduler over the ready queue. Sequential mode runs one
workflow at a time. Parallel mode starts at most the configured batch size and
starts another ready workflow when a slot becomes available. Each worker uses
its plan's branch and an isolated working directory or worktree so concurrent
file and Git operations cannot share a mutable checkout.

A worker owns the complete post-scope path, including BUILD corrections,
verification, immutable review, ACCEPT, and local integration. The worker stops
only after successful completion, an explicit return to SCOPE, a required human
decision, or an unrecoverable workflow-local failure. The scheduler records that
outcome and continues every other worker.

Serialize operations that mutate the shared default branch. Before integration,
revalidate each candidate against the latest default revision. Return only the
stale or conflicting workflow to the appropriate recovery state; do not cancel
the batch.

Show one aggregate dashboard with queued, running, awaiting-human, returned-to-
SCOPE, failed, and completed workflows. Keep each workflow's logs, cancellation,
retry budget, and progress independent. Support cancelling one worker or the
whole batch as distinct actions.

## Trade-offs

- Parallel builds consume more model, CPU, memory, and filesystem resources.
- Concurrent candidates can conflict even when each passes against its original
  base, so integration remains serialized and may trigger workflow-local
  revalidation.
- Per-workflow ownership and process recovery replace the simpler repository-wide
  owner cache.
- Batched human decisions need clear workflow identity to avoid approving the
  wrong candidate.

## Open questions

- Where should the default concurrency limit live, and may a command override it?
- Should dependency links constrain ready-queue ordering before workers start?
- Should a worker awaiting human input occupy a concurrency slot?
- How long should completed and failed batch state remain visible?
