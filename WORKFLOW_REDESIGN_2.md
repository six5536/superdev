# Workflow redesign 2

Status: design draft, double-checked; open design questions remain. This is not
an approved implementation plan.

This document records a redesign discussed outside the existing workflow. No
SCOPE, BUILD, or ACCEPT phase is started by writing it. The `<superdev>` prompt
is correct and remains unchanged. Existing contracts describe the current
implementation until the redesign is approved and implemented.

## Purpose

Keep SCOPE human-driven and make BUILD and ACCEPT automated execution flows
with durable progress. Skills guide the work. The implementation records state,
enforces human authority, and provides session control without duplicating the
skills' judgement.

## SCOPE

```text
Select/create issue
  → Interview with grill-me
  → Write/update issue
  → Double-check issue
  → Human issue approval
  → Write plan
  → Interview with grill-me
  → Update plan
  → Double-check plan
  → Human plan approval
  → BUILD here / BUILD in worker / Exit SCOPE
```

### Invocation and guidance

- The user or LLM invokes the scope skill. It selects an existing issue or
  creates an initial issue from the request. The interview then informs the
  complete issue or its revision.
- SCOPE uses no sub-agents. It invokes the `grill-me` and `double-check` skills
  in the current conversation rather than delegating them to child sessions.
- Interviews ask one question at a time through the ask tool, give a recommended
  answer, and use repository research to settle questions the code can answer.
- Each step transition needs human permission. Permission can cover an explicit
  group of named steps, such as "write the issue and double-check it".
- Explicit chat approval or an approval choice in the ask tool counts. Do not
  require a second confirmation dialog. Clarify ambiguous instructions rather
  than infer approval from general agreement.
- At each decision point, show the current step, recommend the next action,
  and offer Continue, Discuss, Do something else, and Pause.
- Accept typed instructions, including requests to repeat or skip steps. The
  human can advance at any time; the LLM cannot skip steps on its own.
- Preserve progress during discussion and unrelated work. Discussion alone
  neither advances state nor implies approval.
- The user can request another interview or double-check for either document.
  Record skipped checks without claiming they passed.

### Approval and state

- Track SCOPE durably before a plan exists. Record the current step, completed
  and skipped work, outstanding discussion, and document approvals.
- Keep grouped step permission simple: it applies to the current continuation.
  After a pause, show saved progress and the next step, then ask for fresh
  permission to continue. Do not persist unused permissions or build a separate
  permission-tracking mechanism.
- Keep authoritative human approvals and execution state in the durable
  workflow record, outside issue and plan documents. Use that same small state
  record; do not add a separate approval ledger.
- Keep workflow progress, approvals, session ownership, and recovery state
  local and excluded from Git. Issues, plans, and their document lifecycle
  remain Git-tracked. Later sessions in the same checkout can resume from
  local state; a fresh clone receives the documents but needs fresh approval
  before BUILD. Git does not transfer an active workflow between machines.
- Bind approvals to the relevant document identity and revision. Document
  prose or copied metadata never grants approval.
- Keep document lifecycle in the plan: `open`, `done`, or `abandoned`. Lifecycle
  describes the document; it does not prove human approval or authorise another
  workflow. A newly copied plan starts `open` and unapproved.
- Substantive changes clear the affected approval. Substantive issue changes
  also clear plan approval. Formatting-only edits do not clear approval.
- Preserve completed work when an approval is cleared. Explain what changed
  and guide the user back to approval. The human can approve directly without
  repeating interviews or checks.
- BUILD still requires an existing, explicitly approved plan.
- Create and switch to the work branch only when BUILD starts after plan
  approval. Re-scoping an existing workflow retains its work branch.
- Exiting SCOPE preserves the approved plan for a later invocation. Approval
  alone does not start BUILD. Represent approved-but-not-started SCOPE
  explicitly; starting BUILD is a separate authorised action.

### Drafts and publication

- Save draft documents and interview progress between steps. Saving a draft is
  not approval and does not by itself create a Git commit.
- Commit each document when the human approves it: the issue at issue approval,
  then the plan at plan approval. Include only the relevant records.
- If a new SCOPE starts on another branch, ask the user to switch to the default
  branch before authoring its records. Do not automatically create a
  default-branch worktree or keep the new records on the unrelated branch.
- Re-scoping an existing workflow remains on its existing work branch.
- Preserve unrelated edits. The exact durable draft/state format and atomic
  approval/publication failure handling remain implementation design questions.

