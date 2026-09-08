# Superdev workflow UX repair plan

**Status:** Under human design review. Implementation is not approved.

## Purpose

Repair the current Pi workflow experience before continuing
`plan-077-sokf-file-tool-parity`. Perform this work directly on the current
branch after human approval. Do not invoke the Superdev SCOPE, BUILD, or ACCEPT
workflow while implementing this plan.

The repair covers SCOPE, BUILD, and ACCEPT before plan 077 resumes. One phase
invocation must remain visibly active and drive safe automatic work until the
next real human decision or terminal phase outcome. The user must not copy
warnings into later commands, discover session IDs, or relaunch a command for
each review finding.

This sweep also bounds isolated-role output. It does not implement the separate
role-aware compaction redesign in `PI-COMPACTION-AND-ISOLATED-OUTPUT-PLAN.md`.

## Grill status

Agreed decisions:

1. Repair SCOPE, BUILD, and ACCEPT before resuming plan 077.
2. One exhaustive review returns the complete finding set in one pass.
3. Present findings to the user one at a time, not all on one screen.
4. Let the main Pi agent choose question order dynamically from dependencies and
   prior answers.
5. When review contains mixed findings, collect substantive answers first, then
   apply all mechanical fixes and human answers in one correction pass followed
   by one complete re-review.
6. After chat, the main agent summarizes the proposed answer and asks for
   explicit confirmation.
7. Answers remain provisional and editable until the user submits the complete
   answer set.
8. Use one dynamically activated stateful tool named
   `superdev_workflow_questions`.
9. Use one checklist-driven isolated reviewer, not a reviewer panel.
10. Parse and validate oversized reviews internally, keep raw review output
    out of model context, and inject only the current finding.
11. Use one configurable isolated-role timeout, initially 20 minutes, with no
    automatic retry after timeout.
12. Show progress in one focused cancellable progress component and footer.
    Keep timer updates out of the transcript and show sanitized child activity.
13. Persist question queues and provisional answer revisions in Pi custom
    session entries, bound to the reviewed candidate revision.
14. Route BUILD findings outside approved scope back to SCOPE. Route ACCEPT
    findings to BUILD or SCOPE according to the required correction.
15. Return command control to normal chat while awaiting human discussion, then
    resume automatically through the question controller.
16. Populate Rust's existing transient child activity fields so status works
    across Pi instances. A narrow Rust change is allowed.
17. Bound model-visible isolated output to configurable defaults of 8 KiB or
    200 lines, whichever comes first.
18. Store oversized output in secure temporary files and let the normal bounded
    `read` tool inspect them. Do not add an artifact-reading tool.
19. Keep output bounding and artifacts in this sweep; leave workflow-aware
    compaction for its separate plan.
20. Retain artifacts for configurable defaults of 24 hours or 20 artifacts per
    session, cap each at a configurable 10 MiB, and enforce fixed safety
    ceilings.
21. Bound a persisted review queue to configurable defaults of 256 KiB and 100
    findings, with fixed safety ceilings.
22. Permit Back and Pause while questions are pending. Do not silently skip an
    unresolved dependency.
23. After proving the owning parent Pi is dead, automatically terminate its
    orphan child process group, preserve partial changes, mark the phase
    interrupted, and wait for an explicit Resume phase action before spending
    another model call.
24. Block session switching, forking, or replacement while a child runs. Require
    pause and ownership release while awaiting questions or approval, then let
    a Pi instance discover and reopen the same saved session before atomically
    reacquiring ownership.
25. Use a focused Pi custom progress component while a child runs so Esc can
    cancel it idiomatically; retain `/superdev-cancel` for external and recovery
    cancellation rather than as the primary interactive path.
26. In non-interactive modes, run machine-executable stages with structured
    updates but pause fail-closed with a bounded `human-input-required` result
    at every question, approval, recovery, or acceptance gate.
27. Let one confirmed answer resolve multiple dependent findings only when the
    user sees and confirms the explicit finding-to-answer mapping; partially
    covered findings remain pending.
28. Use Pi's immediate `killProcessTree()` behavior for Esc, timeout, and orphan
    termination; do not grant a cancelled model a grace interval for additional
    mutations.
29. Keep one BUILD implementation role in this sweep and defer idea 015's
    dependency-aware work decomposition, parallelism, and batch publication to
    its own plan.
