# Workflow redesign 2 — implementation plan

Status: implementation authorised. The block-1 SDK proof passes under the
accepted interactive-input trust boundary. Pi remains unchanged. Block 2 has a
connected Rust service and CLI with local records, separate approvals,
recoverable publication/startup, and migration tests. Block 3 replaced the v2
adapter with a human-led SCOPE controller, proven against real Pi and real Git.
Block 4 added the persistent worker runtime, its controller, and the durable
execution facts it needs; SCOPE handoff now starts BUILD instead of refusing.
Block 5 completed BUILD, review, ACCEPT, human-only abandonment, and the
instruction rewrite. Block 6 reconciled knowledge, schemas, contracts, the ADR
successor, configuration, and documentation. All six blocks are implemented and
the whole workspace passes. The remaining gap is the human-guided rehearsal in a
disposable repository, which only a human can run.
No existing workflow phase was invoked. Per the user's later instruction, knowledge and schema
reconciliation belongs to the final phase; automatic generated includes may
still be refreshed by the repair hook.

Requirements: [WORKFLOW_REDESIGN_2.md](WORKFLOW_REDESIGN_2.md). That document owns
the agreed behaviour. This plan adds technical choices, affected surfaces, work
order, and verification. Do not copy the requirements into another issue or
plan to make this implementation run through the workflow it replaces.

## Technical design

Use the existing pinned Pi interface without rewriting, patching, or replacing
Pi. See Current evidence for the accepted interactive-input trust boundary.

### Local state

- Store a versioned JSON record per workflow under the Git-ignored
  `.superdev/workflows/` directory. Use an internal stable workflow ID so state
  can exist before a plan and cannot transfer when a document is copied.
- Record identity, default/work branch, phase, SCOPE step, completed/skipped
  steps, pending discussion, document approvals, execution mode, worker session
  reference, checkpoint, candidate evidence, and consumed retry budgets.
  Keep optional fields absent until they apply. No permission ledger.
- Keep transient process ownership and locking in the existing workflow cache.
  The durable record owns progress and approvals; the cache owns only the live
  claim. Deleting cache does not mark work complete or delete approved scope.
- Keep records local to the checkout, including across branch switches. A
  different clone or linked worktree does not inherit their approval. Detect
  missing local state and offer human reconstruction, never infer permission
  from document lifecycle, Git history, or old approval prose.
- Rust owns typed reads/writes and transitions under the existing repository
  lock. Use atomic file replacement and explicit errors for invalid versions
  or corrupt state. Agents supply progress and judgements; Rust does not parse
  plan prose or decide whether an interview/review was good enough.

### Approval and publication

- Bind approval to canonical document IDs, exact content hashes, and the human
  input that authorised it. A plan approval also names its issue revision.
  Store approval only in local state, not in document frontmatter or prose.
- Use one pending human action in the adapter. An ask-tool selection or an
  unambiguous interactive chat response authorises that action without another
  dialog. Ambiguous chat returns to discussion. Messages submitted through
  extension or worker channels and model-supplied approval flags do not confer
  human authority. Do not assume arbitrary RPC input is human-authenticated.
- Trust Pi's existing interactive-input path, including installed input
  transformers. Use the interactive event's text and source, bound to the named
  action and current document revision. This trusts transformers to preserve
  approval intent; it does not certify original pre-transform text. Do not
  require a new Pi interface or a duplicate confirmation dialog.
- Keep grouped permission in the current conversation only. On resume, ask for
  fresh permission for the next step; retain valid document approvals.
- Commit only the approved document and its required generated index changes,
  then make its approval available for later execution. Reuse the current
  knowledge publication and Git refusal helpers, not an unrestricted commit
  of the whole worktree.
- Coordinate commit and local-state updates with a small pending-publication
  field in the same record. Recovery checks the exact expected commit/content
  before completing publication; failure never means approval succeeded.
  Do not pretend a Git commit and a JSON write form one atomic operation.
- Detect changed documents by hash. The main agent reviews the actual diff to
  classify substantive versus formatting-only changes. For a confirmed
  formatting-only edit, retain the original human approval and record the new
  compatible content hash. Substantive changes clear affected approvals as
  specified by the redesign. If classification is uncertain, ask the human;
  do not use whitespace stripping as a semantic equivalence test.
- An unexplained external edit suspends use of the approval until its diff is
  assessed. Copied/new identities always require their own approval.

### Session execution

- Prefer one persistent subprocess hosting the pinned Pi SDK `AgentSession`,
  controlled by the extension. Keep its logical session identity across BUILD,
  review, ACCEPT, and restart. Do not spawn an agent per stage or work block.
- Use a small typed controller/worker channel for status, questions, replies,
  pause, and completion. Return control to the main conversation while the
  worker executes. The controller and worker never write the checkout at the
  same time; unrelated editing requires pausing the worker first.
- Checkpoint into local state before resetting context. Keep the worker's Pi
  transcript private and local; it is recovery/debug data, not approval or
  progress authority. Retain it for an unfinished workflow and make terminal
  cleanup explicit. Do not commit transcripts, secrets, or UI capabilities.
