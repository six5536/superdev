# SCOPE → BUILD → ACCEPT Implementation Plan

## Purpose

Replace Superdev’s contradictory `FILE → SCOPE → BUILD → ACCEPT` / `execute-plan` workflow with one concise, resumable, Pi-orchestrated workflow:

```text
SCOPE → BUILD → ACCEPT
```

`/file` remains an independent capture utility for issues and ideas. It is not a workflow phase.

This document is the complete implementation specification. It is intentionally a root-level engineering plan, not a `schema-plan` document. It contains no interview transcript or double-check log.

## Outcomes

When this work is complete:

- Every scoped change has one primary issue, one plan, and one work branch.
- SCOPE resolves and records user intent, approved design decisions, behavioural promises, and a build-ready plan without implementing source interfaces.
- BUILD alone owns the complete work-block loop, focused checks, commits, final verification, and correction cycles.
- A fresh isolated reviewer performs a mandatory read-only code review after full verification.
- ACCEPT applies one project-wide human-acceptance policy, closes the issue and plan, and merges locally with `--no-ff`.
- The plan is the canonical durable phase/progress record; transient run ownership remains under `.superdev/cache/`.
- Pi provides the supported orchestration adapter. The Rust core owns workflow state and legal transitions.
- Existing Claude Code skills are archived and are no longer active or materialized.
- Existing issues and plans are migrated to one strict schema version.
- Every project declares its user-facing documentation surfaces and verification commands; affected authored, generated, and website documentation is planned, built, rendered, and reviewed as part of the same change.

## Non-goals

- GitHub, GitLab, pull-request, remote-CI, push, deployment, or release orchestration.
- Release tags on issues or a customizable release workflow. File that separately after this work.
- Claude Code workflow support or a premature cross-harness prompt abstraction.
- A second feature/ad-hoc plan shape.
- Plan-level manual test cases.
- A transcript, full reasoning log, or repeated review output in a plan.
- Parallel modifying agents in one working tree.
- Automatically resolving merge conflicts, stashing changes, or discarding user work.
- A large model/provider evaluation matrix.

## Fixed decisions

### Workflow and records

1. The only workflow phases are SCOPE, BUILD, and ACCEPT.
2. `/file` is an out-of-band utility for filing issues or ideas.
3. `/scope <issue-id>` adopts an existing open issue.
4. `/scope <request>` reserves and creates an issue before creating its plan.
5. Every plan has exactly one primary issue; an issue may retain links to completed plans but has at most one open implementing plan.
6. A re-scope while a plan is open retains the same issue, plan, branch, block numbers, and history. Reopening an issue after its plan is done creates a new plan and branch; completed plans remain immutable history.
7. A historical orphan plan receives a migration-derived `done` issue; no plan remains orphaned.
8. Plan lifecycle is `open | done | abandoned`; its durable phase is `scope | build | accept | done | abandoned`.
9. Cancelling pauses automation and releases ownership. It does not abandon or close the plan. Abandonment is a separate explicit human-only action.
10. Issue `done` means implemented, accepted, and integrated into the repository’s default branch. It does not mean released.

### Human authority

1. SCOPE always requires explicit human approval.
2. The project, not the model or an individual plan, decides whether ACCEPT requires a human decision.
3. The model cannot assign manual acceptance work to the human.
4. When human acceptance is required, ACCEPT asks only for accept or reject after presenting the change and evidence. It does not prescribe how the human reviews it.
5. A rejection is recorded verbatim on the primary issue and returns the same plan to SCOPE.
6. A human can reopen an issue after automatic acceptance.

### Contracts, ADRs, and knowledge

1. Source remains authoritative for interface `Definition` sections under ADR-042.
2. SCOPE may commit approved ADR decisions, normative knowledge, and authored contract Behaviour/Stability promises.
3. New or changed prose promises are marked granularly as `PENDING (plan-id/block-n)`.
4. SCOPE plans source-interface changes but does not edit source declarations or generated `Definition` sections.
5. BUILD changes source declarations and runs `superdev validate --fix` to materialize contract definitions.
6. BUILD removes `PENDING` only after implementation and cited automated evidence exist.
7. ADR implementation state is derived from its link to the open implementing plan; accepted ADRs do not gain a separate pending lifecycle.
8. SCOPE may update other authored knowledge when the content is normative and becomes true on approval. BUILD updates current-state and implementation-dependent descriptions after the implementation exists.
9. Schema-owned document/section rules, not prompt guesswork, classify phase mutation ownership.

### Execution and review

1. SCOPE runs one mandatory isolated requirements review. It reruns after substantive corrections until no unresolved requirements finding remains.
2. There is no mandatory model-based buildability review. Deterministic plan validation checks structure; BUILD discovers practical constraints.
3. BUILD uses tests before or alongside implementation where meaningful, without artificial test-first work for generated, documentation-only, or mechanical changes.
4. Every block is focused, checked, and committed before the next block.
5. The full repository verification runs after all blocks, not between blocks.
6. A fresh read-only reviewer examines the complete final diff after full verification.
7. Any actionable review finding returns to BUILD. BUILD fixes it, runs focused checks, reruns full verification, and obtains a fresh review.
8. The final correction loop is bounded by project configuration.
9. Remote CI is outside the workflow; local CI-equivalent commands are authoritative for ACCEPT.

### Git

1. Starting SCOPE requires the default branch and a clean index/worktree.
2. The workflow never stashes, resets, discards, or absorbs pre-existing changes.
3. BUILD commits only on the recorded work branch.
4. Before final verification, BUILD incorporates the current default branch into the work branch.
5. If the default branch advances after verification, BUILD incorporates it and repeats full verification and review.
6. ACCEPT commits issue/plan closure on the work branch, checks out the default branch, and performs `git merge --no-ff`.
7. Merge conflicts and dirty-tree conditions stop automation.
8. ACCEPT does not push or delete the work branch.

## User-facing interface

### Normal path

The normal entry point is:

```text
/scope <issue-id or request>
```

After human scope approval, the Pi extension advances automatically:

```text
SCOPE → BUILD → ACCEPT
```

BUILD returns to SCOPE when unresolved behavioural/design discoveries exhaust the safe independent work. ACCEPT pauses only when configured to require human acceptance.

### Recovery and utility commands

Register these Pi commands:

- `/file <issue-or-idea>` — capture an issue or idea independently of the workflow.
- `/scope <issue-id-or-request>` — start or resume SCOPE.
- `/build [plan-id]` — resume BUILD for the current plan.
- `/accept [plan-id]` — resume ACCEPT.
- `/superdev-status` — render canonical plan phase plus transient owner/worker status.
- `/superdev-resume` — claim and continue a paused run from plan/Git state.
- `/superdev-cancel` — terminate children, release ownership, and leave committed progress intact.
- `/superdev-abandon` — explicitly abandon the current plan after human confirmation and disposition of its issue/knowledge.

Do not register `/execute-plan`, `/integrate`, `/feature-plan`, `/adhoc-plan`, or a workflow-phase `/file`.

### Configuration

Extend `.superdev/config.toml` with a required table:

```toml
[workflow]
human_acceptance_required = true
max_stalled_block_attempts = 3
max_final_correction_cycles = 3
```

Rules:

- A manifest whose blueprint predates this migration may omit the table; parsing assigns these safe defaults and the migration/sync writes them explicitly while advancing the blueprint version.
- New repositories generate the same defaults.
- After migration, a missing table is invalid; malformed values always fail safely and never imply automatic acceptance.
- Both limits are positive integers.
- Plans, prompts, commands, and models cannot override these values.
- Changing the values is an ordinary reviewed project configuration change.

## Canonical plan schema

Replace `schema-plan` with one strict shape. Preserve readable Markdown; do not move the specification into an opaque state file.

### Frontmatter

Require:

```yaml
type: Plan
id: plan-NNN-slug
title: ...
description: ...
lifecycle: open | done | abandoned
phase: scope | build | accept | done | abandoned
branch: work/NNN-slug
links:
  - rel: implements
    to: issue-NNN-slug
```

Validation invariants:

- `lifecycle: done` if and only if `phase: done`; `lifecycle: abandoned` if and only if `phase: abandoned`.
- An open plan resides under `knowledge/plans/open/`; a done plan under `knowledge/plans/done/`; an abandoned plan under `knowledge/plans/abandoned/`.
- Exactly one `implements` link resolves to the primary issue.
- The issue’s incoming SOKF relationships expose every implementing plan; do not duplicate that list in issue prose or metadata.
- At most one open plan implements an issue; older done plans remain visible as incoming history.
- The branch follows the project convention and matches the workflow state.
- A plan cannot enter `build` until its scope approval commit exists.
- A plan cannot enter `accept` until every block is complete and final verification/review evidence is current.

### Ordered sections

Raise the plan limit from 800 to 1,500 lines. This is a safety ceiling, not a target; completeness wins over an arbitrary short document, while transcripts, repeated findings, and raw command logs remain prohibited.

1. `Goal and boundaries`
   - Desired outcome, current gap, constraints, explicit exclusions, and relevant evidence.
2. `Requirements`
   - Settled user intent and non-contract requirements. No unresolved questions.
3. `Contract changes`
   - Complete list of added, changed, or withdrawn promise/criterion keys, or `none`.
4. `ADR decisions`
   - Decisions created, changed, superseded, or explicitly `none`.
5. `Source and interface changes`
   - Exact source declarations/regions to modify and the generated contract definitions expected from them. This section plans source edits; SCOPE does not perform them.
6. `Knowledge changes`
   - Scope-owned normative edits already approved and Build-owned current-state edits still required.
7. `Documentation changes`
   - One entry for every affected documentation surface from the project documentation map: audience, authored source, generated output, required content, owning block, generation command, and verification command; or explicit `none` with evidence that no declared trigger applies.
8. `Work blocks`
   - Dependency-ordered blocks, never renumbered after approval.
9. `Build state`
   - Machine-maintained current block, attempt counts/fingerprints, completion markers, final correction-cycle count, and blocker state. Do not require a commit to contain its own hash; derive block commit IDs from Git trailers/history and summarize them later in the attestation.
10. `Implementation decisions`
   - Concise BUILD-time local choices: block, decision, reason, and affected paths. No chain-of-thought or conversational log.
11. `Follow-up issues`
    - Links to unrelated issues filed during work, or `none`.
12. `Completion evidence`
    - Scope requirements-review and human-approval attestations; focused code and documentation evidence by block; final verification/review attestations bound to the immutable candidate revision; and acceptance mode/result. Do not embed the containing scope, attestation, or closure commit’s own hash; Git history and transition trailers identify those commits and the subsequent merge. Later administrative-only descendants are permitted only by the phase diff policy.

The schema declares section ownership: SCOPE authors sections 1–8; during BUILD, only the Rust service may update completion/evidence fields inside Work blocks. BUILD machine state owns section 9; BUILD appends sections 10–11; the Rust service writes section 12. Re-scoping may revise the authored specification but never erases execution history in sections 9–12.

### Work-block shape

Each block contains:

- Stable number and title.
- Done checkbox.
- Dependencies or `none`.
- Affected source/knowledge areas.
- Required outcome.
- Focused verification commands.
- Test path/name and the contract promise/criterion each test covers.
- Structural evidence for work with no behavioural contract.
- Documentation surfaces delivered by the block, with their generation/render/check commands.

Every affected contract criterion appears in at least one executable evidence entry. A promise without nested criteria is cited directly. Every applicable documentation surface belongs to a block and has executable evidence. There is no `manual` case marker.

## Project documentation map

Add one canonical `Documentation` concept and schema to each managed project, for example `knowledge/documentation.md`. It is the project-specific inventory that lets the workflow reason beyond `README.md` without embedding project layouts in prompts or Rust.

Each surface has a stable local name and declares:

- Audience and purpose.
- Kind: `handwritten`, `generated`, or `site`.
- Authored source paths/globs.
- Generated/derived paths/globs, or `none`.
- Source of truth; a generated target is never its own source.
- Objective applicability triggers: public contract IDs/kinds, components, configuration/CLI/API/UI changes, migration requirements, or source path patterns.
- Named generation/render command from `development-commands`, or `none` for direct prose.
- Named verification command: docs build, link check, doctest/snippet test, schema check, or generated-drift check.
- Whether generated output is committed or build-only.
- Publication/deployment owner as informational metadata; publishing remains outside this workflow.

Seed a minimal map during init and migration. It must explicitly declare the repository README and any known docs roots/generators/sites; a project with no additional surface says so. Humans own the project-level boundary through ordinary reviewed changes to this map. SCOPE may propose a missing surface, but it cannot silently omit a declared one or invent an undocumented publication process.