30. After SCOPE approval, require one explicit Start BUILD action. A clean BUILD
    handoff starts ACCEPT automatically, because starting BUILD authorizes the
    implementation-through-acceptance run up to its human acceptance gate.
31. At final acceptance, offer Accept, Request changes, Discuss, and Pause.
    Clarify and summarize requested changes in chat, classify their destination,
    and require confirmation before routing them to BUILD or SCOPE.
32. Oversized artifacts contain complete final role results, validated review
    payloads, stderr, and minimal parser/process metadata—not raw Pi event JSON,
    reasoning, prompts, intermediate text, or tool payloads.
33. Persist a question queue only in its labeled Pi custom session entries.
    Resume discovers and reopens that saved session before reacquiring workflow
    ownership; never duplicate the queue into a fork or project-level cache.
34. Correlate re-review findings with deterministic semantic fingerprints. Reuse
    confirmed answers for materially unchanged findings and ask again only when
    meaning, choices, dependencies, or impact changed.
35. Treat `max_final_correction_cycles` as one BUILD-through-ACCEPT budget that
    resets only after a new SCOPE approval and explicit Start BUILD action.
36. Consume a semantic correction cycle only after both successful correction
    and a complete valid re-review. Infrastructure, provider, protocol,
    timeout, and artifact failures pause without consuming it.
37. While workflow questions are active, use a temporary read-only discussion
    tool set plus `superdev_workflow_questions`; restore the user's exact prior
    active-tool set on completion or pause.
38. On session restart, validate and visibly restore pending question state but
    do not trigger a model call until the user selects Resume questions.
39. If diagnostics exceed the artifact cap, truncate diagnostics explicitly and
    let the role continue; if the final structured result exceeds the cap, fail
    rather than trusting a truncated result.
40. Use shared `max_review_state_bytes` and `max_review_findings` settings for
    SCOPE reviews, BUILD code reviews, ACCEPT assessments, routed finding sets,
    and provisional question/answer state.
41. Replace whole-file immutable diff returns and the global cumulative diff
    budget with per-review bounded pagination and fail-closed changed-path/chunk
    coverage accounting.
42. Replace final-text JSON with one isolated-child-only typed terminating tool,
    `superdev_submit_result`, whose schema is selected from the trusted child
    role; free-form assistant text is non-authoritative.
43. Do not create a non-blocking advisory category. Every returned finding must
    identify an actionable gate violation; stylistic preferences are omitted and
    out-of-scope improvements use the normal idea process.
44. Reuse `superdev_workflow_questions` and its persistence/confirmation model
    for final-acceptance change requests, including confirmed BUILD-versus-SCOPE
    routing; do not add acceptance-specific state or another tool.
45. At a clean SCOPE human gate, offer Approve, Request changes, Discuss, Pause,
    and confirmed Abandon. Confirmed changes trigger one correction and complete
    requirements re-review before approval is offered again.
46. Keep one simplified child-only `superdev_review_diff` capability whose
    revisions are parent-bound and whose path chunks are bounded and
    coverage-tracked. Do not add an immutable-file tool; use ordinary reads plus
    before/after repository invariants for requirements review and ACCEPT.
47. Render actions as immediate TUI choices while a gate is open; after pause or
    restart, show the next action in status and accept natural-language control
    through existing typed tools. Slash commands remain optional and users never
    supply workflow/session IDs.
48. A corrupt or revision-stale queue is preserved as superseded diagnostics,
    cannot submit or reuse answers, and offers an explicit Restart exhaustive
    review action rather than launching a model call automatically.
49. On graceful Pi shutdown, kill any running child, persist interrupted or
    pending state, clear transient activity, and release workflow ownership.
    Reopening validates state and waits for explicit resume.
50. Preserve `human_acceptance_required = false`: after explicit Start BUILD,
    automatically continue through within-scope review corrections, ACCEPT
    assessment, and local integration unless scope-related or terminal failure
    requires human intervention.
51. Keep the extension entry point thin and extract internal modules for isolated
    execution/artifacts, progress UI, review state, workflow questions, and phase
    drivers; these are not new tools or protocols.
52. Optimize for the cleanest final system without backward-compatibility burden:
    redesign tools, commands, configuration, transient cache, and protocol as
    appropriate; reject or quarantine incompatible transient state and update
    all repository surfaces together while preserving Git history and committed
    canonical knowledge.