- Rebuild each stage from its skill/checklist and the required state/evidence.
  BUILD continuation may read unfinished-work notes; final review and ACCEPT
  do not inherit the implementation conversation or its verdicts.
- Reset only at a settled tool boundary. Use the same session, not `/new` or a
  fork. Choose the supported SDK mechanism after inspecting actual model
  requests and reopening the saved session in block 1.
- Current-session execution uses the same state and checkpoints. Apply special
  context reset only if the public extension API safely supports it; otherwise
  disclose the agreed fallback. Do not mutate a private session manager behind
  the host or assume command-only APIs are available inside tools.
- Keep existing command safety and service snapshot protection. Change tool
  access with the stage; review and acceptance assessment are read-only.
  Intent changes stop the worker and return to human SCOPE in the controller.
- Controller loss stops new work, requests a checkpoint, and allows bounded
  shutdown grace before terminating a stuck process tree. Keep the claim until
  the writer stops. On recovery, inspect partial files and uncertain command
  results before retrying. Preserve retry consumption across every restart.

### Instruction ownership

| Surface | Owns | Does not repeat |
|---|---|---|
| Unchanged `<superdev>` | Project-wide rules | Nothing is rewritten here |
| Skill discovery and tool metadata | When to invoke a skill/tool; interface meaning | Full workflow instructions |
| Extension system additions | Minimum essential tool-use guidance | Project rules or phase checklists |
| Invoked skills | Ordered workflow-specific checklists | Rules already in the effective system prompt |
| Stage context | Current identity, state, and evidence references | A second orchestration prompt |

Rewrite every bundled skill: `file`, `scope`, `build`, `accept`, `grill-me`, and
`double-check`. Audit every file under `prompts/` and every instruction string
in extension code. Delete obsolete prompts; rewrite retained ones. Use
`.pi/skills/sokf-authoring/SKILL.md` as the quality reference, not as text to
copy or a mandate to change that already-approved reference skill.

### Audited instruction inventory

Paths below are relative to `.pi/extensions/superdev/` unless stated otherwise.
This inventory assigns future ownership. This implementation has not yet
rewritten these product assets; concurrent skill edits must be preserved.

| Current surface | Disposition / owning layer |
|---|---|
| `skills/file/SKILL.md` | Rewrite the independent filing checklist; refer to SOKF authoring and inherited preservation rules |
| `skills/scope/SKILL.md` | Replace early branch bootstrap and child-review loops with the human-led SCOPE checklist |
| `skills/build/SKILL.md` | Replace delegation-only instructions with execution, checkpoint, review, and correction steps |
| `skills/accept/SKILL.md` | Replace isolated-assessor delegation with assessment, routing, and configured closure steps |
| `skills/grill-me/SKILL.md` | Rewrite as a one-question interview checklist; SCOPE composes it rather than copying it |
| `skills/double-check/SKILL.md` | Rewrite as an evidence-based review checklist; distinguish review from permission to correct |
| Skill frontmatter descriptions and `allowed-tools` | Keep discovery concise and align stage access with the delivered tools |
| `prompts/scope.md`, `prompts/requirements-review.md` | Remove obsolete child prompts; SCOPE and double-check own these actions |
| `prompts/build.md` | Move retained execution rules to BUILD; remove isolated-role and plan-progress instructions |
| `prompts/code-review.md` | Retain a stage-specific review checklist, loaded only at review; remove duplicated result-protocol prose |
| `prompts/accept.md` | Move assessment rules to ACCEPT; remove isolated-role and repeated result-protocol prose |
| `prompts/orchestrator.md` | Remove duplicated orchestration; route to the selected phase skill |
| `index.ts` `before_agent_start` | Inject state data only; do not repeat workflow rules in the system prompt |
| `index.ts` terminal-repair message and submission-tool metadata | Remove obsolete isolated-result repair instructions with the old runtime |
| `index.ts` review-diff/build-exec tool metadata | Retain only interface meaning and bounded-input/output guidance |
| `lib/phases.ts` task, correction, routing, and recovery strings | Move process instructions to skills; pass stage state and evidence references as data |
| `lib/process.ts` role-prompt assembly | Replace with the single-worker stage-context loader; no second copy of checklists |
| `lib/{phase-tool,intake,questions,review}.ts` tool/schema/result guidance | Keep interface semantics in metadata; move sequencing and interview instructions to the owning skills |
| `pack/pi/extensions/superdev/` mirrors | Ship identical authored assets, not independently maintained instructions |
| `/crates/lib/superdev-core/src/agent-instructions.md`, managed copies | Unchanged; inherited project policy is not reauthored in the workflow |
| `/.pi/skills/sokf-authoring/SKILL.md` | Unchanged quality reference and owner of SOKF-specific authoring rules |

## Ordered work blocks

### 1. Prove session reset and human-input handling