Documentation rules:

1. A user-observable contract/configuration/CLI/API/UI/migration change triggers a SCOPE documentation-impact pass.
2. SCOPE maps every changed promise/criterion and public surface to each applicable documentation surface, or records a checked `none` rationale.
3. The requirements reviewer rejects an unaccounted declared trigger, stale product name/example, missing migration guidance, or generated target with no source/generator.
4. BUILD updates documentation in the block that makes the behaviour true, rather than leaving all prose to an optional final sweep.
5. Handwritten docs are edited at their authored paths.
6. Generated docs are changed only through their declared source and generator. After generation, rerunning the generator must produce a clean diff.
7. Documentation sites are rendered/built locally with their declared command; deployment remains release work.
8. Executable examples, snippets, API references, links, and doctests run through declared checks where the project provides them.
9. Final verification reruns all affected documentation checks and records their bounded results against candidate `H`.
10. The isolated final reviewer receives the rendered/source documentation diff and performs a consumer-facing consistency pass against the issue, contracts, and changed interface. Documentation findings are actionable BUILD findings.
11. ACCEPT refuses missing/stale documentation evidence but never delegates documentation work to the human.

The documentation map is canonical project knowledge. Outward-facing docs summarize and cite canonical contracts/ADRs where appropriate; they do not duplicate internal design detail merely to satisfy the gate.

## Canonical issue schema

Retain `kind: bug | feature | chore` and `lifecycle: open | done | wontfix`, with these changes:

- Use incoming `implements` relationships from plans as the issue’s implementing-plan history; do not store a second reciprocal list.
- Add an optional `Discoveries` section before Resolution/Comments.
- Discoveries are checkboxes with the observed problem, evidence, phase/block, and disposition.
- BUILD appends every behavioural, scope, or ambiguous discovery to the primary issue.
- SCOPE resolves each discovery by updating requirements/contracts/ADRs/plan and ticking it with a concise resolution reference.
- ACCEPT rejection is appended verbatim as an unresolved discovery and returns the plan to SCOPE.
- Resolution is still prohibited while open.
- Define `done` as coded, accepted, and integrated—not released.
- Keep keys and EARS grammar exclusively in contracts.

## Workflow state and Rust authority

### Module placement

Move workflow domain logic into `superdev-core`, for example:

```text
crates/lib/superdev-core/src/workflow/
  mod.rs
  config.rs
  state.rs
  plan.rs
  issue.rs
  transition.rs
  git.rs
  filing.rs
  evidence.rs
  policy.rs
```

Keep `crates/app/superdev/src/workflow_cli.rs` as Clap/JSON rendering only. Retire the Claude-specific behavior in `crates/app/superdev/src/run.rs` after migration.

### Durable versus transient state

The plan is canonical for:

- Phase and lifecycle.
- Current/completed blocks.
- Attempts and stalls.
- Implementation decisions.
- Discoveries/follow-ups.
- Verification, review, and acceptance evidence.

A versioned `.superdev/cache/workflow.toml` is transient and contains only:

- Owning Pi session ID.
- Issue, plan, work branch, and default branch IDs.
- Current child role/PID and start time.
- Cancellation state.
- Last observed plan revision.
- Repository-wide exclusive workflow lock metadata.

Absence of the cache file means no session owns the workflow; it does not mean the plan is complete. Resume reconstructs from the plan, Git, and configuration, then acquires ownership exclusively.

### State transitions

Implement typed transitions and reject every unlisted transition:

```text
(no plan) -> scope
scope -> build          explicit human scope approval + clean requirements review
build -> scope          unresolved design/behaviour discoveries
build -> build          block progress, retries, review corrections, technical stall
build -> accept         all blocks/evidence/pending/verification/review gates pass
accept -> scope         human rejection
accept -> done          configured acceptance passes; closure commit then no-ff merge
done(prepared, not on default) -> build   stale-default recovery before integration
scope|build|accept -> abandoned   explicit human abandonment + approved knowledge disposition
```

`done` becomes terminal when the closure commit is reachable from the default branch. A closure commit prepared on the work branch but not yet merged is an ACCEPT recovery state: integration may be retried, or a stale default may force a machine-authored reopening commit and return to BUILD. `abandoned` is terminal once its knowledge-only abandonment record reaches the default branch. Explicitly reopening an integrated issue permits SCOPE to create a new linked plan and branch; it never mutates completed or abandoned plan history. Ordinary later regressions should normally be filed as new issues.

Cancellation releases transient ownership and kills children but leaves the canonical phase unchanged. During initial SCOPE it also leaves any uncommitted draft files intact on the work branch; resume adopts them. It never commits unapproved scope or deletes the draft implicitly. Abandonment is accepted only from an interactive human command and follows the separate disposition flow below.

### CLI surface

Replace the old Claude-oriented `superdev run` protocol with a versioned workflow protocol used by Pi:

```text
superdev workflow start
superdev workflow status --json
superdev workflow bind
superdev workflow transition
superdev workflow block
superdev workflow evidence
superdev workflow resume
superdev workflow cancel
superdev workflow abandon
superdev workflow integrate
superdev file
```

Use typed subcommands/events in Rust; do not accept arbitrary phase strings or permit TypeScript to rewrite state files. Every mutating command takes the owning session and expected current phase/revision, uses compare-and-swap semantics, and returns bounded machine-readable JSON with a protocol version. Human-readable rendering may wrap the same result.

The exact source enums and structs are the interface source under ADR-042. Materialize their contract `Definition` sections with `superdev validate --fix`; do not hand-copy declarations into contracts during SCOPE.

### Safety and transactions

- Discover the repository root before all path/Git operations.
- Canonicalize and contain all paths.
- Acquire repository locks for workflow transitions and ID allocation.
- Persist a workflow’s reserved issue/plan numbers in transient ownership state until their files are committed; `/file` allocation excludes live reservations so a concurrent default-branch filing cannot take them.
- Write plan/issue updates atomically and run SOKF repair/validation through shared core services.
- Preserve applied invalid intermediate SOKF mutations only where existing agent-safe mutation semantics require it; workflow phase transitions themselves fail closed when gate validation fails.
- Validate branch name/ref arguments and invoke Git without a shell.
- Never interpolate model text into shell commands.
- Compare expected branch tips before synchronization and integration.
- Return bounded diagnostics with references to full local logs where applicable.