## BUILD and ACCEPT

### Session selection and ownership

- Execute in the current conversation or in one persistent worker session,
  selected at the SCOPE handoff or when BUILD or ACCEPT starts separately.
- A separate worker is controlled from the current conversation. It is not a
  new interactive session that the user must switch to.
- The worker executes all work blocks, verification, review, and corrections.
  It creates no further agents and is not replaced for each block or role.
- By default, ACCEPT reuses the worker with reset context.
- Keep one executing workflow owner per checkout. Preserve unrelated work.
- If the controller closes or disconnects, request a pause at a safe
  checkpoint. A later session can resume the worker.
- A crash preserves the last durable checkpoint and partial files. Recovery
  inspects the actual worktree before continuing; it does not assume that an
  interrupted checkpoint completed.

### Context and checkpoints

- Checkpoint after every completed BUILD block and before each context reset.
- Reset context after every completed block, when context fills mid-block,
  before final code review, and before ACCEPT.
- Resume from the approved plan and durable progress, including unfinished
  work and test results. Do not repeat completed blocks merely because the
  conversation context was reset.
- Reconstruct review and acceptance context from the approved plan, issue,
  candidate diff, contracts, and verification evidence rather than from the
  implementation conversation.
- A reset gives the same session fresh assessment context. It does not make
  that session an independent reviewer. Ordinary compaction is not sufficient
  if it retains implementation messages or their conclusions. Confirm the
  actual next-request context, including retained messages and injected state.
- Apply the same reset policy in the current conversation only where the host
  supports it safely. Otherwise continue without special context treatment
  and disclose that limitation.

### Execution and completion

- BUILD stays within the approved plan, verifies and commits completed blocks,
  performs final verification, and reviews the candidate.
- Final code review stays in BUILD, in the executing session after a reset.
- Keep existing routing: within-scope corrections return to BUILD; changes to
  intent return to human-led SCOPE.
- Keep project-configured optional human acceptance.
- Keep bounded retries and corrections. Context resets or resumption must not
  silently grant a new retry budget.
- Acceptance closes the records and releases execution ownership without
  automatically merging, pushing, releasing, or deleting the work branch.

## Ask-tool redesign

- Give choices stable identities and identify the recommended choice directly.
- Show the recommended choice first and mark it; retain a concise explanation.
- Keep presentation markers out of the saved answer.
- Distinguish an answer or approval from Discuss, Do something else, and Pause.
- Support typed answers and ordinary chat without losing the pending question.
- Return control to the conversation for discussion instead of reopening the
  same dialog until the user selects a listed answer.
- Use this interaction vocabulary consistently in SCOPE and worker questions.

## Instruction and prompt rewrite

- Rewrite all skill and instruction/prompt files covered by this redesign,
  including the extension's bundled skills, retained stage prompts, and
  instructions constructed or injected by extension code. Moving existing
  files or making small wording edits is not sufficient.
- Keep `<superdev>` unchanged. Use the current
  `.pi/skills/sokf-authoring/SKILL.md` as the quality reference: concise,
  specific, actionable instructions, with no unnecessary explanation.
- Write workflow instructions as clear checklists in execution order. Each
  skill describes its own workflow, not a copy of the SOKF authoring checklist.
  State the necessary inputs, actions, decisions, stopping points, and handoff.
- Do not repeat information already supplied by the effective system prompt,
  including skill discovery, tool descriptions, extension guidelines, and
  `<superdev>`. Give each instruction one authored home at the appropriate
  layer; refer to an existing skill or canonical document instead of restating
  its rules.
- The extension adds only the minimum high-quality instructions needed in the
  system prompt. Keep full workflow checklists in the invoked skills, not in
  always-present prompt additions. Inject current state as state, not as
  another copy of the process instructions.
- Review the assembled model context, not just individual files. Ensure that
  fresh sessions and context resets load the required instructions once;
  removing duplication must not leave a worker without its stage checklist.
- Remove obsolete prompts and duplicated orchestration rules. Do not preserve
  unused role files merely to rename them.
- Keep required live/pack copies identical. Shipping the same authored asset
  in the pack is not a second source of instruction policy.

## Implementation blocks

Start with an inventory of skill files, prompt files, tool guidance, and
extension-injected instructions. Assign each retained instruction to its
owning layer. Apply the instruction rewrite requirements throughout the blocks
below, including file, grill-me, and double-check, not only the three phase
skills.

### 1. Durable state and human authority