**Change:** Add isolated fixtures against the pinned Pi dependency. Do not
change the product workflow yet. Inventory all effective instructions and
assign them to the ownership table above before drafting replacements.

**Paths:** New focused fixtures under `scripts/test/fixtures/`, a new
`scripts/test/superdev-workflow-runtime.test.mjs`, and the installed Pi public
SDK/type interfaces. Use a scripted provider to inspect requests without paid
model calls or network access.

**Evidence:** Preserve one worker session ID through two BUILD blocks, a
mid-block checkpoint, review, ACCEPT, and process restart. Prove that actual
review requests exclude implementation conversation and old compaction
summaries, retain required instructions, and have read-only tools. Exercise
reset failure, a pending tool call, and checkpoint failure. Check the available
current-session API separately. Prove that a real interactive reply differs
from extension-generated/worker input, and that one reply cannot authorise a
different or changed pending action.

**Verification:** `node --test scripts/test/superdev-workflow-runtime.test.mjs`.

**Stop condition:** If same-worker reset/resumption or non-duplicated human
approval cannot be supported safely, report the exact limitation and revise
this design with the human before proceeding. Documentation alone is not a
passing result. Do not silently replace the worker with successive sessions.

### 2. Introduce local workflow records and migrate document state

**Depends on:** 1.

**Change:** Implement local state and publication rules above. Separate issue
approval, plan approval, and BUILD start; support issue-only and approved but
not started SCOPE. Reserve issue and plan numbers independently under the
existing lock. Create the work branch only on explicit BUILD startup. Remove
execution phase, current progress, approval, and recovery markers as live
sources of state in plans. Retain plan lifecycle and implementation content.

**Paths:** `crates/lib/superdev-core/src/workflow/{state,cache,transition}.rs`,
new small record-storage modules, `crates/app/superdev/src/workflow_cli/`,
`crates/app/superdev/src/workflow_cli.rs`, `knowledge/schemas/plan.md`,
`pack/knowledge/schemas/plan.md`, `.gitignore`, and the generated ignore rules
in `crates/lib/superdev-core/src/pipeline.rs` (the actual owner found during
implementation; `components/item.rs` only contains a generic ignore fixture).

**Migration:** Require incompatible live writers to stop. Preserve old local
files before conversion. Treat legacy open plans and cached phase as recovery
hints, never proof of a new approval. Reconstruct progress for human inspection
without replaying completed work; request fresh approval when no trustworthy
local approval exists. Keep completed/abandoned documents as historical records
and permit their legacy fields without interpreting them as live authority.
Do not bulk-rewrite immutable history. Make conversion repeatable after failure
and refuse unknown state versions with a recovery diagnostic.

**Evidence:** Cold start without a plan; pause/restart; approval without branch
creation; branch creation only at BUILD; copy/new-ID and fresh-clone refusal;
formatting/substantive changes; ID collision; failed commit/state write; dirty
checkout; interrupted migration; cache loss and dead-claim recovery. Confirm
that ignored state is absent from commits and clones.

**Verification:** `cargo nextest run -p superdev-core workflow::`;
`cargo nextest run -p superdev --test workflow_safety --test manage`;
`npm run check:validate`.

### 3. Replace question handling and SCOPE orchestration

**Depends on:** 2.

**Change:** Give choices stable IDs, a separately identified recommendation,
and distinct answer/discuss/other-action/pause results. Implement genuine-input
approval using the mechanism proven in block 1. Use local state for resumption,
not a review-result queue. Rewrite SCOPE, grill-me, and double-check as concise
checklists. Remove SCOPE child authoring, requirements-review spawning, and the
mandatory clean-review barrier to human advancement. Preserve open findings
and record skips without claiming they passed.

**Paths:** `.pi/extensions/superdev/lib/{intake,questions,phase-tool,phases}.ts`,
`.pi/extensions/superdev/index.ts`, bundled skills, and matching
`pack/pi/extensions/superdev/` files. Split or delete obsolete modules instead
of extending the old monolithic driver. Keep source files below 800 lines.

**Evidence:** Complete issue/plan flow without a child; repeated interviews and
checks; grouped permission; fresh permission after pause; direct human
advancement; discussions that return to chat; approval commits only the right
records; wrong-branch startup asks the user to switch; handoff offers current
session, worker, or exit without implementing anything automatically.

**Verification:** Add focused `scripts/test/superdev-workflow-scope.test.mjs`;
run it and the relevant extension tests in
`scripts/test/sokf-pi-adapter.test.mjs`.

### 4. Implement the persistent worker and current-session runner

**Depends on:** 3.

**Change:** Integrate the proven runtime. Replace per-role spawning and terminal
submission repair loops, keeping useful process safety, bounded diagnostics,
progress display, and pinned service execution. Route worker questions through
the same human interface. Keep one controller and at most one worker, with
exclusive writer ownership. Save progress and counters before resets.

**Paths:** `.pi/extensions/superdev/lib/process.ts`, new focused worker/runtime
modules, `lib/{progress,output,phase-tool}.ts`, `index.ts`, and pack mirrors.
Update workflow configuration only where obsolete per-role settings must be
removed or mapped; do not add a configurable orchestration framework.