## `/file` outside the workflow

`/file` remains available while another workflow branch is active.

### Semantics

- Capture either an Issue or Idea, preserving the human’s words and performing duplicate search.
- Do not scope, branch, or create a plan.
- Acquire a short repository-wide filing/ID lock.
- If the clean default branch is the current checkout, show the diff, request confirmation, validate, and create a knowledge-only commit there.
- If a workflow branch is current, use a temporary worktree based on the default branch, create/validate the record there, and advance the default branch without touching the active worktree.
- The active workflow status reports the advanced default immediately. SCOPE rereads the filed record and rechecks duplicate/requirements implications before approval; BUILD incorporates the commit before final verification.
- If the default branch is dirty, checked out unsafely elsewhere, or changes concurrently, abort without writing to either worktree.
- Never silently file on the active work branch as a fallback.
- A human-confirmed `/file` operation may advance the default branch because it is an explicit, knowledge-only filing action.

Test simultaneous numbering, concurrent default advancement, worktree cleanup, validation failure, cancellation, and paths containing spaces.

## Phase algorithms

### SCOPE

1. Ensure configuration is valid, no workflow is owned, the default branch is checked out, and the worktree/index are clean.
2. Search SOKF issues/ideas/contracts/ADRs and source for duplicates and prior decisions.
3. Adopt the supplied open issue or reserve an issue ID for the request.
4. Create and switch to the uniform `work/<issue-number>-<slug>` branch without changing the default branch.
5. Author the issue when the request did not supply one.
6. Read relevant canonical knowledge and inspect affected source, callers, tests, generated interfaces, and history.
7. Interview one substantive uncertainty at a time. Challenge goals, exclusions, alternate interpretations, compatibility, security, migration, failure behavior, and project terminology.
8. Use targeted research/prototyping only when a real external or feasibility uncertainty exists. Preserve reusable findings in canonical knowledge.
9. Record accepted ADR decisions and normative knowledge.
10. Author changed contract promises/criteria with granular `PENDING (plan/block)` markers. Do not edit source declarations or generated Definition sections.
11. Read the project documentation map; map every user-observable change to applicable handwritten, generated, and site surfaces, assigning each update/check to a work block. Record explicit `none` only after evaluating every declared trigger.
12. Write the complete plan, including precise source/interface work and dependency-ordered blocks.
13. Run deterministic schema/reference/dependency/evidence/documentation-impact checks.
14. Spawn a fresh read-only requirements reviewer with the issue, user request, plan, changed contracts/ADRs, documentation map, and scope diff.
15. Resolve every supported finding. A finding may be dismissed only with explicit human rationale included in the next reviewer input. Rerun after substantive corrections until no unresolved finding remains.
16. Present the complete scope diff, documentation impact, and concise approval summary to the human.
17. On rejection, continue SCOPE; do not commit or start BUILD.
18. On explicit approval, set phase to `build`, run validation, and commit the issue, plan, contracts, ADRs, documentation map changes, and other approved normative knowledge together.
19. Automatically launch the isolated BUILD worker.

SCOPE mutation gates allow authored normative knowledge by schema/type/section policy and prohibit product source, tests, generated definitions, and implementation-dependent current-state claims.

### BUILD

1. Claim the run and verify plan, issue, branch, approval commit, phase, and clean tree.
2. Select the first dependency-ready incomplete block.
3. Inspect relevant code/tests/knowledge before editing.
4. Add or update executable evidence before or alongside implementation where meaningful.
5. Implement only the block.
6. Run focused commands and affected tests until pass or stall.
7. Add local implementation choices to `Implementation decisions`.
8. Ensure the diff obeys BUILD’s phase boundary and the block’s scope.
9. For each counted no-progress failure, update Build state and create a work-branch checkpoint commit containing the current partial block plus bounded diagnostics; this makes attempt budgets resumable without claiming the block is done.
10. On success, tick the block, update Build state/evidence, and commit code, tests, and plan together.
11. Repeat until no block is ready.

Discovery handling:

- Local implementation detail consistent with approved scope: decide, record in `Implementation decisions`, continue.
- Required correctness work within the approved outcome: perform now.
- Behaviour/interface/security/architecture/scope discovery: append to the primary issue; continue only independent ready work; when exhausted, checkpoint, set phase to `scope`, and return to the parent.
- Ambiguous discovery: append to the primary issue and handle through SCOPE.
- Unrelated defect/improvement: file a separate issue, link it under Follow-up issues, and continue if current correctness is unaffected.
- Never weaken a promise, criterion, test, review finding, or gate.

Stall handling:

- Compute a stable failure fingerprint from command, exit status, and normalized diagnostics.
- Retry while attempts make observable progress.
- After `max_stalled_block_attempts` consecutive no-progress attempts for the same fingerprint, mark the block stalled and record attempted fixes/diagnostics.
- Continue independent ready blocks.
- When exhausted, return design uncertainty to SCOPE; leave a purely technical blocker in BUILD and request guidance/resume.
- Never mark, defer, or bypass a required stalled block.
- Only meaningful evidence change or approved re-scope resets the affected attempt count.
- Persist every counted attempt in a checkpoint commit with a transition trailer; raw command logs stay outside the plan.

Finalization:

1. Ensure every block is complete.
2. Implement source declarations before regenerating contract Definition sections.
3. Run `superdev validate --fix` and inspect generated changes.
4. Remove each affected `PENDING` marker only after implementation and cited evidence exist.
5. Update current-state knowledge, migration guidance, changelog, and every planned documentation surface through its declared source-of-truth path.
6. Run each affected documentation generator/render/check; rerun generators to prove committed outputs are clean and include these checks in candidate evidence.
7. Incorporate the current default branch into the work branch. Conflicts stop for guidance.
8. Commit finalization changes.
9. Designate the synchronized tip as immutable candidate revision `H` and run the project’s complete local CI-equivalent build/test/typecheck/lint/validate/documentation list once on `H`.
10. Spawn a fresh read-only code reviewer against the complete `merge-base..H` diff, including authored and rendered/generated documentation evidence.
11. Review correctness, regressions, security, contract/ADR conformance, generated definitions, test coverage, migration, user-facing documentation completeness/accuracy, and unnecessary complexity.
12. Require structured findings: severity, path/line, evidence, impact, and required correction.
13. Any actionable finding returns to BUILD. Fix it, run focused checks, create a new candidate, rerun the entire final verification, and obtain a fresh reviewer.
14. Count combined final verification/review correction cycles against `max_final_correction_cycles`.
15. On exhaustion, remain in BUILD and ask for human guidance; never waive a finding automatically.
16. After a clean review of `H`, have the Rust service create one administrative attestation commit `A` that records bounded verification/review evidence, references `H`, and moves the plan to `accept`. The enforced `H..A` diff may touch only machine-owned plan evidence/phase and resulting generated indexes/links; it does not invalidate evidence for `H`.
17. Any non-administrative change after `H` invalidates both attestations and starts a new correction cycle.