53. Discard obsolete transient workflow state. Review abandoned SCOPE edits on
    their merits, keep only correct coherent knowledge, discard flawed content,
    and commit accepted knowledge separately from the UX repair.
54. Commit this root plan as implementation authority and transfer durable
    decisions into canonical knowledge, but leave the root plan in place after
    verification until the user removes it manually.

Pending decision:

- Question 60 will continue the design review.

## Current failure

`.pi/extensions/superdev/index.ts` runs nested Pi processes through `isolated()`.
The function consumes the JSON event stream silently and has no deadline. The
`/scope`, `/build`, and `/accept` handlers await those children without an
initial notification, elapsed-time display, or stage display.

Requirements review returns `findings`, but `/scope` displays only
`reviewed.summary` and exits. The main Pi conversation loses the structured
finding list. Re-running `/scope` reviews and corrects findings serially.

`isolated()` retains up to 1,000,000 characters of stderr and can throw that
entire value into parent context after child failure. Large output needs an
out-of-context artifact with bounded on-demand reads.

Rust's `WorkflowCache` already defines `child_role`, `child_pid`,
`child_started`, and `cancelled`, but the Pi extension does not populate the
child activity fields.

The existing smoke fixture proves registration only. It does not execute phase
handlers or assert UI, review, question, output, timeout, or recovery behavior.

## Configuration

Define `[workflow]` with these defaults:

```toml
[workflow]
human_acceptance_required = true
max_stalled_block_attempts = 3
max_final_correction_cycles = 3
max_scope_review_cycles = 3
isolated_role_timeout_seconds = 1200
max_isolated_context_bytes = 8192
max_isolated_context_lines = 200
max_review_state_bytes = 262144
max_review_findings = 100
max_isolated_artifact_bytes = 10485760
max_isolated_artifacts_per_session = 20
isolated_artifact_retention_hours = 24
```

All numeric values must be positive. Omitted optional policy fields receive
these defaults, and this repository's managed manifest is rewritten to record
them explicitly.

Non-configurable safety ceilings:

- model-visible isolated output: 50 KiB and 2,000 lines;
- serialized review state: 1 MiB and 1,000 findings;
- one isolated artifact: 100 MiB;
- artifacts per session: 100;
- artifact retention: 168 hours.

Configuration may lower or raise defaults only within these ceilings.

## Required behavior

### Visible subprocess lifecycle

- `/scope`, `/build`, and `/accept` acknowledge dispatch immediately.
- A focused custom progress component shows workflow, plan, stage, elapsed time,
  latest completed stage, and the cancellation action while a child runs.
- Esc requests cancellation, matching Pi's abort convention. The component
  closes before normal chat or workflow questions resume.
- A compact footer mirrors the current stage.
- Timer updates replace UI in place and never append transcript or model
  context.
- Transcript entries are limited to command start, meaningful stage transitions,
  human questions, and final outcome.
- Parent stages include initialization, isolated role, checkpoint, review,
  correction, synchronization, verification, human gate, and integration where
  applicable.
- Sanitized child activity may show tool name and safe repository-relative
  target, such as reading one plan or reviewing an immutable diff.
- Never show prompts, contents, reasoning, search queries, shell text, secrets,
  or raw tool arguments.
- Success, cancellation, timeout, malformed result, and child failure replace
  the running display with a concise final outcome.
- Interactive cancellation terminates the process group, preserves partial
  changes, marks the phase interrupted, and offers Resume or Pause and release.
  `/superdev-cancel` remains available for external and recovery cancellation.
- Cleanup runs from `finally`; no stale timer, progress component, status, child
  registration, or dynamically enabled tool remains.
- MCP transport daemons stay quiet unless startup or health fails.

### Non-interactive behavior

In print, JSON, or RPC modes without interactive UI, continue machine-executable
stages and emit structured stage updates where supported. At any question,
approval, timeout-recovery, or acceptance gate, persist state and return a
bounded `human-input-required` result containing workflow, phase, pending action,
and resume guidance. Never invent or default a human answer. A later interactive
session may atomically acquire and resume the persisted state.

### Timeout behavior

- One configured timeout applies to every isolated model role.
- Timeout immediately terminates the complete child process group with Pi's
  `killProcessTree()` behavior and preserves partial working-tree changes.