**Evidence:** Both execution modes; one worker across all stages; responsive
controller; question pause/resume; disconnect; timeout; stuck subprocess;
reconnection; partial command execution; no concurrent writers; private local
transcripts; no reset of retry budgets; public-API-only interactive fallback.

**Verification:** Extend and run
`node --test scripts/test/superdev-workflow-runtime.test.mjs` plus workflow
ownership/process tests from block 2.

### 5. Complete BUILD, review, ACCEPT, and instruction rewrite

**Depends on:** 4.

**Change:** Rewrite BUILD and ACCEPT into stage checklists using the new runner.
Keep focused block verification, product commits, final verification,
candidate-bound review, correction routing, manual cases, and configured human
acceptance. Reassess after candidate changes. Close lifecycle only after
acceptance publication succeeds. Finish the independent file-skill rewrite
without folding filing into the workflow. Remove redundant role/orchestrator
prompts and minimise all extension/tool system instructions.

**Paths:** All six bundled `skills/*/SKILL.md` files, every retained `prompts/`
file, `lib/{phases,review,phase-tool}.ts`, instruction-producing code in
`index.ts` and other extension modules, and pack mirrors.

**Evidence:** Multi-block execution and mid-block recovery; clean-context code
review; within-scope correction; return to SCOPE for changed intent; stale
candidate rejection; manual-case handling; automatic and human acceptance;
accepted/abandoned lifecycle without an implicit merge or push. Inspect actual
assembled startup, invoked-skill, review, and post-reset prompts for duplication
and missing instructions. Human review assesses checklist quality; exact-string
assertions alone do not establish it.

**Verification:** Add `scripts/test/superdev-workflow-execution.test.mjs` for
scripted end-to-end execution; run it with the scope/runtime suites. Retain
applicable existing Git safety tests. Deliberately replace tests of removed
behaviour rather than weakening unrelated tests to make them pass.

### 6. Synchronise contracts, packaging, and documentation

**Depends on:** 5. The user's later instruction overrides the original
alongside-code documentation order: reconcile knowledge, schemas, and contracts
in this final phase. The redesign spec and plan take precedence in the interim.

**Change:** Revise contract-011 and affected CLI/configuration contracts. Add a
successor to ADR-054 where its decisions change. Update plan schema guidance,
configuration, development procedure, user documentation, and changelog. Explain
local-only approval, fresh-clone behaviour, resume controls, and the context
fallback. Keep `<superdev>` and its managed copies unchanged. Do not close
related backlog records unless their own requirements are met.

**Paths:** `knowledge/contracts/`, `knowledge/adrs/`, `knowledge/configuration.md`,
`knowledge/development-procedure.md`, `knowledge/directory-structure.md`,
`knowledge/documentation.md`, relevant pack assets, `README.md`, `CONTRIBUTING.md`,
and `CHANGELOG.md`. Update generated CLI references through their source and
normal generation commands, not by hand.

**Verification:** `cargo run --quiet -- sync`; `npm run check:blueprint`;
`npm run check:validate`; `npm run check:docs`; `npm run test:scripts`;
`cargo nextest run --workspace`; then the remaining CI-equivalent checks listed
in `CONTRIBUTING.md`, including formatting, clippy, doctests, and rustdoc.

## Completion review

- Map every verification case in the redesign document to executable evidence
  or the explicit human prompt-quality review. Report gaps rather than marking
  the redesign complete from a passing subset of tests.
- Confirm that the old and new workflow engines do not coexist as competing
  authorities. Keep only the migration reader needed for old local records.
- Confirm live/pack parity, required skill discovery, local-state exclusion
  from Git, and unchanged `<superdev>` content.
- Run a human-guided SCOPE and a small multi-block BUILD/ACCEPT rehearsal in a
  disposable repository, including pause/resume. This is evidence for the
  delivered workflow, not permission to start it while authoring this plan.

## Current evidence

Command: `node --test scripts/test/superdev-workflow-runtime.test.mjs`.
Five tests pass against the unmodified pinned Pi, including characterisation
of the accepted interactive-input trust boundary. The full
`npm run test:scripts` suite passes all 54 tests. `npm run check:validate`
reports zero errors and zero warnings.

The test-only SDK harness establishes the following against Pi 0.85.1:

- One session file and ID survive simulated BUILD-block and mid-block resets,
  review, ACCEPT, and reopening in a separate OS process. Navigation to a
  non-message root anchor with no summary and a persisted label produces an
  empty restored model context before the next prompt.
- Actual scripted-provider contexts contain the selected checklist once and
  only read-only tools during assessment. Real Pi compaction creates an old
  summary which the later clean reset excludes, along with earlier verdicts.
- A pending tool prevents navigation. Its settled result remains in the saved
  history. Checkpoint-write failure, invalid navigation target, and cancelled
  navigation do not advance the live context.