Support issue-only SCOPE, step history, grouped permissions, revision-bound
approvals, approved-but-not-started SCOPE, pause/resume, and delayed branch
creation. Separate plan approval from BUILD startup. Keep one authority for
state transitions rather than implementing competing state machines in Rust,
Pi, and skill prose. Do not make the core judge interview or review quality.

### 2. Question interface

Give choices stable identities. Connect recommendations to choices and support
answers, discussion, another action, and pause as distinct results. Keep the
ask tool's descriptions and system-prompt guidance minimal; the SCOPE checklist
owns the interview flow.

### 3. SCOPE skill

Compose `grill-me` and `double-check` in the current conversation. Both skills
now belong to `.pi/extensions/superdev/skills/`, with shipped copies under
`pack/pi/extensions/superdev/skills/`. Keep their extension discovery and pack
parity covered by tests. Honour applicable skill adaptations. Remove SCOPE
child authoring and requirements-review loops. Guide the user through issue
and plan decisions without making review findings a barrier to human approval.

### 4. Single-session execution

First prove same-session context reset and restart recovery against the pinned
Pi version. Then replace per-role spawning with a resumable worker and
checkpoint-driven context handling. Keep the controlling conversation usable
for progress, questions, and pause requests while the worker runs. Change the
stage-specific instructions and tool access with the worker's stage; do not
leave BUILD's permissions or instructions active during read-only assessment.

### 5. BUILD and ACCEPT skills

Run implementation, review, corrections, and assessment in the selected
session. Reuse the worker across stages and reset context at the agreed
boundaries. Route intent changes back to the human conversation.

### 6. Migration and verification

Update contracts, documentation, shipped skills, and tests. Preserve existing
workflows without inventing approvals. Remove obsolete role orchestration
rather than maintaining two complete execution systems. Review the rewritten
checklists and assembled prompts against the instruction-quality requirements;
do not judge the rewrite solely by file length or exact-string tests.

## Technical questions to resolve before implementation

- How does the pinned Pi version reset active model context without creating
  another worker session or carrying implementation opinions into review?
- Which safe reset operations are available in the current interactive session?
- Where does durable issue-only SCOPE state live, and how is it recovered when
  the session or transient ownership cache is lost?
- How are explicit chat permissions bound to real human input and document
  revisions without a duplicate confirmation dialog?
- What is the minimal durable local workflow-record format for execution state
  and approvals, separate from the Git-tracked document lifecycle? Choose an
  ignored storage path and distinguish saved progress from transient process
  ownership.
- How are substantive changes distinguished from formatting-only changes?
- How do existing workflows migrate into the new state model?

## Related existing records

- `knowledge/issues/open/issue-086-a-recommended-choice-is-prose-not-a-choice.md`:
  recommended-choice identity and presentation.
- `knowledge/issues/open/issue-091-an-isolated-role-loses-everything-at-its-deadline.md`:
  checkpoints, resumption, retained work, and cumulative execution budgets.
- `knowledge/ideas/idea-010-a-pi-extension-runs-the-superdev-process.md`:
  earlier session, interaction, and compaction proposals.
- `knowledge/adrs/active/adr-054-the-workflow-core-owns-state-branch-and-refusals.md`:
  the current separation of durable state, Git rules, and agent judgement.

## Double-check

### Result

The draft preserves the agreed user flow and session limit. It is not yet
implementation-ready. The following findings need explicit design decisions
or focused technical evidence. Recommendations below are proposals, not
additional approvals from the user.

### Findings and recommendations

1. **Approval and execution are currently the same transition.**
   `crates/lib/superdev-core/src/workflow/transition.rs` maps `ApproveScope`
   directly from SCOPE to BUILD. `WorkflowIdentity` in `workflow/state.rs`
   requires both a plan and a work branch. Neither represents the new
   issue-only or approved-but-not-started state. The implementation outline now
   explicitly separates approval from startup. Decide the durable record
   location and recovery format before changing this transition.

2. **Saving drafts before BUILD needs a Git policy.**
   Delaying the work branch does not specify where issue and plan drafts are
   written or when they are committed. The current bootstrap reserves and
   commits the issue and plan together on the default branch. The new flow
   cannot require a plan just to save an issue interview.
   Resolved publication policy: save drafts and interview progress between
   steps, and commit each document on its own human approval. For a new SCOPE
   on another branch, ask the user to switch to the default branch first; do
   not automatically use a default-branch worktree. Re-scoping stays on the
   existing work branch. Preserve unrelated edits.
   Remaining design: independent identity reservation, durable draft/state
   storage, and atomic handling of approval or commit failure. Do not create a
   temporary plan merely to satisfy the old bootstrap.