- Esc, timeout, and orphan cleanup provide no grace interval in which a
  cancelled model could continue mutating files.
- Timeout never retries automatically.
- Present Retry, Discuss, and Cancel workflow actions. Retry continues the same
  phase without another slash command.

### Immutable review input

Keep one simplified `superdev_review_diff` capability registered only inside
immutable review children. Bind base and candidate through trusted parent state,
not model-controlled tool arguments. Omitting `path` inventories the complete
diff; `path`, `offset`, and `limit` read a bounded changed-file chunk. Bound every
result by configured model-context limits and return total size plus a
continuation offset. Do not grant arbitrary Bash merely to execute Git.

Track changed-path and required-chunk coverage per review run. Completion
metadata must account for every changed path and necessary chunk. Binary or
generated content may be excluded only with an explicit justification. Refuse a
`clean` result when required immutable-diff coverage is incomplete. Large diffs
must be inspectable incrementally rather than rejected by the current 80 KiB
per-file or 400 KiB global budgets.

Do not add a separate immutable-file tool. Immediately before requirements
review or ACCEPT assessment, verify that HEAD equals the candidate, the tracked
worktree and index are clean, and no modifying workflow child is active. Hold
the local modifying-role lock, verify the same invariants afterward, and discard
the result if they changed. A hostile external process that changes and exactly
restores files during review is outside the local workflow concurrency model.

### Exhaustive review

Use one isolated reviewer per review cycle. The reviewer must complete a fixed
checklist covering requirements, contracts, architecture, tests, documentation,
scope boundaries, and internal consistency.

The reviewer must inspect the complete supplied candidate, not fail fast, and
return every finding discoverable in that pass plus completion metadata for
each checklist section.

Reviewers do not emit a separate advisory category. Every finding must identify
an actionable violation of a requirement, contract, acceptance criterion,
project standard, correctness property, or necessary evidence gap and blocks the
gate until resolved or superseded by confirmed scope intent. Omit stylistic
preferences and speculative out-of-scope improvements; worthwhile new work uses
the normal idea process.

Every finding has:

- a stable finding ID;
- classification;
- concise summary;
- affected path, section, requirement, or criterion when known;
- explanation of why it blocks approval;
- dependency finding IDs;
- a user question for substantive SCOPE findings;
- concrete choices and one recommendation when choices exist.

Each isolated child must submit exactly one authoritative result through
`superdev_submit_result`, an isolated-child-only typed tool with `terminate:
true`. Select its TypeBox schema from trusted `SUPERDEV_CHILD_ROLE`. Reject
missing, duplicate, malformed, or role-incompatible submissions. Free-form
assistant text is non-authoritative.

Role-result validation rejects malformed, empty, contradictory, unclassified,
truncated-before-classification, or checklist-incomplete review output. No
truncated or partial result can become clean.

### Mechanical boundary

A finding is mechanical only when one deterministic correction follows from
already-approved intent:

- schema or formatting repair;
- generated link or index repair;
- stale canonical text where an approved source determines the replacement;
- missing test or evidence coverage for an already-settled requirement;
- typo, path, identifier, or reference correction.

A finding is substantive when it changes or introduces observable behavior,
scope, acceptance criteria, architecture, API choices, defaults, limits,
policy, trade-offs, or user intent. Uncertain findings are substantive.

### Stateful workflow questions

After one exhaustive SCOPE review, store the complete finding set extension-side
and persist a bounded revision-bound queue in Pi custom session entries. Preserve
provisional answer revisions. Expose only the latest answer as current.

Dynamically activate one tool:

```text
superdev_workflow_questions
```

The tool supports explicit actions:

```text
inspect
ask
propose-answer
revise
submit
pause
resume
```

- `inspect` returns bounded finding IDs, summaries, dependencies, and answer
  status.
- The main agent chooses the next eligible finding from dependencies and current
  answers. The queue is not hard-coded FIFO.
- `ask` shows one finding at a time with context, choices, recommendation, a
  custom-answer option, Back, Discuss, and Pause SCOPE.
- A direct answer becomes provisional and returns control to the main agent so
  it can adapt the next question.
- Discuss returns to normal Pi chat. The original `/scope` handler is not left
  suspended.