- `sendUserMessage` is labelled `extension`; RPC remains `rpc`. Direct
  interactive approval can be handled without a second dialog or model call.
  The probe binds that reply to one named document revision and rejects reuse.
- Tool/event contexts do not expose tree navigation. The agreed current-session
  fallback remains necessary for tool-driven execution.

### Accepted trust boundary: existing Pi interactive input

The user rejected changes to Pi and approved trusting its interactive-input
path, including installed input transformers. This replaces the earlier
original-input proposal. The patch, patched-SDK helper, and related tests have
been removed. No special Pi build, new interface, dependency change, or extra
confirmation dialog is required. Block 2 is not blocked by Pi compatibility.

The proof retains the measured limitation: an earlier input handler can rewrite
chat text while Pi keeps `source: interactive`. Transformers are therefore
trusted to preserve approval intent. This is not protection against a faulty
or hostile transformer. The test now records the accepted boundary rather
than calling it a failed gate. No arbitrary approval-prose gate in the existing
Superdev workflow was established by this probe.

The same tests still reject extension and RPC messages even after a transform,
model-generated approval flags, approval reuse, and changed document revisions.
Untransformed discussion and ambiguous agreement do not approve anything.
Workers must submit their messages through non-human channels; the controller
alone receives interactive approvals for the named document revision.

Automatic overflow handling, worker/controller IPC, full conversational parsing,
terminal UI interaction, and the complete production workflow remain unproven.
This SDK evidence is not a claim that the redesign is complete.

### Block 2: connected local service

Implemented `workflow apply` with a typed JSON request on standard input and
`workflow status` under protocol v3. The old CLI mutation commands and their
phase-in-plan publication code are removed. The old pure transition table no
longer has an approval-to-BUILD edge. The Pi adapter is intentionally not yet
compatible; block 3 must replace its v2 calls before workflow use resumes.

The Rust service now provides issue-only reservation, independent plan
reservation, saved/repeated/skipped SCOPE actions, discussion, pause/resume,
revision-bound issue and plan approval, conservative index publication,
formatting-diff compatibility, substantive-change invalidation, and separate
BUILD startup. Approval does not create a branch. Resuming does not restore
unused conversational permissions. Private controller capabilities are not
serialised into records or accepted as model approval flags. The interactive
input adapter that supplies this trusted seam remains block-3 work.

Publication records the exact prepared commit before changing its branch ref;
recovery checks its parent, document/index bytes, and branch. Selected index
entries are updated without replacing unrelated staged edits. BUILD startup
also records its expected branch and candidate before branch creation, so a
crash after checkout does not require another approval or create another branch.

Migration preserves stopped legacy ownership bytes before retirement, keeps
tracked historical documents unchanged, and imports agent-reconstructed
progress as unapproved inspection facts. Unknown/live writers and unsupported
local formats refuse. Local records survive cache loss; clones and copied
records from another checkout do not inherit authority. Ignore generation is
owned by `pipeline.rs`, not `components/item.rs`.

Evidence is in `workflow/service/{tests,recovery_tests}.rs`, the replacement
`tests/workflow_safety.rs` CLI journeys, and the management ignore test. These
cover publication on both sides of the commit/JSON boundary, failed commit and
record writes, stale revisions, non-human flags, live/stopped claims, unknown
liveness, interrupted migration, skipped work, retained retries, dirty/wrong
branches, ID collisions, copy/clone refusal, and symlink escape refusal.
Legacy tests asserting early branch creation and approval prose were deliberately
replaced. BUILD execution and ACCEPT closure coverage must be restored against
the new runtime in blocks 4–5; passing these SCOPE/storage cases does not prove
those stages.

Latest verification:

- `cargo nextest run -p superdev-core workflow::`: 56 passed.
- `cargo nextest run -p superdev --test workflow_safety --test manage`: 9 passed.
- `cargo clippy -p superdev --all-targets -- -D warnings`: passed.
- `npm run check:validate`: zero errors and warnings.
- `cargo nextest run --workspace --no-fail-fast`: 894/897 passed. The three
  remaining `normative_shapes` failures concern the Pi-specific SOKF skill text,
  the file skill's managed-content hash, and its `/skill:file` text. These
  assets have concurrent edits; this block did not replace them or weaken
  their assertions. Reconcile them during the instruction/pack work.

Knowledge, plan-schema guidance, the ADR successor, and the live/pack instruction
rewrite remain outstanding. The repair hook may regenerate source includes;
those generated changes are not the final normative knowledge reconciliation.

### Block 3: human-led SCOPE adapter

The v2 orchestrator is replaced by a small adapter: `index.ts` (85 lines) with
`lib/{client,questions,scope,phase-tool,intake,service-exec}.ts`. It registers
`superdev_run_phase` and `superdev_ask`, the `/superdev*` commands, and one
interactive input handler. `lib/phases.ts` and the scope, requirements-review
and orchestrator prompts are deleted in both trees. Child-role processes are
blocked from acquiring v3 authority.