3. **Revision-bound approval needs a precise validity rule.**
   A byte hash changes for formatting as well as requirements. Whitespace
   normalisation is not a general semantic equivalence test: indentation can
   change code or machine-readable structures. Existing human authority also
   depends on an interactive UI capability, while the agreed redesign accepts
   chat approval without another dialog.
   Recommendation: retain the actual human input reference, the approved
   revision, and the named actions. Record a reviewed formatting-only revision
   mapping rather than silently treating all whitespace edits as harmless.
   Clarify who classifies such edits and what happens when classification is
   uncertain. Resolve authority from human input, never from a model-supplied
   boolean or an old conversation summary.

4. **Grouped permission after a pause: resolved in favour of simplicity.**
   Save progress, not unused permission. On resume, show the next step and ask
   for fresh permission. Do not add a permission ledger, consumption tracking,
   or cross-pause grant restoration. Existing document approvals remain subject
   to their revision-validity rule; pausing alone does not clear them.
   Discuss and Do something else leave the current question recoverable without
   silently completing it. A human-directed jump records omitted steps and
   outstanding findings rather than fabricating completion.

5. **Compaction is not a demonstrated clean-context reset.**
   Pi's session and SDK documentation describe persistent sessions, in-place
   tree navigation, and context rebuilding. Its compaction documentation says
   the default retains recent messages and a summary. These are useful
   primitives, not proof that the proposed worker can reset safely at every
   boundary. The example custom-compaction extension also returns the prepared
   kept-entry boundary; its descriptive comment is not evidence that all old
   messages are removed.
   Recommendation: test the pinned package's actual model request and restored
   session, not just the reset command's result. Keep the logical session ID
   unchanged, replace stage-specific context, and ensure an old summary cannot
   reintroduce implementation conclusions into review. Do not cut a pending
   tool call away from its result. Reserve enough context for a checkpoint
   before overflow, and preserve the old state if reset fails. The interactive
   fallback remains permitted, but an unsupported worker reset must be
   reported rather than silently advertised as clean context.

6. **A persistent worker changes failure handling and retained data.**
   `.pi/extensions/superdev/lib/process.ts` currently spawns print-mode
   processes with `--no-session` and kills them on timeout. Changing that flag
   alone does not provide a responsive controller, durable checkpoints, or
   stage changes. `index.ts` also fixes child-role permissions at startup.
   Recommendation: define controller-to-worker question/answer routing,
   process liveness, safe command interruption, and bounded shutdown grace.
   Stop starting new work when controller loss is detected. Release ownership
   only after the writer stops, and resume with exclusive ownership. Persist
   retry consumption across resets. Specify worker-transcript storage, access,
   retention, and cleanup; do not commit private transcripts or authority
   credentials into project knowledge. Canonical records, session history,
   and transient process ownership must not become competing sources of truth.

7. **Evidence must belong to the candidate being assessed.**
   Reusing the worker does not permit reusing a clean review after code changes.
   The current drivers resolve immutable Git candidates for review and ACCEPT.
   Recommendation: retain candidate-bound verification and review evidence;
   route corrections through BUILD, rerun affected checks, and reset before a
   fresh assessment. A resumed read-only stage must not inherit BUILD's write
   access. Keep final-suite execution at completion rather than adding a full
   suite to every block reset. Define the existing policy for manual cases and
   required human acceptance in the rewritten skills rather than dropping it.

8. **Packaging and migration are part of the change.**
   The initial check found `grill-me` and `double-check` only under
   `.pi/skills/`, without shipped copies. This packaging gap is now resolved:
   both skills moved into the Superdev extension and its pack, without changing
   their instructions. Workflow integration remains part of this draft.
   The current contract also requires isolated roles,
   early branch bootstrap, and interactive confirmation; it cannot remain
   unchanged while those mechanisms are removed.
   Recommendation: update contract-011, relevant CLI/configuration contracts,
   ADR-054 or its successor, skill discovery, live and pack copies, and user
   documentation together. Migrate phase and existing evidence without
   inventing separate issue approval or step history. Pause incompatible live
   workers before migration. Keep `/skill:file` independent and `<superdev>`
   unchanged. Treat old issues as related work, not automatically completed by
   adopting this design.