- Discussion mode preserves bounded read, search, graph, status, and immutable
  diff tools, activates `superdev_workflow_questions`, and disables edit, write,
  arbitrary Bash, BUILD execution, and unrelated workflow mutations. Pause is
  required before unrelated repository changes.
- Preserve and restore the user's exact prior active-tool set when discussion
  completes or pauses.
- Once discussion reaches an answer, the main agent calls `propose-answer`.
  The tool summarizes the proposed answer and requires explicit human
  confirmation before recording it provisionally.
- Rejection returns to chat. Confirmation automatically advances question
  processing.
- The user may reopen and revise any provisional answer.
- One answer may cover multiple dependent finding IDs only when the main agent
  identifies each ID, the UI shows an impact summary for each, and the user
  confirms the complete mapping. Partially covered findings remain pending.
- After the last answer, show a summary with Edit, Discuss, and Submit all.
- `submit` is available only when every finding has a confirmed provisional
  answer and every dependency is resolved.
- Pause preserves the queue and answers and exposes one Resume questions action.
  No reconstructed slash command or session ID is required.
- On session start, validate a pending queue and candidate revision, restore the
  read-only tool set and controller, and show current finding and answer
  progress. Do not trigger a model turn before explicit Resume questions.
- Preserve corrupt or revision-stale queue entries as superseded diagnostics,
  disable their submission and answer reuse, explain the invalidation, and offer
  Restart exhaustive review or Pause workflow. Restart requires explicit user
  selection.

Question queues are bound to the immutable reviewed candidate revision. An
unexpected revision change invalidates submission and requires a new complete
review. Confirmed answers become canonical only through the later SCOPE
correction checkpoint.

Use Pi's session-before-switch, session-before-fork, and replacement-session
gates to prevent duplicated active queues. While a child runs, block the session
change and offer Stop and pause workflow. While awaiting questions or approval,
require Pause and release ownership. Persist and label the queue before release.
Resume discovers and reopens the same saved Pi session by workflow, plan, and
revision without exposing a session ID, then atomically reacquires workflow
ownership. Never copy an active queue into another session or fork. If the saved
queue is absent or corrupt, fail closed and offer an explicit Restart exhaustive
review action.

### Batched correction and re-review

- `/scope` performs at most `max_scope_review_cycles` automatic batched
  correction-and-review cycles after the initial review.
- A mixed review presents substantive questions before correction.
- One correction invocation receives the complete mechanical finding set and
  complete submitted human answer set.
- Publish one correction checkpoint and run one complete re-review.
- Never run correction and review once per finding.
- If re-review finds remaining or new issues, correlate them by a deterministic
  semantic fingerprint built from classification, affected requirement/location,
  and normalized issue rather than reviewer-generated IDs.
- Reuse confirmed answers for materially unchanged findings and feed them into
  the next batch automatically. Ask again only when meaning, choices,
  dependencies, or impact changed.
- Consume a correction cycle only after correction succeeds and a complete,
  valid re-review is obtained. Timeout, provider, protocol, malformed-output,
  artifact, or incomplete-review failures pause without consuming the semantic
  budget and still require explicit Retry.
- On cycle exhaustion, preserve all findings and answers and present Discuss,
  Retry cycle, and Pause SCOPE. Do not continue spending automatically.
- A clean review continues to a human gate offering Approve and start BUILD,
  Approve only, Request changes, Discuss, Pause, and Abandon workflow.
- Request changes and Discuss reuse `superdev_workflow_questions`; confirmed
  changes run one correction and complete requirements re-review before approval
  is offered again. Abandon requires explicit destructive confirmation.
- Approve and start BUILD explicitly authorizes the next phase. Approve only
  ends SCOPE without spending a BUILD model call and records Start BUILD as the
  next action.
- If an answer changes after correction but before SCOPE approval, perform one
  new correction and complete re-review. After SCOPE approval, changing intent
  requires explicit return to SCOPE.

### BUILD

- Use the shared progress presenter for implementation, synchronization,
  verification, code review, correction, and handoff.
- Code review returns one exhaustive complete finding set.
- Classify every code-review finding as `correctable-within-scope` or
  `requires-scope`.
- Pass all correctable findings to one BUILD correction cycle before one
  verification and re-review cycle.
- If any finding requires changed requirements, architecture, API, acceptance
  criteria, or approved areas, preserve the complete set, return the workflow
  to SCOPE once, and start the workflow-question controller there.