Questions now carry stable choice IDs, a separately identified recommendation,
and distinct answered/discuss/other-action/paused results. Control words are
reserved, so no model choice can imitate Pause or Discuss. A pending question
is consumed before any asynchronous mutation, so a UI selection and a chat
reply cannot authorise the same action twice. Discussion keeps the question
open and returns to chat without a second dialog.

The controller holds only the current continuation's step permission. Rust
remains the sole owner of approval and durable progress. Answers, open points,
and re-scope feedback accumulate in the record's discussion rather than
overwriting the last entry. A new SCOPE on another branch returns
`branch-switch-required` and creates nothing; `ScopeChange::ReturnToScope`
keeps an existing work branch, approvals, checkpoints, and retries.

Evidence is `scripts/test/superdev-workflow-scope.test.mjs`, which runs the
production extension inside a real pinned Pi session against real Rust and Git:

- Wrong-branch refusal, then issue-only reservation with no plan and no branch.
- `sendUserMessage` and RPC input do not reserve, permit, or approve anything.
- Drafting guards block writes without a current permission for that document.
- Repeated interviews, an intervening discussion, skipped steps recorded as
  skipped, and open findings retained through repeated checks.
- Pause inside a permitted group ends the unused permission; resume keeps the
  approvals and the saved discussion but requires fresh step permission.
- Stale bytes, a model `approved` flag, and transformed extension/RPC text do
  not approve. A trusted interactive transform does. Replay publishes nothing.
- Issue publication commits exactly once; plan approval leaves phase `scope`,
  creates no work branch, and reaches `handoff`.
- The invoked SCOPE checklist appears exactly once in the assembled context,
  and no terminal-repair instruction is injected.

Latest verification:

- `npm run test:scripts`: 55 passed, including the new scope suite and the
  rewritten extension/service smoke fixtures.
- `cargo nextest run -p superdev-core workflow::`: 57 passed.
- `cargo nextest run -p superdev --test workflow_safety --test manage`: 9 passed.
- `cargo clippy -p superdev --all-targets -- -D warnings`: passed.
- `npm run check:validate`: zero errors and warnings.
- `cargo nextest run --workspace --no-fail-fast`: 895/898 passed. The three
  `normative_shapes` failures are unchanged pack/instruction reconciliation
  work for block 6: the SOKF skill text, the lock hashes of replaced Pi assets,
  and the retired `superdev-abandon`/`superdev-force` command assertions.

### Block 4: persistent worker runtime

The runtime is three new modules plus durable execution state:

- `lib/worker-host.ts`: the worker process. It hosts one pinned-SDK
  `AgentSession`, loads project extensions, and speaks a typed message protocol
  over Node's IPC channel, which ordinary command output cannot imitate. It
  holds no controller capability, so it cannot approve, claim, or record.
- `lib/worker.ts`: `WorkerSession`, the controller's handle. It forks at most
  one worker, routes worker questions to the controller's single human-question
  interface, resets at settled boundaries, and stops with a bounded grace
  before terminating the process tree.

  The first version of this block carried stage instructions and tools only in
  the block-1 proof fixture, not in production. The acceptance double-check
  found that; see Corrections after the acceptance double-check.
- `lib/execution.ts`: `ExecutionController`, which starts BUILD, runs stages,
  consumes retries, and pauses. Every durable fact goes to Rust.

Rust gained `ExecutionStage`, `WorkerRecord`, `RETRY_BUDGETS`, and four typed
operations: `AttachWorker`, `DetachWorker`, `RecordProgress`, `ConsumeRetry`.
These use a new active-claim check, so the worker records progress while it
runs, while every checkout mutation still requires a stopped worker. Retry
budgets are consumed in the durable record, so a reset, pause, worker restart,
or controller restart cannot refill one.

Two production defects were found by the new tests and fixed:

- The worker died with `ERR_IPC_CHANNEL_CLOSED` when its controller
  disconnected mid-turn. It now ends at its current boundary and keeps its
  session file and durable record intact.
- `attach_child` accepted a process whose liveness was merely unknown. It now
  requires observed liveness, so nothing is recorded as an owner that cannot
  later be proven stopped.

One measured behaviour changed the design: a *busy* worker stops in an orderly
way, because Pi cancels its running tool. Termination therefore needs a
genuinely unresponsive process, and the test proves it with `SIGSTOP` and an
injected grace period rather than by asserting a convenient outcome.

Evidence is `scripts/test/fixtures/superdev-worker-runtime.ts`, which drives
the shipped controller and worker host — not a re-implementation — through:
one session across consecutive stages; a reset that removes the implementation
conversation while the next stage instructions remain; a worker question
answered by the controller; a refused question that does not strand the worker;
a refused second worker; orderly stop; restart reusing the same session file;
an interrupted stage reported as interrupted rather than finished; and
termination of a stopped process tree. Rust coverage is
`workflow/service/execution_tests.rs` (4 tests): execution facts refused before
BUILD starts, unbudgeted activities refused, one worker at a time, controller
writes refused while a worker runs, progress recorded while it runs, budgets
that survive pause and cache loss, and a stopped worker released without losing
its facts.

