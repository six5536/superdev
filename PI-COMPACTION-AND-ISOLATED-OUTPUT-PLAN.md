# Pi Workflow Context Compaction and Isolated Output Plan

**Status:** Proposed standalone engineering plan.

## Purpose

Make the in-progress Pi workflow extension safe under long-running agent contexts by solving two related problems together:

1. Isolated roles can currently return almost 1 MB of text directly into the parent model context.
2. Pi's automatic compaction remains generic and can omit workflow facts needed to continue safely.

This document is intentionally a root-level engineering plan, like `SOKF-TOOLING-PLAN.md` and `SCOPE-BUILD-ACCEPT-IMPLEMENTATION-PLAN.md`. It is outside the unfinished canonical SCOPE → BUILD → ACCEPT workflow. Implementing it must not start, advance, approve, accept, abandon, or otherwise mutate `plan-059-scope-build-accept-workflow` or transient workflow ownership.

## Outcomes

When this work is complete:

- Every isolated-role result has separate transport and model-context bounds.
- Truncation is explicit, UTF-8 safe, line aware, and covered by tests.
- A truncated or malformed reviewer result can never be interpreted as `CLEAN`.
- Full diagnostics remain available through a local artifact when model-visible output is shortened.
- Parent and child Pi processes use role-aware compaction summaries.
- Compaction preserves canonical workflow pointers and safety state without making the summary authoritative.
- A compacted process reconstructs durable state from the plan, Git, and `superdev workflow status --json` before continuing.
- The project extension and its packaged copy have one tested implementation with no silent drift.

## Non-goals

- Completing the SCOPE → BUILD → ACCEPT workflow or any unfinished block in plan 059.
- Changing Rust workflow transitions, plan schemas, acceptance policy, or Git integration.
- Treating Pi session summaries, child logs, or extension entries as durable workflow authority.
- Preserving unlimited child transcripts in the parent session.
- Hiding child failure, malformed output, omitted review findings, or compaction failure behind a successful result.
- Introducing a provider-specific compaction model or a new model-selection configuration surface.
- Parallel modifying children.

## Fixed design decisions

### Context has three layers

Treat context control as three separate concerns:

1. **Child process isolation** keeps role exploration and tool traffic out of the parent context.
2. **Bounded tool results** control what crosses from a child into the parent.
3. **Compaction** summarizes history within a process when its own context grows.

Compaction is not a substitute for bounded tool output. Output is reduced before it enters the parent context.

### Independent stream, retained-memory, artifact, and context bounds

Keep distinct limits for:

- **Stream parsing:** parse child stdout incrementally as newline-delimited JSON with a bounded partial-line/event buffer. Count observed encoded bytes for diagnostics, but do not retain or reject an otherwise healthy long-running child merely because its disposable JSON event stream cumulatively exceeds the current 1 MB capture limit. Apply a separate generous lifetime/event-count abuse ceiling only if measurements from the longest supported BUILD fixture justify it.
- **Retained transport memory:** retain only the latest complete assistant result, usage aggregates, parser metadata, and a bounded stderr tail. Bound each retained value and the aggregate encoded bytes rather than JavaScript string length.
- **Full-result artifact:** cap the complete extracted terminal answer separately. A result beyond that cap fails closed rather than consuming unbounded disk or memory.
- **Model-visible result:** Pi-compatible limits of at most 50 KB and 2,000 lines, whichever is reached first, with smaller role-specific soft targets where practical. The bound applies to the final composed text across every returned `content` block, including metadata and truncation notices.

Use Pi's exported truncation utilities rather than character slicing for model-visible text. Wrap them with tests against the installed Pi version; do not assume their UTF-8 behavior without verification. Limits are measured on bytes and lines, preserve valid UTF-8, and report original and retained sizes.

### Full output and terminal meaning

Persist an over-limit final child answer to a securely created local temporary artifact before returning a shortened result. Return only:

- terminal state;
- concise summary;
- canonical record or revision references;
- truncation metadata;
- artifact path when one exists.

Raw JSON event-stream stdout is transport material, not a useful parent artifact. Extract the final assistant result first; persist that result only when it exceeds the model-visible bound. Use restrictive file permissions, avoid repository-tracked paths, and remove stale artifacts through a bounded cleanup policy.

Reviewer safety is stricter than ordinary truncation:

- A complete, valid `CLEAN` result may pass.
- Findings must retain their severity headings and exact references.
- If a review result cannot fit the bound without dropping findings, return an explicit `overflow` or failed result with the artifact reference; never return `CLEAN`, and never allow ACCEPT to proceed from it.
- Malformed, empty, terminated, or truncated-before-classification reviewer output is a failed review.

BUILD and SCOPE results should primarily reference durable issue, plan, branch, block, evidence, and revision state. Large narrative detail belongs in canonical records or the local artifact, not the parent tool result.