9. **Approval text can be copied with a knowledge document.**
   The user reports approval written into a knowledge document, sometimes
   appearing in a new document copied from it, and apparently expressed as
   arbitrary prose. The copied-document failure has not been reproduced.
   Inspection confirms that `workflow_cli/transition.rs` changes the plan's
   `phase` field and writes forced-approval notes into Completion evidence.
   `workflow_cli/identity.rs` also writes a prose default-branch marker for
   recovery. The approval transition itself checks UI authority; no arbitrary
   approval-prose gate was established by this inspection.
   Confirmed separation: keep authoritative approval and execution state in
   the small durable workflow record already required for state tracking,
   outside issue and plan documents. Bind approval to exact document identity
   and approved revision. Keep the plan's document lifecycle (`open`, `done`,
   or `abandoned`) in the plan. Lifecycle is not approval evidence. A newly
   copied plan starts `open` and unapproved; inherited prose or metadata cannot
   authorise its workflow. Do not introduce a separate approval ledger or a
   prose parser. Any human-readable approval summary is informational, never
   an alternative source of authority. Workflow progress, approvals, session
   ownership, and recovery state are local and excluded from Git. A later
   session in the same checkout can resume; a fresh clone needs fresh approval
   before BUILD. The exact local workflow-record format and ignored storage
   path remain to be designed.

### Verification cases required by the implementation plan

- Every retained skill and instruction/prompt file has been reviewed and
  rewritten for its own workflow. Checklists are concise, actionable, and in
  execution order, using the current SOKF authoring skill as the quality
  reference without copying its domain-specific content.
- Assembled prompts contain only necessary extension instructions and no
  restatement of rules already supplied by the system prompt. Inspect startup,
  skill invocation, worker stage changes, and post-reset context. Required
  stage instructions remain available, and live/pack copies agree.
- SCOPE runs its complete issue and plan sequence without spawning an agent.
- SCOPE resumes at every step, including before a plan exists and after plan
  approval without starting BUILD or creating a work branch.
- A grouped human permission advances only its named actions in the current
  continuation. After a pause, SCOPE shows saved progress and requests fresh
  permission without repeating completed steps. Explicit chat approval is
  accepted; ambiguous or model-generated approval is not.
- Direct human advancement works with open findings and skipped steps. The
  resulting record does not say the skipped checks passed.
- Discuss, typed answers, another action, pause, and cancellation preserve the
  pending question and do not imply approval.
- Issue edits invalidate dependent plan approval; substantive plan edits clear
  plan approval; reviewed formatting-only edits preserve valid approval.
- Copying an approved document to a new issue or plan does not transfer its
  approval. Approval-like prose, copied metadata, and a model's claim of prior
  approval cannot authorise a transition.
- A later session in the same checkout can resume local workflow state. A fresh
  clone receives no workflow progress, approval, ownership, or recovery state
  through Git and requires fresh approval before BUILD.
- A fresh managed project discovers all required skills, not just SCOPE,
  BUILD, and ACCEPT.
- Current-session and worker execution use the same approved plan. No SCOPE
  child or extra BUILD/review/ACCEPT worker appears in process/session traces.
- Worker identity survives block resets, code review, ACCEPT, and restart.
  Actual model-request context contains only the permitted stage context.
- Mid-block overflow, failed checkpoint writes, failed reset, interrupted
  commands, and crashes preserve partial work without false completion or
  automatic replay of uncertain side effects.
- Controller loss pauses the writer; reconnecting does not create a second
  writer. Stale ownership and outstanding questions are recoverable.
- Resumption and context resets preserve retry/correction consumption.
- Candidate changes invalidate stale review and acceptance evidence. ACCEPT
  follows project human-acceptance policy, and intent changes return to SCOPE.
- Dirty checkouts, unrelated branches, ID collisions, and existing workflows
  migrate or pause without losing work, inventing approvals, or implicitly
  merging, pushing, or releasing.

### Review evidence and limits

This check compared the conversation decisions with the current workflow
state, transition, phase-driver, process, intake, and skill-discovery code. It
also read the installed Pi SDK, sessions, session-format, and compaction
documentation and the custom-compaction example. The project pins
`@earendil-works/pi-coding-agent` to `0.85.1` in `package.json`; integration
claims must be checked against that dependency, not inferred from documentation
alone.

No worker prototype, context-reset integration test, workflow migration, or
product implementation was run. This is a document/design review, not evidence
that the redesign already works.