Worker transcripts are written under the already-ignored `.superdev/cache/`, so
they are never committed. `current` mode discloses `contextReset:
"unavailable-in-current-session"` rather than implying clean stage context.

Latest verification:

- `npm run test:scripts`: 56 passed.
- `cargo nextest run -p superdev-core workflow::`: 61 passed.
- `cargo clippy -p superdev --all-targets -- -D warnings`: passed.
- `npm run check:validate`: zero errors and warnings; `git diff --check` clean.
- `cargo nextest run --workspace --no-fail-fast`: 899/902 passed, with the same
  three block-6 `normative_shapes` failures and no new ones.

Still unproven and owned by block 5: stage checklists, multi-block BUILD with
real verification and commits, candidate-bound review, correction routing, and
ACCEPT closure.

### Block 5: BUILD, ACCEPT, and the instruction rewrite

Five typed closure operations complete the execution path: `CommitBlock`,
`CompleteBuild`, `ReturnToBuild`, `Accept`, and human-only `Abandon`. They route
through the existing pure transition table, so the legal phase edges have one
owner. `crates/app/superdev/src/workflow_cli.rs` now reports
`humanAcceptanceRequired` in `status`; an unreadable manifest reports no policy
and the adapter refuses rather than accepting automatically.

The controller gained matching operations in `lib/execution.ts`, with acceptance
asking the human only when project policy requires it, and a `superdev-abandon`
command. Abandonment is a typed command with no model path, so ending work stays
the person's decision. The retired `superdev-force` command is **not** restored:
it existed to override a SCOPE or ACCEPT review barrier, and human-led SCOPE has
no barrier to override, so an override would now only weaken the human's own
decision. A test asserts it stays absent.

BUILD and ACCEPT skills are rewritten as ordered checklists against the v3
interface. The retired `record-answer`/`submit-answers`/`revise-answer`/`routed`
vocabulary is gone, and a test refuses its return. The last three isolated-role
prompts (`accept.md`, `build.md`, `code-review.md`) are deleted in both trees,
so `.pi/extensions/superdev/prompts/` no longer exists: the invoked skills own
the stage checklists, with no second home to drift from.

Evidence is `scripts/test/superdev-workflow-execution.test.mjs` with
`fixtures/superdev-execution-sdk.ts`, driving the production extension, Rust and
Git through: execution operations refused before BUILD starts; the handoff
creating the work branch; a current-session stage disclosing its absent reset;
a block commit refused for touching paths outside its declared areas, and the
refusal leaving history unchanged; a block committed once; retry budgets
exhausted and unbudgeted activities refused; `complete-build` refused on a dirty
worktree; acceptance refused without a readable policy; ACCEPT findings
returning to BUILD and superseding the candidate while preserving plan approval
and consumed retries; and human acceptance closing the workflow without merging,
pushing, or moving the branch. Rust adds three closure tests in
`workflow/service/execution_tests.rs`.

Two assertions were corrected rather than the code: a document-approval phrase
is ordinary discussion at an acceptance question, and an untracked policy file
leaves the worktree dirty, so there is no candidate.

The three long-standing `normative_shapes` failures are resolved, each at its
cause rather than by weakening the test:

- The lock hashes were stale for the Pi assets this work replaced. `sync`
  regenerated them.
- The SOKF skill assertions checked guidance that was deliberately moved. The
  test now checks its actual homes: tool selection in `.pi/extensions/sokf.ts`,
  and physical-path authoring plus turn-end repair in `agent-instructions.md`.
- The filing skill's `/skill:file` text moved into its frontmatter name, and the
  force-gate command is intentionally retired.

Latest verification:

- `cargo nextest run --workspace --no-fail-fast`: **905 passed, 0 failed**.
- `npm run test:scripts`: 57 passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `npm run check:validate`: zero errors and warnings; `git diff --check` clean.
- Live and pack `.pi/extensions/superdev` trees are byte-identical.

### Block 6: knowledge, contracts, and configuration

[ADR-056][adr] supersedes ADR-054. It records human-led SCOPE, checkout-local
revision-bound approval, and the one persistent worker, and states the costs
plainly: a fresh clone inherits no approval, local state can be deleted, the CLI
surface changes again, and the current conversation cannot reset its own
context.

[adr]: knowledge/adrs/active/adr-056-scope-is-human-led-and-approval-is-checkout-local.md

Contract-011's Behaviour is rewritten against what now exists: local records and
transient claims, human-led SCOPE with per-step permission and revision-bound
approval, execution with one worker and bounded budgets, and recovery. Its
protocol promise names v3. Contract-002 describes `apply` and `status`, the
controller capability, compare-and-swap revisions, bounded block commits, and
local approval; its exit-code table lists the two surviving verbs.

