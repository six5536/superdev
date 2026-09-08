---
type: Idea
id: idea-014-superdev-subprocesses-show-progress-and-control
title: Superdev subprocess workflows stay visible and resumable
description: Make Superdev expose subprocess progress, fix mechanical findings quickly, and present each complete substantive review as one user questionnaire before correction and re-review.
status: draft
---

# Idea: Superdev subprocess workflows stay visible and resumable

Make one workflow command carry work from its current durable state to the next
human decision. Superdev must keep subprocess activity visible, fix mechanical
review findings quickly, and return substantive findings to the main Pi
conversation for the end-user to resolve. Recovery must not ask the user to
copy findings, compare session IDs, cancel stale ownership, or reconstruct a
sequence of internal commands.

## Motivation

`/scope` currently clears the editor and silently awaits an isolated Pi child.
The child can work for several minutes without visible progress. When a
requirements review returns a finding, the command stops and leaves SCOPE
active. Continuing can require the user to identify the owning Pi session,
repeat the finding as `/scope` input, or run a cancel-resume-command sequence.

A lone notification such as `Warning: Post-persistence abort handling lacks
complete contract and test coverage.` neither explains the gap nor tells the
user what decision or action comes next. The finding also disappears from the
main agent's working conversation, leaving the user to carry it into another
command. Returning one finding, correcting it, and rerunning review only to
reveal the next finding turns one review into a slow serial guessing loop.

That interface exposes orchestration mechanics instead of the next meaningful
user decision. The same problems apply to correction cycles, BUILD review,
ACCEPT assessment, and interrupted nested processes.

## Sketch

Treat `/scope`, `/build`, and `/accept` as resumable phase drivers. Each driver
reconstructs canonical state and performs safe automatic steps until a human
decision, exhausted semantic retry budget, or actionable failure.

Each isolated role ends through one typed terminating `superdev_submit_result`
tool. One checklist-driven reviewer inspects the complete candidate without
failing fast and returns every actionable finding together. Code review reads a
parent-bound immutable diff through bounded path pagination and cannot report
clean until every required path is covered. Reviews have no advisory category.

SCOPE divides findings at a strict intent boundary: only deterministic repairs
derived from settled intent are mechanical; uncertainty is substantive. Mixed
findings and confirmed answers enter one correction pass followed by one
complete re-review. Semantic fingerprints reuse answers for materially
unchanged findings, and configured cycle budgets count only a successful
correction plus complete valid re-review.

A revision-bound `superdev_workflow_questions` state machine persists findings
and provisional answer revisions in non-context Pi session entries. The main Pi
agent selects the next dependency-eligible question dynamically, presents one
question at a time, supports normal read-only discussion, summarizes a proposed
answer, and requires explicit confirmation. The user may revise answers before
Submit all. One confirmed answer may cover several findings only when the UI
shows and confirms the mapping.

A focused custom progress component and footer show plan, stage, elapsed time,
and sanitized tool/path activity. Esc immediately kills the detached process
group, preserving partial files. Configured deadlines never retry automatically.
Rust transient state records parent and child process identity for cross-instance
status. Graceful shutdown releases ownership; a new Pi kills only a proven
orphan, marks work interrupted, and waits for explicit resume.

Isolated JSON events are parsed incrementally. Parent-model output is bounded by
configured line and byte limits; full final results and stderr diagnostics go to
owner-only temporary artifacts readable through ordinary bounded `read`. Raw
Pi events, reasoning, prompts, and tool payloads are not persisted. Artifact,
review-state, finding-count, timeout, retention, and SCOPE-cycle defaults live in
`[workflow]` under fixed safety ceilings.

A clean BUILD starts ACCEPT automatically. ACCEPT findings route to BUILD or
SCOPE as one set. When final human acceptance is disabled, explicit Start BUILD
authorizes the nominal automatic path through review corrections, assessment,
and local integration. Human questions, scope changes, timeouts, malformed
results, configured exhaustion, and terminal failures still pause fail-closed.

## Trade-offs

- Complete reviews take longer than fail-fast reviews, but avoid repeated model
  launches and repeated user interruptions for findings discoverable together.
- Mechanical correction batches spend more model time before returning control,
  so each phase needs a small explicit retry budget and a narrow classifier.
- Misclassifying a substantive finding as mechanical could change intent
  without approval, so uncertain findings must return to the end-user.
- Safe ownership recovery requires liveness and identity checks rather than an
  unconditional takeover.
- Durable progress entries improve diagnosis but can add transcript noise; the
  default view must remain compact.

## Open questions

- None. `SUPERDEV-WORKFLOW-UX-PLAN.md` settles progress, cancellation, review,
  question, recovery, output, timeout, artifact, routing, and retry behavior.