BUILD runs as one isolated modifying Pi child. It cannot ask the user, spawn another workflow, run ACCEPT, or mutate the default branch. A replacement child resumes from plan and Git state after cancellation/crash.

### ACCEPT

1. Verify phase `accept`, plan/issue links, all blocks complete, no unresolved discoveries, no affected `PENDING`, current generated definitions, complete documentation-surface evidence, clean tree, immutable candidate `H`, and an administrative-only attestation path from `H` to current tip `A`.
2. Verify the default branch still matches the state included in final BUILD verification. If it advanced, return to BUILD for synchronization, a new candidate, full verification, and fresh review.
3. Present goal, diff summary, contract criteria/evidence, verification result, review result, follow-up issues, and acceptance mode.
4. If `human_acceptance_required = true`, request one accept/reject decision.
5. On rejection, append feedback verbatim to the primary issue, invalidate final evidence, set phase to `scope`, and resume interactive SCOPE on the same plan.
6. If acceptance is automatic, make no model judgment; mechanically apply the configured policy after all gates pass.
7. Set plan `lifecycle: done`, `phase: done`; set issue `lifecycle: done` and write Resolution; update generated indexes/links; validate; commit this administrative closure `C` on the work branch. Enforce that `A..C` contains only ACCEPT-owned records.
8. Recheck the expected default tip, check out the default branch, recheck it again, and run `git merge --no-ff <work-branch>` without a shell.
9. If merge fails, stop and retain `C` as an ACCEPT recovery state. On the default branch, the issue remains open until the merge lands. If the default changed, integration is not attempted; the service writes an administrative reopening commit on the work branch and returns to BUILD for synchronization and fresh evidence.
10. Record transient completion status for the UI. Do not push, delete the work branch, release, or wait for remote CI.

ACCEPT may edit only issue/plan status/evidence, generated indexes/links caused by those edits, and Git integration state. It never fixes product or requirement gaps.

### ABANDONMENT (human-only escape, not a workflow phase)

1. `/superdev-abandon` stops any child and presents the issue, plan, work-branch tip, unmerged commits, pending promises, and knowledge changes.
2. The human chooses whether the underlying issue remains `open` for a future plan or becomes `wontfix`, and explicitly approves which already-settled ADR/normative knowledge remains valid.
3. SCOPE prepares an abandonment record: plan reason and branch tip, issue disposition, no unresolved `PENDING` promise on the default-branch result, and only the approved durable decisions.
4. Show the exact knowledge-only diff for human approval.
5. Commit the approved abandoned status/disposition on the work branch so that branch also tells the truth.
6. Use a temporary worktree at the current default tip to commit only that approved issue/plan/ADR/normative knowledge and generated indexes/links. Do not merge or cherry-pick product, test, generated-interface, changelog, or implementation-dependent commits from the work branch.
7. Mark the copied plan `lifecycle: abandoned`, `phase: abandoned`; validate and advance the default branch with the knowledge-only commit.
8. Retain the work branch for inspection. A future attempt uses the same open issue but creates a new plan/branch.
9. Failure leaves the default branch unchanged and the original work branch intact. The model cannot invoke or approve abandonment itself.

## Pi extension

### Layout

Add a project extension with small modules:

```text
.pi/extensions/superdev/
  index.ts
  commands.ts
  workflow-client.ts
  children.ts
  prompt-loader.ts
  prompts/
    file.md
    scope.md
    build.md
    requirements-review.md
    code-review.md
  gates.ts
  state.ts
  rendering.ts
  filing.ts
```

Treat these Markdown files as private role prompts owned by the extension, not Pi-discovered skills or prompt-template commands. The extension registers the public commands, loads the appropriate prompt by phase/child role, and injects it through `sendUserMessage()` or the child process’s appended system prompt. Keep the files beside the extension rather than embedding long strings in TypeScript. Do not create workflow entries under `.pi/skills/`; independently invocable general capabilities such as `sokf-authoring` remain genuine Pi skills.

Native Pi skills are intentionally not used for workflow roles: they advertise model-invocable capabilities and `/skill:*` entry points, load by model choice, and can run independently of the extension state machine. SCOPE, BUILD, reviewers, and `/file` instead require deterministic role selection, fixed tool sets, structured completion, and legal-transition enforcement. Co-locating their prompts with the extension gives one owned package and removes a second invocation path that could bypass orchestration.

### Responsibilities

- Discover the repository root from subdirectories.
- Verify `superdev` and required tools at `session_start`.
- Register commands and structured child-result tools.
- Call only the Rust workflow CLI for durable transitions.
- Use `ctx.ui.input/editor/select/confirm` only when `ctx.hasUI`; SCOPE cannot approve without UI.
- Automatically advance legal phases and stop on every illegal or ambiguous transition.
- Show compact phase/block/retry/review status with `setStatus`, widgets, and custom entry renderers.
- Store only reconstructable presentation entries via `appendEntry`; rebuild them from the active session branch and `workflow status`.
- Preserve phase, plan, branch, current block, attempts, child role, and next legal action during compaction.
- Warn/block session switch/fork while a modifying child owns the run unless cancellation/release succeeds.
- Propagate Escape/abort signals; terminate child process trees with bounded TERM/KILL behavior.
- Suppress workflow continuation recursively in children through an explicit environment marker.
- Never parse or write `.superdev/cache/workflow.toml` in TypeScript.

### Child agents

Use Pi’s supported subagent process pattern: JSON mode, print mode, no child session, inherited model/thinking level unless explicitly configured later, repository cwd, streamed bounded updates, cancellation propagation, and usage reporting.

Roles:

- Requirements reviewer: fresh context; read, SOKF retrieval, and read-only diff tools only.
- BUILD worker: read/edit/write/bash and SOKF tools; one instance; no nested workflow/acceptance/user UI.
- Code reviewer: fresh context; read, SOKF retrieval, and read-only diff tools only.

Register a read-only diff/search tool for reviewers rather than granting unrestricted Bash. Reviewer tool sets omit edit/write and mutating shell access. Child-role environment and extension `tool_call` gates provide defense in depth.

Child final results use a terminating structured tool rather than prose parsing. Result variants include clean, findings, return-to-scope, stalled, cancelled, and failed, with bounded details and references to durable records.

### Phase mutation enforcement

Enforce twice:

1. Pi `tool_call` preflight blocks obviously illegal tools/paths for the current role and phase.
2. Rust transition-time diff validation compares the phase baseline to the candidate commit and rejects forbidden mutations.

Do not rely on tool-name filtering alone; Bash and generated changes can bypass it.

## ADR and contract work

Create one new workflow ADR that records the complete replacement and supersedes or narrows the workflow portions of:

- ADR-018: Claude skill/Stop-hook loop.
- ADR-019: Claude session-owned run state and hook counter.
- ADR-020: deferred-decision blocked-run behavior.
- ADR-021: human fast-forward default-branch boundary.
- ADR-028: contract-design-specific approval interaction.
- ADR-044: SCOPE/contract-design editing source declarations; preserve the valid rule that definitions materialize from source and pending applies to prose promises.
- ADR-050: FILE as phase, execute-plan, optional manual ACCEPT, BUILD merge/closure; preserve contract keys/EARS and one issue/plan shapes where still valid.

Retain ADR-042 unchanged: source declarations own materialized interface definitions.

Revise/create contracts for:

- `contract-002-cli-superdev`: new workflow/file command tree, JSON protocol, exit behavior; remove retired run verbs after migration.
- `contract-004-config-superdev`: `[workflow]` source struct and defaults.
- `contract-009-interface-run-state`: replace Claude hook seam with canonical plan plus transient workflow ownership, or deprecate it and create a new internal workflow-state contract if the old identity would be misleading.
- `contract-010-interface-document-schemas`: source schema declarations for revised Issue, Plan, and Documentation-map shapes.
- A new internal Pi workflow-adapter contract: commands, child roles, phase gates, cancellation, and persistence seam.
- Pack/materialization contracts affected by Pi assets and Claude-skill retirement.

During the implementation BUILD, modify source declaration regions first and regenerate every `Definition` section with `superdev validate --fix`.

## Claude archive and content-pack migration

1. Move every active Claude skill intact to `archive/claude-code/skills/`, including both knowledge-carried skills and the separate `pack/skills` provider’s materialized skills.
2. Add `archive/claude-code/README.md` stating they are deprecated historical references, naming their retired workflow and last supported revision.
3. Remove all active `.claude/skills/` materialization plus Claude Stop/PostToolUse hook ownership; retire or retarget the Claude-specific `skills` provider so sync cannot recreate any active Claude skill.
4. Do not archive unrelated user-owned Claude files or rely on deleted local `CLAUDE.md`/`AGENTS.md` state during implementation.
5. Update pack ownership so sync removes retired managed Claude assets without treating the archive as active content.
6. Develop/prove `.pi/extensions/superdev/`, including its private Markdown role prompts, in this repository first.
7. Add the complete extension directory as a first-class Pi asset in `pack/` only after integration tests pass.
8. Teach content layout/components to materialize `.pi/extensions/` with adoption, lock hashes, drift, and orphan cleanup. Preserve separately managed genuine `.pi/skills/` such as `sokf-authoring`; this workflow adds none.
9. Existing projects receive Pi assets and workflow configuration through sync. No harness-selection matrix is introduced.
10. Document Pi project trust, permissions, disabling the extension, and non-interactive limitations.

## Existing-record migration

Implement a deterministic Rust migration, covered by fixtures, then run it over this repository and pack copies.

- Rewrite every open and completed issue/plan to the new strict schema.
- Preserve prose and history; map old sections to the nearest new section without model-authored reinterpretation.
- Add `phase: done` to completed plans and `phase: abandoned` to any historically abandoned plan.
- Set every open migrated plan to `phase: scope`; existing checkboxes remain historical input, and human re-scope decides what work/evidence is still valid.
- Create a migration-derived `done` issue for every orphan historical plan, using only its existing goal/history and clearly identifying the derivation.
- Establish one authoritative plan-to-issue `implements` link; rely on incoming graph relationships rather than writing reciprocal duplicates.
- Remove Deferred decisions by moving unresolved substantive items to the primary issue Discoveries section; preserve answered local choices under Implementation decisions.
- Keep the Rust migration mechanical: copy old case text verbatim into clearly labelled legacy evidence/migration notes, and give every open migrated plan an unresolved discovery requiring human re-scope into executable evidence. Preserve completed manual cases only as historical notes; never infer test paths, commands, or coverage from prose.
- Regenerate indexes and lock hashes.
- Support only the new schema after migration.

## Work breakdown

### Bootstrap entry gate for this plan

This root document intentionally predates the workflow it specifies and is the one bootstrap exception to the future canonical plan shape. Before product-source edits, record its approved decisions in the superseding ADR and authored contract promises, with `PENDING` markers tied to the blocks below. Do not create a second prose plan that duplicates this document. Once Block 1 installs the new schema/service, all subsequent workflow plans obey the new issue/plan invariants.

### Block 1 — Plan, issue, and documentation schemas

- Update source schema definitions for Plan and Issue, including phase, links, Discoveries, Build state, Implementation decisions, Documentation changes, and Completion evidence.
- Add the project Documentation concept/schema and seed it in project knowledge/templates.
- Add schema-level phase mutation ownership metadata/rules.
- Update schema snapshots and normative shape tests.

Focused evidence:

- Valid examples for every phase/lifecycle combination, including abandoned.
- Invalid transition/relationship/section ownership fixtures.
- Documentation-map fixtures for handwritten, generated, and site surfaces, including trigger coverage and source-of-truth constraints.
- Manual-case rejection.
- One-open-plan-per-issue and authoritative `implements`-link/incoming-history checks.

### Block 2 — Workflow configuration and state domain

- Add `WorkflowConfig` to `manifest.rs` and generated defaults.
- Add `superdev-core::workflow` state, plan/issue parsing, typed transitions, ownership, retry counters, evidence, and atomic writes.
- Replace hard-coded Claude caps with configured workflow retry limits.
- Keep transient state versioned and session-owned; implement status/cancel/resume.