The plan schema now says that `phase` describes where work has reached and is
not authority, so a copied or cloned plan states a phase it cannot act on. The
glossary defines controller and worker. `development-procedure`,
`software-components`, `directory-structure`, `testing-strategy`, `README`, and
`CHANGELOG` describe the shipped design rather than the retired one.

Two defects were found and fixed at their cause rather than documented around:

- `max_final_correction_cycles` and `max_scope_review_cycles` were configuration
  that claimed to control the correction budgets while the implementation
  hardcoded its own. The budgets now come from project policy, and a test states
  the policy it relies on instead of assuming a default.
- Eight `[workflow]` fields configured the retired isolated-role runtime and
  controlled nothing. They are removed, along with `lib/{output,review,progress}.ts`
  and the dead BUILD command allowlist. Unknown keys are now ignored rather than
  refused, so a manifest written for the old runtime still loads and the next
  managed rewrite drops them.

Issue-086 and issue-091 are closed against this work, each naming the evidence.
The recommendation is now a stable choice ID that validation binds to a supplied
choice; the deadline-killed isolated role no longer exists.

Final verification:

- `cargo nextest run --workspace --no-fail-fast`: **905 passed, 0 failed**.
- `npm run test:scripts`: **57 passed**.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `npm run check:validate`: zero errors and warnings.
- `npm run check:blueprint`, `npm run check:docs`, `cargo run -- sync`: clean.
- `git diff --check` clean; live and pack `.pi/extensions/superdev` identical.

## Corrections after the acceptance double-check

A double-check of the implementation against
[WORKFLOW_REDESIGN_2.md](WORKFLOW_REDESIGN_2.md) found six defects, all in the
worker execution path. Four were substantive: the passing suites did not cover
them, so they are the kind a green run hides. All are fixed.

- **The worker never received its stage checklist.** Its only input was the
  model's free-text note, so a reset left review and acceptance with empty
  context — the reset that makes context clean also made it empty. The proof
  fixture had injected checklists and facts; production had not.
  `ExecutionController.stageContext` now builds each stage's own checklist and
  its durable facts — issue, plan, branch, candidate, completed blocks,
  unfinished work, evidence, consumed retries — and `worker-host.ts` injects
  them through `before_agent_start` on every turn.
- **Tool access did not change with the stage.** The worker kept `write`,
  `edit`, and `bash` during review and acceptance, so a read-only assessor could
  edit the candidate it judged. `stageTools` now scopes tools per stage and the
  worker applies them before each turn.
- **The checkpoint was never updated during execution.** `run()` wrote back
  exactly what it read, so `unfinished` and `evidence` were dead outside
  migration and a mid-block crash lost the notes recovery depends on. The stage
  request now carries `unfinished` and `evidence`, and checkpoints before a
  reset so a failure between them loses no facts.
- **Reset after every completed block was instructed but impossible.**
  `resetBeforeStage` covered only review and acceptance, and no caller could
  request another, so the BUILD skill instructed something with no operation
  behind it. `reset` is now part of the tool surface.
- **Losing the controller aborted the worker mid-turn** instead of requesting a
  pause at a safe boundary. It now settles within a bounded wait, then ends.
  Fixing this exposed a second defect in the fix itself: `process.send` reports
  a closed pipe through an asynchronous `error` event, not a throw, so the
  worker died with `EPIPE` rather than exiting cleanly. It now passes a
  callback and reports the undelivered message.
- **Worker questions lost the control vocabulary.** Discuss and Pause reached
  the worker as the literal answer string `"Question was not answered: discuss"`.
  The human's control now passes through as itself, and a question that could
  not be put to the human is reported as such rather than fabricated.

`verification` was a declared stage nothing ever set. It is now a real stage
with its own tools and a BUILD step.

The worker runtime test previously asserted that a reset *removed* old context
but never that the next stage *received* its own — which is why it passed while
the requirement failed. It now asserts the checklist and facts arrive, that
review loses every mutating tool and implementation regains them, that a
disconnected worker exits cleanly, and that a human's Discuss is delivered as a
control. The execution journey asserts durable `unfinished` and `evidence`, and
that the current-session path returns the stage context a worker would receive.

Contract-011 gained the promises these behaviours now keep:
`P_stage-owns-instructions`, `P_assessment-is-read-only`, `P_reset-on-request`,
`P_progress-is-recordable`, `P_control-not-invented`, and `P_disconnect-pauses`.

Verification after the corrections:

- `cargo nextest run --workspace --no-fail-fast`: **905 passed, 0 failed**.
- `npm run test:scripts`: **57 passed**.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `npm run check:validate`: zero errors and warnings; `git diff --check` clean.
- Live and pack `.pi/extensions/superdev` trees are byte-identical.

## Remaining gap

The redesign's verification cases are covered by executable evidence except the
human-guided rehearsal: a real SCOPE conversation and a small multi-block
BUILD/ACCEPT run in a disposable repository, including pause and resume. The
automated suites drive the production extension, service, and worker, but they
script the human's replies rather than observing a person use the terminal UI.
That rehearsal, and the judgement of whether the checklists read well in
practice, needs a human.