- BUILD continues until ACCEPT, return to SCOPE, configured correction
  exhaustion, cancellation, timeout, or actionable failure. One finding never
  requires another `/build` command.
- A clean BUILD handoff starts ACCEPT automatically. Selecting Start BUILD
  authorizes the implementation-through-assessment run up to the human
  acceptance gate.
- `max_final_correction_cycles` is one budget for the complete run beginning at
  Start BUILD. Code-review corrections and ACCEPT-assessment returns to BUILD
  both consume it; phase changes do not reset it. It resets only after new SCOPE
  approval and a new Start BUILD action. Exhaustion preserves findings and
  evidence and pauses for human discussion.

### ACCEPT

- Use the shared progress presenter for assessment, human decision, closure, and
  integration.
- Route implementation or evidence defects within approved scope to BUILD as
  one complete correction set.
- Route requirement, architecture, API, acceptance-criteria, or scope defects to
  SCOPE as one complete finding set.
- Start the destination phase driver automatically.
- Ask for final human acceptance only after assessment reports no findings and
  `human_acceptance_required` is true.
- When `human_acceptance_required` is false, explicit Start BUILD authorizes an
  automatic nominal path through all within-scope code-review corrections,
  ACCEPT assessment, closure, and local integration. Stop only for a
  scope-related return, configured exhaustion, cancellation, timeout,
  infrastructure/protocol failure, integration failure, or another terminal
  condition requiring human action.
- The human gate offers Accept, Request changes, Discuss, and Pause.
- Request changes and Discuss return to normal chat through
  `superdev_workflow_questions`. Represent the request as a revision-bound
  workflow decision item and reuse the same propose, revise, confirm, pause, and
  resume persistence behavior.
- The main agent clarifies and summarizes the request, classifies it as BUILD
  correction or return to SCOPE, shows affected requirements, and requires
  explicit confirmation before routing.

### Bounded isolated output and artifacts

Treat stream volume, retained transport memory, complete output, model-visible
output, and artifact storage as separate limits.

- Parse child JSON incrementally with bounded partial-line and event buffers.
- Retain only the authoritative typed role submission, parser metadata, usage,
  and a bounded stderr tail. Do not retain or return a 1 MB stderr string.
- Parse and classify a complete review before model-context truncation.
- Keep a complete valid finding set in extension/session state within configured
  queue limits. Inject only the current finding during questions or discussion.
- Apply configured model-visible byte and line limits across all returned content
  blocks, including errors and truncation notices.
- Shared review-state byte and finding limits apply to SCOPE requirements review,
  BUILD code review, ACCEPT assessment, cross-phase finding sets, and provisional
  question/answer state—not to the full Pi event stream or reviewed diff.
- Use Pi's exported UTF-8-safe truncation utilities.
- Persist oversized complete output atomically under an owner-only, session- and
  role-namespaced OS temporary directory.
- Artifact content is limited to the complete authoritative typed role result,
  complete validated review payload, stderr diagnostics, and minimal
  parser/process metadata. Never persist raw Pi JSONL, reasoning deltas,
  free-form assistant text, prompts, unrelated tool-call arguments, or tool
  results.
- Byte-identical artifact guarantees apply separately to the typed role result and
  stderr streams, not to the raw transport stream.
- Return only terminal state, concise summary, dimensions, canonical references,
  truncation metadata, and the artifact path.
- Let the normal `read` tool inspect artifacts with `offset` and `limit`.
- Enforce configured artifact size, count, and age plus fixed hard ceilings.
- If stderr diagnostics exceed the artifact cap, stop persisting further stderr,
  count discarded bytes, mark the artifact incomplete, and let the role finish.
  If a final role result or review payload exceeds the artifact cap, terminate
  and fail with `isolated-output-overflow`; never trust a truncated result.
- Remove expired artifacts on startup and shutdown.
- Artifacts are ephemeral diagnostics and never workflow authority.
- If complete review parsing, queue persistence, or artifact creation fails, the
  review fails closed and cannot satisfy a gate.

### Activity and status

Populate Rust's existing transient child role, PID, and start fields through
narrow session- and revision-bound child-start and child-finish operations.
Record enough process identity to detect PID reuse. Clear activity in `finally`.