Focused evidence:

- Config parse/render/default/migration tests.
- Transition table tests, stale-revision refusal, exclusive ownership, crash/resume, cancellation, and invalid-state diagnostics.
- Attempt fingerprint/progress/reset tests.

### Block 3 — Git and filing services

- Implement clean-tree/default-branch checks, branch creation, synchronization preconditions, closure commit, and no-ff integration without shell interpolation.
- Implement repository-wide ID allocation/filing lock and `/file` temporary-worktree path.
- Ensure failures leave user work and default refs unchanged where promised.

Focused evidence:

- Scratch repositories for dirty/untracked trees, wrong branches, advanced default, conflicts, paths with spaces, concurrent transitions, and no-ff graph shape.
- `/file` on default and during an active work branch.
- Temporary-worktree cleanup and compare-and-swap failures.

### Block 4 — CLI protocol and interface materialization

- Add `workflow_cli.rs` and filing adapter; wire Clap commands in `main.rs`.
- Return versioned bounded JSON for Pi and human rendering from the same domain result.
- Deprecate/remove old `run begin/advance/end` and Claude hook behavior in a controlled migration.
- Regenerate CLI/config/workflow contract Definitions from source.

Focused evidence:

- CLI integration and exit-code tests for every command and refusal.
- JSON golden fixtures and bounded-output tests.
- Contract drift/normative source-region tests.

### Block 5 — Pi commands, state, and interaction

- Build `.pi/extensions/superdev/` command registration, client, state restoration, renderers, UI confirmation, and automatic transitions.
- Add the extension-owned SCOPE and `/file` role prompts and prompt loader.
- Preserve state through compaction/session restart and refuse unsafe session switching/forking.

Focused evidence:

- Fake `ExtensionAPI` command registration and state restoration.
- No-UI scope approval refusal.
- Scope approval starts BUILD exactly once.
- Cancel/resume and stale owner behavior.
- Existing SOKF extension composition remains intact.

### Block 6 — Isolated BUILD and reviewers

- Implement child process invocation, fixed roles/tool sets, streamed updates, cancellation, recursion suppression, and structured terminating results.
- Add requirements-review, BUILD, and code-review prompts.
- Implement phase tool gates and transition-time diff gates.
- Implement block/final retry orchestration using configured limits.

Focused evidence:

- Reviewers cannot call edit/write/unrestricted bash.
- Only one modifying child starts.
- Cancellation kills the process tree and preserves committed progress.
- Counted failed attempts and final correction cycles create resumable checkpoint commits without marking blocks complete.
- Abandonment requires a human and lands only the approved knowledge record, never partial product work.
- Scope discoveries return after independent work is exhausted.
- Review findings cause focused fixes, complete re-verification, and a fresh reviewer.
- Three-default-cycle exhaustion halts without waiver.

### Block 7 — ACCEPT and end-to-end orchestration

- Implement automatic/human ACCEPT behavior, rejection-to-SCOPE, closure, and no-ff merge.
- Implement human-only abandonment with issue disposition and knowledge-only default-branch recording.
- Ensure default advancement invalidates evidence and returns to BUILD.
- Ensure ACCEPT cannot edit product code.

Focused evidence:

- Human-required accept/reject UI flows.
- Human-only abandon flow for open-issue and wontfix dispositions, proving partial implementation never reaches default.
- Automatic acceptance with no model decision.
- Rejection preserves issue text and same plan identity.
- Default advancement, merge conflict, dirty tree, stale candidate/attestation, and pending promise refusal.
- Final Git graph contains a merge commit and closure records on default.

### Block 8 — Migration, archive, and pack shipping

- Run deterministic issue/plan migration.
- Archive Claude skills and remove their active materialization/hooks.
- Add the complete Pi extension, including private role prompts, to the pack and lock/drift/orphan ownership.
- Rewrite workflow knowledge, architecture, components, commands, glossary, testing strategy, definition of done, documentation map, README/help, generated references, docs-site sources/build evidence where present, and changelog.
- Resolve issues 057 and 058 in the migrated workflow; update idea 010 to implemented/retired according to its schema.

Focused evidence:

- Migration fixture corpus and idempotence.
- Every repository issue/plan validates under one schema.
- Every declared documentation surface has a valid source of truth and generation/verification command policy.
- Sync installs Pi assets and removes managed Claude assets.
- Status/drift and adoption tests.
- Archived Claude files are searchable but not active/materialized.

### Block 9 — Final verification and review

Run, on the work branch synchronized with current default:

```text
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
npm run test:scripts
npm run check:validate
git diff --check
```

Also run every affected generation/render/verification command named by the project documentation map and the real-Pi smoke fixture described below. Record exact commands, candidate revision, and bounded results in the plan attestation. Then run the mandatory isolated code review and correction loop.

## Test strategy

### Rust unit tests

- Configuration defaults and validation.
- Plan/issue parsing and exact atomic edits.
- Legal/illegal transitions.
- Phase ownership classification by document type/section.
- Retry fingerprinting and counters.
- Gate evaluation and evidence freshness.
- Documentation trigger matching, source/generated-path ownership, and required evidence.
- Repository locks and cancellation state.
- Git command argument construction without a shell.
- Record migration and idempotence.

### Rust integration tests

Use scratch repositories; never run stateful workflow probes in the real checkout.

- Start/adopt/create issue and branch.
- Scope approval and commit preconditions.
- Block progress/commit/resume.
- Return-to-scope with accumulated discoveries.
- Pending promise, generated-definition, and documentation-surface gates.
- Handwritten, generated, and docs-site fixture builds, including clean second generation.
- Final evidence bound to commit IDs.
- Human and automatic acceptance.
- No-ff merge and failure recovery.
- `/file` during an active branch.
- CLI JSON/exit contract.

### TypeScript tests

Keep reducers, command parsing, output truncation, role policy, and rendering pure where possible. Drive a fake Pi API to cover:

- Registration and command dispatch.
- Repository discovery from a subdirectory.
- UI/no-UI behavior.
- Child role/tool configuration.
- One modifying child and recursion suppression.
- Abort propagation.
- Session-branch state reconstruction and compaction summary.
- Automatic transition scheduling without duplicate follow-ups.
- Tool-call phase gates.
- Coexistence with SOKF’s persistent MCP extension.