### Compaction is role-aware but not authoritative

Register `session_before_compact` before the extension returns for `SUPERDEV_CHILD_ROLE`, so the hook is available to both the parent and isolated children. Override default compaction only when a child role exists or canonical status shows that the current parent Pi session owns the active workflow; another session's ownership and unrelated parent-session compaction must remain Pi's default.

A workflow-aware summary preserves only the facts needed to recover:

- process role and attended/unattended capability;
- issue and plan IDs;
- expected plan revision and workflow phase;
- work/default branches;
- current and completed blocks;
- latest checkpoint or candidate revision;
- verification and review state;
- retries, stalls, discoveries, and deferred human decisions;
- current safety constraints and next legal action;
- files read or modified, using Pi's cumulative file-operation data.

The summary explicitly states that these are cached pointers. Before mutation or transition, the agent must reread the canonical plan and invoke `superdev workflow status --json`. The extension must not write workflow cache or machine-owned plan fields while preparing a summary.

Use Pi's active model for custom summarization and account for its usage. Include `previousSummary`, `messagesToSummarize`, split-turn prefixes, and Pi's file-operation details. Preserve Pi's prepared `firstKeptEntryId` and recent-message tail rather than inventing a more aggressive cut point.

Do not rely on the summarizer to reproduce canonical fields. Build a deterministic, size-bounded recovery checkpoint from an allowlist of parsed workflow-status fields, append the model-generated conversational summary, and validate the combined result before returning it. Store versioned custom compaction `details` containing the checkpoint and cumulative read/modified file lists. Because Pi deliberately does not fold extension-provided compaction details into later default file tracking, reconstruct prior custom details from `branchEntries` on every custom compaction.

If canonical status cannot be read, the active model is unavailable, the summary is empty or length-limited, or custom summarization fails, return no override and let Pi perform its default compaction. Do not cancel threshold or overflow compaction. Emit bounded telemetry/UI feedback without leaking the summary or full child output.

### Trigger policy

Retain Pi's configured automatic threshold as the universal safety net. Add proactive compaction only at an observed durable boundary and only when context use justifies it:

- after a SCOPE proposal/review cycle has been recorded;
- after a BUILD block checkpoint is durably visible;
- after final verification/review state is durably visible;
- at a parent phase handoff.

Do not compact in the middle of an uncheckpointed edit/test cycle merely to meet an arbitrary role quota. Review children should normally stay bounded enough not to compact; if their input is too large, bound or partition the review input rather than rely on lossy summarization for coverage.

The extension may use `ctx.getContextUsage()` and `ctx.compact({ customInstructions })` for these safe-point triggers. Project-level `reserveTokens` and `keepRecentTokens` remain configuration defaults; do not pretend they are dynamically role-scoped Pi settings.

### One maintained extension implementation

Changes affect both:

- `.pi/extensions/superdev/`
- `pack/pi/extensions/superdev/`

Establish the packaged extension as the materialization source, or use the repository's existing declared ownership if it differs. Add a parity/drift assertion so future changes cannot update only one copy. Do not overwrite unrelated local modifications while reconciling the copies.

## Proposed implementation

### Output envelope

Introduce a small internal result shape independent of Pi's raw JSON event stream:

```ts
type IsolatedRoleState =
  | "completed"
  | "clean"
  | "findings"
  | "return-to-scope"
  | "stalled"
  | "overflow"
  | "cancelled"
  | "failed";

interface IsolatedRoleResult {
  role: "scope" | "requirements-review" | "build" | "code-review" | "accept" | "file";
  state: IsolatedRoleState;
  summary: string;
  references: string[];
  truncated: boolean;
  originalBytes: number;
  originalLines: number;
  artifact?: string;
}
```

The exact parser may initially adapt the current terminal Markdown conventions, but validation and bounding happen in pure functions before constructing the Pi tool result. Put the envelope in tool-result `details`; render a concise textual form in `content`.

Longer term, a child terminating structured tool may produce this shape directly. That migration is compatible with this plan but is not required to complete it.

### Compaction module

Extract compaction concerns from `index.ts` into a focused module where practical:

- role-specific summary instructions;
- allowlisted canonical workflow snapshot retrieval and deterministic checkpoint rendering;
- message serialization, split-turn handling, and summary generation;
- reconstruction of cumulative custom compaction details from `branchEntries`;
- fallback behavior;
- safe-point trigger decision;
- compaction telemetry details.

Keep the summary prompt concise and role-specific. For example, BUILD emphasizes the current block, checkpoint, commands/evidence, discoveries, and candidate state; review roles emphasize review scope and unreported findings; the parent emphasizes phase, ownership, child terminal state, and next legal transition.

### Artifact handling

Create artifacts outside tracked project content, preferably under an OS temporary directory namespaced by Pi session and role. Requirements:

- random/non-user-controlled filename;
- owner-only permissions where supported;
- no model text interpolated into a path or shell command;
- atomic write;
- artifact path included only after a successful write;
- bounded retention by age and count;
- ownership by the parent extension process, which creates the artifact only after the child exits;
- cleanup on parent session shutdown where safe, while retaining a failed result long enough for immediate diagnosis (child shutdown must not delete an artifact before the parent can return it);
- explicit treatment as ephemeral diagnostics: a resumed session may find that an old artifact has expired and must never depend on it for workflow recovery.

If artifact creation fails, return a bounded failed/overflow result that states full output was unavailable. Never fall back to injecting the full text.

## Work breakdown

### Block 1 — Characterize and lock the boundaries

- Add pure constants and helpers for partial-event bytes, retained-memory bytes, stderr-tail bytes, full-result/artifact bytes, model-visible bytes, line limits, and role soft targets.
- Add fixtures for ASCII, multibyte UTF-8, very long lines, many lines, empty output, malformed and partial JSON event lines, interleaved stdout/stderr floods, and multiple assistant messages.
- Define terminal classification rules for each role, especially `CLEAN`, findings, overflow, cancellation, and failure.
- Confirm current Pi truncation and compaction APIs against the installed version.

Evidence:

- Unit tests prove byte/line bounds and valid UTF-8.
- A reviewer fixture with findings beyond the retained boundary cannot become `CLEAN`.
- A synthetic event stream larger than 1 MB with a small valid terminal result succeeds with bounded retained memory.
- An oversized single event, stderr tail, or full extracted answer fails or truncates according to its own documented bound.

### Block 2 — Bound isolated-role results and preserve diagnostics

- Incrementally parse newline-delimited JSON from stdout, retaining only a bounded partial-line buffer and the latest complete assistant result; do not retain the whole event stream merely to parse it after exit.
- Track total observed stdout/stderr bytes for diagnostics while independently bounding retained parser state and the stderr tail. Replace the current 1 MB cumulative-capture termination rule; do not merely raise it.
- Classify the complete result before shortening it.
- Persist over-limit final answers as secure temporary artifacts.
- Return the bounded envelope and truncation notice through `superdev_isolated_role`, then enforce the bound once more on the final composed `content`.
- Aggregate nested child usage from non-duplicated terminal events, including compaction usage when Pi exposes it, and test that streaming/update events are not double-counted.
- Update role prompts to require concise terminal summaries and canonical references.

Evidence:

- Parent `content` never exceeds either model-visible limit.
- Full oversized output is byte-identical in its referenced artifact.
- Missing artifact writes fail closed.
- Reviewer overflow blocks clean completion.
- Child cancellation and non-zero exit remain errors with bounded diagnostics.

### Block 3 — Add workflow-aware compaction for parent and children

- Register `session_before_compact` before the child-role early return, but return no override for a parent session with no active canonical workflow.
- Retrieve a read-only canonical status snapshot through `pi.exec("superdev", ["workflow", "status", "--json"], { signal, timeout: ... })`, bound stdout/stderr before parsing its protocol envelope, and allowlist only recovery fields.
- Render the deterministic recovery checkpoint separately from the model summary so required fields cannot be omitted or invented by the summarizer.
- Generate role-aware summaries from Pi's prepared messages, previous summary, split-turn prefix, and file operations.
- Reconstruct previous extension-owned compaction details from `branchEntries`, merge cumulative file lists, and write versioned details on the new compaction.
- Preserve Pi's kept boundary and return usage accounting.
- Fall back to default compaction on unavailable model, invalid status, abort, or summarization failure.
- Ensure threshold and overflow reasons are never cancelled.

Evidence:

- Fake Pi events prove the hook registers in parent and every child role.
- Summary fixtures retain all required recovery fields; the allowlisted status checkpoint excludes unknown status fields and fixture secrets.
- Repeated custom compaction carries previous summary, prior checkpoint data, and cumulative file references even though Pi ignores `fromHook` details in its default file tracker.
- Split-turn compaction includes the early turn prefix.
- Failure and abort paths use default behavior and do not deadlock retry.

### Block 4 — Add conservative safe-point triggering and recovery

- Track only reconstructable presentation state needed to recognize a newly durable boundary.
- Use `ctx.getContextUsage()` to avoid unnecessary proactive compaction.
- Trigger `ctx.compact()` after an observed durable boundary with role-specific custom instructions.
- Observe `session_compact` and mark workflow recovery pending. While a workflow is active, use a short `context` reminder and narrowly scoped tool gates so mutation or transition cannot proceed until canonical status and the referenced plan have been reread successfully; clear the marker only after successful tool results, not tool calls.
- During recovery, permit only the exact shell-free/read-only status operation and the plan read needed to clear the gate; do not accept arbitrary Bash containing matching text as proof. Prefer an extension-owned status helper over command-string recognition.
- Do not impose this gate on unrelated non-workflow use of the parent Pi session.
- Prevent duplicate compactions for the same phase/block/revision boundary.