Actions shown inside an open custom component or human gate are immediate TUI
selections. After Later, pause, or restart, status names the next action. The
user may express it naturally and the main agent invokes the existing typed
workflow-control or question-controller tool; slash commands remain optional
shortcuts. Users never provide plan or session IDs.

`/superdev-status` distinguishes:

- canonical phase;
- running child role and elapsed time;
- owning-instance UI stage when local;
- awaiting questions;
- awaiting approval;
- paused after timeout, findings, or cancellation;
- stale activity;
- next available action.

Cross-instance status uses Rust transient state. Rich sanitized tool activity
remains local to the owning Pi instance.

On graceful `session_shutdown`, kill any running child through Pi's tracked
process-tree cleanup, record interrupted state, preserve partial changes,
persist pending questions, provisional answers, gate state, counters, and next
action, clear transient child activity, and release workflow ownership.
Reopening the saved session validates state and waits for explicit resume.

Track parent and child process identities with start times so PID reuse cannot
prove liveness incorrectly. If the parent remains alive, another instance must
not interfere. If the parent is dead but its child remains alive, terminate the
orphan process group automatically, preserve partial working-tree changes,
clear stale activity, mark the phase interrupted, and visibly offer one Resume
phase action. Do not relaunch the isolated role until the user selects Resume.

## Preflight

1. Confirm the workflow has no live child, then discard obsolete transient
   ownership, cache, and recovery state without adding compatibility machinery.
2. Inspect the uncommitted `idea-012`, issue 077, and plan 077 changes left by
   the cancelled SCOPE run directly against issue 077's intent.
3. Keep only correct coherent knowledge, discard speculative, contradictory, or
   incomplete content, validate it, and commit accepted content in a dedicated
   knowledge-only commit. Do not absorb the root plan or those files into UX
   implementation commits.
4. Confirm the working tree is clean and canonical plan 077 remains paused in
   SCOPE.

## Implementation

1. Refactor the extension into a thin entry point with focused internal modules
   for isolated execution/artifacts, progress UI, review schemas/state,
   persisted workflow questions, and phase drivers. Keep these internal rather
   than adding public tools or protocols.
2. Add workflow configuration fields, validation, defaults, serialization,
   generated contract updates, and status exposure.
3. Add narrow Rust transient child-activity operations over the existing cache
   fields, plus process identity needed for stale detection.
4. Add a shared Pi progress controller for notifications, focused component and
   footer state, elapsed time, sanitized activity, deadline abort, stage
   transitions, and cleanup.
5. Refactor `isolated()` into bounded incremental parsing, typed terminating
   result capture, structured review validation, timeout handling, artifact
   persistence, progress callbacks, and transient child registration.
6. Introduce typed exhaustive review/checklist/finding structures and update
   requirements-review and code-review prompts.
7. Implement revision-bound question queue reconstruction through Pi custom
   session entries.
8. Register and dynamically activate `superdev_workflow_questions` with the
   agreed state-machine actions and explicit confirmation behavior.
9. Refactor `/scope` into batched review, dynamic questioning, correction, and
   re-review cycles.
10. Apply complete-finding batching and progress to `/build` and route
    out-of-scope findings into the SCOPE question flow.
11. Apply progress and defect routing to `/accept`.
12. Extend `/superdev-status` with canonical plus transient activity and one next
    action.
13. Update `.pi/extensions/superdev/` and
    `pack/pi/extensions/superdev/` identically, including prompt copies. Run
    synchronization and update `.superdev/lock.toml`.
14. Update idea 014 and all affected configuration, workflow, component,
    testing, error-handling, and public documentation.

## Tests

Add focused pure seams and a fixture that executes registered handlers against
fake Pi, UI, clock, process, model-role, artifact, question-state, and workflow
service implementations.

Tests must prove:

- each phase command shows immediate progress;
- elapsed time and parent/sanitized-child stages update deterministically;
- start, transition, success, error, cancellation, and timeout produce bounded
  transcript output and clean all UI/timers/state;
- configured timeout kills the child process group and never auto-retries;
- activity registration and cleanup are session/revision bound;
- stale/PID-reused activity cannot masquerade as a live child;
- one reviewer completes every checklist section and returns all findings;
- immutable diff inventory and path/chunk pagination handle a 1 MiB diff without
  one oversized tool result, cross-review budget leakage, or incomplete clean;
- missing, duplicate, malformed, or role-incompatible terminating submissions
  fail, and free-form final text cannot satisfy a role gate;