### Real Pi smoke

Add one deterministic fixture repository and run actual Pi in JSON/print mode to prove:

1. `/scope` creates/adopts an issue and reaches the approval boundary.
2. Approval launches a two-block isolated BUILD.
3. Each block commits focused evidence.
4. Final verification invokes a fresh reviewer.
5. Automatic ACCEPT closes records and creates a no-ff merge while leaving remote state untouched.
6. Cancellation and resume retain plan progress.

Use a fake deterministic model/provider where possible. Do not add an expensive provider/model matrix. Behaviour fixtures may prove protocol and prompt structure but must not be presented as conclusive model-behaviour scores.

## Failure matrix

| Condition | Required result |
|---|---|
| Dirty default at new SCOPE | Refuse; change nothing |
| No issue supplied | Reserve/create/link issue before plan |
| Duplicate issue found | Present existing record; do not create duplicate without human direction |
| No UI during SCOPE approval | Pause/refuse approval; never infer consent |
| Requirements finding | Correct or obtain explicit human disposition, then rerun reviewer |
| Child crash/cancel | Kill process tree, release child state, preserve committed plan/Git progress |
| Human abandons plan | Record approved knowledge only on default; retain work branch; merge no partial implementation |
| Same block failure without progress | Count configured attempts, mark stalled, continue independent blocks |
| Design/behaviour discovery | Record on primary issue; exhaust safe work; return same plan to SCOPE |
| Ambiguous discovery | Record on primary issue; return through SCOPE |
| Unrelated finding | File separate issue; continue only if current correctness is unaffected |
| Pending affected promise at BUILD end | Refuse ACCEPT |
| Stale generated Definition | Regenerate and rerun final verification/review |
| Declared documentation trigger is unplanned | Return to SCOPE; assign the surface to a block |
| Authored/generated/site docs are stale or fail to build | Return to BUILD; update source, regenerate/render, and rerun review |
| Full verification failure | Return to BUILD correction cycle |
| Review finding | Return to BUILD; focused checks + full verification + fresh review |
| Correction budget exhausted | Remain BUILD; request guidance; waive nothing |
| Human rejection | Record verbatim; same plan returns to SCOPE |
| Default advanced after review | Return BUILD; synchronize, verify, review again |
| Merge conflict | Stop; never auto-resolve in ACCEPT |
| Remote unavailable | Irrelevant; no push/poll operation occurs |
| `/file` cannot safely update default | Abort without filing or touching active worktree |

## Documentation changes

Update canonical and outward-facing documentation so it says one thing:

- Architecture and software components: Rust workflow authority and Pi adapter.
- Development procedure: `/file` utility plus `SCOPE → BUILD → ACCEPT`.
- Development commands: workflow CLI and Pi commands.
- Configuration: workflow table and safe defaults.
- Documentation map: audiences, authored/generated/site surfaces, triggers, sources of truth, and named generation/check commands.
- Testing strategy/definition of done: block checks, documentation checks, final verification, isolated review, and acceptance policy.
- Security requirements: trusted Pi project extension, role tool boundaries, shell-free Git, and child cancellation.
- Issue tracker/glossary: primary issue, implementation discovery, human acceptance, reviewed commit, and done-versus-released.
- README/agent instructions: Pi support and removal of active Claude workflow claims.
- Changelog: workflow replacement, migration, and Claude adapter deprecation.

Summarize outward-facing content and cite canonical SOKF concepts rather than duplicating the full workflow.

## Acceptance checklist for this implementation

The implementation is acceptable only when all are true:

- [x] A request can enter `/scope`, automatically gain an issue/plan/branch, receive explicit scope approval, and proceed without a separate FILE phase.
- [x] SCOPE cannot commit product source or generated Definition edits.
- [x] Requirements review is isolated, read-only, mandatory, and clean on the approved revision.
- [x] BUILD is one isolated modifying child and owns the whole dependency-ready block loop.
- [x] Every block has executable focused evidence and a separate commit.
- [x] Every user-observable change is mapped to all applicable project-declared documentation surfaces, or carries a checked `none` rationale.
- [x] Local decisions are recorded in Implementation decisions without transcript/chain-of-thought content.
- [x] Discoveries follow the agreed original-issue/re-scope policy.
- [x] Retry limits come only from project configuration and survive resume.
- [x] No affected promise or criterion can pass with `PENDING` or missing automated evidence.
- [x] Source declarations regenerate contract Definitions through validation.
- [x] Handwritten docs, generated references, and docs-site sources are updated through their declared source of truth; generators are clean on a second run and declared docs checks pass.
- [x] Full local verification and a fresh isolated read-only review—including the consumer documentation lens—are bound to immutable candidate `H`; only diff-gated administrative attestation/closure commits follow it.
- [x] Review/verification correction cycles rerun both gates and stop at the configured limit.
- [x] ACCEPT obeys the project-wide human flag and never delegates a checklist to the human.
- [x] Rejection returns the same issue/plan to SCOPE.
- [x] Human-only abandonment records a truthful issue/plan disposition on default without merging partial product work.
- [x] Successful ACCEPT commits closure and merges locally with `--no-ff`.
- [x] Dirty trees, stale commits, advanced default refs, and conflicts fail safely.
- [x] `/file` can land a human-confirmed issue/idea independently while a workflow branch is active.
- [x] Remote CI, push, release, and branch deletion do not occur.
- [x] Every existing issue/plan validates under the one new schema; historical orphan plans have migration-derived issues.
- [x] Active Claude workflow skills/hooks are gone; archived copies are clearly deprecated and searchable.
- [x] `superdev sync` installs and governs the Pi workflow assets.
- [x] Rust, script, adapter, migration, real-Pi smoke, formatting, lint, validation, and diff checks all pass.
- [x] A final isolated code review reports no actionable finding.

## Implementation completion rule

Do not mark this work complete until the final synchronized commit has:

1. No incomplete work block.
2. No unresolved primary-issue discovery required for the outcome.
3. No affected `PENDING` promise.
4. Current source-materialized contract Definitions and every applicable documentation surface current.
5. Passing focused, documentation, and full verification evidence bound to immutable candidate `H`.
6. A clean isolated review bound to `H`, followed only by allowed administrative commits.
7. Passing configured ACCEPT policy.
8. The plan and issue closed and present on the default branch through a no-fast-forward merge.