Evidence:

- An uncheckpointed BUILD cycle never triggers proactive compaction.
- A completed block above the threshold triggers exactly once.
- Session reload reconstructs the last trigger key without treating it as workflow authority.
- A compacted child is blocked from its next mutation until canonical status and the referenced plan are reread in the real-Pi fixture.
- An unrelated parent-session task is not blocked merely because an earlier workflow compaction occurred.

### Block 5 — Package parity, documentation, and end-to-end verification

- Apply the implementation to the maintained extension source and materialized project copy.
- Add a parity test for extension modules and prompts.
- Extend `scripts/test/fixtures/superdev-extension-smoke.ts` to verify compaction hooks as well as commands/tools.
- Add script tests that run synthetic child JSON streams through result extraction and bounding.
- Document context limits, artifacts, compaction fallback, role behavior, and operator diagnosis in the appropriate Pi/workflow documentation without claiming plan 059 is complete.

Evidence:

```text
npm run test:scripts
npm run check:docs
npm run check:validate
git diff --check
```

Also run a real Pi smoke with a deterministic/fake provider where practical:

1. Produce an oversized BUILD terminal result and prove the parent receives a bounded result plus artifact reference.
2. Produce an oversized reviewer result and prove it cannot report clean completion.
3. Force threshold compaction in a child and verify the role-aware recovery fields.
4. Resume after compaction from canonical status rather than summary-only state.
5. Verify parser/resource-bound overflow, cancellation, and summarizer failure all fail or fall back safely.

## Failure matrix

| Condition | Required result |
|---|---|
| Disposable JSON event stream cumulatively exceeds 1 MB | Continue incremental parsing with bounded retained memory |
| One event, retained parser state, artifact candidate, or abuse ceiling exceeds its explicit bound | Terminate/fail closed with bounded diagnostics |
| Final answer exceeds context bound | Persist artifact; return bounded envelope |
| Reviewer findings exceed context bound | `overflow`/failure; never `CLEAN` |
| Child emits malformed JSON lines | Ignore unrelated lines only within bound; fail if no valid terminal answer |
| Child exits non-zero with huge stderr | Bounded error with size metadata; never inject full stderr |
| Artifact write fails | Bounded failure; do not return full output |
| Automatic threshold compaction | Role-aware summary or default fallback; never cancel |
| Context overflow recovery | Preserve retry semantics and canonical pointers |
| Summary model unavailable/fails | Allow Pi default compaction |
| Workflow status unavailable | Mark snapshot unavailable; default compaction or summary without invented state |
| Summary conflicts with plan/status | Plan/status wins; reread before mutation |
| Review input itself is too large | Bound/partition input; do not claim complete review from lossy context |
| Parent and pack extension copies differ | Test failure with named paths |

## Acceptance checklist

- [ ] No isolated-role tool result can add more than 50 KB or 2,000 lines to model context.
- [ ] Stream volume, retained transport memory, full-result artifact size, and model-visible truncation are separate tested states.
- [ ] A valid long child is not killed solely because its discarded JSON event stream crosses 1 MB.
- [ ] Truncation preserves valid UTF-8 and reports exact original/retained dimensions.
- [ ] Oversized full final answers are available through secure local artifacts when artifact creation succeeds.
- [ ] Review output is classified before truncation, and truncation can never turn findings into `CLEAN`.
- [ ] Empty, malformed, cancelled, failed, and overflowed reviews cannot satisfy a review gate.
- [ ] Parent and child processes register workflow-aware compaction handling.
- [ ] Compaction preserves role, plan, phase, branch, block, evidence/review pointers, discoveries, retries, and next legal action.
- [ ] Repeated and split-turn compactions retain previous-summary and file-operation context.
- [ ] Threshold/overflow compaction falls back safely and is never cancelled by the extension.
- [ ] Agents reread canonical workflow status and plan state after compaction before mutation or transition.
- [ ] Proactive compaction occurs only at a durable observed boundary and is deduplicated.
- [ ] Compaction summaries and ephemeral artifacts are not treated as durable workflow authority or required for resume.
- [ ] `.pi/extensions/superdev/` and `pack/pi/extensions/superdev/` cannot drift silently.
- [ ] Script, documentation, validation, diff, and real-Pi smoke checks pass.

## Completion rule

This standalone plan is complete when bounded child output and role-aware compaction are implemented and independently verified without advancing the unfinished Superdev workflow. Completion must not change the phase, revision, ownership, approval, or evidence of `plan-059-scope-build-accept-workflow`; it may only improve the Pi extension, its packaged assets, tests, and directly affected documentation.