- fail-fast, malformed, empty, contradictory, unclassified, incomplete,
  truncated-before-classification, and oversized-unpersistable reviews fail;
- three substantive findings create one persisted queue and are asked one at a
  time under dynamic dependency-aware model control;
- direct and discussed answers require the correct confirmation path;
- Back, revise, Pause, resume, and final answer-summary behavior preserve state;
- changed candidate revisions invalidate stale queues;
- three mechanical findings and a mixed set produce one correction invocation;
- all submitted answers and mechanical findings reach one correction, checkpoint,
  and complete re-review;
- cycle counting occurs per batch, not per finding;
- a clean review reaches SCOPE approval;
- code review batches all correctable findings and returns scope-changing
  findings to SCOPE once;
- ACCEPT routes findings to BUILD or SCOPE and asks acceptance only when clean;
- with human acceptance disabled, Start BUILD runs nominally through within-scope
  corrections, ACCEPT assessment, and local integration without another prompt;
- model-visible success and error output obey configured and fixed byte/line
  ceilings with valid UTF-8;
- a child stderr flood cannot enter model context;
- uncapped final results and stderr are byte-identical in their secure artifact
  streams and readable in bounded chunks with normal `read`;
- stderr beyond the artifact cap is marked incomplete without failing successful
  work, while an over-cap final structured result fails closed;
- artifact failure fails closed, and age/count/size cleanup obeys configured and
  fixed ceilings;
- an oversized complete review remains fully classified and question-accessible
  without placing raw output in context;
- `/superdev-status` distinguishes canonical phase, live child, awaiting user,
  timeout, stale activity, and next action;
- live and pack extension and prompt copies remain byte-identical.

## Verification

Run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo test --doc --workspace
node --check .pi/extensions/superdev/index.ts
node --test scripts/test/sokf-pi-adapter.test.mjs
npm run test:scripts
cargo run -- sync
cargo run -- sync --dry-run
npm run check:blueprint
npm run check:validate
npm run check:docs
git diff --check
```

Manual Pi verification:

1. Start each phase and confirm immediate plan, role, elapsed time, sanitized
   activity, and cancellation guidance.
2. Force mixed mechanical and substantive SCOPE findings. Confirm one exhaustive
   result, dynamically ordered one-at-a-time questions, optional discussion,
   explicit answer confirmation, final editable summary, one correction, and
   one complete re-review.
3. Restart Pi during a pending question and confirm queue and provisional-answer
   recovery without a session ID.
4. Force multiple BUILD review findings and confirm one correction cycle or one
   return to SCOPE.
5. Force ACCEPT defects and confirm correct automatic phase routing.
6. Produce oversized normal output and stderr. Confirm bounded context, a secure
   artifact reference, and bounded normal `read` access.
7. Cancel and time out children. Confirm retained partial work, no automatic
   retry, correct actions, and no stale UI or child state.

## Boundaries

- Complete SCOPE, BUILD, and ACCEPT UX before resuming plan 077.
- Do not implement idea 015's dependency-aware BUILD decomposition,
  parallelism, batch publication, or multi-workflow scheduling.
- Do not implement workflow-aware compaction in this sweep.
- Do not expose child reasoning, raw event JSON, unbounded output, or secrets.
- Do not make Pi session entries or temporary artifacts canonical authority.
- Do not weaken Rust-owned Git publication, human authority, immutable review,
  retry, phase, or acceptance gates.
- Permit focused Rust workflow protocol and transient-cache redesign where it
  produces the cleanest final authority, recovery, and status model. Bump the
  protocol when semantics materially change; do not add legacy shims without a
  current design need.
- Do not add another file-tool protocol.

## Root-plan lifecycle and approval gate

Commit this root plan as the approved implementation authority before coding.
Use it throughout implementation and verification, and transfer durable
architecture, configuration, contracts, UX behavior, and testing decisions into
canonical SOKF knowledge. Leave this root plan in place after verification; the
user will remove it manually.

Continue grilling from the pending question recorded in Grill status.
Implementation starts only after every design branch is resolved and the user
approves this root plan. After approval, implement directly on the current
branch without invoking the Superdev workflow. Report verification evidence and
commit the repair without merging, pushing, deleting branches, rewriting
existing commits, or absorbing unrelated changes.
